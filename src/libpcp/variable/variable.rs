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

use kernel::event::*;
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
