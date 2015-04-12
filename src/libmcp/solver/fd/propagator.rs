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

use solver::variable::*;
use solver::fd::event::*;
use solver::fd::event::FDEvent::*;
use solver::propagator::*;
use solver::propagator::Status::*;
use solver::merge::Merge;
use interval::interval::*;
use interval::ncollections::ops::*;
use std::ops::{Deref, DerefMut};
use std::collections::HashMap;

pub type SharedVarI32 = SharedVar<Interval<i32>>;

// x < y
pub struct XLessThanY;

impl XLessThanY {
  pub fn new(x: SharedVarI32, y: SharedVarI32) -> XLessThanYPlusC {
    XLessThanYPlusC::new(x, y, 0)
  }
}

// x <= y
pub struct XLessEqThanY;

impl XLessEqThanY {
  pub fn new(x: SharedVarI32, y: SharedVarI32) -> XLessThanYPlusC {
    XLessThanYPlusC::new(x, y, 1)
  }
}

// x <= y + c
pub struct XLessEqThanYPlusC;

impl XLessEqThanYPlusC {
  pub fn new(x: SharedVarI32, y: SharedVarI32, c: i32) -> XLessThanYPlusC {
    XLessThanYPlusC::new(x, y, c + 1)
  }
}

// x > y
pub struct XGreaterThanY;

impl XGreaterThanY {
  pub fn new(x: SharedVarI32, y: SharedVarI32) -> XLessThanYPlusC {
    XLessThanY::new(y, x)
  }
}

// x >= y
pub struct XGreaterEqThanY;

impl XGreaterEqThanY {
  pub fn new(x: SharedVarI32, y: SharedVarI32) -> XLessThanYPlusC {
    XLessEqThanY::new(y, x)
  }
}

// x > y + c
pub struct XGreaterThanYPlusC;

impl XGreaterThanYPlusC {
  pub fn new(x: SharedVarI32, y: SharedVarI32, c: i32) -> XLessThanYPlusC {
    XLessThanYPlusC::new(y, x, -c)
  }
}

// x >= y + c
pub struct XGreaterEqThanYPlusC;

impl XGreaterEqThanYPlusC {
  pub fn new(x: SharedVarI32, y: SharedVarI32, c: i32) -> XLessThanYPlusC {
    XLessThanYPlusC::new(y, x, 1 - c)
  }
}

// x = y
#[derive(Debug)]
pub struct XEqualY {
  x: SharedVarI32,
  y: SharedVarI32
}

impl XEqualY {
  pub fn new(x: SharedVarI32, y: SharedVarI32) -> XEqualY {
    XEqualY { x: x, y: y }
  }
}

impl Propagator<FDEvent> for XEqualY {
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

    if x.is_disjoint(y.deref()) {
      Disentailed
    }
    else if x.lower() == y.upper() && x.upper() == y.lower() {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self, events: &mut Vec<(usize, FDEvent)>) -> bool {
    let mut x = self.x.borrow_mut();
    let mut y = self.y.borrow_mut();
    x.deref_mut().event_intersection(y.deref_mut(), events)
  }

  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    vec![(self.x.borrow().index(), Inner), (self.y.borrow().index(), Inner)]
  }
}

impl DeepClone<Vec<SharedVarI32>> for XEqualY
{
  fn deep_clone(&self, state: &Vec<SharedVarI32>) -> XEqualY {
    XEqualY::new(
      self.x.deep_clone(state),
      self.y.deep_clone(state))
  }
}

// x < y + c
#[derive(Debug)]
pub struct XLessThanYPlusC {
  x: SharedVarI32,
  y: SharedVarI32,
  c: i32
}

impl XLessThanYPlusC {
  pub fn new(x: SharedVarI32, y: SharedVarI32, c: i32) -> XLessThanYPlusC {
    XLessThanYPlusC { x: x, y: y, c: c }
  }
}

impl Propagator<FDEvent> for XLessThanYPlusC {
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

    if x.lower() > y.upper() + self.c {
      Disentailed
    }
    else if x.upper() < y.lower() + self.c {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self, events: &mut Vec<(usize, FDEvent)>) -> bool {
    let mut x = self.x.borrow_mut();
    let mut y = self.y.borrow_mut();
    x.event_shrink_right(y.upper() - 1 + self.c, events) &&
    y.event_shrink_left(x.lower() + 1 - self.c, events)
  }

  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    vec![(self.x.borrow().index(), Bound), (self.y.borrow().index(), Bound)]
  }
}

