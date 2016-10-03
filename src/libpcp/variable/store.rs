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
use variable::concept::*;
use term::identity::*;
use gcollections::ops::*;
use vec_map::{Drain, VecMap};
use std::marker::PhantomData;
use std::slice;
use std::fmt::{Formatter, Display, Error};
use std::ops::Index;

pub struct Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  memory: Memory,
  delta: VecMap<Event>,
  phantom: PhantomData<Domain>
}

impl<Memory, Domain, Event> ImmutableMemoryConcept<Domain> for
  Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{}

impl<Memory, Domain, Event> StoreConcept<Domain> for
  Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept,
 Event: EventConcept<Domain>
{}

impl<Memory, Domain, Event> Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  fn from_memory(memory: Memory) -> Self {
    Store {
      memory: memory,
      delta: VecMap::new(),
      phantom: PhantomData
    }
  }
}

impl<Memory, Domain, Event> Empty for Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  fn empty() -> Store<Memory, Domain, Event> {
    Store::from_memory(Memory::empty())
  }
}

impl<Memory, Domain, Event> Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept,
 Event: EventConcept<Domain>
{
  // FIXME: Need a rustc fix on borrowing rule, `updated` not needed.
  fn update_delta(&mut self, key: usize, old_dom: &Domain) {
    if let Some(delta) = Event::new(&self[key], old_dom) {
      let mut updated = false;
      if let Some(old_delta) = self.delta.get_mut(key) {
        *old_delta = Merge::merge(old_delta.clone(), delta.clone());
        updated = true;
      }
      if !updated {
        self.delta.insert(key, delta);
      }
    }
  }
}

impl<Memory, Domain, Event> Cardinality for Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  type Size = usize;

  fn size(&self) -> usize {
    self.memory.size()
  }
}

impl<Memory, Domain, Event> Iterable for Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  type Item = Domain;

  fn iter<'a>(&'a self) -> slice::Iter<'a, Self::Item> {
    self.memory.iter()
  }
}

impl<Memory, Domain, Event> Alloc<Domain> for Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  type Location = Identity<Domain>;

  fn alloc(&mut self, dom: Domain) -> Identity<Domain> {
    assert!(!dom.is_empty());
    let var_idx = self.memory.size();
    self.memory.push(dom);
    Identity::new(var_idx)
  }
}

impl<Memory, Domain, Event> MonotonicUpdate<usize, Domain> for Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept,
 Event: EventConcept<Domain>
{
  // We update the domain located at `loc` if `dom` is not empty and is a strictly smaller than the current value.
  fn update(&mut self, loc: usize, dom: Domain) -> bool {
    assert!(dom.is_subset(&self.memory[loc]),
      "Domain update must be monotonic.");
    if dom.is_empty() {
      false
    }
    else {
      if dom.size() < self[loc].size() {
        let old_dom = self.memory.replace(loc, dom);
        self.update_delta(loc, &old_dom);
      }
      true
    }
  }
}

impl<Memory, Domain, Event> Index<usize> for Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  type Output = Domain;
  fn index<'a>(&'a self, index: usize) -> &'a Domain {
    assert!(index < self.memory.size(),
      "Variable not registered in the store. Variable index must be obtained with `alloc`.");
    &self.memory[index]
  }
}

impl<Memory, Domain, Event> Display for Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    self.memory.fmt(formatter)
  }
}

impl<Memory, Domain, Event> DrainDelta<Event> for Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  fn drain_delta<'a>(&'a mut self) -> Drain<'a, Event> {
    self.delta.drain()
  }

  fn has_changed(&self) -> bool {
    !self.delta.is_empty()
  }
}

impl<Memory, Domain, Event> Freeze for Store<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  type FrozenState = FrozenStore<Memory, Domain, Event>;
  fn freeze(self) -> Self::FrozenState
  {
    FrozenStore::new(self)
  }
}

pub struct FrozenStore<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  frozen_memory: Memory::FrozenState,
  phantom_domain: PhantomData<Domain>,
  phantom_event: PhantomData<Event>
}

impl<Memory, Domain, Event> FrozenStore<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  fn new(store: Store<Memory, Domain, Event>) -> Self {
    FrozenStore {
      frozen_memory: store.memory.freeze(),
      phantom_domain: PhantomData,
      phantom_event: PhantomData
    }
  }
}

impl<Memory, Domain, Event> Snapshot for FrozenStore<Memory, Domain, Event> where
 Memory: MemoryConcept<Domain>,
 Domain: DomainConcept
{
  type Label = <Memory::FrozenState as Snapshot>::Label;
  type State = Store<Memory, Domain, Event>;

  fn label(&mut self) -> Self::Label {
    self.frozen_memory.label()
  }

  fn restore(self, label: Self::Label) -> Self::State {
    Store::from_memory(self.frozen_memory.restore(label))
  }
}

#[cfg(test)]
pub mod test {
  use kernel::Alloc;
  use variable::VStoreFD;
  use variable::ops::*;
  use term::ops::*;
  use term::identity::*;
  use propagation::events::*;
  use propagation::events::FDEvent::*;
  use interval::interval::*;
  use gcollections::ops::*;

  pub type Domain = Interval<i32>;
  pub type VStore = VStoreFD;

  pub fn consume_delta(store: &mut VStore, delta_expected: Vec<(usize, FDEvent)>) {
    let res: Vec<(usize, FDEvent)> = store.drain_delta().collect();
    assert_eq!(res, delta_expected);
    assert!(store.drain_delta().next().is_none());
  }

