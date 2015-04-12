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

use solver::variable::*;
use solver::entailment::*;

pub trait Propagator<Event>
{
  // The propagator is stable if no event are added into `events`.
  // Returns `false` if the propagator is failed.
  fn propagate(&mut self, events: &mut Vec<(usize, Event)>) -> bool;
}

pub trait PropagatorDependencies<Event>
{
  // Each event on a variable that can change the result of
  // the `is_entailed` method should be listed here.
  fn dependencies(&self) -> Vec<(usize, Event)>;
}

pub trait DeepClone<State>
{
  fn deep_clone(&self, state: &State) -> Self;
}

impl<D> DeepClone<Vec<SharedVar<D>>> for SharedVar<D>
{
  fn deep_clone(&self, state: &Vec<SharedVar<D>>) -> SharedVar<D> {
    state[self.borrow().index()].clone()
  }
}

pub trait BoxedDeepClone<E, D>
{
  fn boxed_deep_clone(&self, state: &Vec<SharedVar<D>>) -> Box<PropagatorErasure<E, D>>;
}

impl<E, R, D> BoxedDeepClone<E, D> for R where
  R: DeepClone<Vec<SharedVar<D>>>,
  R: Propagator<E>,
  R: PropagatorDependencies<E>,
  R: Entailment,
  R: 'static
{
  fn boxed_deep_clone(&self, state: &Vec<SharedVar<D>>) -> Box<PropagatorErasure<E, D>> {
    Box::new(self.deep_clone(state))
  }
}

pub trait PropagatorErasure<E, D>:
    Propagator<E>
  + PropagatorDependencies<E>
  + Entailment
  + BoxedDeepClone<E, D>
{}

impl<
  D,
  E,
  R: Propagator<E>
   + PropagatorDependencies<E>
   + Entailment
   + BoxedDeepClone<E, D>
> PropagatorErasure<E, D> for R {}
