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
use gcollections::kind::*;
use kernel::*;
use model::*;
use propagation::events::*;
use term::ops::*;

#[derive(Debug)]
pub struct Sum<VStore> {
    vars: Vec<Var<VStore>>,
}

impl<VStore> Sum<VStore> {
    pub fn new(vars: Vec<Var<VStore>>) -> Self {
        Sum { vars }
    }
}

impl<VStore> Clone for Sum<VStore>
where
    VStore: Collection,
{
    fn clone(&self) -> Self {
        Sum::new(self.vars.iter().map(|v| v.bclone()).collect())
    }
}

impl<VStore> DisplayStateful<Model> for Sum<VStore> {
    fn display(&self, model: &Model) {
        print!("sum(");
        self.vars[0].display(model);
        let mut i = 1;
        while i < self.vars.len() {
            print!(" + ");
            self.vars[i].display(model);
            i += 1;
        }
        print!(")");
    }
}

impl<VStore, Domain, Bound> StoreMonotonicUpdate<VStore> for Sum<VStore>
where
    VStore: VStoreConcept<Item = Domain>,
    Domain: IntDomain<Item = Bound>,
    Bound: IntBound,
{
    fn update(&mut self, store: &mut VStore, value: Domain) -> bool {
        if self.vars.len() == 1 {
            self.vars[0].update(store, value)
        } else {
            let sum = self.read(store);
            sum.overlap(&value)
        }
    }
}

impl<VStore, Domain, Bound> StoreRead<VStore> for Sum<VStore>
where
    VStore: VStoreConcept<Item = Domain>,
    Domain: IntDomain<Item = Bound>,
    Bound: IntBound,
{
    fn read(&self, store: &VStore) -> Domain {
        let mut iter = self.vars.iter();
        let sum = iter.next().expect("At least one variable in sum.");
        iter.fold(sum.read(store), |a: Domain, v| a + v.read(store))
    }
}

impl<VStore> ViewDependencies<FDEvent> for Sum<VStore> {
    fn dependencies(&self, event: FDEvent) -> Vec<(usize, FDEvent)> {
        self.vars
            .iter()
            .flat_map(|v| v.dependencies(event.clone()))
            .collect()
    }
}
