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

use gcollections::kind::*;
use gcollections::ops::*;
use kernel::*;
use model::*;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::ops::Index;
use std::slice;
use term::identity::*;
use variable::concept::*;
use variable::ops::*;
use vec_map::{Drain, VecMap};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Store<Memory, Event> {
    memory: Memory,
    delta: VecMap<Event>,
    has_changed: bool,
}

impl<Memory, Event> Collection for Store<Memory, Event>
where
    Memory: MemoryConcept,
{
    type Item = <Memory as Collection>::Item;
}

impl<Memory, Event> AssociativeCollection for Store<Memory, Event>
where
    Memory: MemoryConcept,
{
    type Location = Identity<<Memory as Collection>::Item>;
}

impl<Memory, Event, Domain> ImmutableMemoryConcept for Store<Memory, Event>
where
    Memory: MemoryConcept<Item = Domain>,
    Event: Debug,
{
}

impl<Memory, Domain, Bound, Event> VStoreConcept for Store<Memory, Event>
where
    Memory: MemoryConcept<Item = Domain>,
    Domain: Subset + Cardinality + Bounded<Item = Bound> + Display,
    Event: EventConcept<Domain>,
{
}

impl<Memory, Event> Store<Memory, Event>
where
    Memory: MemoryConcept,
{
    fn from_memory(memory: Memory) -> Self {
        Store {
            memory,
            delta: VecMap::new(),
            has_changed: false,
        }
    }
}

impl<Memory, Event> Empty for Store<Memory, Event>
where
    Memory: MemoryConcept,
{
    fn empty() -> Store<Memory, Event> {
        Store::from_memory(Memory::empty())
    }
}

impl<Memory, Domain, Event> Store<Memory, Event>
where
    Memory: MemoryConcept,
    Memory: Collection<Item = Domain>,
    Domain: Subset + Cardinality + Bounded,
    Event: EventConcept<Domain>,
{
    // FIXME: Need a rustc fix on borrowing rule, `updated` not needed.
    fn update_delta(&mut self, key: usize, old_dom: &Domain) {
        if let Some(delta) = Event::new(&self[key], old_dom) {
            self.has_changed = true;
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

impl<Memory, Event> Cardinality for Store<Memory, Event>
where
    Memory: MemoryConcept,
{
    type Size = usize;

    fn size(&self) -> usize {
        self.memory.size()
    }
}

impl<Memory, Event> Iterable for Store<Memory, Event>
where
    Memory: MemoryConcept,
{
    fn iter(&self) -> slice::Iter<'_, Self::Item> {
        self.memory.iter()
    }
}

impl<Memory, Domain, Event> Alloc for Store<Memory, Event>
where
    Memory: MemoryConcept,
    Memory: Collection<Item = Domain>,
    Domain: Cardinality + IsEmpty,
{
    fn alloc(&mut self, dom: Self::Item) -> Self::Location {
        assert!(!dom.is_empty());
        let var_idx = self.memory.size();
        self.memory.push(dom);
        Identity::new(var_idx)
    }
}

impl<Memory, Domain, Event> MonotonicUpdate for Store<Memory, Event>
where
    Memory: MemoryConcept,
    Memory: Collection<Item = Domain>,
    Domain: Subset + Cardinality + Bounded,
    Event: EventConcept<Domain>,
{
    // We update the domain located at `loc` if `dom` is not empty and is a strictly smaller than the current value.
    fn update(&mut self, loc: &Identity<Domain>, dom: Self::Item) -> bool {
        let idx = loc.index();
        assert!(
            dom.is_subset(&self.memory[idx]),
            "Domain update must be monotonic."
        );
        if dom.is_empty() {
            false
        } else {
            if dom.size() < self[idx].size() {
                let old_dom = self.memory.replace(idx, dom);
                self.update_delta(idx, &old_dom);
            }
            true
        }
    }
}

impl<Memory, Event> Index<usize> for Store<Memory, Event>
where
    Memory: MemoryConcept,
{
    type Output = <Memory as Collection>::Item;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(
            index < self.memory.size(),
            "Variable not registered in the store. Variable index must be obtained with `alloc`."
        );
        &self.memory[index]
    }
}

impl<Memory, Event, Domain> Store<Memory, Event>
where
    Memory: MemoryConcept<Item = Domain>,
    Domain: Display + IsSingleton,
{
    fn display_var_matrix(&self, model: &Model, var_idx: Vec<usize>, header: &str) {
        let header_width = 15;
        let var_width = 20;
        let num_columns = 8;
        print!("{:>width$} ", header, width = header_width);
        for (i, idx) in var_idx.into_iter().enumerate() {
            if (i + 1) % (num_columns + 1) == 0 {
                print!("\n{:>width$} ", "", width = header_width);
            }
            let var_str = format!("{:<6} = {}", model.var_name(idx), self[idx]);
            print!("{:<width$}", var_str, width = var_width);
        }
        println!();
    }
}

impl<Memory, Event, Domain> DisplayStateful<Model> for Store<Memory, Event>
where
    Memory: MemoryConcept<Item = Domain>,
    Domain: Display + IsSingleton,
{
    fn display(&self, model: &Model) {
        let mut idx_assigned = vec![];
        let mut idx_others = vec![];
        for (i, dom) in self.memory.iter().enumerate() {
            if dom.is_singleton() {
                idx_assigned.push(i);
            } else {
                idx_others.push(i);
            }
        }
        self.display_var_matrix(model, idx_assigned, "assigned:");
        self.display_var_matrix(model, idx_others, "not assigned:");
    }
}

impl<Memory, Event> DrainDelta<Event> for Store<Memory, Event> {
    fn drain_delta(&mut self) -> Drain<'_, Event> {
        self.delta.drain()
    }

    fn has_changed(&self) -> bool {
        self.has_changed
    }

    fn reset_changed(&mut self) {
        self.has_changed = false;
    }
}

