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
use interval::ncollections::ops::*;
use std::fmt::{Formatter, Display, Error};
use std::ops::Deref;

#[derive(Copy, PartialEq, Eq, Debug, Clone)]
pub struct Variable<Domain> {
  idx: usize,
  dom: Domain
}

impl<Domain> Deref for Variable<Domain> {
  type Target = Domain;

  fn deref<'a>(&'a self) -> &'a Domain {
    &self.dom
  }
}

impl<Domain: Display> Display for Variable<Domain> {
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("({}, {})", self.idx, self.dom))
  }
}

impl<Domain> Variable<Domain> where
  Domain: Cardinality
{
  pub fn new(idx: usize, dom: Domain) -> Variable<Domain> {
    assert!(!dom.is_empty());
    Variable {
      idx: idx,
      dom: dom
    }
  }
}

impl<Domain> VarIndex for Variable<Domain>
{
  fn index(&self) -> usize {
    self.idx
  }
}

impl<Domain> Failure for Variable<Domain> where
  Domain: Cardinality
{
  fn is_failed(&self) -> bool {
    self.dom.is_empty()
  }
}
