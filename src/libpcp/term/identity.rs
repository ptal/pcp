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
use model::*;
use propagation::events::*;
use std::marker::PhantomData;
use term::ops::*;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct Identity<Domain> {
    idx: usize,
    phantom: PhantomData<Domain>,
}

impl<Domain> Identity<Domain> {
    pub fn new(idx: usize) -> Identity<Domain> {
        Identity {
            idx,
            phantom: PhantomData,
        }
    }

    pub fn index(&self) -> usize {
        self.idx
    }
}

impl<Domain> DisplayStateful<Model> for Identity<Domain> {
    fn display(&self, model: &Model) {
        print!("{}", model.var_name(self.idx));
    }
}

impl<VStore, Domain> StoreMonotonicUpdate<VStore> for Identity<Domain>
where
    VStore: VStoreConcept<Item = Domain>,
{
    fn update(&mut self, store: &mut VStore, value: VStore::Item) -> bool {
        store.update(self, value)
    }
}

impl<VStore, Domain> StoreRead<VStore> for Identity<Domain>
where
    VStore: VStoreConcept<Item = Domain>,
    Domain: Clone,
{
    fn read(&self, store: &VStore) -> VStore::Item {
        store[self.idx].clone()
    }
}

impl<Domain> ViewDependencies<FDEvent> for Identity<Domain> {
    fn dependencies(&self, event: FDEvent) -> Vec<(usize, FDEvent)> {
        vec![(self.idx, event)]
    }
}

#[cfg(test)]
mod test {
    use gcollections::ops::*;
    use interval::interval::*;
    use term::ops::*;
    use variable::VStoreFD;

    type VStore = VStoreFD;

    #[test]
    fn identity_read_update() {
        let dom0_10 = (0, 10).to_interval();
        let dom0_5 = (0, 5).to_interval();
        let mut store = VStore::empty();
        let mut v = store.alloc(dom0_10);

        assert_eq!(v.read(&store), dom0_10);
        assert_eq!(v.update(&mut store, dom0_5), true);
        assert_eq!(v.read(&store), dom0_5);
    }
}
