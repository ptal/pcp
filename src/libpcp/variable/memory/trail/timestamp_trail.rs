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

use variable::memory::trail::SingleValueTrail;

use gcollections::kind::*;
use gcollections::ops::*;
use std::fmt::{Display, Error, Formatter};
use variable::memory::ops::*;
use vec_map::VecMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampTrail<Domain> {
    trail: SingleValueTrail<Domain>,
    /// Contains the change on the domain of the current instant.
    timestamp: VecMap<Domain>,
}

impl<Domain> Collection for TimestampTrail<Domain> {
    type Item = Domain;
}

impl<Domain> AssociativeCollection for TimestampTrail<Domain> {
    type Location = usize;
}

impl<Domain> TrailVariable for TimestampTrail<Domain> {
    /// We do not trail the intermediate value, just the first change.
    fn trail_variable(&mut self, loc: usize, value: Domain) {
        self.timestamp.entry(loc).or_insert(value);
    }
}

impl<Domain> TrailRestoration for TimestampTrail<Domain>
where
    Domain: Clone,
{
    type Mark = usize;

    fn commit(&mut self) {
        for (loc, value) in self.timestamp.drain() {
            self.trail.trail_variable(loc, value.clone());
        }
    }

    fn mark(&mut self) -> Self::Mark {
        self.trail.mark()
    }

    fn undo(&mut self, mark: Self::Mark, memory: &mut Vec<Domain>) {
        self.trail.undo(mark, memory);
    }
}

impl<Domain> Display for TimestampTrail<Domain>
where
    Domain: Display,
{
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        self.trail.fmt(formatter)
    }
}

impl<Domain> Empty for TimestampTrail<Domain> {
    fn empty() -> TimestampTrail<Domain> {
        TimestampTrail {
            trail: SingleValueTrail::empty(),
            timestamp: VecMap::new(),
        }
    }
}
