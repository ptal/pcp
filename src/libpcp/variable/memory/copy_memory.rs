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

use gcollections::kind::*;
use gcollections::ops::sequence::ordering::*;
use gcollections::ops::*;
use kernel::*;
use std::fmt::Debug;
use std::fmt::{Display, Error, Formatter};
use std::mem;
use std::ops::{Deref, DerefMut, Index};
use std::rc::*;
use std::slice;
use variable::concept::*;
use variable::ops::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyMemory<Domain> {
    variables: Vec<Domain>,
}

impl<Domain> MemoryConcept for CopyMemory<Domain> where Domain: Clone + Display + Debug {}

impl<Domain> ImmutableMemoryConcept for CopyMemory<Domain> where Domain: Clone + Display + Debug {}

impl<Domain> Collection for CopyMemory<Domain> {
    type Item = Domain;
}

impl<Domain> AssociativeCollection for CopyMemory<Domain> {
    type Location = usize;
}

impl<Domain> Replace for CopyMemory<Domain> {
    fn replace(&mut self, key: usize, dom: Domain) -> Domain {
        mem::replace(&mut self.variables[key], dom)
    }
}

impl<Domain> Index<usize> for CopyMemory<Domain> {
    type Output = Domain;
    fn index(&self, index: usize) -> &Domain {
        &self.variables[index]
    }
}

impl<Domain> CopyMemory<Domain> {
    fn restore(variables: Vec<Domain>) -> CopyMemory<Domain> {
        CopyMemory { variables }
    }
}

impl<Domain> Deref for CopyMemory<Domain> {
    type Target = Vec<Domain>;
    fn deref(&self) -> &Self::Target {
        &self.variables
    }
}

impl<Domain> DerefMut for CopyMemory<Domain> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.variables
    }
}

impl<Domain> Empty for CopyMemory<Domain> {
    fn empty() -> CopyMemory<Domain> {
        CopyMemory { variables: vec![] }
    }
}

impl<Domain> Cardinality for CopyMemory<Domain> {
    type Size = usize;

    fn size(&self) -> usize {
        self.variables.len()
    }
}

impl<Domain> Iterable for CopyMemory<Domain> {
    fn iter(&self) -> slice::Iter<'_, Self::Item> {
        self.variables.iter()
    }
}

impl<Domain> Push<Back> for CopyMemory<Domain> {
    fn push(&mut self, value: Domain) {
        self.variables.push(value);
    }
}

impl<Domain> Display for CopyMemory<Domain>
where
    Domain: Display,
{
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        for v in &self.variables {
            formatter.write_fmt(format_args!("{} ", v))?;
        }
        Ok(())
    }
}

impl<Domain> Freeze for CopyMemory<Domain>
where
    Domain: Clone,
{
    type FrozenState = FrozenCopyMemory<Domain>;
    fn freeze(self) -> Self::FrozenState {
        FrozenCopyMemory::new(self)
    }
}

pub struct FrozenCopyMemory<Domain> {
    variables: Rc<Vec<Domain>>,
}

impl<Domain> FrozenCopyMemory<Domain> {
    fn new(store: CopyMemory<Domain>) -> FrozenCopyMemory<Domain> {
        FrozenCopyMemory {
            variables: Rc::new(store.variables),
        }
    }
}

impl<Domain> Snapshot for FrozenCopyMemory<Domain>
where
    Domain: Clone,
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
