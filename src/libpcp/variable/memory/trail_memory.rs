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
use std::fmt::{Debug, Display, Error, Formatter};
use std::ops::{DerefMut, Index};
use std::slice;
use variable::concept::*;
use variable::memory::copy_memory::*;
use variable::memory::ops::*;
use variable::ops::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrailMemory<Trail, Domain> {
    variables: CopyMemory<Domain>,
    trail: Trail,
}

impl<Trail, Domain> MemoryConcept for TrailMemory<Trail, Domain>
where
    Trail: TrailRestoration
        + AssociativeCollection<Location = usize, Item = Domain>
        + TrailVariable
        + Empty
        + Debug,
    Domain: Clone + Display + Debug,
{
}

impl<Trail, Domain> ImmutableMemoryConcept for TrailMemory<Trail, Domain>
where
    Trail:
        TrailRestoration + AssociativeCollection<Location = usize, Item = Domain> + Empty + Debug,
    Domain: Clone + Display + Debug,
{
}

impl<Trail, Domain> Collection for TrailMemory<Trail, Domain> {
    type Item = Domain;
}

impl<Trail, Domain> AssociativeCollection for TrailMemory<Trail, Domain> {
    type Location = usize;
}

impl<Trail, Domain> Empty for TrailMemory<Trail, Domain>
where
    Trail: Empty,
{
    fn empty() -> TrailMemory<Trail, Domain> {
        TrailMemory {
            variables: CopyMemory::empty(),
            trail: Trail::empty(),
        }
    }
}

impl<Trail, Domain> Cardinality for TrailMemory<Trail, Domain> {
    type Size = usize;
    fn size(&self) -> usize {
        self.variables.size()
    }
}

impl<Trail, Domain> Iterable for TrailMemory<Trail, Domain> {
    fn iter(&self) -> slice::Iter<'_, Domain> {
        self.variables.iter()
    }
}

impl<Trail, Domain> Push<Back> for TrailMemory<Trail, Domain>
where
    Trail: TrailVariable + AssociativeCollection<Location = usize, Item = Domain>,
    Domain: Clone,
{
    fn push(&mut self, dom: Domain) {
        self.variables.push(dom);
    }
}

impl<Trail, Domain> Replace for TrailMemory<Trail, Domain>
where
    Trail: TrailVariable + AssociativeCollection<Location = usize, Item = Domain>,
    Domain: Clone,
{
    fn replace(&mut self, key: usize, dom: Domain) -> Domain {
        let dom = self.variables.replace(key, dom);
        self.trail.trail_variable(key, dom.clone());
        dom
    }
}

impl<Trail, Domain> Index<usize> for TrailMemory<Trail, Domain> {
    type Output = Domain;
    fn index(&self, index: usize) -> &Domain {
        &self.variables[index]
    }
}

impl<Trail, Domain> Display for TrailMemory<Trail, Domain>
where
    Domain: Display,
{
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        self.variables.fmt(formatter)
    }
}

impl<Trail, Domain> Freeze for TrailMemory<Trail, Domain>
where
    Trail: TrailRestoration + Collection<Item = Domain>,
{
    type FrozenState = FrozenTrailMemory<Trail, Domain>;
    fn freeze(mut self) -> Self::FrozenState {
        self.trail.commit();
        FrozenTrailMemory::new(self)
    }
}

pub struct FrozenTrailMemory<Trail, Domain> {
    store: TrailMemory<Trail, Domain>,
}

impl<Trail, Domain> FrozenTrailMemory<Trail, Domain> {
    fn new(store: TrailMemory<Trail, Domain>) -> FrozenTrailMemory<Trail, Domain> {
        FrozenTrailMemory { store }
    }
}

impl<Trail, Domain> Snapshot for FrozenTrailMemory<Trail, Domain>
where
    Trail: TrailRestoration + Collection<Item = Domain>,
{
    type Label = Trail::Mark;
    type State = TrailMemory<Trail, Domain>;

    fn label(&mut self) -> Self::Label {
        self.store.trail.mark()
    }

    fn restore(mut self, label: Self::Label) -> Self::State {
        self.store
            .trail
            .undo(label, self.store.variables.deref_mut());
        self.store
    }
}
