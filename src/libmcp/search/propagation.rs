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
use solver::space::Space;
use solver::space::Status as SpaceStatus;

pub struct Propagation<C> {
  child: C
}

impl<S,C> SearchTreeVisitor<S> for Propagation<C> where
  S: Space,
  C: SearchTreeVisitor<S>
{
  fn start(&mut self, root: &S) {
    self.child.start(root);
  }

  fn enter(&mut self, mut current: S) -> (S, Status<S>) {
    let status = current.solve();
    match status {
      SpaceStatus::Satisfiable => (current, Status::Satisfiable),
      SpaceStatus::Unsatisfiable => (current, Status::Unsatisfiable),
      SpaceStatus::Unknown => self.child.enter(current)
    }
  }
}