impl DeepClone<Vec<SharedVarI32>> for XLessThanYPlusC
{
  fn deep_clone(&self, state: &Vec<SharedVarI32>) -> XLessThanYPlusC {
    XLessThanYPlusC::new(
      self.x.deep_clone(state),
      self.y.deep_clone(state),
      self.c)
  }
}

// x > c
pub struct XGreaterThanC;

impl XGreaterThanC {
  pub fn new(x: SharedVarI32, c: i32) -> XGreaterEqThanC {
    XGreaterEqThanC::new(x, c + 1)
  }
}

// x >= c
#[derive(Debug)]
pub struct XGreaterEqThanC {
  x: SharedVarI32,
  c: i32
}

impl XGreaterEqThanC {
  pub fn new(x: SharedVarI32, c: i32) -> XGreaterEqThanC {
    XGreaterEqThanC { x: x, c: c }
  }
}

impl Propagator<FDEvent> for XGreaterEqThanC {
  fn status(&self) -> Status {
    let x = self.x.borrow();

    if x.upper() < self.c {
      Disentailed
    }
    else if x.lower() >= self.c {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self, events: &mut Vec<(usize, FDEvent)>) -> bool {
    self.x.borrow_mut().event_shrink_left(self.c, events)
  }

  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    vec![(self.x.borrow().index(), Bound)]
  }
}

impl DeepClone<Vec<SharedVarI32>> for XGreaterEqThanC
{
  fn deep_clone(&self, state: &Vec<SharedVarI32>) -> XGreaterEqThanC {
    XGreaterEqThanC::new(
      self.x.deep_clone(state),
      self.c)
  }
}

// x < c
pub struct XLessThanC;

impl XLessThanC {
  pub fn new(x: SharedVarI32, c: i32) -> XLessEqThanC {
    XLessEqThanC::new(x, c - 1)
  }
}

// x <= c
#[derive(Debug)]
pub struct XLessEqThanC {
  x: SharedVarI32,
  c: i32
}

impl XLessEqThanC {
  pub fn new(x: SharedVarI32, c: i32) -> XLessEqThanC {
    XLessEqThanC { x: x, c: c }
  }
}

impl Propagator<FDEvent> for XLessEqThanC {
  fn status(&self) -> Status {
    let x = self.x.borrow();

    if x.lower() > self.c {
      Disentailed
    }
    else if x.upper() <= self.c {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self, events: &mut Vec<(usize, FDEvent)>) -> bool {
    self.x.borrow_mut().event_shrink_right(self.c, events)
  }

  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    vec![(self.x.borrow().index(), Bound)]
  }
}

impl DeepClone<Vec<SharedVarI32>> for XLessEqThanC
{
  fn deep_clone(&self, state: &Vec<SharedVarI32>) -> XLessEqThanC {
    XLessEqThanC::new(
      self.x.deep_clone(state),
      self.c)
  }
}

// x != c
#[derive(Debug)]
pub struct XNotEqualC {
  x: SharedVarI32,
  c: i32
}

impl XNotEqualC {
  pub fn new(x: SharedVarI32, c: i32) -> XNotEqualC {
    XNotEqualC { x: x, c: c }
  }
}

impl Propagator<FDEvent> for XNotEqualC {
  fn status(&self) -> Status {
    let x = self.x.borrow();

    if x.lower() == self.c && x.upper() == self.c {
      Disentailed
    }
    else if !x.contains(&self.c) {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self, events: &mut Vec<(usize, FDEvent)>) -> bool {
    self.x.borrow_mut().event_remove(self.c, events)
  }

  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    vec![(self.x.borrow().index(), Inner)]
  }
}

impl DeepClone<Vec<SharedVarI32>> for XNotEqualC
{
  fn deep_clone(&self, state: &Vec<SharedVarI32>) -> XNotEqualC {
    XNotEqualC::new(
      self.x.deep_clone(state),
      self.c)
  }
}

// x != y
#[derive(Debug)]
pub struct XNotEqualY;

