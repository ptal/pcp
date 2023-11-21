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

//! A variable is a pair `(location, value)` where the value lives inside a block of memory, called the *variables store*. The store is parametrized by the type of the values it will contain, it is the *domain* of the variables.
//!
//! We initialize a variable in the store by providing the associated value and the store gives back a location associated to it. Further read or write operations of the value must be done explicitly through the store with the location returned during the allocation. An important property is that only monotonic updates of a value are allowed, it means that the domains can only become smaller. For example, the variable `x: [1..10]` can be updated to `x: [2..10]` or `x: [5..5]` but not to `x: [1..11]`.
//!
//! A subset of arithmetics is provided with *views* on variables. It allows to manipulate an expression such as `x + 5` as if it was a single variable and to avoid implementing specific instances of propagation algorithms such as `x + c < y` which is just `x < y` with `x` being a view. The view acts as a proxy between operations on variable and the store. It implies that operations must be called on the views instead of applying them directly to the store.

pub mod concept;
pub mod memory;
pub mod ops;
pub mod store;

pub use variable::ops::Iterable;

use interval::interval::*;
use interval::interval_set::*;
use propagation::events::FDEvent;
use variable::memory::CopyMemory;
use variable::memory::TimestampTrailMemory;
use variable::store::*;

pub type VStoreTrail<Domain> = Store<TimestampTrailMemory<Domain>, FDEvent>;
pub type VStoreCopy<Domain> = Store<CopyMemory<Domain>, FDEvent>;
pub type VStoreFD = VStoreTrail<Interval<i32>>;
pub type VStoreSet = VStoreTrail<IntervalSet<i32>>;
