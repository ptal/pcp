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

use solver::fd::var::*;
use solver::fd::var::FDEvent::*;
use solver::propagator::*;
use solver::propagator::Status::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

pub type SharedFDVar = Rc<RefCell<FDVar>>;

// x < y
#[derive(Copy)]
pub struct XLessThanY;

impl XLessThanY {
  pub fn new(x: SharedFDVar, y: SharedFDVar) -> XLessThanYPlusC {
    XLessThanYPlusC::new(x, y, 0)
  }
}

// x <= y
#[derive(Copy)]
pub struct XLessEqThanY;

impl XLessEqThanY {
  pub fn new(x: SharedFDVar, y: SharedFDVar) -> XLessThanYPlusC {
    XLessThanYPlusC::new(x, y, 1)
  }
}

// x <= y + c
#[derive(Copy)]
pub struct XLessEqThanYPlusC;

impl XLessEqThanYPlusC {
  pub fn new(x: SharedFDVar, y: SharedFDVar, c: i32) -> XLessThanYPlusC {
    XLessThanYPlusC::new(x, y, c + 1)
  }
}

// x > y
#[derive(Copy)]
pub struct XGreaterThanY;

impl XGreaterThanY {
  pub fn new(x: SharedFDVar, y: SharedFDVar) -> XLessThanYPlusC {
    XLessThanY::new(y, x)
  }
}

// x >= y
#[derive(Copy)]
pub struct XGreaterEqThanY;

impl XGreaterEqThanY {
  pub fn new(x: SharedFDVar, y: SharedFDVar) -> XLessThanYPlusC {
    XLessEqY::new(y, x)
  }
}

// x > y + c
#[derive(Copy)]
pub struct XGreaterThanYPlusC;

impl XGreaterThanYPlusC {
  pub fn new(x: SharedFDVar, y: SharedFDVar, c: i32) -> XLessThanYPlusC {
    XLessThanYPlusC::new(y, x, -c)
  }
}

// x >= y + c
#[derive(Copy)]
pub struct XGreaterThanYPlusC;

impl XGreaterThanYPlusC {
  pub fn new(x: SharedFDVar, y: SharedFDVar, c: i32) -> XLessThanYPlusC {
    XLessThanYPlusC::new(y, x, 1 - c)
  }
}

// x = y
#[derive(Show)]
pub struct XEqualY {
  x: SharedFDVar,
  y: SharedFDVar
}

impl XEqualY {
  pub fn new(x: SharedFDVar, y: SharedFDVar) -> XEqualY {
    XEqualY { x: x, y: y }
  }
}

impl Propagator for XEqualY {
  type Event = FDEvent;

  fn status(&self) -> Status {
    // Disentailed:
    // |--|
    //     |--|
    //
    // Entailed:
    // |-|
    // |-|
    //
    // Unknown: Everything else.

    let x = self.x.borrow();
    let y = self.y.borrow();

    if FDVar::is_disjoint(x.deref(), y.deref()) {
      Disentailed
    }
    else if x.lb() == y.ub() && x.ub() == y.lb() {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self) -> Option<Vec<(u32, <XEqualY as Propagator>::Event)>> {
    let mut x = self.x.borrow_mut();
    let mut y = self.y.borrow_mut();
    FDVar::intersection(x.deref_mut(), y.deref_mut())
  }

  fn dependencies(&self) -> Vec<(u32, <XEqualY as Propagator>::Event)> {
    vec![(self.x.borrow().id(), Inner), (self.y.borrow().id(), Inner)]
  }
}

// x < y + c
#[derive(Show)]
pub struct XLessThanYPlusC {
  x: SharedFDVar,
  y: SharedFDVar,
  c: i32
}

impl XLessThanYPlusC {
  pub fn new(x: SharedFDVar, y: SharedFDVar, c: i32) -> XLessThanYPlusC {
    XLessThanYPlusC { x: x, y: y, c: c }
  }
}

impl Propagator for XLessThanYPlusC {
  type Event = FDEvent;

