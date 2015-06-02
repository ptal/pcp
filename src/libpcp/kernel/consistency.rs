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

use kernel::subsumption::Subsumption;
use kernel::propagator::Propagator;
use kernel::trilean::Trilean;
use kernel::trilean::Trilean::*;

pub trait Consistency<VStore> {
  fn consistency(&mut self, store: &mut VStore) -> Trilean;
}

// FIXME: Without this trait, `Consistency` impl for the Propagator store conflicts with the following one.
pub trait PropagatorKind {}

impl<VStore, R> Consistency<VStore> for R where
  R: Subsumption<VStore>,
  R: Propagator<VStore>,
  R: PropagatorKind
{
  fn consistency(&mut self, store: &mut VStore) -> Trilean {
    if self.propagate(store) {
      self.is_subsumed(store)
    } else {
      False
    }
  }
}
