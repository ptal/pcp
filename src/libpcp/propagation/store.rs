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
use kernel::Trilean::*;
use propagation::Reactor;
use propagation::Scheduler;
use propagation::propagator::*;
use variable::ops::*;
use interval::ncollections::ops::*;

pub trait BoxedDeepClone<VStore, Event>
{
  fn boxed_deep_clone(&self) -> Box<PropagatorRequirements<VStore, Event>>;
}

impl<VStore, Event, R> BoxedDeepClone<VStore, Event> for R where
  R: DeepClone,
  R: Consistency<VStore>,
  R: PropagatorDependencies<Event>,
  R: 'static
{
  fn boxed_deep_clone(&self) -> Box<PropagatorRequirements<VStore, Event>> {
    Box::new(self.deep_clone())
  }
}

pub trait PropagatorRequirements<VStore, Event> :
    Consistency<VStore>
  + PropagatorDependencies<Event>
  + BoxedDeepClone<VStore, Event>
{}

impl<VStore, Event, R> PropagatorRequirements<VStore, Event> for R where
 R: Consistency<VStore>,
 R: PropagatorDependencies<Event>,
 R: BoxedDeepClone<VStore, Event>
{}

pub struct Store<VStore, Event, Reactor, Scheduler>
{
  propagators: Vec<Box<PropagatorRequirements<VStore, Event> + 'static>>,
  reactor: Reactor,
  scheduler: Scheduler
}

impl<VStore, Event, R, S> Store<VStore, Event, R, S> where
 Event: EventIndex,
 R: Reactor,
 S: Scheduler
{
  pub fn new() -> Store<VStore, Event, R, S> {
    Store {
      propagators: vec![],
      reactor: Reactor::new(0,0),
      scheduler: Scheduler::new(0)
    }
  }
}

impl<VStore, Event, R, S> Store<VStore, Event, R, S> where
 VStore: Cardinality<Size=usize> + DrainDelta<Event>,
 Event: EventIndex,
 R: Reactor,
 S: Scheduler
{
  fn prepare(&mut self, store: &VStore) {
    self.init_reactor(store);
    self.init_scheduler();
  }

  fn init_reactor(&mut self, store: &VStore) {
    self.reactor = Reactor::new(store.size(), Event::size());
    for (p_idx, p) in self.propagators.iter().enumerate() {
      let p_deps = p.dependencies();
      for (v, ev) in p_deps {
        self.reactor.subscribe(v as usize, ev, p_idx);
      }
    }
  }

  fn init_scheduler(&mut self) {
    let num_props = self.propagators.len();
    self.scheduler = Scheduler::new(num_props);
    for p_idx in 0..num_props {
      self.scheduler.schedule(p_idx);
    }
  }

  fn propagation_loop(&mut self, store: &mut VStore) -> Trilean {
    let mut unsatisfiable = false;
    while let Some(p_idx) = self.scheduler.pop() {
      if !self.propagate_one(p_idx, store) {
        unsatisfiable = true;
        break;
      }
    }
    if unsatisfiable { False }
    else if self.reactor.is_empty() { True }
    else { Unknown }
  }

  fn propagate_one(&mut self, p_idx: usize, store: &mut VStore) -> bool {
    let subsumed = self.propagators[p_idx].consistency(store);
    match subsumed {
      False => return false,
      True => self.unlink_prop(p_idx),
      Unknown => self.reschedule_prop(p_idx, store)
    };
    self.react(store);
    true
  }

  fn reschedule_prop(&mut self, p_idx: usize, store: &mut VStore) {
    if !store.drain_delta().peekable().is_empty() {
      self.scheduler.schedule(p_idx);
    }
  }

  fn react(&mut self, store: &mut VStore) {
    for (v, ev) in store.drain_delta() {
      let reactions = self.reactor.react(v, ev);
      for p in reactions.into_iter() {
        self.scheduler.schedule(p);
      }
    }
  }

  fn unlink_prop(&mut self, p_idx: usize) {
    self.scheduler.unschedule(p_idx);
    let deps = self.propagators[p_idx].dependencies();
    for &(var, ev) in deps.iter() {
      self.reactor.unsubscribe(var, ev, p_idx)
    }
  }
}

impl<Prop, VStore, Event, R, S> Assign<Prop> for Store<VStore, Event, R, S> where
 Prop: PropagatorRequirements<VStore, Event> + 'static
{
  type Variable = ();
  fn assign(&mut self, p: Prop) {
    self.propagators.push(Box::new(p));
  }
}