impl XNotEqualY {
  pub fn new(x: SharedVarI32, y: SharedVarI32) -> XNotEqualYPlusC {
    XNotEqualYPlusC { x: x, y: y, c: 0 }
  }
}

// x != y + c
#[derive(Debug)]
pub struct XNotEqualYPlusC {
  x: SharedVarI32,
  y: SharedVarI32,
  c: i32
}

impl XNotEqualYPlusC {
  pub fn new(x: SharedVarI32, y: SharedVarI32, c: i32) -> XNotEqualYPlusC {
    XNotEqualYPlusC { x: x, y: y, c: c }
  }
}

impl Propagator<FDEvent> for XNotEqualYPlusC {
  fn status(&self) -> Status {
    let x = self.x.borrow();
    let y = self.y.borrow();

    if x.lower() == y.upper() + self.c && x.upper() == y.lower() + self.c {
      Disentailed
    }
    else if x.lower() > y.upper() + self.c || x.upper() < y.lower() + self.c {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self, events: &mut Vec<(usize, FDEvent)>) -> bool {
    let mut x = self.x.borrow_mut();
    let mut y = self.y.borrow_mut();
    if x.lower() == x.upper() {
      y.event_remove(x.lower() - self.c, events)
    }
    else if y.lower() == y.upper() {
      x.event_remove(y.lower() + self.c, events)
    }
    else {
      true
    }
  }

  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    vec![(self.x.borrow().index(), Inner), (self.y.borrow().index(), Inner)]
  }
}

impl DeepClone<Vec<SharedVarI32>> for XNotEqualYPlusC
{
  fn deep_clone(&self, state: &Vec<SharedVarI32>) -> XNotEqualYPlusC {
    XNotEqualYPlusC::new(
      self.x.deep_clone(state),
      self.y.deep_clone(state),
      self.c)
  }
}

// distinct(x1,..,xN)
// #[derive(Debug)]
pub struct Distinct {
  vars: Vec<SharedVarI32>,
  props: Vec<XNotEqualYPlusC>
}

impl Distinct {
  pub fn new(vars: Vec<SharedVarI32>) -> Distinct {
    let mut props = vec![];
    for i in 0..vars.len()-1 {
      for j in i+1..vars.len() {
        let i_neq_j = XNotEqualY::new(vars[i].clone(), vars[j].clone());
        props.push(i_neq_j);
      }
    }
    Distinct { vars: vars, props: props }
  }

  fn merge_keys(key: usize, value: FDEvent, vars_events: &mut HashMap<usize, FDEvent>) {
    let old = vars_events.insert(key, value);
    match old {
      None => (),
      Some(x) => {
        vars_events.insert(key, FDEvent::merge(value, x));
      }
    }
  }
}

impl Propagator<FDEvent> for Distinct {
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

  fn propagate(&mut self, events: &mut Vec<(usize, FDEvent)>) -> bool {
    let mut unique_events = HashMap::new();
    for p in self.props.iter_mut() {
      let mut events = vec![];
      if p.propagate(&mut events) {
        for (var_id, ev) in events.into_iter() {
          Distinct::merge_keys(var_id, ev, &mut unique_events);
        }
      } else {
        return false;
      }
    }
    for id_ev in unique_events.into_iter() {
      events.push(id_ev);
    }
    true
  }

  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    self.vars.iter().map(|x| (x.borrow().index(), Inner)).collect()
  }
}

