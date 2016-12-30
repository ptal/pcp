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

use term::ops::*;
use concept::*;
use gcollections::kind::*;
use std::fmt::{Formatter, Debug, Error};

#[derive(Clone)]
pub struct Sum<V>
{
  vars: Vec<V>
}

impl<V> Sum<V> {
  pub fn new(vars: Vec<V>) -> Self {
    Sum {
      vars: vars
    }
  }
}

impl<V> Debug for Sum<V> where
  V: Debug
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("sum({:?})", self.vars))
  }
}

impl<Domain, Bound, V, Store> StoreMonotonicUpdate<Store> for Sum<V> where
 Store: Collection<Item=Domain>,
 Domain: IntDomain<Item=Bound>,
 Bound: IntBound,
 V: StoreRead<Store>
{
  fn update(&mut self, store: &mut Store, value: Domain) -> bool {
    let sum = self.read(store);
    sum.overlap(&value)
  }
}

impl<Domain, Bound, V, Store> StoreRead<Store> for Sum<V> where
 Store: Collection<Item=Domain>,
 Domain: IntDomain<Item=Bound>,
 Bound: IntBound,
 V: StoreRead<Store>
{
  fn read(&self, store: &Store) -> Domain {
    let mut iter = self.vars.iter();
    let sum = iter.next().expect("At least one variable in sum.");
    iter.fold(sum.read(store), |a: Domain, v| a + v.read(store))
  }
}

impl<V, Event> ViewDependencies<Event> for Sum<V> where
  V: ViewDependencies<Event>,
  Event: Clone
{
  fn dependencies(&self, event: Event) -> Vec<(usize, Event)> {
    self.vars.iter()
      .flat_map(|v| v.dependencies(event.clone()))
      .collect()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use propagators::XEqY;
  use variable::VStoreFD;
  use interval::interval::*;

  type VStore = VStoreFD;

  fn bool2int_read_x_eq_y(narrow_x: Interval<i32>, narrow_y: Interval<i32>, expected: Interval<i32>) {
    let dom0_10 = (0,10).to_interval();
    let dom0_1 = (0,1).to_interval();
    let mut store = VStore::empty();
    let mut x = store.alloc(dom0_10);
    let mut y = store.alloc(dom0_10);

    // z = bool2int(x == y)
    let z = Sum::new(XEqY::new(x, y));
    assert_eq!(z.read(&store), dom0_1);

    x.update(&mut store, narrow_x);
    y.update(&mut store, narrow_y);
    assert_eq!(z.read(&store), expected);
  }

  #[test]
  fn bool2int_read() {
    let dom10_10 = (10,10).to_interval();
    let dom9_9 = (9,9).to_interval();
    let dom9_10 = (9,10).to_interval();
    let dom0_0 = (0,0).to_interval();
    let dom1_1 = (1,1).to_interval();
    let dom0_1 = (0,1).to_interval();

    bool2int_read_x_eq_y(dom10_10, dom10_10, dom1_1);
    bool2int_read_x_eq_y(dom9_9, dom10_10, dom0_0);
    bool2int_read_x_eq_y(dom9_10, dom9_10, dom0_1);
  }

  fn bool2int_update_x_eq_y(dom_x: Interval<i32>, dom_y: Interval<i32>,
    narrow_z: Interval<i32>, expected_x: Interval<i32>, expected: bool)
  {
    let dom0_1 = (0,1).to_interval();
    let mut store = VStore::empty();
    let x = store.alloc(dom_x);
    let y = store.alloc(dom_y);

    // z = bool2int(x == y)
    let mut z = Sum::new(XEqY::new(x, y));
    if expected {
      assert_eq!(z.read(&store), dom0_1);
    }

    let res_z = z.update(&mut store, narrow_z);
    assert_eq!(res_z, expected);
    assert_eq!(x.read(&store), expected_x);
  }

  #[test]
  fn bool2int_update() {
    let dom1_1 = (1,1).to_interval();
    let dom0_1 = (0,1).to_interval();
    let dom10_10 = (10,10).to_interval();
    let dom9_9 = (9,9).to_interval();
    let dom9_10 = (9,10).to_interval();

    bool2int_update_x_eq_y(dom9_10, dom10_10, dom1_1, dom10_10, true);
    bool2int_update_x_eq_y(dom9_10, dom9_10, dom1_1, dom9_10, true);
    bool2int_update_x_eq_y(dom9_9, dom10_10, dom1_1, dom9_9, false);
    bool2int_update_x_eq_y(dom9_10, dom9_10, dom0_1, dom9_10, true);
  }

  #[test]
  #[should_panic]
  fn bool2int_panic_update() {
    let dom10_10 = (10,10).to_interval();
    let dom9_9 = (9,9).to_interval();
    let dom9_10 = (9,10).to_interval();
    let dom0_0 = (0,0).to_interval();
    bool2int_update_x_eq_y(dom9_10, dom10_10, dom0_0, dom9_9, true);
  }
}
