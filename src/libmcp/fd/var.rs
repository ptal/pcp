// Copyright 2014 Pierre Talbot (IRCAM)

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
use self::VarEvent::*;
use std::cmp::min;
use std::num::FromPrimitive;

// we try to take the most precise first
// which is failure, then Ass, then Bound,...
// the union between two events is min(e1, e2)
#[derive(Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
pub enum VarEvent {
  Failure,
  Assignment,
  Bound,
  Inner,
  Nothing
}

impl VarEvent {
  pub fn merge(self, other: VarEvent) -> VarEvent {
    FromPrimitive::from_int(min(self as int, other as int)).unwrap()
  }
}

#[derive(Copy, PartialEq, Eq)]
pub struct Var {
  id: uint,
  dom: Interval
}

impl Var {
  pub fn new(id: uint, dom: Interval) -> Var {
    Var {
      id: id,
      dom: dom
    }
  }

  // Precondition: Accept only monotonic updates. `dom` must be a subset of self.dom.
  pub fn update(&mut self, dom: Interval) -> VarEvent {
    assert!(dom.is_subset_of(self.dom));
    let old = self.dom;
    self.dom = dom;
    if dom.is_empty() { Failure }
    else if dom.is_singleton() { Assignment }
    else if dom.lower() != old.lower() ||
            dom.upper() != old.upper() { Bound }
    else if dom != old { Inner }
    else { Nothing }
  }

  pub fn id(&self) -> uint { self.id }
}

#[cfg(test)]
mod test {
  use super::*;
  use super::VarEvent::*;

  #[test]
  fn event_test() {
    let fail = Failure;
    let ass = Assignment;
    assert!(fail.merge(ass) == Failure);
    assert!(ass.merge(fail) == Failure);
    let nothing = Nothing;
    assert!(nothing.merge(ass) == Assignment);
    assert!(ass.merge(nothing) == Assignment);
  }

  #[test]
  fn var_update_test() {
    let dom0_10 = (0,10).to_interval();
    let dom0_9 = (0,5).to_interval();
    let dom1_10 = (5,10).to_interval();
    let dom1_9 = (1,9).to_interval();
    let dom0_0 = (0,0).to_interval();
    let empty = Interval::empty();
    let var0_10 = Var::new(0, dom0_10);

    var_update_test_one(var0_10, dom0_10, Nothing);
    var_update_test_one(var0_10, empty, Failure);
    var_update_test_one(var0_10, dom0_0, Assignment);
    var_update_test_one(var0_10, dom1_10, Bound);
    var_update_test_one(var0_10, dom0_9, Bound);
    var_update_test_one(var0_10, dom1_9, Bound);
  }

  fn var_update_test_one(var: Var, dom: Interval, expect: VarEvent) {
    let mut var = var;
    assert!(var.update(dom) == expect);
    assert!(var.dom == dom);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_singleton() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10 = Var::new(0, dom0_10);
    let dom11_11 = 11.to_interval();

    var0_10.update(dom11_11);
  }

  #[test]
  #[should_fail]
  fn var_non_monotonic_update_widen() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10 = Var::new(0, dom0_10);
    let domm5_15 = (-5, 15).to_interval();

    var0_10.update(domm5_15);
  }
}