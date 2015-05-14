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

use variable::ops::*;
use std::marker::PhantomData;

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
}

impl<Domain> VarIndex for Identity<Domain>
{
  fn index(&self) -> usize {
    self.idx
  }
}

impl<Domain, Store> StoreMonotonicUpdate<Store, Domain> for Identity<Domain> where
  Store: MonotonicUpdate<usize, Domain>
{
  fn update(&self, store: &mut Store, value: Domain) -> bool {
    store.update(self.idx, value)
  }
}

impl<Domain, Store> StoreRead<Store> for Identity<Domain> where
  Store: Read<usize, Value=Domain>
{
  type Value = Domain;
  fn read(&self, store: &Store) -> Domain {
    store.read(self.idx)
  }
}


#[cfg(test)]
mod test {
  use super::*;
  use variable::store::*;
  use variable::ops::*;
  use interval::interval::*;

  #[test]
  fn identity_read_update() {
    let dom0_10 = (0,10).to_interval();
    let dom0_5 = (0,5).to_interval();
    let mut store: Store<Interval<i32>> = Store::new();
    let x = store.assign(dom0_10);
    let v: Identity<Interval<i32>> = Identity::new(x);

    assert_eq!(v.read(&store), dom0_10);
    assert_eq!(v.update(&mut store, dom0_5), true);
    assert_eq!(v.read(&store), dom0_5);
  }
}
