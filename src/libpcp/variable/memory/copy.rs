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

use variable::ops::*;
use gcollections::ops::constructor::*;
use gcollections::ops::cardinality::*;
use gcollections::ops::sequence::*;
use gcollections::ops::sequence::ordering::*;
use std::slice;
use std::ops::{Deref, Index};
use std::fmt::{Formatter, Display, Error};
use std::rc::*;
use std::mem;

#[derive(Clone)]
pub struct CopyStore<Domain>
{
  variables: Vec<Domain>
}

impl<Domain> CopyStore<Domain>
{
  pub fn new() -> CopyStore<Domain> {
    CopyStore::empty()
  }

  pub fn len(&self) -> usize {
    self.size()
  }

  fn restore(variables: Vec<Domain>) -> CopyStore<Domain> {
    CopyStore {
      variables: variables
    }
  }
}

impl<Domain> Empty for CopyStore<Domain>
{
  fn empty() -> CopyStore<Domain> {
    CopyStore {
      variables: vec![]
    }
  }
}

impl<Domain> Cardinality for CopyStore<Domain>
{
  type Size = usize;

  fn size(&self) -> usize {
    self.variables.len()
  }
}

impl<Domain> Iterable for CopyStore<Domain>
{
  type Value = Domain;

  fn iter<'a>(&'a self) -> slice::Iter<'a, Domain> {
    self.variables.iter()
  }
}

impl<'a, Domain> IntoIterator for &'a CopyStore<Domain>
{
  type Item = &'a Domain;
  type IntoIter = ::std::slice::Iter<'a, Domain>;

  fn into_iter(self) -> Self::IntoIter {
    self.variables.iter()
  }
}

impl<Domain> Push<Back, Domain> for CopyStore<Domain>
{
  fn push(&mut self, value: Domain) {
    self.variables.push(value);
  }
}

impl<Domain> Update<usize, Domain> for CopyStore<Domain>
{
  fn update(&mut self, key: usize, mut dom: Domain) -> Option<Domain> {
    mem::swap(&mut dom, &mut self.variables[key]);
    Some(dom)
  }
}

impl<Domain> Index<usize> for CopyStore<Domain>
{
  type Output = Domain;
  fn index<'a>(&'a self, index: usize) -> &'a Domain {
    &self.variables[index]
  }
}

impl<Domain> Display for CopyStore<Domain> where
 Domain: Display
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    for v in &self.variables {
      try!(formatter.write_fmt(format_args!("{} ", v)));
    }
    Ok(())
  }
}

impl<Domain> Freeze for CopyStore<Domain> where
 Domain: Clone
{
  type FrozenState = FrozenCopyStore<Domain>;
  fn freeze(self) -> FrozenCopyStore<Domain>
  {
    FrozenCopyStore::new(self)
  }
}

pub struct FrozenCopyStore<Domain>
{
  variables: Rc<Vec<Domain>>
}

impl<Domain> FrozenCopyStore<Domain>
{
  fn new(store: CopyStore<Domain>) -> FrozenCopyStore<Domain> {
    FrozenCopyStore {
      variables: Rc::new(store.variables)
    }
  }
}

impl<Domain> Snapshot for FrozenCopyStore<Domain> where
 Domain: Clone
{
  type Label = Rc<Vec<Domain>>;
  type UnfrozenState = CopyStore<Domain>;

  fn snapshot(&mut self) -> Rc<Vec<Domain>> {
    self.variables.clone()
  }

  fn restore(self, label: Rc<Vec<Domain>>) -> CopyStore<Domain> {
    let variables = Rc::try_unwrap(label).unwrap_or_else(|l| l.deref().clone());
    CopyStore::restore(variables)
  }
}

// #[cfg(test)]
// mod test {
//   use super::*;
//   use variable::memory::ops::*;
//   use std::ops::Deref;

//   fn make_frozen_store(initial_data: Vec<u32>) -> FrozenCopyStore<u32> {
//     let mut copy_store = CopyStore::new();
//     copy_store.extend(initial_data);
//     copy_store.freeze()
//   }

//   #[test]
//   fn save_and_restore_identity() {
//     let initial_data = vec![1,2,3];
//     let mut frozen = make_frozen_store(initial_data.clone());
//     let snapshots: Vec<_> = (1..10).map(|_| frozen.snapshot()).collect();

//     for snapshot in snapshots {
//       let copy_store = frozen.restore(snapshot);
//       assert_eq!(initial_data.clone(), copy_store.deref().clone());
//       frozen = copy_store.freeze();
//     }
//   }
// }
