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
use kernel::Trilean::*;
use propagators::PropagatorKind;
use propagation::*;
use propagation::events::*;
use term::ops::*;
use gcollections::ops::*;
use std::fmt::{Formatter, Debug, Error};
use num::traits::Num;

#[derive(Clone)]
pub struct Cumulative<V>
{
  starts: Vec<V>,
  durations: Vec<usize>,
  resources: Vec<usize>,
  capacity: usize,
}

impl<V> PropagatorKind for Cumulative<V> {}

impl<V> Cumulative<V> {
  pub fn new(starts: Vec<V>, durations: Vec<usize>, resources: Vec<usize>, capacity: usize) -> Cumulative<V> {
    Cumulative { starts: starts, durations: durations, resources: resources, capacity: capacity }
  }
}

impl<V> Debug for Cumulative<V> where
  V: Debug,
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("TO BE DONE capacity:{:?}", self.capacity))
  }
}

impl<Store, B, DomV, V> Subsumption<Store> for Cumulative<V> where
  V: StoreRead<Store, Value=DomV>,
  DomV: Bounded<Bound=B> + Debug,
  B: PartialOrd + Num
{
  fn is_subsumed(&self, store: &Store) -> Trilean {

    let starts = self.starts.read(store);

    if x.upper() <= y.lower() + z.lower() {
      False
    }
    else if x.lower() > y.upper() + z.upper() {
      True
    }
    else {
      Unknown
    }
  }
}

impl<Store, B, DomV, V> Propagator<Store> for Cumulative<V> where
  V: StoreRead<Store, Value=DomV> + StoreMonotonicUpdate<Store, DomV>,
  DomV: Bounded<Bound=B> + StrictShrinkLeft<B>,
  B: PartialOrd + Num,
{
  fn propagate(&mut self, store: &mut Store) -> bool {
    let x = self.x.read(store);
    let y = self.y.read(store);
    let z = self.z.read(store);
    self.x.update(store, x.strict_shrink_left(y.lower() + z.lower())) &&
    self.y.update(store, y.strict_shrink_right(x.upper() - z.lower())) &&
    self.z.update(store, z.strict_shrink_right(x.upper() - y.lower()))
  }
}

impl<X, Y, Z> PropagatorDependencies<FDEvent> for XGreaterYPlusZ<X, Y, Z> where
  X: ViewDependencies<FDEvent>,
  Y: ViewDependencies<FDEvent>,
  Z: ViewDependencies<FDEvent>
{
  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    let mut deps = self.x.dependencies(FDEvent::Bound);
    deps.append(&mut self.y.dependencies(FDEvent::Bound));
    deps.append(&mut self.z.dependencies(FDEvent::Bound));
    deps
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::*;
  use kernel::Trilean::*;
  use propagation::events::*;
  use propagation::events::FDEvent::*;
  use interval::interval::*;
  use propagators::test::*;

  #[test]
  fn x_greater_y_plus_z_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom10_11 = (10,11).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom1_10 = (1,10).to_interval();
    let dom5_10 = (5,10).to_interval();
    let dom6_10 = (6,10).to_interval();
    let dom1_1 = (1,1).to_interval();
    let dom2_2 = (2,2).to_interval();

    x_greater_y_plus_z_test_one(1, dom0_10, dom0_10, dom0_10,
      Unknown, Unknown, vec![(0, Bound), (1, Bound), (2, Bound)], true);
    x_greater_y_plus_z_test_one(2, dom10_11, dom5_15, dom5_15,
      Unknown, True, vec![(0, Assignment), (1, Assignment), (2, Assignment)], true);
    x_greater_y_plus_z_test_one(3, dom10_20, dom1_1, dom1_1,
      True, True, vec![], true);
    x_greater_y_plus_z_test_one(4, dom1_1, dom1_1, dom1_1,
      False, False, vec![], false);
    x_greater_y_plus_z_test_one(5, dom2_2, dom1_1, dom1_1,
      False, False, vec![], false);
    x_greater_y_plus_z_test_one(6, dom6_10, dom5_10, dom1_10,
      Unknown, Unknown, vec![(0, Bound), (1, Bound), (2, Bound)], true);
  }

  fn x_greater_y_plus_z_test_one(test_num: u32,
    x: Interval<i32>, y: Interval<i32>, z: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    trinary_propagator_test(test_num, XGreaterYPlusZ::new, x, y, z, before, after, delta_expected, propagate_success);
  }
}
