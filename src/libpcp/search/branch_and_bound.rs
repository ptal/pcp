// Copyright 2016 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use kernel::*;
use search::space::*;
use search::search_tree_visitor::*;
use search::search_tree_visitor::Status::*;
use term::*;
use term::ops::*;
use propagators::cmp::*;
use propagation::concept::*;
use propagation::events::*;
use gcollections::ops::*;
use gcollections::*;
use num::Integer;
use std::fmt::Debug;

pub enum Mode {
  Minimize,
  Maximize
}

pub struct BranchAndBound<V, Bound, C> {
  pub mode: Mode,
  pub var: V,
  pub value: Option<Bound>,
  pub child: C
}

impl<V, Bound, C> BranchAndBound<V, Bound, C> {
  pub fn new(mode: Mode, var: V, child: C) -> Self {
    BranchAndBound {
      mode: mode,
      var: var,
      value: None,
      child: child,
    }
  }
}

impl<C, Bound, Dom, V, VStore, CStore> SearchTreeVisitor<Space<VStore, CStore>> for
  BranchAndBound<V, Bound, C> where
 VStore: Freeze + Collection<Item=Dom>,
 V: StoreRead<VStore> + ViewDependencies<FDEvent>,
 V: StoreMonotonicUpdate<VStore> + Debug + Clone + 'static,
 CStore: Freeze + Alloc + Collection<Item=Box<PropagatorConcept<VStore, FDEvent>>>,
 C: SearchTreeVisitor<Space<VStore, CStore>>,
 Dom: Bounded + Collection<Item=Bound> + Cardinality + ShrinkLeft + ShrinkRight + Empty,
 Dom: StrictShrinkLeft + StrictShrinkRight + Singleton + 'static,
 Bound: Clone + Integer + Debug + 'static
{
  fn start(&mut self, root: &Space<VStore, CStore>) {
    self.child.start(root);
  }

  fn enter(&mut self, mut current: Space<VStore, CStore>)
    -> (<Space<VStore, CStore> as Freeze>::FrozenState, Status<Space<VStore, CStore>>)
  {
    if let Some(bound) = self.value.clone() {
      let bound = Constant::new(bound);
      match self.mode {
        Mode::Minimize => current.cstore.alloc(box XLessY::new(self.var.clone(), bound)),
        Mode::Maximize => current.cstore.alloc(box x_greater_y(self.var.clone(), bound)),
      };
    }
    let (mut immutable_state, status) = self.child.enter(current);
    if status == Satisfiable {
      let space = immutable_state.unfreeze();
      self.value = Some(self.var.read(&space.vstore).lower());
      immutable_state = space.freeze();
    }
    (immutable_state, status)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use search::test::*;
  use search::engine::all_solution::*;
  use search::engine::one_solution::*;
  use search::propagation::*;
  use search::branching::binary_split::*;
  use search::branching::brancher::*;
  use search::branching::first_smallest_var::*;
  use interval::interval::*;
  use gcollections::VectorStack;

  #[test]
  fn simple_maximize_test() {
    simple_optimization_test(Mode::Maximize, 9);
  }

  #[test]
  fn simple_minimize_test() {
    simple_optimization_test(Mode::Minimize, 0);
  }

  fn simple_optimization_test(mode: Mode, expect: i32) {
    let mut space = FDSpace::empty();
    let x = space.vstore.alloc((0,10).to_interval());
    let y = space.vstore.alloc((0,10).to_interval());
    space.cstore.alloc(box XLessY::new(x.clone(), y));

    let mut search: AllSolution<OneSolution<_, VectorStack<_>, FDSpace>>
      = AllSolution::new(
          OneSolution::new(
            BranchAndBound::new(mode, x.clone(),
              Propagation::new(Brancher::new(FirstSmallestVar, BinarySplit)))));
    search.start(&space);
    let (_, status) = search.enter(space);
    assert_eq!(status, EndOfSearch);
    assert_eq!(search.child.child.value, Some(expect));
  }
}

