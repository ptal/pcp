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

use concept::*;
use kernel::*;
use search::search_tree_visitor::*;
use search::space::*;
use trilean::SKleene::*;

pub struct Propagation<C> {
    child: C,
}

impl<C> Propagation<C> {
    pub fn new(child: C) -> Propagation<C> {
        Propagation { child }
    }
}

impl<VStore, CStore, R, C> SearchTreeVisitor<Space<VStore, CStore, R>> for Propagation<C>
where
    VStore: VStoreConcept,
    CStore: IntCStore<VStore>,
    C: SearchTreeVisitor<Space<VStore, CStore, R>>,
    R: FreezeSpace<VStore, CStore> + Snapshot<State = Space<VStore, CStore, R>>,
{
    fn start(&mut self, root: &Space<VStore, CStore, R>) {
        self.child.start(root);
    }

    fn enter(
        &mut self,
        mut current: Space<VStore, CStore, R>,
    ) -> (
        <Space<VStore, CStore, R> as Freeze>::FrozenState,
        Status<Space<VStore, CStore, R>>,
    ) {
        let status = current.consistency();
        match status {
            True => (current.freeze(), Status::Satisfiable),
            False => (current.freeze(), Status::Unsatisfiable),
            Unknown => self.child.enter(current),
        }
    }
}
