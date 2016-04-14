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

use kernel::consistency::*;
use propagation::ops::*;

pub trait PropagatorConcept<VStore, Event> :
    Consistency<VStore>
  + PropagatorDependencies<Event>
  + BoxedClone<VStore, Event>
{}

impl<VStore, Event, R> PropagatorConcept<VStore, Event> for R where
 R: Consistency<VStore>,
 R: PropagatorDependencies<Event>,
 R: BoxedClone<VStore, Event>
{}

