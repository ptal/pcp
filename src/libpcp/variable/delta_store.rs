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

use kernel::*;
use variable::ops::*;
use variable::store::*;
use term::identity::*;
use interval::ncollections::ops::*;
use std::slice;
use vec_map::{Drain, VecMap};
use std::fmt::{Formatter, Display, Error};
use std::default::Default;
use std::ops::Index;

pub struct DeltaStore<Domain, Event> {
  store: Store<Domain>,
  delta: VecMap<Event>
}

impl<Domain, Event> DeltaStore<Domain, Event>
{
  pub fn new() -> DeltaStore<Domain, Event> {
    DeltaStore::from_store(Store::new())
  }

  fn from_store(store: Store<Domain>) -> DeltaStore<Domain, Event> {
    DeltaStore {
      store: store,
      delta: VecMap::new()
    }
  }
}

impl<Domain, Event> DeltaStore<Domain, Event> where
  Domain: Bounded + Cardinality + Subset,
  Event: MonotonicEvent<Domain> + Merge + Clone
{
  // FIXME: Need a rustc fix on borrowing rule, `updated` not needed.
  fn merge_delta(&mut self, key: usize, old_dom: &Domain) {
    if let Some(delta) = Event::new(&self.store[key], old_dom) {
      let mut updated = false;
      if let Some(old_delta) = self.delta.get_mut(&key) {
        *old_delta = Merge::merge(old_delta.clone(), delta.clone());
        updated = true;
      }
      if !updated {
        self.delta.insert(key, delta);
      }
    }
  }
}

impl<Domain, Event> Default for DeltaStore<Domain, Event>
{
  fn default() -> DeltaStore<Domain, Event> {
    DeltaStore::new()
  }
}

impl<Domain, Event> DrainDelta<Event> for DeltaStore<Domain, Event> {
  fn drain_delta<'a>(&'a mut self) -> Drain<'a, Event> {
    self.delta.drain()
  }

  fn has_changed(&self) -> bool {
    !self.delta.is_empty()
  }
}

impl<Domain, Event> Clone for DeltaStore<Domain, Event> where
  Domain: Clone
{
  fn clone(&self) -> Self {
    DeltaStore {
      store: self.store.clone(),
      delta: VecMap::new()
    }
  }
}

impl<Domain, Event> State for DeltaStore<Domain, Event> where
 Domain: Clone
{
  type Label = Store<Domain>;

  fn mark(&self) -> Store<Domain> {
    self.store.mark()
  }

  fn restore(self, label: Store<Domain>) -> Self {
    DeltaStore::from_store(label)
  }
}

impl<Domain, Event> Iterable for DeltaStore<Domain, Event> {
  type Value = Domain;

  fn iter<'a>(&'a self) -> slice::Iter<'a, Domain> {
    self.store.iter()
  }
}

impl<'a, Domain, Event> IntoIterator for &'a DeltaStore<Domain, Event> {
  type Item = <&'a Store<Domain> as IntoIterator>::Item;
  type IntoIter = <&'a Store<Domain> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.store.into_iter()
  }
}

impl<Domain, Event> Cardinality for DeltaStore<Domain, Event>
{
  type Size = usize;

  fn size(&self) -> usize {
    self.store.size()
  }
}

impl<Domain, Event> Alloc<Domain> for DeltaStore<Domain, Event> where
  Domain: Cardinality
{
  type Location = Identity<Domain>;

  fn alloc(&mut self, dom: Domain) -> Identity<Domain> {
    let var = self.store.alloc(dom);
    self.delta.reserve_len(var.index());
    var
  }
}

impl<Domain, Event> Update<usize, Domain> for DeltaStore<Domain, Event> where
  Domain: Bounded + Cardinality + Subset + Clone,
  Event: MonotonicEvent<Domain> + Merge + Clone
{
  fn update(&mut self, key: usize, dom: Domain) -> Option<Domain> {
    self.store.update(key, dom)
      .map(|old_dom| {
        self.merge_delta(key, &old_dom);
        old_dom
      })
  }
}

impl<Domain, Event> Index<usize> for DeltaStore<Domain, Event>
{
  type Output = Domain;
  fn index<'a>(&'a self, index: usize) -> &'a Domain {
    &self.store[index]
  }
}

impl<Domain, Event> Display for DeltaStore<Domain, Event> where
 Domain: Display
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    self.store.fmt(formatter)
  }
}

#[cfg(test)]
pub mod test {
  use super::*;
  use kernel::Alloc;
  use variable::ops::*;
  use term::identity::*;
  use propagation::events::*;
  use propagation::events::FDEvent::*;
  use interval::interval::*;
  use interval::ncollections::ops::*;

  pub type FDStore = DeltaStore<Interval<i32>, FDEvent>;

