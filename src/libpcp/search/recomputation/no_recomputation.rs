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

use kernel::*;
use search::recomputation::ops::*;
use search::space::Space;

pub struct NoRecomputation<VStore, CStore>
where
    VStore: Freeze,
    CStore: Freeze,
{
    frozen_vstore: VStore::FrozenState,
    frozen_cstore: CStore::FrozenState,
}

impl<VStore, CStore> FreezeSpace<VStore, CStore> for NoRecomputation<VStore, CStore>
where
    VStore: Freeze,
    CStore: Freeze,
{
    fn freeze_space(vstore: VStore, cstore: CStore) -> Self {
        NoRecomputation {
            frozen_vstore: vstore.freeze(),
            frozen_cstore: cstore.freeze(),
        }
    }
}

impl<VStore, CStore> Snapshot for NoRecomputation<VStore, CStore>
where
    VStore: Freeze,
    CStore: Freeze,
{
    type Label = (
        <VStore::FrozenState as Snapshot>::Label,
        <CStore::FrozenState as Snapshot>::Label,
    );
    type State = Space<VStore, CStore, Self>;

    fn label(&mut self) -> Self::Label {
        (self.frozen_vstore.label(), self.frozen_cstore.label())
    }

    fn restore(self, label: Self::Label) -> Self::State {
        Space::new(
            self.frozen_vstore.restore(label.0),
            self.frozen_cstore.restore(label.1),
        )
    }
}
