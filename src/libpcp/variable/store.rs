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
use variable::ops::*;
use variable::memory::*;
use term::identity::*;
use gcollections::ops::*;
use std::marker::PhantomData;
use std::slice;
use std::fmt::{Formatter, Display, Error};
use std::ops::Index;

#[derive(Clone)]
pub struct Store<Memory, Domain> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  variables: Memory,
  phantom: PhantomData<Domain>
}

impl<Memory, Domain> Empty for Store<Memory, Domain> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  fn empty() -> Store<Memory, Domain> {
    Store {
      variables: Memory::empty(),
      phantom: PhantomData
    }
  }
}

impl<Memory, Domain> State for Store<Memory, Domain> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  type Label = Store<Memory, Domain>;

  fn mark(&self) -> Store<Memory, Domain> {
    self.clone()
  }

  fn restore(self, label: Store<Memory, Domain>) -> Self {
    label
  }
}

impl<Memory, Domain> Cardinality for Store<Memory, Domain> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  type Size = usize;

  fn size(&self) -> usize {
    self.variables.size()
  }
}

impl<Memory, Domain> Iterable for Store<Memory, Domain> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  type Item = Domain;

  fn iter<'a>(&'a self) -> slice::Iter<'a, Self::Item> {
    self.variables.iter()
  }
}

impl<Memory, Domain> Alloc<Domain> for Store<Memory, Domain> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  type Location = Identity<Domain>;

  fn alloc(&mut self, dom: Domain) -> Identity<Domain> {
    assert!(!dom.is_empty());
    let var_idx = self.variables.size();
    self.variables.push(dom);
    Identity::new(var_idx)
  }
}

impl<Memory, Domain> Update<usize, Domain> for Store<Memory, Domain> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  fn update(&mut self, key: usize, dom: Domain) -> Option<Domain> {
    assert!(dom.is_subset(&self.variables[key]), "Domain update must be monotonic.");
    if dom.is_empty() {
      None
    }
    else {
      self.variables.update(key, dom)
    }
  }
}

impl<Memory, Domain> Index<usize> for Store<Memory, Domain> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  type Output = Domain;
  fn index<'a>(&'a self, index: usize) -> &'a Domain {
    assert!(index < self.variables.size(),
      "Variable not registered in the store. Variable index must be obtained with `alloc`.");
    &self.variables[index]
  }
}

impl<Memory, Domain> Display for Store<Memory, Domain> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    self.variables.fmt(formatter)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::Alloc;
  use variable::ops::*;
  use variable::memory::*;
  use term::identity::*;
  use interval::interval::*;
  use gcollections::ops::*;

  type Domain = Interval<i32>;
  type CopyFDStore = Store<CopyStore<Domain>, Domain>;

  #[test]
  fn ordered_assign_10_vars() {
    let dom0_10 = (0, 10).to_interval();
    let mut store: CopyFDStore = Store::empty();

    for i in 0..10 {
      assert_eq!(store.alloc(dom0_10), Identity::new(i));
    }
  }

  #[test]
  fn valid_read_update() {
    let dom0_10 = (0, 10).to_interval();
    let dom5_5 = (5, 5).to_interval();
    let mut store: CopyFDStore = Store::empty();

    let vars: Vec<_> = (0..10).map(|_| store.alloc(dom0_10)).collect();
    for var in vars {
      assert_eq!(var.read(&store), dom0_10);
      assert_eq!(var.update(&mut store, dom5_5), true);
      assert_eq!(var.read(&store), dom5_5);
    }
  }

  #[test]
  fn empty_update() {
    let mut store: CopyFDStore = Store::empty();
    let dom5_5 = (5, 5).to_interval();

    let var = store.alloc(dom5_5);
    assert_eq!(var.update(&mut store, Interval::empty()), false);
  }

  #[test]
  #[should_panic]
  fn empty_assign() {
    let mut store: CopyFDStore = Store::empty();
    store.alloc(Interval::<i32>::empty());
  }

  #[test]
  #[should_panic]
  fn non_monotonic_update_singleton() {
    let dom0_10 = (0,10).to_interval();
    let dom11_11 = 11.to_interval();

    let mut store: CopyFDStore = Store::empty();
    let var = store.alloc(dom0_10);
    var.update(&mut store, dom11_11);
  }

  #[test]
  #[should_panic]
  fn non_monotonic_update_widen() {
    let dom0_10 = (0,10).to_interval();
    let domm5_15 = (-5, 15).to_interval();

    let mut store: CopyFDStore = Store::empty();
    let var = store.alloc(dom0_10);
    var.update(&mut store, domm5_15);
  }
}