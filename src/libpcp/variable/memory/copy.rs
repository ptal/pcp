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
use variable::concept::*;
use variable::ops::*;
use gcollections::ops::constructor::*;
use gcollections::ops::cardinality::*;
use gcollections::ops::sequence::*;
use gcollections::ops::sequence::ordering::*;
use std::slice;
use std::ops::{Deref, DerefMut, Index};
use std::fmt::{Formatter, Display, Error};
use std::rc::*;
use std::mem;

pub struct CopyStore<Domain>
{
  variables: Vec<Domain>
}

impl<Domain> MemoryConcept<Domain> for CopyStore<Domain> where
 Domain: DomainConcept
{}

impl<Domain> ImmutableMemoryConcept<Domain> for CopyStore<Domain> where
 Domain: DomainConcept
{}

impl<Domain> CopyStore<Domain>
{
  fn restore(variables: Vec<Domain>) -> CopyStore<Domain> {
    CopyStore {
      variables: variables
    }
  }
}

impl<Domain> Deref for CopyStore<Domain>
{
  type Target = Vec<Domain>;
  fn deref(&self) -> &Self::Target {
    &self.variables
  }
}

impl<Domain> DerefMut for CopyStore<Domain>
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.variables
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
  type Item = Domain;

  fn iter<'a>(&'a self) -> slice::Iter<'a, Self::Item> {
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
  type ImmutableState = ImmutableCopyStore<Domain>;
  fn freeze(self) -> Self::ImmutableState
  {
    ImmutableCopyStore::new(self)
  }
}

pub struct ImmutableCopyStore<Domain>
{
  variables: Rc<Vec<Domain>>
}

impl<Domain> ImmutableCopyStore<Domain>
{
  fn new(store: CopyStore<Domain>) -> ImmutableCopyStore<Domain> {
    ImmutableCopyStore {
      variables: Rc::new(store.variables)
    }
  }
}

impl<Domain> Snapshot for ImmutableCopyStore<Domain> where
 Domain: Clone
{
  type Label = Rc<Vec<Domain>>;
  type MutableState = CopyStore<Domain>;

  fn label(&mut self) -> Self::Label {
    self.variables.clone()
  }

  fn restore(self, label: Self::Label) -> Self::MutableState {
    let variables = Rc::try_unwrap(label).unwrap_or_else(|l| l.deref().clone());
    CopyStore::restore(variables)
  }
}
