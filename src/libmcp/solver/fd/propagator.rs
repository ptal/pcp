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
use std::collections::HashMap;

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
    XLessEqThanY::new(y, x)
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
pub struct XGreaterEqThanYPlusC;

impl XGreaterEqThanYPlusC {
  pub fn new(x: SharedFDVar, y: SharedFDVar, c: i32) -> XLessThanYPlusC {
    XLessThanYPlusC::new(y, x, 1 - c)
  }
}

// x = y
#[derive(Debug)]
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
#[derive(Debug)]
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

// x > c
#[derive(Copy)]
pub struct XGreaterThanC;

impl XGreaterThanC {
  pub fn new(x: SharedFDVar, c: i32) -> XGreaterEqThanC {
    XGreaterEqThanC::new(x, c + 1)
  }
}

// x >= c
#[derive(Debug)]
pub struct XGreaterEqThanC {
  x: SharedFDVar,
  c: i32
}

impl XGreaterEqThanC {
  pub fn new(x: SharedFDVar, c: i32) -> XGreaterEqThanC {
    XGreaterEqThanC { x: x, c: c }
  }
}

impl Propagator for XGreaterEqThanC {
  type Event = FDEvent;

  fn status(&self) -> Status {
    let x = self.x.borrow();

    if x.ub() < self.c {
      Disentailed
    }
    else if x.lb() >= self.c {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self) -> Option<Vec<(u32, <XGreaterEqThanC as Propagator>::Event)>> {
    let mut events = vec![];
    let mut x = self.x.borrow_mut();
    if x.lb() < self.c && !x.update_lb(self.c, &mut events) {
      return None;
    }
    Some(events)
  }

  fn dependencies(&self) -> Vec<(u32, <XGreaterEqThanC as Propagator>::Event)> {
    vec![(self.x.borrow().id(), Bound)]
  }
}

// x < c
#[derive(Copy)]
pub struct XLessThanC;

impl XLessThanC {
  pub fn new(x: SharedFDVar, c: i32) -> XLessEqThanC {
    XLessEqThanC::new(x, c - 1)
  }
}

// x <= c
#[derive(Debug)]
pub struct XLessEqThanC {
  x: SharedFDVar,
  c: i32
}

impl XLessEqThanC {
  pub fn new(x: SharedFDVar, c: i32) -> XLessEqThanC {
    XLessEqThanC { x: x, c: c }
  }
}

impl Propagator for XLessEqThanC {
  type Event = FDEvent;

  fn status(&self) -> Status {
    let x = self.x.borrow();

    if x.lb() > self.c {
      Disentailed
    }
    else if x.ub() <= self.c {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self) -> Option<Vec<(u32, <XLessEqThanC as Propagator>::Event)>> {
    let mut events = vec![];
    let mut x = self.x.borrow_mut();
    if x.ub() > self.c && !x.update_ub(self.c, &mut events) {
      return None;
    }
    Some(events)
  }

  fn dependencies(&self) -> Vec<(u32, <XLessEqThanC as Propagator>::Event)> {
    vec![(self.x.borrow().id(), Bound)]
  }
}

// x != c
#[derive(Debug)]
pub struct XNotEqualC {
  x: SharedFDVar,
  c: i32
}

impl XNotEqualC {
  pub fn new(x: SharedFDVar, c: i32) -> XNotEqualC {
    XNotEqualC { x: x, c: c }
  }
}

impl Propagator for XNotEqualC {
  type Event = FDEvent;

  fn status(&self) -> Status {
    let x = self.x.borrow();

    if x.lb() == self.c && x.ub() == self.c {
      Disentailed
    }
    else if x.is_disjoint_value(self.c) {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self) -> Option<Vec<(u32, <XNotEqualC as Propagator>::Event)>> {
    self.x.borrow_mut().remove_value(self.c)
  }

  fn dependencies(&self) -> Vec<(u32, <XNotEqualC as Propagator>::Event)> {
    vec![(self.x.borrow().id(), Inner)]
  }
}

// x != y
#[derive(Copy, Debug)]
pub struct XNotEqualY;

impl XNotEqualY {
  pub fn new(x: SharedFDVar, y: SharedFDVar) -> XNotEqualYPlusC {
    XNotEqualYPlusC { x: x, y: y, c: 0 }
  }
}

// x != y + c
#[derive(Debug)]
pub struct XNotEqualYPlusC {
  x: SharedFDVar,
  y: SharedFDVar,
  c: i32
}

impl XNotEqualYPlusC {
  pub fn new(x: SharedFDVar, y: SharedFDVar, c: i32) -> XNotEqualYPlusC {
    XNotEqualYPlusC { x: x, y: y, c: c }
  }
}

impl Propagator for XNotEqualYPlusC {
  type Event = FDEvent;

