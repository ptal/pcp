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

use solver::merge::Merge;
use solver::event::*;
use solver::fd::event::FDEvent::*;
use interval::ncollections::ops::*;
use std::cmp::{Eq, min};

// Failure or Nothing events are absents on purpose because there are not events that propagator
// should subscribe to. If a failure occurs, it's over. If nothing occurs, we don't care.

#[derive(Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, Debug)]
pub enum FDEvent {
  Assignment,
  Bound,
  Inner
}

impl Merge for FDEvent {
  fn merge(e: FDEvent, f: FDEvent) -> FDEvent {
    min(e, f)
  }
}

impl EventIndex for FDEvent {
  fn to_index(self) -> usize {
    self as usize
  }

  fn size() -> usize {
    Inner.to_index() + 1
  }
}

impl<Domain> MonotonicEvent<Domain> for FDEvent
where Domain: Subset + Cardinality + Bounded + Eq
{
  fn new(little: &Domain, big: &Domain) -> Option<Self>
  {
    assert!(little.is_subset(big),
      "Events are computed on the difference between `little` and `big`.\
       So `little` must be a subset of `big`.");
    if little != big {
      let ev =
        if little.is_singleton() { Assignment }
        else if little.lower() != big.lower() ||
                little.upper() != big.upper() { Bound }
        else { Inner };
      Some(ev)
    } else {
      None
    }
  }
}
