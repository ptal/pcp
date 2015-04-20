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
use solver::variable::VarIndex;
use search::branching::*;
use interval::ncollections::ops::*;
use num::traits::Unsigned;

pub struct FirstSmallestVar;

impl<S, D, Size> VarSelection<S> for FirstSmallestVar where
  S: VariableIterator<Domain=D>,
  D: Cardinality<Size=Size>,
  Size: Ord + Unsigned
{
  fn select(&mut self, space: &S) -> usize {
    space.vars_iter()
      .map(|v| v.borrow())
      .filter(|v| v.size() > Size::one())
      .min_by(|v| v.size())
      .expect("Cannot select a variable in a space where all variables are assigned.")
      .index()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use search::branching::VarSelection;
  use interval::interval::*;
  use interval::ops::*;
  use solver::solver::*;
  use solver::space::*;
  use solver::fd::event::*;
  use solver::variable::VarIndex;
  use solver::agenda::RelaxedFifoAgenda;
  use solver::dependencies::VarEventDepsVector;

  type FDSolver = Solver<FDEvent, Interval<i32>, VarEventDepsVector, RelaxedFifoAgenda>;

  #[test]
  fn smallest_var_selection() {
    let mut solver: FDSolver = Solver::new();

    solver.newvar(Interval::new(1,10));
    let var2 = solver.newvar(Interval::new(2,4));
    solver.newvar(Interval::new(1,1));

    let mut var_selector = FirstSmallestVar;
    assert_eq!(var2.borrow().index(), var_selector.select(&solver));
  }

  #[test]
  fn smallest_var_selection_2() {
    let mut solver: FDSolver = Solver::new();

    solver.newvar(Interval::new(1,1));
    solver.newvar(Interval::new(1,1));
    solver.newvar(Interval::new(1,10));
    solver.newvar(Interval::new(1,1));
    let var5 = solver.newvar(Interval::new(2,4));
    solver.newvar(Interval::new(1,1));
    solver.newvar(Interval::new(1,1));

    let mut var_selector = FirstSmallestVar;
    assert_eq!(var5.borrow().index(), var_selector.select(&solver));
  }

  #[test]
  fn smallest_var_selection_first_if_equals() {
    let mut solver: FDSolver = Solver::new();

    solver.newvar(Interval::new(1,10));
    let var2 = solver.newvar(Interval::new(2,4));
    solver.newvar(Interval::new(2,4));

    let mut var_selector = FirstSmallestVar;
    assert_eq!(var2.borrow().index(), var_selector.select(&solver));
  }

  #[should_panic]
  #[test]
  fn smallest_var_selection_all_assigned() {
    let mut solver: FDSolver = Solver::new();

    solver.newvar(Interval::new(0,0));
    solver.newvar(Interval::new(2,2));
    solver.newvar(Interval::new(1,1));

    let mut var_selector = FirstSmallestVar;
    var_selector.select(&solver);
  }
}