  fn status(&self) -> Status {
    let x = self.x.borrow();
    let y = self.y.borrow();

    if x.lb() == y.ub() + self.c && x.ub() == y.lb() + self.c {
      Disentailed
    }
    else if x.lb() > y.ub() + self.c || x.ub() < y.lb() + self.c {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self) -> Option<Vec<(u32, <XNotEqualYPlusC as Propagator>::Event)>> {
    let mut x = self.x.borrow_mut();
    let mut y = self.y.borrow_mut();
    if x.lb() == x.ub() {
      y.remove_value(x.lb() - self.c)
    }
    else if y.lb() == y.ub() {
      x.remove_value(y.lb() + self.c)
    }
    else {
      Some(vec![])
    }
  }

  fn dependencies(&self) -> Vec<(u32, <XNotEqualYPlusC as Propagator>::Event)> {
    vec![(self.x.borrow().id(), Inner), (self.y.borrow().id(), Inner)]
  }
}

// distinct(x1,..,xN)
#[derive(Debug)]
pub struct Distinct {
  vars: Vec<SharedFDVar>,
  props: Vec<XNotEqualYPlusC>
}

impl Distinct {
  pub fn new(vars: Vec<SharedFDVar>) -> Distinct {
    let mut props = vec![];
    for i in range(0, vars.len()-1) {
      for j in range(i+1, vars.len()) {
        let i_neq_j = XNotEqualY::new(vars[i].clone(), vars[j].clone());
        props.push(i_neq_j);
      }
    }
    Distinct { vars: vars, props: props }
  }

  fn merge_keys(key: u32, value: FDEvent, vars_events: &mut HashMap<u32, FDEvent>) {
    let old = vars_events.insert(key, value);
    match old {
      None => (),
      Some(x) => {
        vars_events.insert(key, value.merge(x));
      }
    }
  }
}

impl Propagator for Distinct {
  type Event = FDEvent;

  fn status(&self) -> Status {
    let mut all_entailed = true;
    for p in self.props.iter() {
      match p.status() {
        Disentailed => return Disentailed,
        Unknown => all_entailed = false,
        _ => ()
      }
    }
    if all_entailed { Entailed }
    else { Unknown }
  }

  fn propagate(&mut self) -> Option<Vec<(u32, <Distinct as Propagator>::Event)>> {
    let mut events = HashMap::new();
    for p in self.props.iter_mut() {
      match p.propagate() {
        None => return None,
        Some(prop_events) => {
          for (var_id, ev) in prop_events.into_iter() {
            Distinct::merge_keys(var_id, ev, &mut events);
          }
        }
      }
    }
    Some(events.into_iter().collect())
  }

