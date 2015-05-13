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

use solver::event::*;
use variable::ops::*;
use interval::ncollections::ops::*;
use interval::ops::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{Formatter, Display, Error};
use std::ops::Deref;

pub type SharedVar<Domain> = Rc<RefCell<Variable<Domain>>>;

#[derive(Copy, PartialEq, Eq, Debug, Clone)]
pub struct Variable<Domain> {
  idx: usize,
  dom: Domain
}

impl<Domain> Deref for Variable<Domain> {
  type Target = Domain;

  fn deref<'a>(&'a self) -> &'a Domain {
    &self.dom
  }
}

impl<Domain: Display> Display for Variable<Domain> {
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("({}, {})", self.idx, self.dom))
  }
}

impl<Domain> Variable<Domain> where
  Domain: Cardinality
{
  pub fn new(idx: usize, dom: Domain) -> Variable<Domain> {
    assert!(!dom.is_empty());
    Variable {
      idx: idx,
      dom: dom
    }
  }
}

impl<Domain> VarIndex for Variable<Domain>
{
  fn index(&self) -> usize {
    self.idx
  }
}

impl<Domain> Failure for Variable<Domain> where
  Domain: Cardinality
{
  fn is_failed(&self) -> bool {
    self.dom.is_empty()
  }
}

impl<Domain> EventUpdate<Domain> for Variable<Domain> where
  Domain: VarDomain
{
  fn event_update<Event>(&mut self, dom: Domain,
    events: &mut Vec<(usize, Event)>) -> bool
   where
    Event: MonotonicEvent<Domain>
  {
    assert!(dom.is_subset(&self.dom), "Domain update must be monotonic.");
    if dom.is_empty() { false } // Failure
    else {
      if let Some(event) = Event::new(&dom, &self.dom) {
        events.push((self.idx, event));
        self.dom = dom;
      }
      true
    }
  }
}

impl<Domain> EventShrinkLeft<Domain> for Variable<Domain> where
  Domain: VarDomain + ShrinkLeft<<Domain as Bounded>::Bound>
{
  fn event_shrink_left<Event>(&mut self, lb: Domain::Bound,
    events: &mut Vec<(usize, Event)>) -> bool
   where
    Event: MonotonicEvent<Domain>
  {
    let new = self.dom.shrink_left(lb);
    self.event_update(new, events)
  }
}

impl<Domain> EventShrinkRight<Domain> for Variable<Domain> where
  Domain: VarDomain + ShrinkRight<<Domain as Bounded>::Bound>
{
  fn event_shrink_right<Event>(&mut self, ub: Domain::Bound,
    events: &mut Vec<(usize, Event)>) -> bool
   where
    Event: MonotonicEvent<Domain>
  {
    let new = self.dom.shrink_right(ub);
    self.event_update(new, events)
  }
}

impl<Domain> EventRemove<Domain> for Variable<Domain> where
  Domain: VarDomain + Difference<<Domain as Bounded>::Bound, Output=Domain>
{
  fn event_remove<Event>(&mut self, value: Domain::Bound,
    events: &mut Vec<(usize, Event)>) -> bool
   where
    Event: MonotonicEvent<Domain>
  {
    let new = self.dom.difference(&value);
    self.event_update(new, events)
  }
}

impl<Domain> EventIntersection<Domain> for Variable<Domain> where
  Domain: VarDomain + Intersection<Output=Domain> + Clone
{
  fn event_intersection<Event>(&mut self, other: &mut Variable<Domain>,
    events: &mut Vec<(usize, Event)>) -> bool
   where
    Event: MonotonicEvent<Domain>
  {
    let new = self.dom.intersection(&other.dom);
    self.event_update(new.clone(), events) &&
    other.event_update(new, events)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use variable::ops::*;
  use solver::fd::event::*;
  use solver::fd::event::FDEvent::*;
  use interval::interval::*;
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

  fn var_update_test_one(mut var: Variable<Interval<i32>>, dom: Interval<i32>, expect: Vec<FDEvent>, expect_success: bool) {
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

  fn var_update_lb_test_one(mut var: Variable<Interval<i32>>, lb: i32, expect: Vec<FDEvent>, expect_success: bool) {
    let ub = var.upper();
    let mut events = vec![];
    assert_eq!(var.event_shrink_left(lb, &mut events), expect_success);
    if expect_success {
      assert_eq_events(events, expect);
      assert_eq!(var.dom, (lb,ub).to_interval());
    }
  }

  fn var_update_ub_test_one(mut var: Variable<Interval<i32>>, ub: i32, expect: Vec<FDEvent>, expect_success: bool) {
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

  fn var_intersection_test_one(mut v1: Variable<Interval<i32>>, mut v2: Variable<Interval<i32>>, events: Option<Vec<(usize, FDEvent)>>) {
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
}
