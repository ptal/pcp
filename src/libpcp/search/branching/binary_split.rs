// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use solver::iterator::*;
use search::branching::*;
use search::branching::branch::*;
use propagators::cmp::*;
use variable::arithmetics::*;
use solver::solver::*;
use kernel::Space;
use solver::fd::event::*;
use variable::*;
use interval::ncollections::ops::*;
use num::traits::Num;
use num::PrimInt;

pub struct BinarySplit;

impl<S,B,D,R> Distributor<S> for BinarySplit where
  S: VariableIterator<Variable=D> + Space<Constraint=Box<PropagatorErasure<FDEvent, D>>>,
  D: ExprInference<Output=R> + Clone + Cardinality + Bounded<Bound=B> + ShrinkLeft<B> + ShrinkRight<B> + Subset + 'static,
  R: Bounded<Bound=B>,
  B: PrimInt + Num + PartialOrd + Clone + 'static
{
  fn distribute(&mut self, space: &S, var_idx: usize) -> Vec<Branch<S>> {
    let var = nth_var(space, var_idx);
    assert!(!var.is_singleton() && !var.is_empty(),
      "Can not distribute over assigned or failed variables.");
    let mid = (var.lower() + var.upper()) / (B::one() + B::one());
    let mid2 = mid.clone();

    Branch::distribute(space,
      vec![
        Box::new(move |s: &mut S| {
          s.add(Box::new(x_leq_y(Identity::<D>::new(var_idx), Constant::new(mid))))
        }),
        Box::new(move |s: &mut S| {
          s.add(Box::new(x_greater_y(Identity::<D>::new(var_idx), Constant::new(mid2))))
        })
      ]
    )
  }
}

pub fn nth_var<S, D>(s: &S, var_idx: usize) -> D where
  S: VariableIterator<Variable=D>,
  D: Clone
{
  s.vars_iter()
  .nth(var_idx)
  .expect("Number of variable in a space can not decrease.")
  .clone()
}

#[cfg(test)]
mod test {
  use super::*;
  use search::branching::Distributor;
  use interval::interval::*;
  use interval::ops::*;
  use solver::solver::*;
  use solver::space::*;
  use solver::fd::event::*;
  use variable::ops::VarIndex;
  use solver::agenda::RelaxedFifoAgenda;
  use solver::dependencies::VarEventDepsVector;
  use std::ops::Deref;

  type FDSolver = Solver<FDEvent, Interval<i32>, VarEventDepsVector, RelaxedFifoAgenda>;

  #[test]
  fn binary_split_distribution() {
    let mut space: FDSolver = Solver::new();

    space.newvar(Interval::new(1,10));
    let var = space.newvar(Interval::new(2,4));
    space.newvar(Interval::new(1,1));

    let mut distributor = BinarySplit;
    let branches = distributor.distribute(&space, var.borrow().index());

    assert_eq!(branches.len(), 2);

    let expected_dom = vec![Interval::new(2,3), Interval::new(4,4)];
    let var_idx = var.borrow().index();
    for (branch, expected) in branches.into_iter().zip(expected_dom.iter()) {
      space = branch.commit(space);
      assert_eq!(space.solve(), Status::Satisfiable);
      let space_var = nth_var(&space, var_idx);
      assert_eq!(space_var.borrow().deref().deref(), expected);
    }
  }

  #[test]
  #[should_panic]
  fn binary_split_impossible_distribution() {
    let mut space: FDSolver = Solver::new();

    let var = space.newvar(Interval::new(1,1));

    let mut distributor = BinarySplit;
    distributor.distribute(&space, var.borrow().index());
  }
}

