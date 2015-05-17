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

use kernel::deep_clone::*;
use variable::ops::*;
use interval::ncollections::ops::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Constant<Domain> where
  Domain: Bounded
{
  value: Domain::Bound
}

impl<Domain> Constant<Domain> where
  Domain: Bounded
{
  pub fn new(value: Domain::Bound) -> Constant<Domain> {
    Constant {
      value: value
    }
  }
}

impl<Domain, Bound> DeepClone for Constant<Domain> where
  Domain: Bounded<Bound=Bound>,
  Bound: Clone
{
  fn deep_clone(&self) -> Constant<Domain> {
    Constant::new(self.value.clone())
  }
}

impl<Domain, Store> StoreMonotonicUpdate<Store, Domain> for Constant<Domain> where
  Domain: Bounded
{
  fn update(&self, _store: &mut Store, _value: Domain) -> bool {
    true
  }
}

impl<Domain, Bound, Store> StoreRead<Store> for Constant<Domain> where
  Domain: Bounded<Bound=Bound> + Singleton<Bound>,
  Bound: Clone
{
  type Value = Domain;
  fn read(&self, _store: &Store) -> Domain {
    Domain::singleton(self.value.clone())
  }
}

impl<Domain, Event> ViewDependencies<Event> for Constant<Domain> where
  Domain: Bounded
{
  fn dependencies(&self, _event: Event) -> Vec<(usize, Event)> {
    vec![]
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::trilean::Trilean::*;
  use solver::fd::event::FDEvent::*;
  use variable::delta_store::*;
  use variable::ops::*;
  use propagators::test::*;
  use propagators::cmp::XLessY;
  use interval::interval::*;

  #[test]
  fn x_less_constant() {
    let dom0_10 = (0,10).to_interval();
    let dom0_4 = (0,4).to_interval();
    let mut store: FDStore = DeltaStore::new();
    let x = store.assign(dom0_10);
    let c: Constant<Interval<i32>> = Constant::new(5)

    let x_less_c = XLessY::new(x, c);
    subsumption_propagate(x_less_c, &mut store, Unknown, True, vec![(0, Bound)], true);
    assert_eq!(x.read(&store), dom0_4);
  }
}
