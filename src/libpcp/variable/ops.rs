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

use gcollections::kind::*;
use std::slice;
use vec_map::Drain;

pub trait DrainDelta<Event> {
    fn drain_delta<'a>(&'a mut self) -> Drain<'a, Event>;
    fn has_changed(&self) -> bool;
    fn reset_changed(&mut self);
}

pub trait Iterable: Collection {
    fn iter<'a>(&'a self) -> slice::Iter<'a, Self::Item>;
}

pub trait MonotonicUpdate: AssociativeCollection {
    fn update(&mut self, loc: &Self::Location, value: Self::Item) -> bool;
}
