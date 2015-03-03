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
pub use solver::event::VarEvent;
pub use solver::variable::Variable;

use self::FDEvent::*;
use std::cmp::min;

// We don't have a Nothing event because some functions have two ways of returning
// 'nothing', with an empty vector or `Nothing`, we select only one.
// We don't have Failure event because it's not an event that propagator
// should subscribe too. If a failure occurs, it's over.
#[derive(Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, Debug)]
pub enum FDEvent {
  Assignment,
  Bound,
  Inner
}

impl FDEvent {
  pub fn merge(self, ev: FDEvent) -> FDEvent {
    min(self, ev)
  }
}

impl VarEvent for FDEvent {
  fn to_index(self) -> usize {
    self as usize
  }

  fn size() -> usize {
    Inner.to_index() + 1
  }
}

#[derive(Copy, PartialEq, Eq, Debug, Clone)]
pub struct FDVar {
  id: u32,
  dom: Interval<i32>
}

impl Variable for FDVar {
  type Domain = Interval<i32>;
  type Event = FDEvent;

  fn new(id: u32, dom: Interval<i32>) -> FDVar {
    assert!(!dom.is_empty());
    FDVar {
      id: id,
      dom: dom
    }
  }
}

impl FDVar {
  pub fn id(&self) -> u32 { self.id }

  // Precondition: Accept only monotonic updates. `dom` must be a subset of self.dom.
  pub fn update(&mut self, dom: Interval<i32>, events: &mut Vec<(u32, FDEvent)>) -> bool {
    assert!(dom.is_subset_of(self.dom));
    if dom.is_empty() { false } // Failure
    else {
      let old = self.dom;
      self.dom = dom;
      if old != dom {
        let ev =
          if dom.is_singleton() { Assignment }
          else if dom.lower() != old.lower() ||
                  dom.upper() != old.upper() { Bound }
          else { Inner };
        events.push((self.id, ev));
      }
      true
    }
  }

  pub fn update_lb(&mut self, lb: i32, events: &mut Vec<(u32, FDEvent)>) -> bool {
    let ub =  self.dom.upper();
    self.update((lb, ub).to_interval(), events)
  }

  pub fn update_ub(&mut self, ub: i32, events: &mut Vec<(u32, FDEvent)>) -> bool {
    let lb = self.dom.lower();
    self.update((lb, ub).to_interval(), events)
  }

  pub fn lb(&self) -> i32 { self.dom.lower() }
  pub fn ub(&self) -> i32 { self.dom.upper() }

  pub fn is_failed(&self) -> bool { self.dom.is_empty() }

  pub fn remove_value(&mut self, x: i32) -> Option<Vec<(u32, FDEvent)>> {
    let mut events = vec![];
    let new = self.dom.difference(Interval::singleton(x));
    if self.update(new, &mut events) {
      Some(events)
    }
    else { None }
  }

  pub fn intersection(v1: &mut FDVar, v2: &mut FDVar) -> Option<Vec<(u32, FDEvent)>> {
    let mut events = vec![];
    let new = v1.dom.intersection(v2.dom);
    if v1.update(new, &mut events) {
      if v2.update(new, &mut events) {
        return Some(events);
      }
    }
    None
  }

  pub fn is_disjoint(v1: &FDVar, v2: &FDVar) -> bool {
    v1.dom.is_disjoint(v2.dom)
  }

