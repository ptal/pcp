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
use model::*;
use term::ops::*;
use variable::ops::*;
use gcollections::kind::*;
use std::marker::PhantomData;
use std::ops::Index;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Identity<Domain> {
  idx: usize,
  phantom: PhantomData<Domain>
}

impl<Domain> Identity<Domain> {
  pub fn new(idx: usize) -> Identity<Domain> {
    Identity {
      idx: idx,
      phantom: PhantomData
    }
  }

  pub fn index(&self) -> usize {
    self.idx
  }
}

impl<Domain> DisplayStateful<Model> for Identity<Domain>
{
  fn display(&self, model: &Model) {
    print!("{}", model.var_name(self.idx));
  }
}

impl<Domain, Store> StoreMonotonicUpdate<Store> for Identity<Domain> where
  Store: MonotonicUpdate,
  Store: AssociativeCollection<Location=Identity<Domain>>
{
  fn update(&mut self, store: &mut Store, value: Store::Item) -> bool {
    store.update(self, value)
  }
}

impl<Domain, Store> StoreRead<Store> for Identity<Domain> where
  Store: Collection<Item=Domain>,
  Store: Index<usize, Output=Domain>,
  Domain: Clone
{
  fn read(&self, store: &Store) -> Store::Item {
    store[self.idx].clone()
  }
}

impl<Domain, Event> ViewDependencies<Event> for Identity<Domain>
{
  fn dependencies(&self, event: Event) -> Vec<(usize, Event)> {
    vec![(self.idx, event)]
  }
}

#[cfg(test)]
mod test {
  use gcollections::ops::*;
  use variable::VStoreFD;
  use term::ops::*;
  use interval::interval::*;

  type VStore = VStoreFD;

  #[test]
  fn identity_read_update() {
    let dom0_10 = (0,10).to_interval();
    let dom0_5 = (0,5).to_interval();
    let mut store = VStore::empty();
    let mut v = store.alloc(dom0_10);

    assert_eq!(v.read(&store), dom0_10);
    assert_eq!(v.update(&mut store, dom0_5), true);
    assert_eq!(v.read(&store), dom0_5);
  }
}
