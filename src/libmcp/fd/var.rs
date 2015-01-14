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

pub use interval::interval::*;
pub use event::VarEvent;

use self::FDEvent::*;
use std::cmp::min;
use std::num::FromPrimitive;

// we try to take the most precise first
// which is failure, then Ass, then Bound,...
// the union between two events is min(e1, e2)
#[derive(Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, Show)]
pub enum FDEvent {
  Failure,
  Assignment,
  Bound,
  Inner
}

impl VarEvent for FDEvent {
  fn merge(self, other: FDEvent) -> FDEvent {
    FromPrimitive::from_int(min(self as isize, other as isize)).unwrap()
  }

  fn to_index(self) -> usize {
    self as usize
  }

  fn size() -> usize {
    Inner.to_index() + 1
  }
}

#[derive(Copy, PartialEq, Eq)]
pub struct FDVar {
  id: u32,
  dom: Interval
}

impl FDVar {
  pub fn new(id: u32, dom: Interval) -> FDVar {
    FDVar {
      id: id,
      dom: dom
    }
  }

  pub fn id(&self) -> u32 { self.id }

  // Precondition: Accept only monotonic updates. `dom` must be a subset of self.dom.
  pub fn update(&mut self, dom: Interval, events: &mut Vec<(u32, FDEvent)>) {
    assert!(dom.is_subset_of(self.dom));
    let old = self.dom;
    self.dom = dom;
    if old == dom { return; }
    else {
      let ev =
        if dom.is_empty() { Failure }
        else if dom.is_singleton() { Assignment }
        else if dom.lower() != old.lower() ||
                dom.upper() != old.upper() { Bound }
        else { Inner };
      events.push((self.id, ev));
    }
  }

  pub fn update_lb(&mut self, lb: i32, events: &mut Vec<(u32, FDEvent)>) {
    let ub =  self.dom.upper();
    self.update((lb, ub).to_interval(), events);
  }

  pub fn update_ub(&mut self, ub: i32, events: &mut Vec<(u32, FDEvent)>) {
    let lb = self.dom.lower();
    self.update((lb, ub).to_interval(), events);
  }

  pub fn lb(&self) -> i32 { self.dom.lower() }
  pub fn ub(&self) -> i32 { self.dom.upper() }

  pub fn is_failed(&self) -> bool { self.dom.is_empty() }
  pub fn failed(&mut self, events: &mut Vec<(u32, FDEvent)>) {
    if !self.is_failed() {
      self.dom = Interval::empty();
      events.push((self.id, Failure));
    }
  }

  pub fn intersection(v1: &mut FDVar, v2: &mut FDVar) -> Vec<(u32, FDEvent)> {
    let mut events = vec![];
    let new = v1.dom.intersection(v2.dom);
    v1.update(new, &mut events);
    v2.update(new, &mut events);
    events
  }

