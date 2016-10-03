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

use kernel::*;
use search::search_tree_visitor::*;
use search::search_tree_visitor::Status::*;
use search::branching::branch::*;
use gcollections::ops::multiset::*;
use std::marker::PhantomData;

pub struct OneSolution<C, Q, Space> {
  child: C,
  queue: Q,
  exploring: bool,
  phantom_space: PhantomData<Space>
}

impl<C, Q, Space> OneSolution<C, Q, Space> where
 Space: Freeze,
 C: SearchTreeVisitor<Space>,
 Q: Multiset<Branch<Space>>
{
  pub fn new(child: C) -> OneSolution<C, Q, Space>
  {
    OneSolution {
      queue: Q::empty(),
      child: child,
      exploring: false,
      phantom_space: PhantomData
    }
  }

  fn push_branches(&mut self, branches: Vec<Branch<Space>>)
  {
    for branch in branches {
      self.queue.insert(branch);
    }
  }

  fn enter_child(&mut self, current: Space, status: &mut Status<Space>) -> Space::FrozenState
  {
    let (immutable_state, child_status) = self.child.enter(current);
    match child_status {
      Unknown(ref branches) if branches.is_empty() => *status = Status::pruned(),
      Unknown(branches) => self.push_branches(branches),
      Satisfiable => *status = Satisfiable,
      _ => ()
    }
    immutable_state
  }

  // Only visit the root if we didn't visit it before (based on the queue emptiness).
  fn enter_root(&mut self, root: Space, status: &mut Status<Space>) -> Space::FrozenState
  {
    if self.queue.is_empty() && !self.exploring {
      self.exploring = true;
      self.enter_child(root, status)
    } else {
      root.freeze()
    }
  }
}

impl<C, Q, Space> SearchTreeVisitor<Space> for OneSolution<C, Q, Space> where
 Space: Freeze,
 C: SearchTreeVisitor<Space>,
 Q: Multiset<Branch<Space>>
{
  fn start(&mut self, root: &Space) {
    self.queue = Q::empty();
    self.exploring = false;
    self.child.start(root);
  }

  fn enter(&mut self, root: Space) -> (Space::FrozenState, Status<Space>) {
    let mut status = Unsatisfiable;
    let mut immutable_state = self.enter_root(root, &mut status);
    while status != Satisfiable && !self.queue.is_empty() {
      let branch = self.queue.extract().unwrap();
      let child = branch.commit(immutable_state);
      immutable_state = self.enter_child(child, &mut status);
    }
    (immutable_state, status)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::*;
  use variable::VStoreFD;
  use propagation::CStoreFD;
  use propagators::cmp::*;
  use propagators::distinct::*;
  use term::*;
  use search::search_tree_visitor::*;
  use search::search_tree_visitor::Status::*;
  use search::space::*;
  use search::propagation::*;
  use search::branching::binary_split::*;
  use search::branching::brancher::*;
  use search::branching::first_smallest_var::*;
  use interval::interval::*;
  use gcollections::VectorStack;
  use gcollections::ops::*;
  use test::Bencher;

  type Domain = Interval<i32>;
  type VStore = VStoreFD;
  type CStore = CStoreFD<VStore>;
  type FDSpace = Space<VStore, CStore>;

  #[test]
  fn example_nqueens() {
    nqueens(1, Satisfiable);
    nqueens(2, Unsatisfiable);
    nqueens(3, Unsatisfiable);
    for i in 4..12 {
      nqueens(i, Satisfiable);
    }
  }

  #[bench]
  fn bench_nqueens10(b: &mut Bencher) {
    b.iter(|| {
        nqueens(10, Satisfiable)
    });
  }

  fn nqueens(n: usize, expect: Status<FDSpace>) {
    let mut space = FDSpace::empty();
    let mut queens = vec![];
    // 2 queens can't share the same line.
    for _ in 0..n {
      queens.push(space.vstore.alloc((1, n as i32).to_interval()));
    }
    for i in 0..n-1 {
      for j in i + 1..n {
        // 2 queens can't share the same diagonal.
        let q1 = (i + 1) as i32;
        let q2 = (j + 1) as i32;
        // Xi + i != Xj + j
        space.cstore.alloc(XNeqY::new(queens[i].clone(), Addition::new(queens[j].clone(), q2 - q1)));
        // Xi - i != Xj - j
        space.cstore.alloc(XNeqY::new(queens[i].clone(), Addition::new(queens[j].clone(), -q2 + q1)));
      }
    }
    // 2 queens can't share the same column.
    space.cstore.alloc(Distinct::new(queens));

    let mut search: OneSolution<_, VectorStack<_>, FDSpace> = OneSolution::new(Propagation::new(Brancher::new(FirstSmallestVar, BinarySplit)));
    search.start(&space);
    let (_, status) = search.enter(space);
    assert_eq!(status, expect);
  }
}
