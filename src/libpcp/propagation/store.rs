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

//! Represents the *constraint store* which is a conjunction of constraints, it also comes with an algorithm checking the consistency of the store. It is not a complete method for solving a constraint problem because the output can be `Unknown`. A complete solver is obtained using a search algorithm on top of the consistency algorithm.

use bit_set::BitSet;
use gcollections::kind::*;
use gcollections::ops::*;
use kernel::*;
use model::*;
use propagation::concept::*;
use propagation::ops::*;
use propagation::Reactor;
use propagation::Scheduler;
use std::ops::{Index, IndexMut};
use trilean::SKleene;
use trilean::SKleene::*;
use variable::ops::*;

#[derive(Debug)]
pub struct Store<VStore, Event, Reactor, Scheduler> {
    propagators: Vec<Box<dyn PropagatorConcept<VStore, Event> + 'static>>,
    active: BitSet,
    reactor: Reactor,
    scheduler: Scheduler,
}

impl<VStore, Event, R, S> Empty for Store<VStore, Event, R, S>
where
    Event: EventIndex,
    R: Reactor,
    S: Scheduler,
{
    fn empty() -> Store<VStore, Event, R, S> {
        Store {
            propagators: vec![],
            active: BitSet::new(),
            reactor: Reactor::new(0, 0),
            scheduler: Scheduler::new(0),
        }
    }
}

impl<VStore, Event, R, S> Collection for Store<VStore, Event, R, S> {
    type Item = Box<dyn PropagatorConcept<VStore, Event>>;
}

impl<VStore, Event, R, S> AssociativeCollection for Store<VStore, Event, R, S> {
    type Location = usize;
}

impl<VStore, Event, R, S> Cardinality for Store<VStore, Event, R, S> {
    type Size = usize;

    fn size(&self) -> usize {
        self.propagators.len()
    }
}

impl<VStore, Event, R, S> Store<VStore, Event, R, S> {
    fn display_constraints(&self, model: &Model, indexes: Vec<usize>, header: &str) {
        let header_width = 15;
        print!("{:>width$} ", header, width = header_width);
        let mut idx = 0;
        while idx < indexes.len() {
            self.propagators[indexes[idx]].display(model);
            if idx < indexes.len() - 1 {
                print!(" /\\ \n{:>width$} ", "", width = header_width);
            }
            idx += 1;
        }
        println!();
    }
}

impl<VStore, Event, R, S> DisplayStateful<(Model, VStore)> for Store<VStore, Event, R, S> {
    fn display(&self, (model, vstore): &(Model, VStore)) {
        let mut subsumed = vec![];
        let mut unknown = vec![];
        let mut unsatisfiable = vec![];
        for (i, p) in self.propagators.iter().enumerate() {
            match p.is_subsumed(vstore) {
                False => unsatisfiable.push(i),
                True => subsumed.push(i),
                Unknown => unknown.push(i),
            };
        }
        self.display_constraints(model, unsatisfiable, "unsatisfiable:");
        self.display_constraints(model, subsumed, "subsumed:");
        self.display_constraints(model, unknown, "unknown:");
    }
}

impl<VStore, Event, R, S> DisplayStateful<Model> for Store<VStore, Event, R, S> {
    fn display(&self, model: &Model) {
        let mut i = 0;
        while i < self.propagators.len() {
            self.propagators[i].display(model);
            if i < self.propagators.len() - 1 {
                print!(" /\\ ");
            }
            i += 1;
        }
    }
}

