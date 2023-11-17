// Copyright 2017 Pierre Talbot (IRCAM)

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
use gcollections::kind::*;
use gcollections::ops::*;
use interval::ops::Range;
use kernel::*;
use logic::{BooleanNeg, NotFormula};
use model::*;
use num::traits::Num;
use propagation::events::*;
use propagation::*;
use std::fmt::{Debug, Formatter, Result};
use term::ops::*;
use trilean::SKleene;

pub struct Boolean<VStore> {
    var: Var<VStore>,
}

impl<VStore, Domain, Bound> Boolean<VStore>
where
    VStore: VStoreConcept<Item = Domain>,
    Domain: Range + Collection<Item = Bound> + Clone + Debug + 'static,
    Bound: Num,
{
    pub fn new(vstore: &mut VStore) -> Self {
        let v = vstore.alloc(Domain::new(Bound::zero(), Bound::one()));
        Boolean {
            var: Box::new(v) as Var<VStore>,
        }
    }
}

impl<VStore> Debug for Boolean<VStore> {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        fmt.debug_struct("Boolean").field("var", &self.var).finish()
    }
}

impl<VStore> Clone for Boolean<VStore>
where
    VStore: Collection,
{
    fn clone(&self) -> Self {
        Boolean {
            var: self.var.bclone(),
        }
    }
}

impl<VStore> DisplayStateful<Model> for Boolean<VStore> {
    fn display(&self, model: &Model) {
        self.var.display(model);
    }
}

impl<VStore, Domain, Bound> NotFormula<VStore> for Boolean<VStore>
where
    VStore: VStoreConcept<Item = Domain> + 'static,
    Domain: IntDomain<Item = Bound> + 'static,
    Bound: IntBound + 'static,
{
    fn not(&self) -> Formula<VStore> {
        Box::new(BooleanNeg::new(self.clone()))
    }
}

impl<VStore> StoreMonotonicUpdate<VStore> for Boolean<VStore>
where
    VStore: VStoreConcept,
{
    fn update(&mut self, store: &mut VStore, value: VStore::Item) -> bool {
        self.var.update(store, value)
    }
}

impl<VStore> StoreRead<VStore> for Boolean<VStore>
where
    VStore: VStoreConcept,
{
    fn read(&self, store: &VStore) -> VStore::Item {
        self.var.read(store)
    }
}

impl<VStore> ViewDependencies<FDEvent> for Boolean<VStore> {
    fn dependencies(&self, event: FDEvent) -> Vec<(usize, FDEvent)> {
        self.var.dependencies(event)
    }
}

impl<VStore, Dom, Bound> Subsumption<VStore> for Boolean<VStore>
where
    VStore: Collection<Item = Dom>,
    Dom: Bounded<Item = Bound> + IsSingleton,
    Bound: Num,
{
    fn is_subsumed(&self, store: &VStore) -> SKleene {
        use trilean::SKleene::*;
        let x = self.var.read(store);
        if x.is_singleton() {
            if x.lower() == Bound::one() {
                True
            } else {
                False
            }
        } else {
            Unknown
        }
    }
}

impl<VStore, Dom, Bound> Propagator<VStore> for Boolean<VStore>
where
    VStore: VStoreConcept<Item = Dom>,
    Dom: Collection<Item = Bound> + Singleton,
    Bound: Num,
{
    fn propagate(&mut self, vstore: &mut VStore) -> bool {
        self.update(vstore, Dom::singleton(Bound::one()))
    }
}

impl<VStore> PropagatorDependencies<FDEvent> for Boolean<VStore> {
    fn dependencies(&self) -> Vec<(usize, FDEvent)> {
        self.var.dependencies(FDEvent::Bound)
    }
}
