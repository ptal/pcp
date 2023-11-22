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

use concept::*;
use gcollections::ops::*;
use kernel::*;
use model::*;
use search::search_tree_visitor::*;
use search::space::*;
use std::io::{self};

pub struct Debugger<C> {
    model: Model,
    child: C,
}

impl<C> Debugger<C> {
    pub fn new(model: Model, child: C) -> Debugger<C> {
        Debugger { model, child }
    }
}

impl<VStore, CStore, Domain, R, C> SearchTreeVisitor<Space<VStore, CStore, R>> for Debugger<C>
where
    VStore: VStoreConcept<Item = Domain> + Clone,
    CStore: IntCStore<VStore> + DisplayStateful<(Model, VStore)>,
    C: SearchTreeVisitor<Space<VStore, CStore, R>>,
    Domain: IsSingleton,
    R: FreezeSpace<VStore, CStore> + Snapshot<State = Space<VStore, CStore, R>>,
{
    fn start(&mut self, root: &Space<VStore, CStore, R>) {
        self.child.start(root);
    }

    fn enter(
        &mut self,
        current: Space<VStore, CStore, R>,
    ) -> (
        <Space<VStore, CStore, R> as Freeze>::FrozenState,
        Status<Space<VStore, CStore, R>>,
    ) {
        let (frozen, status) = self.child.enter(current);
        let current = frozen.unfreeze();
        println!("Variable store:");
        println!("  Number of variables: {}", current.vstore.size());
        println!(
            "  Number of variables assigned: {}",
            current.vstore.iter().filter(|v| v.is_singleton()).count()
        );
        current.vstore.display(&self.model);
        println!("Constraint store:");
        current
            .cstore
            .display(&(self.model.clone(), current.vstore.clone()));
        println!("Status {:?}", status);
        println!("Press enter to continue...");
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        (current.freeze(), status)
    }
}