impl<VStore, Event, R, S> Store<VStore, Event, R, S>
where
    VStore: Cardinality<Size = usize> + DrainDelta<Event>,
    Event: EventIndex,
    R: Reactor + Cardinality<Size = usize>,
    S: Scheduler,
{
    fn prepare(&mut self, vstore: &VStore) {
        self.init_reactor(vstore);
        self.init_scheduler();
    }

    fn init_reactor(&mut self, vstore: &VStore) {
        self.reactor = Reactor::new(vstore.size(), Event::size());
        for p_idx in self.active.iter() {
            let p_deps = self[p_idx].dependencies();
            for (v, ev) in p_deps {
                debug_assert!(v < vstore.size(),
          "The propagator {:?} has a dependency to the variable {} which is not in the vstore (of size {}).\n\
          Hint: you should not manually create `Identity` struct, if you do make sure they contain relevant index to the variable vstore.",
          self[p_idx], v, vstore.size());
                self.reactor.subscribe(v, ev, p_idx);
            }
        }
    }

    fn init_scheduler(&mut self) {
        self.scheduler = Scheduler::new(self.propagators.len());
        for p_idx in self.active.iter() {
            self.scheduler.schedule(p_idx);
        }
    }

    fn propagation_loop(&mut self, vstore: &mut VStore) -> bool {
        let mut consistent = true;
        while !self.scheduler.is_empty() && consistent {
            while let Some(p_idx) = self.scheduler.pop() {
                if !self.propagate_one(p_idx, vstore) {
                    consistent = false;
                    break;
                }
                self.react(vstore);
            }
            // self.react(vstore); // For bulk reaction.
        }
        consistent
    }

    fn propagate_one(&mut self, p_idx: usize, vstore: &mut VStore) -> bool {
        vstore.reset_changed();
        let subsumed = self.propagator_consistency(p_idx, vstore);
        match subsumed {
            False => return false,
            True => self.unlink_prop(p_idx),
            Unknown => self.reschedule_prop(p_idx, vstore),
        };
        true
    }

    fn propagator_consistency(&mut self, p_idx: usize, vstore: &mut VStore) -> SKleene {
        if self[p_idx].propagate(vstore) {
            self[p_idx].is_subsumed(vstore)
        } else {
            False
        }
    }

    fn reschedule_prop(&mut self, p_idx: usize, vstore: &mut VStore) {
        if vstore.has_changed() {
            self.scheduler.schedule(p_idx);
        }
    }

    fn react(&mut self, vstore: &mut VStore) {
        for (v, ev) in vstore.drain_delta() {
            let reactions = self.reactor.react(v, ev);
            for p in reactions.into_iter() {
                self.scheduler.schedule(p);
            }
        }
    }

    fn unlink_prop(&mut self, p_idx: usize) {
        self.active.remove(p_idx);
        self.scheduler.unschedule(p_idx);
        let deps = self[p_idx].dependencies();
        for &(var, ev) in deps.iter() {
            self.reactor.unsubscribe(var, ev, p_idx)
        }
    }
}

impl<VStore, Event, R, S> Index<usize> for Store<VStore, Event, R, S> {
    type Output = Box<dyn PropagatorConcept<VStore, Event> + 'static>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.propagators[index]
    }
}

impl<VStore, Event, R, S> IndexMut<usize> for Store<VStore, Event, R, S> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.propagators[index]
    }
}

impl<VStore, Event, R, S> Alloc for Store<VStore, Event, R, S> {
    fn alloc(&mut self, p: Self::Item) -> usize {
        let idx = self.propagators.len();
        self.propagators.push(p);
        self.active.insert(idx);
        idx
    }
}

impl<VStore, Event, R, S> Subsumption<VStore> for Store<VStore, Event, R, S> {
    fn is_subsumed(&self, vstore: &VStore) -> SKleene {
        self.propagators
            .iter()
            .fold(True, |x, p| x.and(p.is_subsumed(vstore)))
    }
}

impl<VStore, Event, R, S> Consistency<VStore> for Store<VStore, Event, R, S>
where
    VStore: Cardinality<Size = usize> + DrainDelta<Event>,
    Event: EventIndex,
    R: Reactor + Cardinality<Size = usize>,
    S: Scheduler,
{
    fn consistency(&mut self, vstore: &mut VStore) -> SKleene {
        self.prepare(vstore);
        let consistent = self.propagation_loop(vstore);
        if !consistent {
            False
        } else if self.reactor.is_empty() {
            True
        } else {
            Unknown
        }
    }
}

impl<VStore, Event, R, S> Clone for Store<VStore, Event, R, S>
where
    Event: EventIndex,
    R: Reactor,
    S: Scheduler,
{
    fn clone(&self) -> Self {
        let mut cstore = Store::empty();
        cstore.propagators = self.propagators.iter().map(|p| p.bclone()).collect();
        cstore.active = self.active.clone();
        cstore
    }
}

impl<VStore, Event, R, S> Freeze for Store<VStore, Event, R, S>
where
    Event: EventIndex,
    R: Reactor + Clone,
    S: Scheduler,
{
    type FrozenState = FrozenStore<VStore, Event, R, S>;
    fn freeze(self) -> Self::FrozenState {
        FrozenStore::new(self)
    }
}

pub struct FrozenStore<VStore, Event, R, S>
where
    Event: EventIndex,
    R: Reactor + Clone,
    S: Scheduler,
{
    cstore: Store<VStore, Event, R, S>,
}