  pub fn is_disjoint_value(&self, x: i32) -> bool {
    self.dom.is_disjoint(Interval::singleton(x))
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use super::FDEvent::*;

  #[test]
  fn var_update_test() {
    let dom0_10 = (0,10).to_interval();
    let dom0_9 = (0,5).to_interval();
    let dom1_10 = (5,10).to_interval();
    let dom1_9 = (1,9).to_interval();
    let dom0_0 = (0,0).to_interval();
    let empty = Interval::empty();
    let var0_10 = Variable::new(0, dom0_10);

    var_update_test_one(var0_10, dom0_10, vec![], true);
    var_update_test_one(var0_10, empty, vec![], false);
    var_update_test_one(var0_10, dom0_0, vec![Assignment], true);
    var_update_test_one(var0_10, dom1_10, vec![Bound], true);
    var_update_test_one(var0_10, dom0_9, vec![Bound], true);
    var_update_test_one(var0_10, dom1_9, vec![Bound], true);
  }

  fn var_update_test_one(var: FDVar, dom: Interval<i32>, expect: Vec<FDEvent>, expect_success: bool) {
    let mut var = var;
    let mut events = vec![];
    assert_eq!(var.update(dom, &mut events), expect_success);
    if expect_success {
      assert_eq_events(events, expect);
      assert_eq!(var.dom, dom);
    }
  }

  fn assert_eq_events(events: Vec<(u32, FDEvent)>, expect: Vec<FDEvent>) {
    for ((_,ev), expect) in events.into_iter().zip(expect.into_iter()) {
      assert_eq!(ev, expect);
    }
  }

  #[test]
  fn var_update_bound() {
    let dom0_10 = (0,10).to_interval();
    let var0_10 = Variable::new(0, dom0_10);

    var_update_lb_test_one(var0_10, 0, vec![], true);
    var_update_lb_test_one(var0_10, 10, vec![Assignment], true);
    var_update_lb_test_one(var0_10, 1, vec![Bound], true);
    var_update_lb_test_one(var0_10, 11, vec![], false);

    var_update_ub_test_one(var0_10, 10, vec![], true);
    var_update_ub_test_one(var0_10, 0, vec![Assignment], true);
    var_update_ub_test_one(var0_10, 1, vec![Bound], true);
    var_update_ub_test_one(var0_10, -1, vec![], false);
  }

  fn var_update_lb_test_one(var: FDVar, lb: i32, expect: Vec<FDEvent>, expect_success: bool) {
    let mut var = var;
    let ub = var.ub();
    let mut events = vec![];
    assert_eq!(var.update_lb(lb, &mut events), expect_success);
    if expect_success {
      assert_eq_events(events, expect);
      assert_eq!(var.dom, (lb,ub).to_interval());
    }
  }

  fn var_update_ub_test_one(var: FDVar, ub: i32, expect: Vec<FDEvent>, expect_success: bool) {
    let mut var = var;
    let lb = var.lb();
    let mut events = vec![];
    assert_eq!(var.update_ub(ub, &mut events), expect_success);
    if expect_success {
      assert_eq_events(events, expect);
      assert_eq!(var.dom, (lb,ub).to_interval());
    }
  }

  #[test]
  fn var_intersection_test() {
    let var0_10 = Variable::new(1, (0,10).to_interval());
    let var10_20 = Variable::new(2, (10,20).to_interval());
    let var11_20 = Variable::new(3, (11,20).to_interval());
    let var1_9 = Variable::new(4, (1,9).to_interval());

    var_intersection_test_one(var0_10, var10_20, Some(vec![(1, Assignment), (2, Assignment)]));
    var_intersection_test_one(var0_10, var1_9, Some(vec![(1, Bound)]));
    var_intersection_test_one(var1_9, var0_10, Some(vec![(1, Bound)]));
    var_intersection_test_one(var0_10, var11_20, None);
  }

  fn var_intersection_test_one(mut v1: FDVar, mut v2: FDVar, events: Option<Vec<(u32, FDEvent)>>) {
    let v1 = &mut v1;
    let v2 = &mut v2;
    assert_eq!(FDVar::intersection(v1, v2), events);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_lb() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10: FDVar = Variable::new(0, dom0_10);

    var0_10.update_lb(-1, &mut vec![]);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_ub() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10: FDVar = Variable::new(0, dom0_10);

    var0_10.update_ub(11, &mut vec![]);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_singleton() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10: FDVar = Variable::new(0, dom0_10);
    let dom11_11 = 11.to_interval();

    var0_10.update(dom11_11, &mut vec![]);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_widen() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10: FDVar = Variable::new(0, dom0_10);
    let domm5_15 = (-5, 15).to_interval();

    var0_10.update(domm5_15, &mut vec![]);
  }
}