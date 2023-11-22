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

use concept::*;
use kernel::*;
use search::branching::*;
use search::space::*;
use term::ops::*;
use term::*;

pub struct Brancher<Var, Val, D> {
    var_selector: Var,
    val_selector: Val,
    distributor: D,
}

impl<Var, Val, D> Brancher<Var, Val, D> {
    pub fn new(var_selector: Var, val_selector: Val, distributor: D) -> Self {
        Brancher {
            var_selector,
            val_selector,
            distributor,
        }
    }
}

impl<Var, Val, D, VStore, CStore, R, Domain, Bound> SearchTreeVisitor<Space<VStore, CStore, R>>
    for Brancher<Var, Val, D>
where
    VStore: VStoreConcept<Item = Domain, Location = Identity<Domain>, Output = Domain>,
    CStore: IntCStore<VStore>,
    R: FreezeSpace<VStore, CStore> + Snapshot<State = Space<VStore, CStore, R>>,
    Var: VarSelection<Space<VStore, CStore, R>>,
    Val: ValSelection<Domain>,
    Domain: IntDomain<Item = Bound>,
    Bound: IntBound,
    D: Distributor<Space<VStore, CStore, R>, Bound>,
{
    fn enter(
        &mut self,
        current: Space<VStore, CStore, R>,
    ) -> (
        <Space<VStore, CStore, R> as Freeze>::FrozenState,
        Status<Space<VStore, CStore, R>>,
    ) {
        let var_idx = self.var_selector.select(&current);

        let x = Identity::<Domain>::new(var_idx);
        let dom = x.read(&current.vstore);
        assert!(
            !dom.is_singleton() && !dom.is_empty(),
            "Can not distribute over assigned or failed variables."
        );
        let val = self.val_selector.select(dom);

        let (immutable_space, branches) = self.distributor.distribute(current, var_idx, val);
        (immutable_space, Status::Unknown(branches))
    }
}
