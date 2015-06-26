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
use std::default::Default;
use std::rc::*;
use std::ops::Deref;

pub struct Space<VStore, CStore> {
  pub vstore: VStore,
  pub cstore: CStore
}

impl<VStore, CStore> Space<VStore, CStore> where
  CStore: Consistency<VStore>
{
  pub fn consistency(&mut self) -> Trilean {
    self.cstore.consistency(&mut self.vstore)
  }
}

impl<VStore, CStore> State for Space<VStore, CStore> where
  VStore: State,
  CStore: State
{
  type Label = Rc<(VStore::Label, CStore::Label)>;

  fn mark(&self) -> Rc<(VStore::Label, CStore::Label)>
  {
    Rc::new((self.vstore.mark(), self.cstore.mark()))
  }

  fn restore(mut self, label: Rc<(VStore::Label, CStore::Label)>) -> Space<VStore, CStore>
  {
    let label = Rc::try_unwrap(label).unwrap_or_else(|l| l.deref().clone());
    self.vstore = self.vstore.restore(label.0);
    self.cstore = self.cstore.restore(label.1);
    self
  }
}

impl<VStore, CStore> Default for Space<VStore, CStore> where
  VStore: Default,
  CStore: Default
{
  fn default() -> Space<VStore, CStore> {
    Space {
      vstore: VStore::default(),
      cstore: CStore::default()
    }
  }
}
