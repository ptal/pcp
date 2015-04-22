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

use search::search_tree_visitor::*;
use search::search_tree_visitor::Status::*;
use search::branching::branch::*;
use search::engine::queue::*;
use search::engine::PartialExploration;
use solver::space::Space;

pub struct OneSolution<Q, C> {
  queue: Q,
  child: C,
  exploring: bool
}

impl<Q,C> PartialExploration for OneSolution<Q,C> {}

impl<Q, C> OneSolution<Q, C>
{
  pub fn new<S>(child: C) -> OneSolution<Q, C> where
    S: Space,
    Q: Queue<Branch<S>>,
    C: SearchTreeVisitor<S>
  {
    OneSolution {
      queue: Q::empty(),
      child: child,
      exploring: false
    }
  }

  fn push_branches<S>(&mut self, branches: Vec<Branch<S>>) where
    S: Space,
    Q: Queue<Branch<S>>
  {
    for branch in branches {
      self.queue.push(branch);
    }
  }

  fn enter_child<S>(&mut self, current: S, status: &mut Status<S>) -> S where
    S: Space,
    Q: Queue<Branch<S>>,
    C: SearchTreeVisitor<S>
  {
    let (current, child_status) = self.child.enter(current);
    match child_status {
      Unknown(branches) => self.push_branches(branches),
      Satisfiable => *status = Satisfiable,
      Pruned => *status = Pruned,
      _ => ()
    }
    current
  }

  // Only visit the root if we didn't visit it before (based on the queue emptiness).
  fn enter_root<S>(&mut self, root: S, status: &mut Status<S>) -> S where
    S: Space,
    Q: Queue<Branch<S>>,
    C: SearchTreeVisitor<S>
  {
    if self.queue.is_empty() && !self.exploring {
      self.exploring = true;
      self.enter_child(root, status)
    } else {
      root
    }
  }
}

impl<S, Q, C> SearchTreeVisitor<S> for OneSolution<Q, C> where
 S: Space,
 Q: Queue<Branch<S>>,
 C: SearchTreeVisitor<S>
{
  fn start(&mut self, root: &S) {
    self.queue = Q::empty();
    self.exploring = false;
    self.child.start(root);
  }

  fn enter(&mut self, root: S) -> (S, Status<S>) {
    let mut status = Unsatisfiable;
    let mut current = self.enter_root(root, &mut status);
    while !status.is_satisfiable() && !self.queue.is_empty() {
      let branch = self.queue.pop().unwrap();
      let child = branch.commit(current);
      current = self.enter_child(child, &mut status);
    }
    (current, status)
  }
}
