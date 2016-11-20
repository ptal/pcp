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

pub use variable::memory::concept::*;
use kernel::*;
use variable::ops::*;
use gcollections::ops::*;

pub trait EventConcept<Domain>:
  MonotonicEvent<Domain> + Merge + Clone
{}

impl<Domain, R> EventConcept<Domain> for R where
  R: MonotonicEvent<Domain> + Merge + Clone
{}

pub trait StoreConcept:
   ImmutableMemoryConcept
 + Alloc
 + MonotonicUpdate
{}