  pub fn is_disjoint(v1: &FDVar, v2: &FDVar) -> bool {
    v1.dom.is_disjoint(v2.dom)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use super::FDEvent::*;

  #[test]
  fn event_test() {
    let fail = Failure;
    let ass = Assignment;
    let bound = Bound;
    assert_eq!(fail.merge(ass), Failure);
    assert_eq!(fail.merge(fail), Failure);
    assert_eq!(ass.merge(fail), Failure);
    assert_eq!(bound.merge(fail), Failure);
    assert_eq!(bound.merge(ass), Assignment);
    assert_eq!(ass.merge(bound), Assignment);
  }

  #[test]
  fn var_update_test() {
    let dom0_10 = (0,10).to_interval();
    let dom0_9 = (0,5).to_interval();
    let dom1_10 = (5,10).to_interval();
    let dom1_9 = (1,9).to_interval();
    let dom0_0 = (0,0).to_interval();
    let empty = Interval::empty();
    let var0_10 = FDVar::new(0, dom0_10);

    var_update_test_one(var0_10, dom0_10, vec![]);
    var_update_test_one(var0_10, empty, vec![Failure]);
    var_update_test_one(var0_10, dom0_0, vec![Assignment]);
    var_update_test_one(var0_10, dom1_10, vec![Bound]);
    var_update_test_one(var0_10, dom0_9, vec![Bound]);
    var_update_test_one(var0_10, dom1_9, vec![Bound]);
  }

  fn var_update_test_one(var: FDVar, dom: Interval, expect: Vec<FDEvent>) {
    let mut var = var;
    let mut events = vec![];
    var.update(dom, &mut events);
    assert_eq_events(events, expect);
    assert_eq!(var.dom, dom);
  }

  fn assert_eq_events(events: Vec<(u32, FDEvent)>, expect: Vec<FDEvent>) {
    for ((_,ev), expect) in events.into_iter().zip(expect.into_iter()) {
      assert_eq!(ev, expect);
    }
  }

  #[test]
  fn var_update_bound() {
    let dom0_10 = (0,10).to_interval();
    let var0_10 = FDVar::new(0, dom0_10);

    var_update_lb_test_one(var0_10, 0, vec![]);
    var_update_lb_test_one(var0_10, 10, vec![Assignment]);
    var_update_lb_test_one(var0_10, 1, vec![Bound]);
    var_update_lb_test_one(var0_10, 11, vec![Failure]);

    var_update_ub_test_one(var0_10, 10, vec![]);
    var_update_ub_test_one(var0_10, 0, vec![Assignment]);
    var_update_ub_test_one(var0_10, 1, vec![Bound]);
    var_update_ub_test_one(var0_10, -1, vec![Failure]);
  }

  fn var_update_lb_test_one(var: FDVar, lb: i32, expect: Vec<FDEvent>) {
    let mut var = var;
    let ub = var.ub();
    let mut events = vec![];
    var.update_lb(lb, &mut events);
    assert_eq_events(events, expect);
    assert_eq!(var.dom, (lb,ub).to_interval());
  }

  fn var_update_ub_test_one(var: FDVar, ub: i32, expect: Vec<FDEvent>) {
    let mut var = var;
    let lb = var.lb();
    let mut events = vec![];
    var.update_ub(ub, &mut events);
    assert_eq_events(events, expect);
    assert_eq!(var.dom, (lb,ub).to_interval());
  }

  #[test]
  fn var_intersection_test() {
    let empty = FDVar::new(0, Interval::empty());
    let var0_10 = FDVar::new(1, (0,10).to_interval());
    let var10_20 = FDVar::new(2, (10,20).to_interval());
    let var11_20 = FDVar::new(3, (11,20).to_interval());
    let var1_9 = FDVar::new(4, (1,9).to_interval());

    var_intersection_test_one(var0_10, var10_20, vec![(1, Assignment), (2, Assignment)]);
    var_intersection_test_one(var0_10, var1_9, vec![(1, Bound)]);
    var_intersection_test_one(var1_9, var0_10, vec![(1, Bound)]);
    var_intersection_test_one(var0_10, var11_20, vec![(1, Failure), (3, Failure)]);
    var_intersection_test_one(var0_10, empty, vec![(1, Failure)]);
    var_intersection_test_one(empty, empty, vec![]);
  }

  fn var_intersection_test_one(mut v1: FDVar, mut v2: FDVar, events: Vec<(u32, FDEvent)>) {
    let v1 = &mut v1;
    let v2 = &mut v2;
    assert_eq!(FDVar::intersection(v1, v2), events);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_lb() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10 = FDVar::new(0, dom0_10);

    var0_10.update_lb(-1, &mut vec![]);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_ub() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10 = FDVar::new(0, dom0_10);

    var0_10.update_ub(11, &mut vec![]);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_singleton() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10 = FDVar::new(0, dom0_10);
    let dom11_11 = 11.to_interval();

    var0_10.update(dom11_11, &mut vec![]);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_widen() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10 = FDVar::new(0, dom0_10);
    let domm5_15 = (-5, 15).to_interval();

    var0_10.update(domm5_15, &mut vec![]);
  }
}