  fn test_op<Op>(source: Interval<i32>, target: Interval<i32>, delta_expected: Vec<FDEvent>, update_success: bool, op: Op) where
    Op: FnOnce(&FDStore, Identity<Interval<i32>>) -> Interval<i32>
  {
    let mut store = DeltaStore::new();
    let var = store.alloc(source);

    let new = op(&store, var);
    assert_eq!(var.update(&mut store, new), update_success);
    assert_eq!(new, target);

    if update_success {
      let delta_expected = delta_expected.into_iter().map(|d| (var.index(), d)).collect();
      consume_delta(&mut store, delta_expected);
      assert_eq!(var.read(&store), target);
    }
  }

  fn test_binary_op<Op>(source1: Interval<i32>, source2: Interval<i32>, target: Interval<i32>, delta_expected: Vec<(usize, FDEvent)>, update_success: bool, op: Op) where
    Op: FnOnce(&FDStore, Identity<Interval<i32>>, Identity<Interval<i32>>) -> Interval<i32>
  {
    let mut store = DeltaStore::new();
    let var1 = store.alloc(source1);
    let var2 = store.alloc(source2);

    let new = op(&store, var1, var2);
    assert_eq!(var1.update(&mut store, new), update_success);
    assert_eq!(var2.update(&mut store, new), update_success);
    assert_eq!(new, target);

    if update_success {
      consume_delta(&mut store, delta_expected);
      assert_eq!(var1.read(&store), target);
      assert_eq!(var2.read(&store), target);
    }
  }

  pub fn consume_delta(store: &mut FDStore, delta_expected: Vec<(usize, FDEvent)>) {
    let res: Vec<(usize, FDEvent)> = store.drain_delta().collect();
    assert_eq!(res, delta_expected);
    assert!(store.drain_delta().next().is_none());
  }

  #[test]
  fn var_update_test() {
    let dom0_10 = (0,10).to_interval();
    let dom0_9 = (0,5).to_interval();
    let dom1_10 = (5,10).to_interval();
    let dom1_9 = (1,9).to_interval();
    let dom0_0 = (0,0).to_interval();
    let empty = Interval::empty();

    var_update_test_one(dom0_10, dom0_10, vec![], true);
    var_update_test_one(dom0_10, empty, vec![], false);
    var_update_test_one(dom0_10, dom0_0, vec![Assignment], true);
    var_update_test_one(dom0_10, dom1_10, vec![Bound], true);
    var_update_test_one(dom0_10, dom0_9, vec![Bound], true);
    var_update_test_one(dom0_10, dom1_9, vec![Bound], true);
  }

  fn var_update_test_one(source: Interval<i32>, target: Interval<i32>, delta_expected: Vec<FDEvent>, update_success: bool) {
    test_op(source, target, delta_expected, update_success, |_,_| target);
  }

  #[test]
  fn var_shrink_bound() {
    let dom0_10 = (0,10).to_interval();

    var_shrink_lb_test_one(dom0_10, 0, vec![], true);
    var_shrink_lb_test_one(dom0_10, 10, vec![Assignment], true);
    var_shrink_lb_test_one(dom0_10, 1, vec![Bound], true);
    var_shrink_lb_test_one(dom0_10, 11, vec![], false);

    var_shrink_ub_test_one(dom0_10, 10, vec![], true);
    var_shrink_ub_test_one(dom0_10, 0, vec![Assignment], true);
    var_shrink_ub_test_one(dom0_10, 1, vec![Bound], true);
    var_shrink_ub_test_one(dom0_10, -1, vec![], false);
  }

  fn var_shrink_lb_test_one(source: Interval<i32>, target_lb: i32, delta_expected: Vec<FDEvent>, update_success: bool) {
    let expected_dom = (target_lb, source.upper()).to_interval();

    test_op(source, expected_dom, delta_expected, update_success,
      |store, var| var.read(store).shrink_left(target_lb));
  }

  fn var_shrink_ub_test_one(source: Interval<i32>, target_ub: i32, delta_expected: Vec<FDEvent>, update_success: bool) {
    let expected_dom = (source.lower(), target_ub).to_interval();

    test_op(source, expected_dom, delta_expected, update_success,
      |store, var| var.read(store).shrink_right(target_ub));
  }

  #[test]
  fn var_intersection_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom10_10 = (10,10).to_interval();
    let dom11_20 = (11,20).to_interval();
    let dom1_9 = (1,9).to_interval();

    var_intersection_test_one(dom0_10, dom10_20, dom10_10, vec![(0, Assignment), (1, Assignment)], true);
    var_intersection_test_one(dom0_10, dom1_9, dom1_9, vec![(0, Bound)], true);
    var_intersection_test_one(dom1_9, dom0_10, dom1_9, vec![(1, Bound)], true);
    var_intersection_test_one(dom0_10, dom11_20, Interval::empty(), vec![], false);
  }

  fn var_intersection_test_one(source1: Interval<i32>, source2: Interval<i32>, target: Interval<i32>, delta_expected: Vec<(usize, FDEvent)>, update_success: bool) {
    test_binary_op(source1, source2, target, delta_expected, update_success,
      |store, v1, v2| v1.read(store).intersection(&v2.read(store)));
  }
}
