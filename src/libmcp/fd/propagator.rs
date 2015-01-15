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

use fd::var::*;
use fd::var::FDEvent::*;
use propagator::*;
use propagator::Status::*;

pub struct XEqualY {
  x: FDVar,
  y: FDVar
}

impl XEqualY {
  pub fn new(x: FDVar, y: FDVar) -> XEqualY {
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

    if FDVar::is_disjoint(&self.x, &self.y) {
      Disentailed
    }
    else if self.x.lb() == self.y.ub() && self.x.ub() == self.y.lb() {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self) -> Option<Vec<(u32, <XEqualY as Propagator>::Event)>> {
    FDVar::intersection(&mut self.x, &mut self.y)
  }

  fn dependencies(&self) -> Vec<(u32, <XEqualY as Propagator>::Event)> {
    vec![(self.x.id(), Inner), (self.y.id(), Inner)]
  }
}

pub struct XLessThanY {
  x: FDVar,
  y: FDVar
}

impl XLessThanY {
  pub fn new(x: FDVar, y: FDVar) -> XLessThanY {
    XLessThanY { x: x, y: y }
  }
}

impl Propagator for XLessThanY {
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

    if self.x.lb() > self.y.ub() {
      Disentailed
    }
    else if self.x.ub() < self.y.lb() {
      Entailed
    }
    else {
      Unknown
    }
  }

  fn propagate(&mut self) -> Option<Vec<(u32, <XLessThanY as Propagator>::Event)>> {
    let mut events = vec![];
    if self.x.ub() >= self.y.ub() {
      if !self.x.update_ub(self.y.ub() - 1, &mut events) {
        return None;
      }
    }
    if self.x.lb() >= self.y.lb() {
      if !self.y.update_lb(self.x.lb() + 1, &mut events) {
        return None;
      }
    }
    Some(events)
  }

  fn dependencies(&self) -> Vec<(u32, <XEqualY as Propagator>::Event)> {
    vec![(self.x.id(), Bound), (self.y.id(), Bound)]
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use fd::var::*;
  use fd::var::FDEvent::*;
  use propagator::Status::*;
  use propagator::*;

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

  #[test]
  fn equalxy_propagate_test() {
    let var0_10 = Variable::new(1, (0,10).to_interval());
    let var10_20 = Variable::new(2, (10,20).to_interval());
    let var5_15 = Variable::new(3, (5,15).to_interval());
    let var11_20 = Variable::new(4, (11,20).to_interval());
    let var1_1 = Variable::new(5, (1,1).to_interval());

    xequaly_propagate_test_one(var0_10, var10_20, Unknown, Entailed, Some(vec![(1, Assignment), (2, Assignment)]));
    xequaly_propagate_test_one(var5_15, var10_20, Unknown, Unknown, Some(vec![(3, Bound), (2, Bound)]));
    xequaly_propagate_test_one(var1_1, var0_10, Unknown, Entailed, Some(vec![(1, Assignment)]));
    xequaly_propagate_test_one(var0_10, var0_10, Unknown, Unknown, Some(vec![]));
    xequaly_propagate_test_one(var0_10, var11_20, Disentailed, Disentailed, None);
  }

  fn xequaly_propagate_test_one(v1: FDVar, v2: FDVar, before: Status, after: Status, expected: Option<Vec<(u32, FDEvent)>>) {
    let propagator = XEqualY::new(v1, v2);
    propagate_test_one(propagator, before, after, expected);
  }

  #[test]
  fn xlessy_propagate_test() {
    let var0_10 = Variable::new(1, (0,10).to_interval());
    let var10_20 = Variable::new(2, (10,20).to_interval());
    let var5_15 = Variable::new(3, (5,15).to_interval());
    let var11_20 = Variable::new(4, (11,20).to_interval());
    let var1_1 = Variable::new(5, (1,1).to_interval());

    xlessy_propagate_test_one(var0_10, var10_20, Unknown, Unknown, Some(vec![]));
    xlessy_propagate_test_one(var5_15, var10_20, Unknown, Unknown, Some(vec![]));
    xlessy_propagate_test_one(var5_15, var0_10, Unknown, Unknown, Some(vec![(3, Bound), (1, Bound)]));
    xlessy_propagate_test_one(var0_10, var11_20, Entailed, Entailed, Some(vec![]));
    xlessy_propagate_test_one(var11_20, var0_10, Disentailed, Disentailed, None);
    xlessy_propagate_test_one(var1_1, var0_10, Unknown, Entailed, Some(vec![(1, Bound)]));
  }

  fn xlessy_propagate_test_one(v1: FDVar, v2: FDVar, before: Status, after: Status, expected: Option<Vec<(u32, FDEvent)>>) {
    let propagator = XLessThanY::new(v1, v2);
    propagate_test_one(propagator, before, after, expected);
  }
}