impl<VStore, Event, R, S> Consistency<VStore> for Store<VStore, Event, R, S> where
 VStore: Cardinality<Size=usize> + DrainDelta<Event>,
 Event: EventIndex,
 R: Reactor,
 S: Scheduler
{
  fn consistency(&mut self, store: &mut VStore) -> Trilean {
    self.prepare(store);
    self.propagation_loop(store)
  }
}

impl<VStore, Event, R, S> State for Store<VStore, Event, R, S> where
 Event: EventIndex,
 R: Reactor,
 S: Scheduler
{
  type Label = Store<VStore, Event, R, S>;

  fn mark(&self) -> Store<VStore, Event, R, S> {
    self.deep_clone()
  }

  fn restore(self, label: Store<VStore, Event, R, S>) -> Self {
    label
  }
}

impl<VStore, Event, R, S> DeepClone for Store<VStore, Event, R, S> where
 Event: EventIndex,
 R: Reactor,
 S: Scheduler
{
  fn deep_clone(&self) -> Self {
    let mut store = Store::new();
    store.propagators = self.propagators.iter()
      .map(|p| p.boxed_deep_clone())
      .collect();
    store
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use interval::interval::*;
  use interval::ops::*;
  use kernel::*;
  use kernel::Trilean::*;
  use propagation::events::*;
  use propagation::reactors::*;
  use propagation::schedulers::*;
  use propagators::cmp::*;
  use propagators::distinct::*;
  use variable::ops::*;
  use variable::arithmetics::*;
  use variable::delta_store::DeltaStore;

  type VStore = DeltaStore<Interval<i32>, FDEvent>;
  type CStore = Store<VStore, FDEvent, IndexedDeps, RelaxedFifo>;

  #[test]
  fn basic_test() {
    let variables: &mut VStore = &mut VStore::new();
    let mut constraints: CStore = CStore::new();

    assert_eq!(constraints.consistency(variables), True);

    let var1 = variables.assign(Interval::new(1,4));
    let var2 = variables.assign(Interval::new(1,4));
    let var3 = variables.assign(Interval::new(1,1));

    assert_eq!(constraints.consistency(variables), True);

    constraints.assign(XLessY::new(var1.clone(), var2));
    assert_eq!(constraints.consistency(variables), Unknown);

    constraints.assign(XEqY::new(var1, var3));
    assert_eq!(constraints.consistency(variables), True);
  }

  fn chained_lt(n: usize, expect: Trilean) {
    // X1 < X2 < X3 < ... < XN, all in dom [1, 10]
    let variables: &mut VStore = &mut VStore::new();
    let mut constraints: CStore = CStore::new();
    let mut vars = vec![];
    for _ in 0..n {
      vars.push(variables.assign(Interval::new(1,10)));
    }
    for i in 0..n-1 {
      constraints.assign(XLessY::new(vars[i].clone(), vars[i+1].clone()));
    }
    assert_eq!(constraints.consistency(variables), expect);
  }

  #[test]
  fn chained_lt_tests() {
    chained_lt(1, True);
    chained_lt(2, Unknown);
    chained_lt(5, Unknown);
    chained_lt(9, Unknown);
    chained_lt(10, True);
    chained_lt(11, False);
  }

  #[test]
  fn example_nqueens() {
    nqueens(1, True);
    nqueens(2, Unknown);
    nqueens(3, Unknown);
    nqueens(4, Unknown);
  }

  fn nqueens(n: usize, expect: Trilean) {
    let variables: &mut VStore = &mut VStore::new();
    let mut constraints: CStore = CStore::new();
    let mut queens = vec![];
    // 2 queens can't share the same line.
    for _ in 0..n {
      queens.push(variables.assign((1, n as i32).to_interval()));
    }
    for i in 0..n-1 {
      for j in i + 1..n {
        // 2 queens can't share the same diagonal.
        let q1 = (i + 1) as i32;
        let q2 = (j + 1) as i32;
        // Xi + i != Xj + j
        constraints.assign(XNeqY::new(queens[i].clone(), Addition::new(queens[j].clone(), q2 - q1)));
        // Xi - i != Xj - j
        constraints.assign(XNeqY::new(queens[i].clone(), Addition::new(queens[j].clone(), -q2 + q1)));
      }
    }
    // 2 queens can't share the same column.
    constraints.assign(Distinct::new(queens));
    assert_eq!(constraints.consistency(variables), expect);
  }
}
