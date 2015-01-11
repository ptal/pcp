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
  Inner,
  Nothing
}

impl VarEvent for FDEvent {
  fn merge(self, other: FDEvent) -> FDEvent {
    FromPrimitive::from_int(min(self as isize, other as isize)).unwrap()
  }

  fn to_index(self) -> usize {
    self as usize
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
  pub fn update(&mut self, dom: Interval) -> FDEvent {
    assert!(dom.is_subset_of(self.dom));
    let old = self.dom;
    self.dom = dom;
    if old == dom { Nothing }
    else if dom.is_empty() { Failure }
    else if dom.is_singleton() { Assignment }
    else if dom.lower() != old.lower() ||
            dom.upper() != old.upper() { Bound }
    else { Inner }
  }

  pub fn update_lb(&mut self, lb: i32) -> FDEvent {
    let ub =  self.dom.upper();
    self.update((lb, ub).to_interval())
  }

  pub fn update_ub(&mut self, ub: i32) -> FDEvent {
    let lb = self.dom.lower();
    self.update((lb, ub).to_interval())
  }

  pub fn lb(&self) -> i32 { self.dom.lower() }
  pub fn ub(&self) -> i32 { self.dom.upper() }

  pub fn intersection(v1: &mut FDVar, v2: &mut FDVar) -> Vec<(u32, FDEvent)> {
    let new = v1.dom.intersection(v2.dom);
    vec![(v1.id, v1.update(new)), (v2.id, v2.update(new))]
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
    assert_eq!(fail.merge(ass), Failure);
    assert_eq!(ass.merge(fail), Failure);
    let nothing = Nothing;
    assert_eq!(nothing.merge(ass), Assignment);
    assert_eq!(ass.merge(nothing), Assignment);
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

    var_update_test_one(var0_10, dom0_10, Nothing);
    var_update_test_one(var0_10, empty, Failure);
    var_update_test_one(var0_10, dom0_0, Assignment);
    var_update_test_one(var0_10, dom1_10, Bound);
    var_update_test_one(var0_10, dom0_9, Bound);
    var_update_test_one(var0_10, dom1_9, Bound);
  }

  fn var_update_test_one(var: FDVar, dom: Interval, expect: FDEvent) {
    let mut var = var;
    assert_eq!(var.update(dom), expect);
    assert_eq!(var.dom, dom);
  }

  #[test]
  fn var_update_bound() {
    let dom0_10 = (0,10).to_interval();
    let var0_10 = FDVar::new(0, dom0_10);

    var_update_lb_test_one(var0_10, 0, Nothing);
    var_update_lb_test_one(var0_10, 10, Assignment);
    var_update_lb_test_one(var0_10, 1, Bound);
    var_update_lb_test_one(var0_10, 11, Failure);

    var_update_ub_test_one(var0_10, 10, Nothing);
    var_update_ub_test_one(var0_10, 0, Assignment);
    var_update_ub_test_one(var0_10, 1, Bound);
    var_update_ub_test_one(var0_10, -1, Failure);
  }

  fn var_update_lb_test_one(var: FDVar, lb: i32, expect: FDEvent) {
    let mut var = var;
    let ub = var.ub();
    assert_eq!(var.update_lb(lb), expect);
    assert_eq!(var.dom, (lb,ub).to_interval());
  }

  fn var_update_ub_test_one(var: FDVar, ub: i32, expect: FDEvent) {
    let mut var = var;
    let lb = var.lb();
    assert_eq!(var.update_ub(ub), expect);
    assert_eq!(var.dom, (lb,ub).to_interval());
  }

  #[test]
  fn var_intersection_test() {
    let empty = FDVar::new(0, Interval::empty());
    let var0_10 = FDVar::new(0, (0,10).to_interval());
    let var10_20 = FDVar::new(0, (10,20).to_interval());
    let var11_20 = FDVar::new(0, (11,20).to_interval());
    let var1_9 = FDVar::new(0, (1,9).to_interval());

    var_intersection_test_one(var0_10, var10_20, (Assignment, Assignment));
    var_intersection_test_one(var0_10, var1_9, (Bound, Nothing));
    var_intersection_test_one(var1_9, var0_10, (Nothing, Bound));
    var_intersection_test_one(var0_10, var11_20, (Failure, Failure));
    var_intersection_test_one(var0_10, empty, (Failure, Nothing));
    var_intersection_test_one(empty, empty, (Nothing, Nothing));
  }

  fn var_intersection_test_one(v1: FDVar, v2: FDVar, (e1, e2): (FDEvent, FDEvent)) {
    let mut v1 = v1;
    let mut v2 = v2;
    assert_eq!(FDVar::intersection(&mut v1, &mut v2), vec![(0,e1), (0,e2)]);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_lb() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10 = FDVar::new(0, dom0_10);

    var0_10.update_lb(-1);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_ub() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10 = FDVar::new(0, dom0_10);

    var0_10.update_ub(11);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_singleton() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10 = FDVar::new(0, dom0_10);
    let dom11_11 = 11.to_interval();

    var0_10.update(dom11_11);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_widen() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10 = FDVar::new(0, dom0_10);
    let domm5_15 = (-5, 15).to_interval();

    var0_10.update(domm5_15);
  }
}