  fn status(&self) -> Status {
    // Disentailed:
    //     |--|
    // |--|
    //
    // Entailed:
    // |--|
    //     |--|
    //
    // Unknown: Everything else.

    let x = self.x.borrow();
    let y = self.y.borrow();

    if x.lb() > y.ub() + self.c {
      Disentailed
    }
    else if x.ub() < y.lb() + self.c {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self) -> Option<Vec<(u32, <XLessThanYPlusC as Propagator>::Event)>> {
    let mut events = vec![];
    let mut x = self.x.borrow_mut();
    let mut y = self.y.borrow_mut();
    if x.ub() >= y.ub() + self.c {
      if !x.update_ub(y.ub() - 1 + self.c, &mut events) {
        return None;
      }
    }
    if x.lb() >= y.lb() + self.c {
      if !y.update_lb(x.lb() + 1 - self.c, &mut events) {
        return None;
      }
    }
    Some(events)
  }

  fn dependencies(&self) -> Vec<(u32, <XLessThanYPlusC as Propagator>::Event)> {
    vec![(self.x.borrow().id(), Bound), (self.y.borrow().id(), Bound)]
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use solver::fd::var::*;
  use solver::fd::var::FDEvent::*;
  use solver::propagator::Status::*;
  use solver::propagator::*;
  use std::rc::Rc;
  use std::cell::RefCell;

  fn propagate_only_test<P>(prop: &mut P, expected: Option<Vec<(u32, FDEvent)>>)
   where P: Propagator<Event=FDEvent> {
    let events = prop.propagate();
    assert_eq!(events, expected);
  }

  fn propagate_test_one<P>(mut prop: P, before: Status, after: Status, expected: Option<Vec<(u32, FDEvent)>>)
   where P: Propagator<Event=FDEvent> {
    assert_eq!(prop.status(), before);
    propagate_only_test(&mut prop, expected);
    assert_eq!(prop.status(), after);
  }

  fn make_var(var: FDVar) -> SharedFDVar {
    Rc::new(RefCell::new(var))
  }

  #[test]
  fn equalxy_propagate_test() {
    let var0_10 = Variable::new(1, (0,10).to_interval());
    let var10_20 = Variable::new(2, (10,20).to_interval());
    let var5_15 = Variable::new(3, (5,15).to_interval());
    let var11_20 = Variable::new(4, (11,20).to_interval());
    let var1_1 = Variable::new(5, (1,1).to_interval());

    xequaly_propagate_test_one(make_var(var0_10), make_var(var10_20), Unknown, Entailed, Some(vec![(1, Assignment), (2, Assignment)]));
    xequaly_propagate_test_one(make_var(var5_15), make_var(var10_20), Unknown, Unknown, Some(vec![(3, Bound), (2, Bound)]));
    xequaly_propagate_test_one(make_var(var1_1), make_var(var0_10), Unknown, Entailed, Some(vec![(1, Assignment)]));
    xequaly_propagate_test_one(make_var(var0_10), make_var(var0_10), Unknown, Unknown, Some(vec![]));
    xequaly_propagate_test_one(make_var(var0_10), make_var(var11_20), Disentailed, Disentailed, None);
  }

  fn xequaly_propagate_test_one(v1: SharedFDVar, v2: SharedFDVar, before: Status, after: Status, expected: Option<Vec<(u32, FDEvent)>>) {
    let propagator = XEqualY::new(v1, v2);
    propagate_test_one(propagator, before, after, expected);
  }

  #[test]
  fn xlessy_propagate_test() {
    let var0_10 = Variable::new(1, (0,10).to_interval());
    let var0_10_ = Variable::new(12, (0,10).to_interval());
    let var10_20 = Variable::new(2, (10,20).to_interval());
    let var5_15 = Variable::new(3, (5,15).to_interval());
    let var11_20 = Variable::new(4, (11,20).to_interval());
    let var1_1 = Variable::new(5, (1,1).to_interval());

    xlessy_propagate_test_one(make_var(var0_10), make_var(var0_10_), Unknown, Unknown, Some(vec![(1, Bound), (12, Bound)]));
    xlessy_propagate_test_one(make_var(var0_10), make_var(var10_20), Unknown, Unknown, Some(vec![]));
    xlessy_propagate_test_one(make_var(var5_15), make_var(var10_20), Unknown, Unknown, Some(vec![]));
    xlessy_propagate_test_one(make_var(var5_15), make_var(var0_10), Unknown, Unknown, Some(vec![(3, Bound), (1, Bound)]));
    xlessy_propagate_test_one(make_var(var0_10), make_var(var11_20), Entailed, Entailed, Some(vec![]));
    xlessy_propagate_test_one(make_var(var11_20), make_var(var0_10), Disentailed, Disentailed, None);
    xlessy_propagate_test_one(make_var(var1_1), make_var(var0_10), Unknown, Entailed, Some(vec![(1, Bound)]));
  }

  fn xlessy_propagate_test_one(v1: SharedFDVar, v2: SharedFDVar, before: Status, after: Status, expected: Option<Vec<(u32, FDEvent)>>) {
    let propagator = XLessThanY::new(v1, v2);
    propagate_test_one(propagator, before, after, expected);
  }

  #[test]
  fn xlessyplusc_propagate_test() {
    let var0_10 = Variable::new(1, (0,10).to_interval());
    let var10_20 = Variable::new(2, (10,20).to_interval());
    let var5_15 = Variable::new(3, (5,15).to_interval());
    let var1_1 = Variable::new(5, (1,1).to_interval());

    // Same test as x < y but we shift y.
    xlessyplusc_propagate_test_one(make_var(var0_10), make_var(var5_15), -5, Unknown, Unknown, Some(vec![(1, Bound), (3, Bound)]));
    xlessyplusc_propagate_test_one(make_var(var0_10), make_var(var0_10), 10, Unknown, Unknown, Some(vec![]));
    xlessyplusc_propagate_test_one(make_var(var5_15), make_var(var5_15), 5, Unknown, Unknown, Some(vec![]));
    xlessyplusc_propagate_test_one(make_var(var5_15), make_var(var10_20), -10, Unknown, Unknown, Some(vec![(3, Bound), (2, Bound)]));
    xlessyplusc_propagate_test_one(make_var(var0_10), make_var(var0_10), 11, Entailed, Entailed, Some(vec![]));
    xlessyplusc_propagate_test_one(make_var(var0_10), make_var(var0_10), -11, Disentailed, Disentailed, None);
    xlessyplusc_propagate_test_one(make_var(var1_1), make_var(var5_15), -5, Unknown, Entailed, Some(vec![(3, Bound)]));
  }

  fn xlessyplusc_propagate_test_one(v1: SharedFDVar, v2: SharedFDVar, c: i32, before: Status, after: Status, expected: Option<Vec<(u32, FDEvent)>>) {
    let propagator = XLessThanYPlusC::new(v1, v2, c);
    propagate_test_one(propagator, before, after, expected);
  }
}
