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
use interval::ncollections::ops::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Constant<V>
{
  value: V
}

impl<V> ExprInference for Constant<V>
{
  type Output = V;
}

impl<V> Constant<V>
{
  pub fn new(value: V) -> Constant<V> {
    Constant {
      value: value
    }
  }
}

impl<V, Domain, Store> StoreMonotonicUpdate<Store, Domain> for Constant<V> where
  Domain: Cardinality
{
  fn update(&self, _store: &mut Store, value: Domain) -> bool {
    !value.is_empty()
  }
}

impl<V, Store> StoreRead<Store> for Constant<V> where
  V: Clone
{
  type Value = Option<V>;
  fn read(&self, _store: &Store) -> Option<V> {
    Some(self.value.clone())
  }
}

impl<V, Event> ViewDependencies<Event> for Constant<V>
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
    let c: Constant<i32> = Constant::new(5);

    let x_less_c = XLessY::new(x, c);
    subsumption_propagate(1, x_less_c, &mut store, Unknown, True, vec![(0, Bound)], true);
    assert_eq!(x.read(&store), dom0_4);
  }
}
