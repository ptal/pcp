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
use term::ops::*;
use model::*;
use concept::*;
use gcollections::kind::*;

#[derive(Clone)]
pub struct Sum<X>
{
  vars: Vec<X>
}

impl<X> Sum<X> {
  pub fn new(vars: Vec<X>) -> Self {
    Sum {
      vars: vars
    }
  }
}

impl<X> DisplayStateful<Model> for Sum<X> where
 X: DisplayStateful<Model>
{
  fn display(&self, model: &Model) {
    print!("sum(");
    self.vars[0].display(model);
    let mut i = 1;
    while i < self.vars.len() {
      print!(" + ");
      self.vars[i].display(model);
      i += 1;
    }
    print!(")");
  }
}

impl<Domain, Bound, X, Store> StoreMonotonicUpdate<Store> for Sum<X> where
 Store: Collection<Item=Domain>,
 Domain: IntDomain<Item=Bound>,
 Bound: IntBound,
 X: StoreRead<Store>
{
  fn update(&mut self, store: &mut Store, value: Domain) -> bool {
    let sum = self.read(store);
    sum.overlap(&value)
  }
}

impl<Domain, Bound, X, Store> StoreRead<Store> for Sum<X> where
 Store: Collection<Item=Domain>,
 Domain: IntDomain<Item=Bound>,
 Bound: IntBound,
 X: StoreRead<Store>
{
  fn read(&self, store: &Store) -> Domain {
    let mut iter = self.vars.iter();
    let sum = iter.next().expect("At least one variable in sum.");
    iter.fold(sum.read(store), |a: Domain, v| a + v.read(store))
  }
}

impl<X, Event> ViewDependencies<Event> for Sum<X> where
  X: ViewDependencies<Event>,
  Event: Clone
{
  fn dependencies(&self, event: Event) -> Vec<(usize, Event)> {
    self.vars.iter()
      .flat_map(|v| v.dependencies(event.clone()))
      .collect()
  }
}
