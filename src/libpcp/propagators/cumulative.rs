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
use term::bool2int::*;
use std::marker::PhantomData;
use concept::*;

pub struct Cumulative<VS, VD, VR, VC, VStore>
{
  starts: Vec<Box<VS>>,
  durations: Vec<Box<VD>>,
  resources: Vec<Box<VR>>,
  capacity: Box<VC>,
  vstore_phantom: PhantomData<VStore>
}

impl<VS, VD, VR, VC, VStore> Cumulative<VS, VD, VR, VC, VStore>
{
  pub fn new(starts: Vec<Box<VS>>, durations: Vec<Box<VD>>,
   resources: Vec<Box<VR>>, capacity: Box<VC>) -> Self
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

impl<V, VS, VD, VR, VC, Bound, VStore, Dom> Cumulative<VS, VD, VR, VC, VStore> where
  VStore: IntVStore<Item=Dom, Location=V>,
  V: IntVariable<VStore> + 'static,
  VS: IntVariable<VStore> + 'static,
  VD: IntVariable<VStore> + 'static,
  VR: IntVariable<VStore> + 'static,
  VC: IntVariable<VStore> + 'static,
  Dom: IntDomain<Item=Bound> + 'static,
  Bound: IntBound + 'static,
{
  // Decomposition described in `Why cumulative decomposition is not as bad as it sounds`, Schutt and al., 2009.
  // forall( j in tasks ) (
  //   c >= r[j] + sum( i in tasks where i != j ) (
  //     bool2int( s[i] <= s[j] /\ s[j] < s[i] + d[i] ) * r[i]));
  pub fn join<CStore>(&self, vstore: &mut VStore, cstore: &mut CStore) where
    CStore: IntCStore<VStore> + 'static
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
          conj.alloc(box x_leq_y::<_,_,Bound>(self.start_at(i), self.start_at(j)));
          // s[j] < s[i] + d[i]
          conj.alloc(box XLessYPlusZ::new(self.start_at(j), self.start_at(i), self.duration_at(i)));
          let b2i = Bool2Int::new(conj);

          // r = b2i * r[i]
          let ri = self.resource_at(i);
          let ri_ub = ri.read(vstore).upper();
          let r = vstore.alloc(Dom::new(Bound::zero(), ri_ub));
          cstore.alloc(box XEqYMulZ::new(r.clone(), b2i, ri));
          resource_vars.push(r);
        }
      }
      //  sum( i in tasks where i != j )(...)
      let mut sum = resource_vars.pop().expect("Need at least two tasks.");
      for r in resource_vars {
        let sum2_dom = sum.read(vstore) + r.read(vstore);
        let sum2 = vstore.alloc(sum2_dom);
        cstore.alloc(box XEqYPlusZ::<_,_,_,Bound>::new(sum2.clone(), sum, r));
        sum = sum2;
      }
      // c >= r[j] + sum
      cstore.alloc(box x_geq_y_plus_z::<_,_,_,Bound>(self.capacity_var(), self.resource_at(j), sum));
    }
  }

  fn start_at(&self, i: usize) -> VS {
    *self.starts[i].clone()
  }
  fn duration_at(&self, i: usize) -> VD {
    *self.durations[i].clone()
  }
  fn resource_at(&self, i: usize) -> VR {
    *self.resources[i].clone()
  }
  fn capacity_var(&self) -> VC {
    *self.capacity.clone()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::*;
  use kernel::Trilean::*;
  use variable::VStoreCopy;
  use propagation::CStoreFD;
  use interval::interval::*;
  use interval::ops::Range;
  use gcollections::ops::*;

  type VStoreFD = VStoreCopy<Interval<i32>>;

  struct CumulativeTest {
    starts: Vec<Interval<i32>>,
    durations: Vec<Interval<i32>>,
    resources: Vec<Interval<i32>>,
    capacity: Interval<i32>,
  }

  impl CumulativeTest {
    fn new(starts: Vec<Interval<i32>>, durations: Vec<Interval<i32>>,
      resources: Vec<Interval<i32>>, capacity: Interval<i32>) -> Self
    {
      CumulativeTest {
        starts: starts,
        durations: durations,
        resources: resources,
        capacity: capacity
      }
    }

    fn new_assignment(starts: Vec<i32>, durations: Vec<i32>,
      resources: Vec<i32>, capacity: i32) -> Self
    {
      CumulativeTest::new(
        starts.into_iter().map(|s| Interval::new(s, s)).collect(),
        durations.into_iter().map(|d| Interval::new(d, d)).collect(),
        resources.into_iter().map(|r| Interval::new(r, r)).collect(),
        Interval::new(capacity, capacity)
      )
    }

    fn instantiate(self, vstore: &mut VStoreFD, cstore: &mut CStoreFD<VStoreFD>) {
      let cumulative = Cumulative::new(
        self.starts.into_iter().map(|s| box vstore.alloc(s)).collect(),
        self.durations.into_iter().map(|d| box vstore.alloc(d)).collect(),
        self.resources.into_iter().map(|r| box vstore.alloc(r)).collect(),
        box vstore.alloc(self.capacity)
      );
      cumulative.join(vstore, cstore);
    }

    fn test(self, test_num: usize, before: Trilean, after: Trilean, propagate_success: bool) {
      println!("Test number {}", test_num);
      let vstore = &mut VStoreFD::empty();
      let cstore = &mut CStoreFD::empty();
      self.instantiate(vstore, cstore);
      assert_eq!(cstore.is_subsumed(vstore), before);
      assert_eq!(cstore.propagate(vstore), propagate_success);
      assert_eq!(cstore.is_subsumed(vstore), after);
    }
  }

  #[test]
  fn cumulative_assignment_test() {
    // The task 2 and 3 overlaps and consume 4 resources altogether.
    let test = CumulativeTest::new_assignment(
      vec![0,1,4], vec![3,4,2], vec![1,2,2], 3);
    test.test(1, Unknown, False, false);

    // We can delay the task 3 to fix the problem.
    let test = CumulativeTest::new_assignment(
      vec![0,1,5], vec![3,4,2], vec![1,2,2], 3);
    test.test(2, Unknown, True, true);

    // Another possibility is to reduce the resource of task 3.
    let test = CumulativeTest::new_assignment(
      vec![0,1,4], vec![3,4,2], vec![1,2,1], 3);
    test.test(3, Unknown, True, true);

    // Or augment the total amount of resources available.
    let test = CumulativeTest::new_assignment(
      vec![0,1,4], vec![3,4,2], vec![1,2,2], 4);
    test.test(4, Unknown, True, true);

    // Or reduce the duration of task 2.
    let test = CumulativeTest::new_assignment(
      vec![0,1,4], vec![3,3,2], vec![1,2,2], 3);
    test.test(4, Unknown, True, true);
  }

  #[test]
  fn cumulative_test() {
    let mut test = CumulativeTest::new_assignment(
      vec![0,1,4], vec![3,4,2], vec![1,2,2], 3);
    // Widden the start date of task 1, should fail anyway.
    test.starts[0] = Interval::new(0,4);
    test.test(1, Unknown, False, false);

    let mut test = CumulativeTest::new_assignment(
      vec![0,1,4], vec![3,4,2], vec![1,2,2], 3);
    // Widden the start date of task 2, succeed when schedule at start=0.
    test.starts[1] = Interval::new(0,1);
    test.test(2, Unknown, Unknown, true);

    let mut test = CumulativeTest::new_assignment(
      vec![0,1,4], vec![3,4,2], vec![1,2,2], 3);
    // Widden the start date of task 3, succeed when schedule at start=5.
    test.starts[2] = Interval::new(4,5);
    test.test(3, Unknown, Unknown, true);
  }
}