impl DeepClone<Vec<SharedVarI32>> for Distinct
{
  fn deep_clone(&self, state: &Vec<SharedVarI32>) -> Distinct {
    Distinct {
      vars: self.vars.iter().map(|v| v.deep_clone(state)).collect(),
      props: self.props.iter().map(|p| p.deep_clone(state)).collect()
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use solver::variable::*;
  use solver::fd::event::*;
  use solver::fd::event::FDEvent::*;
  use solver::propagator::Status::*;
  use solver::propagator::*;
  use interval::interval::*;
  use interval::ncollections::ops::*;
  use std::rc::Rc;
  use std::cell::RefCell;
  use std::collections::VecMap;

  fn make_vec_map(events: Vec<(usize, FDEvent)>) -> VecMap<FDEvent> {
    let mut new_events = VecMap::new();
    for (id, ev) in events.into_iter() {
      let old = new_events.insert(id as usize, ev);
      assert!(old.is_none(), "Duplications are not allowed from propagate().");
    }
    new_events
  }

  fn propagate_only_test<P>(prop: &mut P, expected: Option<Vec<(usize, FDEvent)>>)
   where P: Propagator<FDEvent> {
    let mut events = vec![];
    if prop.propagate(&mut events) && expected != None {
      let events = make_vec_map(events);
      let expected = make_vec_map(expected.unwrap());
      assert_eq!(events, expected);
    } else {
      assert!(expected.is_none(), "The propagator returned `false`, it failed.");
    }
  }

  fn propagate_test_one<P>(mut prop: P, before: Status, after: Status, expected: Option<Vec<(usize, FDEvent)>>)
   where P: Propagator<FDEvent> {
    assert_eq!(prop.status(), before);
    propagate_only_test(&mut prop, expected);
    assert_eq!(prop.status(), after);
  }

  fn make_var(var: Variable<Interval<i32>>) -> SharedVarI32 {
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

  fn xequaly_propagate_test_one(v1: SharedVarI32, v2: SharedVarI32, before: Status, after: Status, expected: Option<Vec<(usize, FDEvent)>>) {
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

  fn xlessy_propagate_test_one(v1: SharedVarI32, v2: SharedVarI32, before: Status, after: Status, expected: Option<Vec<(usize, FDEvent)>>) {
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

  fn xlessyplusc_propagate_test_one(v1: SharedVarI32, v2: SharedVarI32, c: i32, before: Status, after: Status, expected: Option<Vec<(usize, FDEvent)>>) {
    let propagator = XLessThanYPlusC::new(v1, v2, c);
    propagate_test_one(propagator, before, after, expected);
  }

  #[test]
  fn unary_propagator_test() {
    let var0_10 = Variable::new(1, (0,10).to_interval());
    let var0_0 = Variable::new(2, (0,0).to_interval());
    let make_xlessc = |c| XLessThanC::new(make_var(var0_10), c);
    propagate_test_one(make_xlessc(0), Disentailed, Disentailed, None);
    propagate_test_one(make_xlessc(11), Entailed, Entailed, Some(vec![]));
    propagate_test_one(make_xlessc(10), Unknown, Entailed, Some(vec![(1, Bound)]));

    let make_xlesseqc = |c| XLessEqThanC::new(make_var(var0_10), c);
    propagate_test_one(make_xlesseqc(-1), Disentailed, Disentailed, None);
    propagate_test_one(make_xlesseqc(10), Entailed, Entailed, Some(vec![]));
    propagate_test_one(make_xlesseqc(9), Unknown, Entailed, Some(vec![(1, Bound)]));

    let make_xgreaterc = |c| XGreaterThanC::new(make_var(var0_10), c);
    propagate_test_one(make_xgreaterc(10), Disentailed, Disentailed, None);
    propagate_test_one(make_xgreaterc(-1), Entailed, Entailed, Some(vec![]));
    propagate_test_one(make_xgreaterc(0), Unknown, Entailed, Some(vec![(1, Bound)]));

    let make_xgreatereqc = |c| XGreaterEqThanC::new(make_var(var0_10), c);
    propagate_test_one(make_xgreatereqc(11), Disentailed, Disentailed, None);
    propagate_test_one(make_xgreatereqc(0), Entailed, Entailed, Some(vec![]));
    propagate_test_one(make_xgreatereqc(1), Unknown, Entailed, Some(vec![(1, Bound)]));

    let make_xnotequalc = |c| XNotEqualC::new(make_var(var0_10), c);
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

  fn x_neq_y_plus_c_test_one(v1: SharedVarI32, v2: SharedVarI32, c: i32, before: Status, after: Status, expected: Option<Vec<(usize, FDEvent)>>) {
    let propagator = XNotEqualYPlusC::new(v1, v2, c);
    propagate_test_one(propagator, before, after, expected);
  }

  #[test]
  fn distinct_test() {
    let mut vars: Vec<Variable<Interval<i32>>> = (0..3)
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

  fn distinct_test_one(vars: Vec<SharedVarI32>, before: Status, after: Status, expected: Option<Vec<(usize, FDEvent)>>) {
    let propagator = Distinct::new(vars);
    propagate_test_one(propagator, before, after, expected);
  }
}