impl<VStore, Event, R, S> FrozenStore<VStore, Event, R, S>
where
    Event: EventIndex,
    R: Reactor + Clone,
    S: Scheduler,
{
    fn new(cstore: Store<VStore, Event, R, S>) -> Self {
        FrozenStore { cstore }
    }
}

impl<VStore, Event, R, S> Snapshot for FrozenStore<VStore, Event, R, S>
where
    Event: EventIndex,
    R: Reactor + Clone,
    S: Scheduler,
{
    type Label = (usize, BitSet);
    type State = Store<VStore, Event, R, S>;

    fn label(&mut self) -> Self::Label {
        (self.cstore.propagators.len(), self.cstore.active.clone())
    }

    fn restore(mut self, label: Self::Label) -> Self::State {
        self.cstore.propagators.truncate(label.0);
        self.cstore.active = label.1;
        self.cstore
    }
}

// #[cfg(test)]
// mod test {
//   use kernel::*;
//   use trilean::SKleene::*;
//   use variable::VStoreFD;
//   use propagation::*;
//   use propagators::cmp::*;
//   use propagators::distinct::*;
//   use term::addition::Addition;
//   use interval::interval::*;
//   use interval::ops::*;
//   use gcollections::ops::*;

//   type VStore = VStoreFD;
//   type CStore = CStoreFD<VStore>;

//   #[test]
//   fn basic_test() {
//     let variables: &mut VStore = &mut VStore::empty();
//     let mut constraints: CStore = CStore::empty();

//     assert_eq!(constraints.consistency(variables), True);

//     let var1 = variables.alloc(Interval::new(1,4));
//     let var2 = variables.alloc(Interval::new(1,4));
//     let var3 = variables.alloc(Interval::new(1,1));

//     assert_eq!(constraints.consistency(variables), True);

//     constraints.alloc(Box::new(XLessY::new(var1.clone(), var2)));
//     assert_eq!(constraints.consistency(variables), Unknown);

//     constraints.alloc(Box::new(XEqY::new(var1, var3)));
//     assert_eq!(constraints.consistency(variables), True);
//   }

//   fn chained_lt(n: usize, expect: SKleene) {
//     // X1 < X2 < X3 < ... < XN, all in dom [1, 10]
//     let variables: &mut VStore = &mut VStore::empty();
//     let mut constraints: CStore = CStore::empty();
//     let mut vars = vec![];
//     for _ in 0..n {
//       vars.push(variables.alloc(Interval::new(1,10)));
//     }
//     for i in 0..n-1 {
//       constraints.alloc(Box::new(XLessY::new(vars[i].clone(), vars[i+1].clone())));
//     }
//     assert_eq!(constraints.consistency(variables), expect);
//   }

//   #[test]
//   fn chained_lt_tests() {
//     chained_lt(1, True);
//     chained_lt(2, Unknown);
//     chained_lt(5, Unknown);
//     chained_lt(9, Unknown);
//     chained_lt(10, True);
//     chained_lt(11, False);
//   }

//   #[test]
//   fn example_nqueens() {
//     nqueens(1, True);
//     nqueens(2, Unknown);
//     nqueens(3, Unknown);
//     nqueens(4, Unknown);
//   }

//   fn nqueens(n: usize, expect: SKleene) {
//     let variables: &mut VStore = &mut VStore::empty();
//     let mut constraints: CStore = CStore::empty();
//     let mut queens = vec![];
//     // 2 queens can't share the same line.
//     for _ in 0..n {
//       queens.push(variables.alloc((1, n as i32).to_interval()));
//     }
//     for i in 0..n-1 {
//       for j in i + 1..n {
//         // 2 queens can't share the same diagonal.
//         let q1 = (i + 1) as i32;
//         let q2 = (j + 1) as i32;
//         // Xi + i != Xj + j
//         constraints.alloc(Box::new(XNeqY::new(Addition::new(queens[i], q1), Addition::new(queens[j], q2))));
//         // constraints.alloc(XNeqY::new(queens[i].clone(), Addition::new(queens[j].clone(), q2 - q1)));
//         // Xi - i != Xj - j
//         constraints.alloc(Box::new(XNeqY::new(queens[i].clone(), Addition::new(queens[j].clone(), -q2 + q1))));
//       }
//     }
//     // 2 queens can't share the same column.
//     constraints.alloc(Box::new(Distinct::new(queens)));
//     assert_eq!(constraints.consistency(variables), expect);
//   }
// }
