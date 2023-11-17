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

use gcollections::ops::*;
use gcollections::*;
use kernel::*;
use model::*;
use std::fmt::Debug;
use term::identity::*;
pub use variable::memory::concept::*;
use variable::ops::*;

pub trait EventConcept<Domain>: MonotonicEvent<Domain> + Merge + Clone + Debug {}

impl<Domain, R> EventConcept<Domain> for R where R: MonotonicEvent<Domain> + Merge + Clone + Debug {}

pub trait VStoreConcept:
    ImmutableMemoryConcept
    + AssociativeCollection<Location = Identity<<Self as Collection>::Item>>
    + Alloc
    + MonotonicUpdate
    + DisplayStateful<Model>
{
}
