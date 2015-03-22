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
pub use solver::fd::event::*;
use solver::event::*;
use solver::variable::Variable;
use interval::ncollections::ops::*;
use interval::ops::*;

use std::fmt::{Formatter, Display, Error};

#[derive(Copy, PartialEq, Eq, Debug, Clone)]
pub struct FDVar<Domain> {
  idx: usize,
  dom: Domain
}

impl<Domain: Display> Display for FDVar<Domain> {
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("({}, {})", self.idx, self.dom))
  }
}

impl<Domain> Variable for FDVar<Domain> where
  Domain: Cardinality
{
  type Domain = Domain;
  type Event = FDEvent;

  fn new(idx: usize, dom: Domain) -> FDVar<Domain> {
    assert!(!dom.is_empty());
    FDVar {
      idx: idx,
      dom: dom
    }
  }
}

pub trait VarIndex
{
  fn index(&self) -> usize;
}

impl<Domain> VarIndex for FDVar<Domain>
{
  fn index(&self) -> usize {
    self.idx
  }
}

pub trait Failure
{
  fn is_failed(&self) -> bool;
}

impl<Domain> Failure for FDVar<Domain> where
  Domain: Cardinality
{
  fn is_failed(&self) -> bool {
    self.dom.is_empty()
  }
}

trait VarDomain: Bounded + Cardinality + Subset
{}

impl<R> VarDomain for R where
  R: Bounded + Cardinality + Subset
{}

pub trait EventUpdate<Item>
{
  fn event_update(&mut self, value: Item,
    events: &mut Vec<(usize, FDEvent)>) -> bool;
}

impl<Domain> EventUpdate<Domain> for FDVar<Domain> where
  Domain: VarDomain
{
  fn event_update(&mut self, dom: Domain, events: &mut Vec<(usize, FDEvent)>) -> bool {
    assert!(dom.is_subset(&self.dom), "Domain update must be monotonic.");
    if dom.is_empty() { false } // Failure
    else {
      if let Some(event) = FDEvent::new(&dom, &self.dom) {
        events.push((self.idx, event));
        self.dom = dom;
      }
      true
    }
  }
}

impl<Domain> Bounded for FDVar<Domain> where
  Domain: Bounded
{
  type Bound = Domain::Bound;

  fn lower(&self) -> Domain::Bound {
    self.dom.lower()
  }

  fn upper(&self) -> Domain::Bound {
    self.dom.upper()
  }
}

impl<Domain> Disjoint for FDVar<Domain> where
  Domain: Disjoint
{
  fn is_disjoint(&self, other: &FDVar<Domain>) -> bool {
    self.dom.is_disjoint(&other.dom)
  }
}

impl<Domain> Contains<Domain::Bound> for FDVar<Domain> where
  Domain: Bounded + Contains<<Domain as Bounded>::Bound>
{
  fn contains(&self, value: &Domain::Bound) -> bool {
    self.dom.contains(value)
  }
}

pub trait EventShrinkLeft<Bound>
{
  fn event_shrink_left(&mut self, lb: Bound,
    events: &mut Vec<(usize, FDEvent)>) -> bool;
}

impl<Domain> EventShrinkLeft<Domain::Bound> for FDVar<Domain> where
  Domain: VarDomain + ShrinkLeft<<Domain as Bounded>::Bound> + Clone
{
  fn event_shrink_left(&mut self, lb: Domain::Bound,
    events: &mut Vec<(usize, FDEvent)>) -> bool
  {
    let new = self.dom.clone().shrink_left(lb);
    self.event_update(new, events)
  }
}

pub trait EventShrinkRight<Bound>
{
  fn event_shrink_right(&mut self, ub: Bound,
    events: &mut Vec<(usize, FDEvent)>) -> bool;
}

