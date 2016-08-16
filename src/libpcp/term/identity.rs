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
use term::ExprInference;
use std::marker::PhantomData;
use std::fmt::{Formatter, Debug, Error};
use std::ops::Index;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Identity<Domain> {
  idx: usize,
  phantom: PhantomData<Domain>
}

impl<Domain> ExprInference for Identity<Domain> {
  type Output = Domain;
}

impl<Domain> Identity<Domain> {
  pub fn new(idx: usize) -> Identity<Domain> {
    Identity {
      idx: idx,
      phantom: PhantomData
    }
  }
}

impl<Domain> Debug for Identity<Domain>
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("_{}", self.idx))
  }
}

impl<Domain> VarIndex for Identity<Domain> {
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
  Store: Index<usize, Output=Domain>,
  Domain: Clone
{
  type Value = Domain;
  fn read(&self, store: &Store) -> Domain {
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
  use kernel::Alloc;
  use variable::store::*;
  use variable::memory::*;
  use variable::ops::*;
  use interval::interval::*;

  type Domain = Interval<i32>;

  #[test]
  fn identity_read_update() {
    let dom0_10 = (0,10).to_interval();
    let dom0_5 = (0,5).to_interval();
    let mut store: Store<CopyMemory<Domain>, Domain> = Store::empty();
    let v = store.alloc(dom0_10);

    assert_eq!(v.read(&store), dom0_10);
    assert_eq!(v.update(&mut store, dom0_5), true);
    assert_eq!(v.read(&store), dom0_5);
  }
}
