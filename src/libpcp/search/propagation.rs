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
use kernel::Trilean::*;
use search::space::*;
use search::search_tree_visitor::*;

pub struct Propagation<C> {
  child: C
}

impl<C> Propagation<C> {
  pub fn new(child: C) -> Propagation<C> {
    Propagation {
      child: child
    }
  }
}

impl<VStore, CStore, C> SearchTreeVisitor<Space<VStore, CStore>> for Propagation<C> where
  VStore: Freeze,
  CStore: Freeze + Consistency<VStore>,
  C: SearchTreeVisitor<Space<VStore, CStore>>
{
  fn start(&mut self, root: &Space<VStore, CStore>) {
    self.child.start(root);
  }

  fn enter(&mut self, mut current: Space<VStore, CStore>)
    -> (<Space<VStore, CStore> as Freeze>::FrozenState, Status<Space<VStore, CStore>>)
  {
    let status = current.consistency();
    match status {
      True => (current.freeze(), Status::Satisfiable),
      False => (current.freeze(), Status::Unsatisfiable),
      Unknown => self.child.enter(current)
    }
  }
}
