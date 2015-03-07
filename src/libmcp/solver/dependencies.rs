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

use solver::event::VarEvent;
use std::iter::{FromIterator, repeat};

pub trait VarEventDependencies {

  fn new(num_vars: usize, num_events: usize) -> Self;
  fn subscribe<E>(&mut self, var: usize, ev: E, prop: usize) where E: VarEvent;
  fn unsubscribe<E>(&mut self, var: usize, ev: E, prop: usize) where E: VarEvent;
  fn react<E>(&self, var: usize, ev: E) -> Vec<usize> where E: VarEvent;
  fn is_empty(&self) -> bool;
}

pub struct VarEventDepsVector {
  num_events: usize,
  num_subscriptions: usize,
  deps: Vec<Vec<usize>>
}

impl VarEventDepsVector {
  fn index_of<E>(&self, var: usize, ev: E) -> usize where E: VarEvent {
    self.num_events*var + ev.to_index()
  }

  fn deps_of_mut<'a, E>(&'a mut self, var: usize, ev: E) -> &'a mut Vec<usize> where E: VarEvent {
    let idx = self.index_of(var, ev);
    &mut self.deps[idx]
  }
}

impl VarEventDependencies for VarEventDepsVector {
  fn new(num_vars: usize, num_events: usize) -> VarEventDepsVector {
    VarEventDepsVector {
      num_events: num_events,
      num_subscriptions: 0,
      deps: FromIterator::from_iter(repeat(vec![]).take(num_vars * num_events))
    }
  }

  fn subscribe<E>(&mut self, var: usize, ev: E, prop: usize) where E: VarEvent {
    assert!(self.deps.iter()
      .skip(var*self.num_events).take(self.num_events)
      .flat_map(|x| x.iter()).all(|&x| x != prop),
      "propagator already subscribed to this variable");
    self.num_subscriptions += 1;
    let mut props = self.deps_of_mut(var, ev);
    props.push(prop);
  }

  fn unsubscribe<E>(&mut self, var: usize, ev: E, prop: usize) where E: VarEvent {
    self.num_subscriptions -= 1;
    let mut props = self.deps_of_mut(var, ev);
    let idx = props.iter().position(|&v| v == prop);
    assert!(idx.is_some(), "cannot unsubscribe propagator not registered.");
    props.swap_remove(idx.unwrap());
  }

  fn react<E>(&self, var: usize, ev: E) -> Vec<usize> where E: VarEvent {
    let from = self.index_of(var, ev);
    let len = self.num_events - ev.to_index();
    self.deps.iter()
      .skip(from).take(len)
      .flat_map(|x| x.iter())
      .cloned()
      .collect()
  }

  fn is_empty(&self) -> bool {
    self.num_subscriptions == 0
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use solver::fd::var::FDEvent;
  use solver::fd::var::FDEvent::*;
  use solver::event::VarEvent;

  fn make_deps() -> VarEventDepsVector {
    let res: VarEventDepsVector = VarEventDependencies::new(3, <FDEvent as VarEvent>::size());
    res
  }

  fn iter_deps(deps: &mut VarEventDepsVector, var: usize, ev: FDEvent, expect: Vec<usize>) {
    let mut it = deps.react(var, ev).into_iter();
    let mut it_e = expect.into_iter();
    loop {
      let next = it.next();
      let next_e = it_e.next();
      if next.is_none() || next_e.is_none() { break; }
      assert_eq!(next.unwrap(), next_e.unwrap());
    }
    assert_eq!(it.next(), None);
    assert_eq!(it_e.next(), None);
  }

  #[test]
  fn subscribe_test() {
    let mut deps = make_deps();
    assert!(deps.is_empty());
    deps.subscribe(0, Assignment, 4);
    assert!(!deps.is_empty());

    iter_deps(&mut deps, 0, Assignment, vec![4]);

    // Assignment is more precise than Inner, so we don't care.
    iter_deps(&mut deps, 0, Inner, vec![]);

    // If we subscribe to Inner, we must react on Inner
    // or more precise events.
    deps.subscribe(0, Inner, 5);
    iter_deps(&mut deps, 0, Inner, vec![5]);
    iter_deps(&mut deps, 0, Assignment, vec![4,5]);

    deps.subscribe(1, Bound, 6);
    deps.subscribe(1, Assignment, 7);
    iter_deps(&mut deps, 1, Inner, vec![]);
    iter_deps(&mut deps, 1, Bound, vec![6]);
    iter_deps(&mut deps, 1, Assignment, vec![7,6]);

    iter_deps(&mut deps, 2, Assignment, vec![]);

    deps.subscribe(2, Assignment, 8);
    iter_deps(&mut deps, 1, Inner, vec![]);
  }

  #[test]
  fn unsubscribe_test() {
    let mut deps = make_deps();
    deps.subscribe(0, Assignment, 4);
    deps.subscribe(0, Bound, 5);

    iter_deps(&mut deps, 0, Assignment, vec![4,5]);

    deps.unsubscribe(0, Bound, 5);
    iter_deps(&mut deps, 0, Assignment, vec![4]);

    deps.unsubscribe(0, Assignment, 4);
    iter_deps(&mut deps, 0, Assignment, vec![]);

    deps.subscribe(0, Assignment, 4);
    iter_deps(&mut deps, 0, Assignment, vec![4]);
  }

  #[test]
  #[should_fail]
  fn subscribe_fail_test() {
    let mut deps = make_deps();

    deps.subscribe(0, Assignment, 0);
    deps.subscribe(0, Assignment, 0);
  }

  #[test]
  #[should_fail]
  fn unsubscribe_fail_test() {
    let mut deps = make_deps();

    deps.unsubscribe(0, Assignment, 0);
  }

  #[test]
  #[should_fail]
  fn subscribe_two_events_fail_test() {
    let mut deps = make_deps();

    deps.subscribe(0, Assignment, 0);
    deps.subscribe(0, Bound, 0); // already subscribed to this variable
  }
}
