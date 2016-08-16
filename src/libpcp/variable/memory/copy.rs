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

pub struct CopyMemory<Domain>
{
  variables: Vec<Domain>
}

impl<Domain> MemoryConcept<Domain> for CopyMemory<Domain> where
 Domain: DomainConcept
{}

impl<Domain> ImmutableMemoryConcept<Domain> for CopyMemory<Domain> where
 Domain: DomainConcept
{}

impl<Domain> CopyMemory<Domain>
{
  fn restore(variables: Vec<Domain>) -> CopyMemory<Domain> {
    CopyMemory {
      variables: variables
    }
  }
}

impl<Domain> Deref for CopyMemory<Domain>
{
  type Target = Vec<Domain>;
  fn deref(&self) -> &Self::Target {
    &self.variables
  }
}

impl<Domain> DerefMut for CopyMemory<Domain>
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.variables
  }
}

impl<Domain> Empty for CopyMemory<Domain>
{
  fn empty() -> CopyMemory<Domain> {
    CopyMemory {
      variables: vec![]
    }
  }
}

impl<Domain> Cardinality for CopyMemory<Domain>
{
  type Size = usize;

  fn size(&self) -> usize {
    self.variables.len()
  }
}

impl<Domain> Iterable for CopyMemory<Domain>
{
  type Item = Domain;

  fn iter<'a>(&'a self) -> slice::Iter<'a, Self::Item> {
    self.variables.iter()
  }
}

impl<Domain> Push<Back, Domain> for CopyMemory<Domain>
{
  fn push(&mut self, value: Domain) {
    self.variables.push(value);
  }
}

impl<Domain> Update<usize, Domain> for CopyMemory<Domain>
{
  fn update(&mut self, key: usize, mut dom: Domain) -> Option<Domain> {
    mem::swap(&mut dom, &mut self.variables[key]);
    Some(dom)
  }
}

impl<Domain> Index<usize> for CopyMemory<Domain>
{
  type Output = Domain;
  fn index<'a>(&'a self, index: usize) -> &'a Domain {
    &self.variables[index]
  }
}

impl<Domain> Display for CopyMemory<Domain> where
 Domain: Display
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    for v in &self.variables {
      try!(formatter.write_fmt(format_args!("{} ", v)));
    }
    Ok(())
  }
}

impl<Domain> Freeze for CopyMemory<Domain> where
 Domain: Clone
{
  type FrozenState = FrozenCopyMemory<Domain>;
  fn freeze(self) -> Self::FrozenState
  {
    FrozenCopyMemory::new(self)
  }
}

pub struct FrozenCopyMemory<Domain>
{
  variables: Rc<Vec<Domain>>
}

impl<Domain> FrozenCopyMemory<Domain>
{
  fn new(store: CopyMemory<Domain>) -> FrozenCopyMemory<Domain> {
    FrozenCopyMemory {
      variables: Rc::new(store.variables)
    }
  }
}

impl<Domain> Snapshot for FrozenCopyMemory<Domain> where
 Domain: Clone
{
  type Label = Rc<Vec<Domain>>;
  type State = CopyMemory<Domain>;

  fn label(&mut self) -> Self::Label {
    self.variables.clone()
  }

  fn restore(self, label: Self::Label) -> Self::State {
    let variables = Rc::try_unwrap(label).unwrap_or_else(|l| l.deref().clone());
    CopyMemory::restore(variables)
  }
}