  #[test]
  fn ordered_assign_10_vars() {
    let dom0_10 = (0, 10).to_interval();
    let mut store = VStore::empty();

    for i in 0..10 {
      assert_eq!(store.alloc(dom0_10), Identity::new(i));
    }
  }

  #[test]
  fn valid_read_update() {
    let dom0_10 = (0, 10).to_interval();
    let dom5_5 = (5, 5).to_interval();
    let mut store = VStore::empty();

    let vars: Vec<_> = (0..10).map(|_| store.alloc(dom0_10)).collect();
    for var in vars {
      assert_eq!(var.read(&store), dom0_10);
      assert_eq!(var.update(&mut store, dom5_5), true);
      assert_eq!(var.read(&store), dom5_5);
    }
  }

  #[test]
  fn empty_update() {
    let mut store = VStore::empty();
    let dom5_5 = (5, 5).to_interval();

    let var = store.alloc(dom5_5);
    assert_eq!(var.update(&mut store, Interval::empty()), false);
  }

  #[test]
  #[should_panic]
  fn empty_assign() {
    let mut store = VStore::empty();
    store.alloc(Interval::<i32>::empty());
  }

  #[test]
  #[should_panic]
  fn non_monotonic_update_singleton() {
    let dom0_10 = (0,10).to_interval();
    let dom11_11 = 11.to_interval();

    let mut store = VStore::empty();
    let var = store.alloc(dom0_10);
    var.update(&mut store, dom11_11);
  }

  #[test]
  #[should_panic]
  fn non_monotonic_update_widen() {
    let dom0_10 = (0,10).to_interval();
    let domm5_15 = (-5, 15).to_interval();

    let mut store = VStore::empty();
    let var = store.alloc(dom0_10);
    var.update(&mut store, domm5_15);
  }

  fn test_op<Op>(test_num: u32, source: Domain, target: Domain, delta_expected: Vec<FDEvent>, update_success: bool, op: Op) where
    Op: FnOnce(&VStore, Identity<Domain>) -> Domain
  {
    println!("Test number {}", test_num);
    let mut store = VStore::empty();
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

  fn test_binary_op<Op>(source1: Domain, source2: Domain, target: Domain, delta_expected: Vec<(usize, FDEvent)>, update_success: bool, op: Op) where
    Op: FnOnce(&VStore, Identity<Domain>, Identity<Domain>) -> Domain
  {
    let mut store = VStore::empty();
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

  #[test]
  fn var_update_test() {
    let dom0_10 = (0,10).to_interval();
    let dom0_9 = (0,5).to_interval();
    let dom1_10 = (5,10).to_interval();
    let dom1_9 = (1,9).to_interval();
    let dom0_0 = (0,0).to_interval();
    let empty = Interval::empty();

    var_update_test_one(1, dom0_10, dom0_10, vec![], true);
    var_update_test_one(2, dom0_10, empty, vec![], false);
    var_update_test_one(3, dom0_10, dom0_0, vec![Assignment], true);
    var_update_test_one(4, dom0_10, dom1_10, vec![Bound], true);
    var_update_test_one(5, dom0_10, dom0_9, vec![Bound], true);
    var_update_test_one(6, dom0_10, dom1_9, vec![Bound], true);
  }

  fn var_update_test_one(test_num: u32, source: Domain, target: Domain, delta_expected: Vec<FDEvent>, update_success: bool) {
    test_op(test_num, source, target, delta_expected, update_success, |_,_| target);
  }

  #[test]
  fn var_shrink_bound() {
    let dom0_10 = (0,10).to_interval();

    var_shrink_lb_test_one(1, dom0_10, 0, vec![], true);
    var_shrink_lb_test_one(2, dom0_10, 10, vec![Assignment], true);
    var_shrink_lb_test_one(3, dom0_10, 1, vec![Bound], true);
    var_shrink_lb_test_one(4, dom0_10, 11, vec![], false);

    var_shrink_ub_test_one(5, dom0_10, 10, vec![], true);
    var_shrink_ub_test_one(6, dom0_10, 0, vec![Assignment], true);
    var_shrink_ub_test_one(7, dom0_10, 1, vec![Bound], true);
    var_shrink_ub_test_one(8, dom0_10, -1, vec![], false);
  }

  fn var_shrink_lb_test_one(test_num: u32, source: Domain, target_lb: i32, delta_expected: Vec<FDEvent>, update_success: bool) {
    let expected_dom = (target_lb, source.upper()).to_interval();

    test_op(test_num, source, expected_dom, delta_expected, update_success,
      |store, var| var.read(store).shrink_left(target_lb));
  }

  fn var_shrink_ub_test_one(test_num: u32, source: Domain, target_ub: i32, delta_expected: Vec<FDEvent>, update_success: bool) {
    let expected_dom = (source.lower(), target_ub).to_interval();

    test_op(test_num, source, expected_dom, delta_expected, update_success,
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

  fn var_intersection_test_one(source1: Domain, source2: Domain, target: Domain, delta_expected: Vec<(usize, FDEvent)>, update_success: bool) {
    test_binary_op(source1, source2, target, delta_expected, update_success,
      |store, v1, v2| v1.read(store).intersection(&v2.read(store)));
  }
}