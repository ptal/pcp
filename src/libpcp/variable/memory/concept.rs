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
use std::ops::Index;
use variable::ops::*;

pub trait ImmutableMemoryConcept:
    Collection
    + AssociativeCollection
    + Cardinality<Size = usize>
    + Iterable
    + Empty
    + Index<usize, Output = <Self as Collection>::Item>
    + Freeze
    + Debug
{
}

pub trait MemoryConcept:
    ImmutableMemoryConcept + AssociativeCollection<Location = usize> + Push<Back> + Replace
{
}
