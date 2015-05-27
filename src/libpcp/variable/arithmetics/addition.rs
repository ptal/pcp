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

use variable::ops::*;
use variable::arithmetics::ExprInference;
use std::ops::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Addition<X, V>
{
  x: X,
  v: V
}

impl<X, V, R> ExprInference for Addition<X, V> where
  X: Add<V, Output=R>
{
  type Output = R;
}

impl<X, V> Addition<X, V> {
  pub fn new(x: X, v: V) -> Addition<X, V> {
    Addition {
      x: x,
      v: v
    }
  }
}

impl<X, V, Domain, Store> StoreMonotonicUpdate<Store, Domain> for Addition<X, V> where
  Domain: Sub<V, Output=Domain>,
  V: Clone,
  X: StoreMonotonicUpdate<Store, Domain>
{
  fn update(&self, store: &mut Store, value: Domain) -> bool {
    self.x.update(store, value - self.v.clone())
  }
}

impl<X, V, Domain, Store> StoreRead<Store> for Addition<X, V> where
  Domain: Add<V, Output=Domain>,
  V: Clone,
  X: StoreRead<Store, Value=Domain>
{
  type Value = Domain;
  fn read(&self, store: &Store) -> Domain {
    self.x.read(store) + self.v.clone()
  }
}

impl<X, V, Event> ViewDependencies<Event> for Addition<X, V> where
  X: ViewDependencies<Event>
{
  fn dependencies(&self, event: Event) -> Vec<(usize, Event)> {
    self.x.dependencies(event)
  }
}
