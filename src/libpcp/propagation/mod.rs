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

//! Propagation is a set of algorithmic components to verify the consistency of a constraint conjunction, called the *constraints store*.
//!
//! The constraints store is parametrized by the type of the variables store to keep both concrete implementation independents. Stores are stacked in a hierarchical manner and they communicate only from top to bottom; the variables store is not aware of the constraints store.

pub mod concept;
pub mod events;
pub mod ops;
pub mod reactor;
pub mod reactors;
pub mod scheduler;
pub mod schedulers;
pub mod store;

pub use propagation::concept::*;
pub use propagation::ops::*;
pub use propagation::reactor::Reactor;
pub use propagation::scheduler::Scheduler;

pub type CStoreFD<VStore> =
    store::Store<VStore, events::FDEvent, reactors::IndexedDeps, schedulers::RelaxedFifo>;
