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
use variable::ops::*;
use gcollections::ops::*;
use gcollections::ops::sequence::ordering::*;
use std::ops::Index;
use std::fmt::Display;

pub trait DomainConcept :
  Clone + Display + Bounded + Cardinality + Subset
{}

impl<R> DomainConcept for R where
  R: Clone + Display + Bounded + Cardinality + Subset
{}

pub trait ReadOnlyMemoryConcept<Domain> :
   Cardinality<Size=usize>
 + Iterable<Item=Domain>
 + Empty
 + Index<usize, Output=Domain>
 + Display
 + Clone // TO DELETE
{}

impl<Domain, R> ReadOnlyMemoryConcept<Domain> for R where
 R: Cardinality<Size=usize>
 + Iterable<Item=Domain>
 + Empty
 + Index<usize, Output=Domain>
 + Display
 + Clone // TO DELETE
{}

pub trait MemoryConcept<Domain> :
   ReadOnlyMemoryConcept<Domain>
 + Push<Back, Domain>
 + Update<usize, Domain>
 + Freeze
 where
  Domain: DomainConcept
{}

impl<Domain, R> MemoryConcept<Domain> for R where
 R: ReadOnlyMemoryConcept<Domain>
 + Push<Back, Domain>
 + Update<usize, Domain>
 + Freeze,
 Domain: DomainConcept
{}

pub trait StoreConcept<Domain> :
   ReadOnlyMemoryConcept<Domain>
 + Alloc<Domain>
 + Update<usize, Domain>
 + State
where
 Domain: DomainConcept
{}

impl<Domain, R> StoreConcept<Domain> for R where
 R: ReadOnlyMemoryConcept<Domain>
 + Alloc<Domain>
 + Update<usize, Domain>
 + State,
 Domain: DomainConcept
{}
