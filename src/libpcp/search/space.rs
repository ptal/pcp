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
use gcollections::ops::*;

pub struct Space<VStore, CStore> {
  pub vstore: VStore,
  pub cstore: CStore
}

impl<VStore, CStore> Space<VStore, CStore>
{
  pub fn new(vstore: VStore, cstore: CStore) -> Space<VStore, CStore> {
    Space {
      vstore: vstore,
      cstore: cstore
    }
  }
}

impl<VStore, CStore> Space<VStore, CStore> where
  CStore: Consistency<VStore>
{
  pub fn consistency(&mut self) -> Trilean {
    self.cstore.consistency(&mut self.vstore)
  }
}

impl<VStore, CStore> Empty for Space<VStore, CStore> where
  VStore: Empty,
  CStore: Empty
{
  fn empty() -> Space<VStore, CStore> {
    Space {
      vstore: VStore::empty(),
      cstore: CStore::empty()
    }
  }
}

impl<VStore, CStore> Freeze for Space<VStore, CStore> where
 VStore: Freeze,
 CStore: Freeze
{
  type ImmutableState = ImmutableStore<VStore, CStore>;
  fn freeze(self) -> Self::ImmutableState
  {
    ImmutableStore::new(self)
  }
}

pub struct ImmutableStore<VStore, CStore> where
 VStore: Freeze,
 CStore: Freeze
{
  immutable_vstore: VStore::ImmutableState,
  immutable_cstore: CStore::ImmutableState
}

impl<VStore, CStore> ImmutableStore<VStore, CStore> where
 VStore: Freeze,
 CStore: Freeze
{
  fn new(space: Space<VStore, CStore>) -> Self {
    ImmutableStore {
      immutable_vstore: space.vstore.freeze(),
      immutable_cstore: space.cstore.freeze()
    }
  }
}

impl<VStore, CStore> Snapshot for ImmutableStore<VStore, CStore> where
 VStore: Freeze,
 CStore: Freeze
{
  type Label = (
    <VStore::ImmutableState as Snapshot>::Label,
    <CStore::ImmutableState as Snapshot>::Label);
  type MutableState = Space<VStore, CStore>;

  fn label(&mut self) -> Self::Label {
    (self.immutable_vstore.label(), self.immutable_cstore.label())
  }

  fn restore(self, label: Self::Label) -> Self::MutableState {
    Space {
      vstore: self.immutable_vstore.restore(label.0),
      cstore: self.immutable_cstore.restore(label.1)
    }
  }
}