  fn dependencies(&self) -> Vec<(u32, <Distinct as Propagator>::Event)> {
    self.vars.iter().map(|x| (x.borrow().id(), Inner)).collect()
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
  use std::iter::range;
  use std::collections::VecMap;

  fn make_vec_map(events: Vec<(u32, FDEvent)>) -> VecMap<FDEvent> {
    let mut new_events = VecMap::new();
    for (id, ev) in events.into_iter() {
      let old = new_events.insert(id as usize, ev);
      assert!(old.is_none(), "Duplications are not allowed from propagate().");
    }
    new_events
  }

  fn propagate_only_test<P>(prop: &mut P, expected: Option<Vec<(u32, FDEvent)>>)
   where P: Propagator<Event=FDEvent> {
    let events = prop.propagate();
    if events != None && expected != None {
      let events = make_vec_map(events.unwrap());
      let expected = make_vec_map(expected.unwrap());
      assert_eq!(events, expected);
    } else {
      assert_eq!(events, expected);
    }
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

  #[test]
  fn unary_propagator_test() {
    let var0_10 = Variable::new(1, (0,10).to_interval());
    let var0_0 = Variable::new(2, (0,0).to_interval());
    let make_xlessc = |&:c| XLessThanC::new(make_var(var0_10), c);
    propagate_test_one(make_xlessc(0), Disentailed, Disentailed, None);
    propagate_test_one(make_xlessc(11), Entailed, Entailed, Some(vec![]));
    propagate_test_one(make_xlessc(10), Unknown, Entailed, Some(vec![(1, Bound)]));

    let make_xlesseqc = |&:c| XLessEqThanC::new(make_var(var0_10), c);
    propagate_test_one(make_xlesseqc(-1), Disentailed, Disentailed, None);
    propagate_test_one(make_xlesseqc(10), Entailed, Entailed, Some(vec![]));
    propagate_test_one(make_xlesseqc(9), Unknown, Entailed, Some(vec![(1, Bound)]));

    let make_xgreaterc = |&:c| XGreaterThanC::new(make_var(var0_10), c);
    propagate_test_one(make_xgreaterc(10), Disentailed, Disentailed, None);
    propagate_test_one(make_xgreaterc(-1), Entailed, Entailed, Some(vec![]));
    propagate_test_one(make_xgreaterc(0), Unknown, Entailed, Some(vec![(1, Bound)]));

    let make_xgreatereqc = |&:c| XGreaterEqThanC::new(make_var(var0_10), c);
    propagate_test_one(make_xgreatereqc(11), Disentailed, Disentailed, None);
    propagate_test_one(make_xgreatereqc(0), Entailed, Entailed, Some(vec![]));
    propagate_test_one(make_xgreatereqc(1), Unknown, Entailed, Some(vec![(1, Bound)]));

    let make_xnotequalc = |&:c| XNotEqualC::new(make_var(var0_10), c);
    propagate_test_one(make_xnotequalc(5), Unknown, Unknown, Some(vec![]));
    propagate_test_one(make_xnotequalc(0), Unknown, Entailed, Some(vec![(1, Bound)]));
    propagate_test_one(make_xnotequalc(10), Unknown, Entailed, Some(vec![(1, Bound)]));
    propagate_test_one(XNotEqualC::new(make_var(var0_0), 0), Disentailed, Disentailed, None);
  }

  #[test]
  fn x_neq_y_plus_c_test() {
    let var0_10 = Variable::new(1, (0,10).to_interval());
    let var10_20 = Variable::new(2, (10,20).to_interval());
    let var0_0 = Variable::new(5, (0,0).to_interval());

    x_neq_y_plus_c_test_one(make_var(var0_10), make_var(var0_10), 0, Unknown, Unknown, Some(vec![]));
    x_neq_y_plus_c_test_one(make_var(var0_10), make_var(var10_20), 0, Unknown, Unknown, Some(vec![]));
    x_neq_y_plus_c_test_one(make_var(var0_10), make_var(var10_20), 1, Entailed, Entailed, Some(vec![]));
    x_neq_y_plus_c_test_one(make_var(var0_10), make_var(var0_0), 0, Unknown, Entailed, Some(vec![(1, Bound)]));
    x_neq_y_plus_c_test_one(make_var(var0_10), make_var(var0_0), 10, Unknown, Entailed, Some(vec![(1, Bound)]));
    x_neq_y_plus_c_test_one(make_var(var0_10), make_var(var0_0), 5, Unknown, Unknown, Some(vec![]));
    x_neq_y_plus_c_test_one(make_var(var0_0), make_var(var0_0), 10, Entailed, Entailed, Some(vec![]));
    x_neq_y_plus_c_test_one(make_var(var0_0), make_var(var0_0), 0, Disentailed, Disentailed, None);
  }

  fn x_neq_y_plus_c_test_one(v1: SharedFDVar, v2: SharedFDVar, c: i32, before: Status, after: Status, expected: Option<Vec<(u32, FDEvent)>>) {
    let propagator = XNotEqualYPlusC::new(v1, v2, c);
    propagate_test_one(propagator, before, after, expected);
  }

  #[test]
  fn distinct_test() {
    let mut vars: Vec<FDVar> = range(0, 3)
      .map(|v| Variable::new(v, Interval::singleton(v as i32)))
      .collect();
    vars.push(Variable::new(3, (0,3).to_interval()));
    vars.push(Variable::new(4, (0,1).to_interval()));
    vars.push(Variable::new(5, (0,3).to_interval()));

    distinct_test_one(vec![make_var(vars[0]), make_var(vars[1]), make_var(vars[2])],
      Entailed, Entailed, Some(vec![]));
    distinct_test_one(vec![make_var(vars[0]), make_var(vars[0]), make_var(vars[2])],
      Disentailed, Disentailed, None);
    distinct_test_one(vec![make_var(vars[0]), make_var(vars[1]), make_var(vars[3])],
      Unknown, Entailed, Some(vec![(3,Bound)]));
    distinct_test_one(vec![make_var(vars[0]), make_var(vars[1]), make_var(vars[4])],
      Unknown, Disentailed, None);
    distinct_test_one(vec![make_var(vars[0]), make_var(vars[3]), make_var(vars[5])],
      Unknown, Unknown, Some(vec![(3,Bound),(5,Bound)]));
    distinct_test_one(vec![make_var(vars[3])], Entailed, Entailed, Some(vec![]));
  }

  fn distinct_test_one(vars: Vec<SharedFDVar>, before: Status, after: Status, expected: Option<Vec<(u32, FDEvent)>>) {
    let propagator = Distinct::new(vars);
    propagate_test_one(propagator, before, after, expected);
  }
}
