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

use kernel::DisplayStateful;
use logic::ops::*;
use model::*;
use propagation::ops::*;
use std::fmt::Debug;

pub trait PropagatorConcept_<VStore, Event>:
    Propagator<VStore>
    + Subsumption<VStore>
    + PropagatorDependencies<Event>
    + DisplayStateful<Model>
    + Debug
    + NotFormula<VStore>
{
}

impl<VStore, Event, R> PropagatorConcept_<VStore, Event> for R
where
    R: Propagator<VStore>,
    R: Subsumption<VStore>,
    R: PropagatorDependencies<Event>,
    R: DisplayStateful<Model> + Debug,
    R: NotFormula<VStore>,
{
}

pub trait PropagatorConcept<VStore, Event>: PropagatorConcept_<VStore, Event> {
    fn bclone(&self) -> Box<dyn PropagatorConcept<VStore, Event>>;
}

impl<VStore, Event, R> PropagatorConcept<VStore, Event> for R
where
    R: PropagatorConcept_<VStore, Event>,
    R: Clone + NotFormula<VStore> + 'static,
{
    fn bclone(&self) -> Box<dyn PropagatorConcept<VStore, Event>> {
        Box::new(self.clone())
    }
}
