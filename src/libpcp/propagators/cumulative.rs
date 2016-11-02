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
use propagators::cmp::{x_leq_y, XLessYPlusZ};
use propagation::*;
use propagation::events::*;
use term::ops::*;
use gcollections::ops::*;
use std::fmt::{Formatter, Debug, Error};
use num::traits::Num;
use std::ops::{Add, Sub};
//use propagation::CStoreFD;

pub type CStore<VStore> = CStoreFD<VStore>;

pub struct Cumulative<V>
{
  starts: Vec<V>,
  durations: Vec<usize>,
  resources: Vec<usize>,
  capacity: usize,
}

impl<V> PropagatorKind for Cumulative<V> {}

impl<V> Cumulative<V>  where V: Clone {
  pub fn new(starts: Vec<V>, durations: Vec<usize>, resources: Vec<usize>, capacity: usize) -> Cumulative<V> {
    Cumulative { starts: starts, durations: durations, resources: resources, capacity: capacity}
  }
}

impl<V> Debug for Cumulative<V> where
  V: Debug,
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("TO BE DONE capacity:{:?}", self.capacity))
  }
}

impl<Store, DomV, V> Subsumption<Store> for Cumulative<V> where
  V: StoreRead<Store, Value=DomV> + Clone + Debug,
  DomV: Bounded + Disjoint + Debug + Add + Sub + Clone,
{
  fn is_subsumed(&self, store: &Store) -> Trilean {

    let mut cstore = CStore::empty();
    for j in 0..self.starts.len() - 1 {
      for i in 0..self.starts.len() - 1 {
        if i != j {
          // `s[i] <= s[j] /\ s[j] < s[i] + d[i]`
          cstore.alloc(x_leq_y(self.starts[i].clone(), self.starts[j].clone())); 
          cstore.alloc(XLessYPlusZ::new(self.starts[j].clone(), self.starts[i].clone(), self.durations[i].clone())); 
        }
      }
    }

    cstore.is_subsumed(store)
  }
}

impl<Store, B, DomV, V> Propagator<Store> for Cumulative<V> where
  V: StoreRead<Store, Value=DomV> + StoreMonotonicUpdate<Store, DomV>,
  DomV: Bounded<Bound=B> + StrictShrinkLeft<B>,
  B: PartialOrd + Num,
{
  fn propagate(&mut self, store: &mut Store) -> bool {
  }
}

impl<V> PropagatorDependencies<FDEvent> for Cumulative<V> where
  V: ViewDependencies<FDEvent>
{
  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    let mut deps = self.starts.iter().flat_map(|v| v.dependencies(FDEvent::Bound)).collect();
    deps.append(self.durations.iter().flat_map(|v| v.dependencies(FDEvent::Bound)).collect());
    deps.append(self.resources.iter().flat_map(|v| v.dependencies(FDEvent::Bound)).collect());
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
  fn cumulative_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom10_11 = (10,11).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom1_10 = (1,10).to_interval();
    let dom5_10 = (5,10).to_interval();
    let dom6_10 = (6,10).to_interval();
    let dom1_1 = (1,1).to_interval();
    let dom2_2 = (2,2).to_interval();

    culmulative_test_one(1, dom0_10, dom0_10, dom0_10,
      Unknown, Unknown, vec![(0, Bound), (1, Bound), (2, Bound)], true);
    culmulative_test_one(2, dom10_11, dom5_15, dom5_15,
      Unknown, True, vec![(0, Assignment), (1, Assignment), (2, Assignment)], true);
    culmulative_test_one(3, dom10_20, dom1_1, dom1_1,
      True, True, vec![], true);
    culmulative_test_one(4, dom1_1, dom1_1, dom1_1,
      False, False, vec![], false);
    culmulative_test_one(5, dom2_2, dom1_1, dom1_1,
      False, False, vec![], false);
    culmulative_test_one(6, dom6_10, dom5_10, dom1_10,
      Unknown, Unknown, vec![(0, Bound), (1, Bound), (2, Bound)], true);
  }

  fn culmulative_test_one(test_num: u32,
    x: Interval<i32>, y: Interval<i32>, z: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    trinary_propagator_test(test_num, XGreaterYPlusZ::new, x, y, z, before, after, delta_expected, propagate_success);
  }
}
