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

use gcollections::ops::*;
use kernel::*;
pub use search::recomputation::*;
use std::marker::PhantomData;
use trilean::SKleene;

pub struct Space<VStore, CStore, Restoration> {
    pub vstore: VStore,
    pub cstore: CStore,
    phantom_restoration: PhantomData<Restoration>,
}

impl<VStore, CStore, Restoration> Space<VStore, CStore, Restoration> {
    pub fn new(vstore: VStore, cstore: CStore) -> Self {
        Space {
            vstore,
            cstore,
            phantom_restoration: PhantomData,
        }
    }
}

impl<VStore, CStore, Restoration> Space<VStore, CStore, Restoration>
where
    CStore: Consistency<VStore>,
{
    pub fn consistency(&mut self) -> SKleene {
        self.cstore.consistency(&mut self.vstore)
    }
}

impl<VStore, CStore, Restoration> Empty for Space<VStore, CStore, Restoration>
where
    VStore: Empty,
    CStore: Empty,
{
    fn empty() -> Self {
        Space::new(VStore::empty(), CStore::empty())
    }
}

impl<VStore, CStore, Restoration> Freeze for Space<VStore, CStore, Restoration>
where
    VStore: Freeze,
    CStore: Freeze,
    Restoration: FreezeSpace<VStore, CStore> + Snapshot<State = Space<VStore, CStore, Restoration>>,
{
    type FrozenState = Restoration;
    fn freeze(self) -> Self::FrozenState {
        Restoration::freeze_space(self.vstore, self.cstore)
    }
}
