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

use propagators::*;
use propagation::*;
use propagation::events::*;
use term::ops::*;
use term::bool2int::*;
use interval::ops::{Range, Whole};
use gcollections::ops::*;
use gcollections::*;
use gcollections::IntervalKind;
use num::{Integer, PrimInt, Signed};
use std::ops::{Add, Sub};
use std::marker::PhantomData;

pub struct Cumulative<V, VStore>
{
  starts: Vec<Box<V>>,
  durations: Vec<Box<V>>,
  resources: Vec<Box<V>>,
  capacity: Box<V>,
  vstore_phantom: PhantomData<VStore>
}

impl<V, VStore> Cumulative<V, VStore>
{
  pub fn new(starts: Vec<Box<V>>, durations: Vec<Box<V>>,
   resources: Vec<Box<V>>, capacity: Box<V>) -> Self
  {
    assert_eq!(starts.len(), durations.len());
    assert_eq!(starts.len(), resources.len());
    Cumulative {
      starts: starts,
      durations: durations,
      resources: resources,
      capacity: capacity,
      vstore_phantom: PhantomData
    }
  }
}

impl<V, Bound, VStore, Dom> Cumulative<V, VStore> where
  VStore: AssociativeCollection<Item=Dom, Location=V> + Alloc,
  V: ViewDependencies<FDEvent>,
  V: StoreMonotonicUpdate<VStore>,
  V: StoreRead<VStore>,
  V: Clone + 'static,
  Dom: Bounded<Item=Bound> + Add<Output=Dom> + Sub<Output=Dom> + Clone + Whole,
  Dom: Singleton + Overlap + Intersection<Output=Dom> + Cardinality + Range,
  Dom: Empty + ShrinkLeft + ShrinkRight + IntervalKind + 'static + PrimInt + Signed,
  Bound: PartialOrd,
  Bound: Integer + PrimInt + 'static
{
  // Decomposition described in `Why cumulative decomposition is not as bad as it sounds`, Schutt and al., 2009.
  // forall( j in tasks ) (
  //   c >= r[j] + sum( i in tasks where i != j ) (
  //     bool2int( s[i] <= s[j] /\ s[j] < s[i] + d[i] ) * r[i]));
  pub fn join<CStore>(&self, vstore: &mut VStore, cstore: &mut CStore) where
    CStore: Alloc + Collection<Item=Box<PropagatorConcept<VStore, FDEvent>>>
          + Empty + Clone + PropagatorConcept<VStore, FDEvent> + Propagator<VStore> + 'static
  {
    let tasks = self.starts.len();
    // forall( j in tasks ) (...)
    for j in 0..tasks {
      let mut resource_vars = vec![];
      for i in 0..tasks {
        if i != j {
          // bool2int(s[i] <= s[j] /\ s[j] < s[i] + d[i])
          let mut conj: CStore = CStore::empty();
          // s[i] <= s[j]
          conj.alloc(box x_leq_y(self.start_at(i), self.start_at(j)));
          // s[j] < s[i] + d[i]
          conj.alloc(box XLessYPlusZ::new(self.start_at(j), self.start_at(i), self.duration_at(i)));
          let b2i = Bool2Int::new(conj);

          // r = b2i * r[i]
          let r = vstore.alloc(Dom::whole());
          cstore.alloc(box XEqYMulZ::new(r.clone(), b2i, self.resource_at(i)));
          resource_vars.push(r);
        }
      }
      //  sum( i in tasks where i != j )(...)
      let mut sum = resource_vars.pop().expect("Need at least two tasks.");
      for r in resource_vars {
        let sum2 = vstore.alloc(Dom::whole());
        cstore.alloc(box XEqYPlusZ::new(sum2.clone(), sum, r));
        sum = sum2;
      }
      // c >= r[j] + sum
      cstore.alloc(box XGreaterYPlusZ::new(self.capacity_var(), self.resource_at(j), sum));
    }
  }

  fn start_at(&self, i: usize) -> V {
    *self.starts[i].clone()
  }
  fn duration_at(&self, i: usize) -> V {
    *self.durations[i].clone()
  }
  fn resource_at(&self, i: usize) -> V {
    *self.resources[i].clone()
  }
  fn capacity_var(&self) -> V {
    *self.capacity.clone()
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
