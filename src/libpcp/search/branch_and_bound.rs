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
use propagators::cmp::*;
use gcollections::*;
use concept::*;

pub enum Mode {
  Minimize,
  Maximize
}

pub struct BranchAndBound<VStore, C> where
 VStore: VStoreConcept,
 VStore::Item: Collection
{
  pub mode: Mode,
  pub var: Var<VStore>,
  pub value: Option<<VStore::Item as Collection>::Item>,
  pub child: C
}

impl<VStore, C> BranchAndBound<VStore, C> where
  VStore: VStoreConcept,
  VStore::Item: Collection
{
  pub fn new(mode: Mode, var: Var<VStore>, child: C) -> Self {
    BranchAndBound {
      mode: mode,
      var: var,
      value: None,
      child: child,
    }
  }
}

impl<C, Bound, Dom, VStore, CStore, R> SearchTreeVisitor<Space<VStore, CStore, R>> for
  BranchAndBound<VStore, C> where
 VStore: VStoreConcept<Item=Dom> + 'static,
 CStore: IntCStore<VStore>,
 Dom: IntDomain<Item=Bound> + 'static,
 Bound: IntBound + 'static,
 C: SearchTreeVisitor<Space<VStore, CStore, R>>,
 R: FreezeSpace<VStore, CStore> + Snapshot<State=Space<VStore, CStore, R>>
{
  fn start(&mut self, root: &Space<VStore, CStore, R>) {
    self.child.start(root);
  }

  fn enter(&mut self, mut current: Space<VStore, CStore, R>)
    -> (<Space<VStore, CStore, R> as Freeze>::FrozenState, Status<Space<VStore, CStore, R>>)
  {
    if let Some(bound) = self.value.clone() {
      let bound = Box::new(Constant::new(bound)) as Var<VStore>;
      match self.mode {
        Mode::Minimize => current.cstore.alloc(Box::new(XLessY::new(self.var.bclone(), bound.bclone()))),
        Mode::Maximize => current.cstore.alloc(Box::new(x_greater_y(self.var.bclone(), bound.bclone()))),
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

// #[cfg(test)]
// mod test {
//   use super::*;
//   use search::test::*;
//   use search::engine::all_solution::*;
//   use search::engine::one_solution::*;
//   use search::propagation::*;
//   use search::branching::binary_split::*;
//   use search::branching::brancher::*;
//   use search::branching::first_smallest_var::*;
//   use search::branching::middle_val::*;
//   use interval::interval_set::*;
//   use gcollections::VectorStack;
//   use gcollections::ops::*;

//   #[test]
//   fn simple_maximize_test() {
//     simple_optimization_test(Mode::Maximize, 9);
//   }

//   #[test]
//   fn simple_minimize_test() {
//     simple_optimization_test(Mode::Minimize, 0);
//   }

//   fn simple_optimization_test(mode: Mode, expect: i32) {
//     let mut space = FDSpace::empty();
//     let x = space.vstore.alloc((0,10).to_interval_set());
//     let y = space.vstore.alloc((0,10).to_interval_set());
//     space.cstore.alloc(Box::new(XLessY::new(x.clone(), y)));

//     let mut search: AllSolution<OneSolution<_, VectorStack<_>, FDSpace>>
//       = AllSolution::new(
//           OneSolution::new(
//             BranchAndBound::new(mode, x.clone(),
//               Propagation::new(Brancher::new(FirstSmallestVar, MiddleVal, BinarySplit)))));
//     search.start(&space);
//     let (_, status) = search.enter(space);
//     assert_eq!(status, EndOfSearch);
//     assert_eq!(search.child.child.value, Some(expect));
//   }
// }