impl<Domain> EventShrinkRight<Domain::Bound> for FDVar<Domain> where
  Domain: VarDomain + ShrinkRight<<Domain as Bounded>::Bound> + Clone
{
  fn event_shrink_right(&mut self, ub: Domain::Bound,
    events: &mut Vec<(usize, FDEvent)>) -> bool
  {
    let new = self.dom.clone().shrink_right(ub);
    self.event_update(new, events)
  }
}

pub trait EventRemove<Item>
{
  fn event_remove(&mut self, value: Item,
    events: &mut Vec<(usize, FDEvent)>) -> bool;
}

impl<Domain> EventRemove<Domain::Bound> for FDVar<Domain> where
  Domain: VarDomain + Difference<<Domain as Bounded>::Bound, Output=Domain> + Clone
{
  fn event_remove(&mut self, value: Domain::Bound,
    events: &mut Vec<(usize, FDEvent)>) -> bool
  {
    let new = self.dom.clone().difference(value);
    self.event_update(new, events)
  }
}

pub trait EventIntersection<RHS = Self>
{
  fn event_intersection(&mut self, other: &mut RHS,
    events: &mut Vec<(usize, FDEvent)>) -> bool;
}

impl<Domain> EventIntersection for FDVar<Domain> where
  Domain: VarDomain + Intersection<Output=Domain> + Clone
{
  fn event_intersection(&mut self, other: &mut FDVar<Domain>,
    events: &mut Vec<(usize, FDEvent)>) -> bool
  {
    let new = self.dom.clone().intersection(other.dom.clone());
    self.event_update(new.clone(), events) &&
    other.event_update(new, events)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use super::FDEvent::*;
  use solver::variable::Variable;
  use interval::ncollections::ops::*;

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

  fn var_update_test_one(mut var: FDVar<Interval<i32>>, dom: Interval<i32>, expect: Vec<FDEvent>, expect_success: bool) {
    let mut events = vec![];
    assert_eq!(var.event_update(dom, &mut events), expect_success);
    if expect_success {
      assert_eq_events(events, expect);
      assert_eq!(var.dom, dom);
    }
  }

  fn assert_eq_events(events: Vec<(usize, FDEvent)>, expect: Vec<FDEvent>) {
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

  fn var_update_lb_test_one(mut var: FDVar<Interval<i32>>, lb: i32, expect: Vec<FDEvent>, expect_success: bool) {
    let ub = var.upper();
    let mut events = vec![];
    assert_eq!(var.event_shrink_left(lb, &mut events), expect_success);
    if expect_success {
      assert_eq_events(events, expect);
      assert_eq!(var.dom, (lb,ub).to_interval());
    }
  }

  fn var_update_ub_test_one(mut var: FDVar<Interval<i32>>, ub: i32, expect: Vec<FDEvent>, expect_success: bool) {
    let lb = var.lower();
    let mut events = vec![];
    assert_eq!(var.event_shrink_right(ub, &mut events), expect_success);
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

  fn var_intersection_test_one(mut v1: FDVar<Interval<i32>>, mut v2: FDVar<Interval<i32>>, events: Option<Vec<(usize, FDEvent)>>) {
    let mut ev = vec![];
    let res =
      if v1.event_intersection(&mut v2, &mut ev) {
        Some(ev)
      }
      else {
        None
      };
    assert_eq!(res, events);
  }

  #[test]
  #[should_panic]
  fn var_non_monotonic_update_singleton() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10: FDVar<Interval<i32>> = Variable::new(0, dom0_10);
    let dom11_11 = 11.to_interval();

    var0_10.event_update(dom11_11, &mut vec![]);
  }

  #[test]
  #[should_panic]
  fn var_non_monotonic_update_widen() {
    let dom0_10 = (0,10).to_interval();
    let mut var0_10: FDVar<Interval<i32>> = Variable::new(0, dom0_10);
    let domm5_15 = (-5, 15).to_interval();

    var0_10.event_update(domm5_15, &mut vec![]);
  }
}