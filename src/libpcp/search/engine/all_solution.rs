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
use search::engine::PartialExploration;
use kernel::Space;

pub struct AllSolution<C> {
  child: C
}

impl<C> AllSolution<C>
{
  pub fn new<S>(child: C) -> AllSolution<C> where
    S: Space,
    C: SearchTreeVisitor<S>
  {
    AllSolution {
      child: child
    }
  }
}

impl<S, C> SearchTreeVisitor<S> for AllSolution<C> where
 S: Space,
 C: SearchTreeVisitor<S> + PartialExploration
{
  fn start(&mut self, root: &S) {
    self.child.start(root);
  }

  fn enter(&mut self, root: S) -> (S, Status<S>) {
    let mut status_sum = Unsatisfiable;
    let mut status = Satisfiable;
    let mut current = root;
    while status == Satisfiable {
      let child = self.child.enter(current);
      current = child.0;
      status = child.1;
      status_sum = status_sum.or(&status);
    }
    (current, status_sum)
  }
}
