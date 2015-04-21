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
use solver::space::Space;

pub struct Engine<Q, C> {
  queue: Q,
  child: C
}

impl<Q, C> Engine<Q, C>
{
  pub fn new<S>(child: C) -> Engine<Q, C> where
    S: Space,
    Q: Queue<Branch<S>>,
    C: SearchTreeVisitor<S>
  {
    Engine {
      queue: Q::empty(),
      child: child
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

  fn enter_child<S>(&mut self, current: S) -> S where
    S: Space,
    Q: Queue<Branch<S>>,
    C: SearchTreeVisitor<S>
  {
    let (current, status) = self.child.enter(current);
    if let Unknown(branches) = status {
      self.push_branches(branches);
    }
    current
  }
}

impl<S, Q, C> SearchTreeVisitor<S> for Engine<Q, C> where
 S: Space,
 Q: Queue<Branch<S>>,
 C: SearchTreeVisitor<S>
{
  fn start(&mut self, root: &S) {
    self.queue = Q::empty();
    self.child.start(root);
  }

  fn enter(&mut self, root: S) -> (S, Status<S>) {
    let mut current = self.enter_child(root);
    while let Some(branch) = self.queue.pop() {
      current = branch.commit(current);
      current = self.enter_child(current);
    }
    (current, Pruned)
  }
}