impl<Memory, Event> Freeze for Store<Memory, Event>
where
    Memory: MemoryConcept,
{
    type FrozenState = FrozenStore<Memory, Event>;
    fn freeze(self) -> Self::FrozenState {
        FrozenStore::new(self)
    }
}

pub struct FrozenStore<Memory, Event>
where
    Memory: MemoryConcept,
{
    frozen_memory: Memory::FrozenState,
    phantom_event: PhantomData<Event>,
}

impl<Memory, Event> FrozenStore<Memory, Event>
where
    Memory: MemoryConcept,
{
    fn new(store: Store<Memory, Event>) -> Self {
        FrozenStore {
            frozen_memory: store.memory.freeze(),
            phantom_event: PhantomData,
        }
    }
}

impl<Memory, Event> Snapshot for FrozenStore<Memory, Event>
where
    Memory: MemoryConcept,
{
    type Label = <Memory::FrozenState as Snapshot>::Label;
    type State = Store<Memory, Event>;

    fn label(&mut self) -> Self::Label {
        self.frozen_memory.label()
    }

    fn restore(self, label: Self::Label) -> Self::State {
        Store::from_memory(self.frozen_memory.restore(label))
    }
}

#[cfg(test)]
pub mod test {
    use gcollections::ops::*;
    use interval::interval::*;
    use propagation::events::FDEvent::*;
    use propagation::events::*;
    use term::identity::*;
    use term::ops::*;
    use variable::ops::*;
    use variable::VStoreFD;

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
        for mut var in vars {
            assert_eq!(var.read(&store), dom0_10);
            assert_eq!(var.update(&mut store, dom5_5), true);
            assert_eq!(var.read(&store), dom5_5);
        }
    }

    #[test]
    fn empty_update() {
        let mut store = VStore::empty();
        let dom5_5 = (5, 5).to_interval();

        let mut var = store.alloc(dom5_5);
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
        let dom0_10 = (0, 10).to_interval();
        let dom11_11 = 11.to_interval();

        let mut store = VStore::empty();
        let mut var = store.alloc(dom0_10);
        var.update(&mut store, dom11_11);
    }

    #[test]
    #[should_panic]
    fn non_monotonic_update_widen() {
        let dom0_10 = (0, 10).to_interval();
        let domm5_15 = (-5, 15).to_interval();

        let mut store = VStore::empty();
        let mut var = store.alloc(dom0_10);
        var.update(&mut store, domm5_15);
    }

    fn test_op<Op>(
        test_num: u32,
        source: Domain,
        target: Domain,
        delta_expected: Vec<FDEvent>,
        update_success: bool,
        op: Op,
    ) where
        Op: FnOnce(&VStore, Identity<Domain>) -> Domain,
    {
        println!("Test number {}", test_num);
        let mut store = VStore::empty();
        let mut var = store.alloc(source);

        let new = op(&store, var);
        assert_eq!(var.update(&mut store, new), update_success);
        assert_eq!(new, target);

        if update_success {
            let delta_expected = delta_expected
                .into_iter()
                .map(|d| (var.index(), d))
                .collect();
            consume_delta(&mut store, delta_expected);
            assert_eq!(var.read(&store), target);
        }
    }

    fn test_binary_op<Op>(
        source1: Domain,
        source2: Domain,
        target: Domain,
        delta_expected: Vec<(usize, FDEvent)>,
        update_success: bool,
        op: Op,
    ) where
        Op: FnOnce(&VStore, Identity<Domain>, Identity<Domain>) -> Domain,
    {
        let mut store = VStore::empty();
        let mut var1 = store.alloc(source1);
        let mut var2 = store.alloc(source2);

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
        let dom0_10 = (0, 10).to_interval();
        let dom0_9 = (0, 5).to_interval();
        let dom1_10 = (5, 10).to_interval();
        let dom1_9 = (1, 9).to_interval();
        let dom0_0 = (0, 0).to_interval();
        let empty = Interval::empty();

        var_update_test_one(1, dom0_10, dom0_10, vec![], true);
        var_update_test_one(2, dom0_10, empty, vec![], false);
        var_update_test_one(3, dom0_10, dom0_0, vec![Assignment], true);
        var_update_test_one(4, dom0_10, dom1_10, vec![Bound], true);
        var_update_test_one(5, dom0_10, dom0_9, vec![Bound], true);
        var_update_test_one(6, dom0_10, dom1_9, vec![Bound], true);
    }

    fn var_update_test_one(
        test_num: u32,
        source: Domain,
        target: Domain,
        delta_expected: Vec<FDEvent>,
        update_success: bool,
    ) {
        test_op(
            test_num,
            source,
            target,
            delta_expected,
            update_success,
            |_, _| target,
        );
    }

    #[test]
    fn var_shrink_bound() {
        let dom0_10 = (0, 10).to_interval();

        var_shrink_lb_test_one(1, dom0_10, 0, vec![], true);
        var_shrink_lb_test_one(2, dom0_10, 10, vec![Assignment], true);
        var_shrink_lb_test_one(3, dom0_10, 1, vec![Bound], true);
        var_shrink_lb_test_one(4, dom0_10, 11, vec![], false);

        var_shrink_ub_test_one(5, dom0_10, 10, vec![], true);
        var_shrink_ub_test_one(6, dom0_10, 0, vec![Assignment], true);
        var_shrink_ub_test_one(7, dom0_10, 1, vec![Bound], true);
        var_shrink_ub_test_one(8, dom0_10, -1, vec![], false);
    }

    fn var_shrink_lb_test_one(
        test_num: u32,
        source: Domain,
        target_lb: i32,
        delta_expected: Vec<FDEvent>,
        update_success: bool,
    ) {
        let expected_dom = (target_lb, source.upper()).to_interval();

        test_op(
            test_num,
            source,
            expected_dom,
            delta_expected,
            update_success,
            |store, var| var.read(store).shrink_left(target_lb),
        );
    }

    fn var_shrink_ub_test_one(
        test_num: u32,
        source: Domain,
        target_ub: i32,
        delta_expected: Vec<FDEvent>,
        update_success: bool,
    ) {
        let expected_dom = (source.lower(), target_ub).to_interval();

        test_op(
            test_num,
            source,
            expected_dom,
            delta_expected,
            update_success,
            |store, var| var.read(store).shrink_right(target_ub),
        );
    }

    #[test]
    fn var_intersection_test() {
        let dom0_10 = (0, 10).to_interval();
        let dom10_20 = (10, 20).to_interval();
        let dom10_10 = (10, 10).to_interval();
        let dom11_20 = (11, 20).to_interval();
        let dom1_9 = (1, 9).to_interval();

        var_intersection_test_one(
            dom0_10,
            dom10_20,
            dom10_10,
            vec![(0, Assignment), (1, Assignment)],
            true,
        );
        var_intersection_test_one(dom0_10, dom1_9, dom1_9, vec![(0, Bound)], true);
        var_intersection_test_one(dom1_9, dom0_10, dom1_9, vec![(1, Bound)], true);
        var_intersection_test_one(dom0_10, dom11_20, Interval::empty(), vec![], false);
    }

    fn var_intersection_test_one(
        source1: Domain,
        source2: Domain,
        target: Domain,
        delta_expected: Vec<(usize, FDEvent)>,
        update_success: bool,
    ) {
        test_binary_op(
            source1,
            source2,
            target,
            delta_expected,
            update_success,
            |store, v1, v2| v1.read(store).intersection(&v2.read(store)),
        );
    }
}
