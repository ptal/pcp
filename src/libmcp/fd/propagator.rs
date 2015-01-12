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
use propagator::*;
use propagator::Status::*;

pub struct EqualXY {
  x: FDVar,
  y: FDVar
}

impl EqualXY {
  pub fn new(x: FDVar, y: FDVar) -> EqualXY {
    EqualXY { x: x, y: y }
  }
}

impl Propagator for EqualXY {
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

  fn propagate(&mut self) -> Vec<(u32, <EqualXY as Propagator>::Event)> {
    FDVar::intersection(&mut self.x, &mut self.y)
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

  fn propagate(&mut self) -> Vec<(u32, <XLessThanY as Propagator>::Event)> {
    let mut events = vec![];
    if self.x.lb() >= self.y.lb() {
      let ev = self.y.update_lb(self.x.lb() + 1);
      events.push((self.x.id(), ev));
    }
    if self.x.ub() >= self.y.ub() {
      let ev = self.x.update_ub(self.y.ub() - 1);
      events.push((self.y.id(), ev));
    }
    events
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use fd::var::*;
  use fd::var::FDEvent::*;
  use propagator::Status::*;
  use propagator::*;

  #[test]
  fn equalxy_propagate_test() {
    let empty = FDVar::new(0, Interval::empty());
    let var0_10 = FDVar::new(0, (0,10).to_interval());
    let var10_20 = FDVar::new(0, (10,20).to_interval());
    let var5_15 = FDVar::new(0, (5,15).to_interval());
    let var11_20 = FDVar::new(0, (11,20).to_interval());
    let var1_1 = FDVar::new(0, (1,1).to_interval());

    equalxy_propagate_test_one(var0_10, var10_20, Unknown, Entailed, vec![Assignment, Assignment]);
    equalxy_propagate_test_one(var5_15, var10_20, Unknown, Unknown, vec![Bound, Bound]);
    equalxy_propagate_test_one(var1_1, var0_10, Unknown, Entailed, vec![Nothing, Assignment]);
    equalxy_propagate_test_one(var0_10, var0_10, Unknown, Unknown, vec![Nothing, Nothing]);
    equalxy_propagate_test_one(var0_10, empty, Disentailed, Disentailed, vec![Failure, Nothing]);
    equalxy_propagate_test_one(var1_1, empty, Disentailed, Disentailed, vec![Failure, Nothing]);
    equalxy_propagate_test_one(var0_10, var11_20, Disentailed, Disentailed, vec![Failure, Failure]);
  }

  fn equalxy_propagate_test_one(v1: FDVar, v2: FDVar, before: Status, after: Status, events_expected: Vec<FDEvent>) {
    let mut propagator = EqualXY::new(v1, v2);
    assert_eq!(propagator.status(), before);
    let var_events = propagator.propagate();
    for ((_,ev), expected) in var_events.into_iter().zip(events_expected.into_iter()) {
      assert_eq!(ev, expected);
    }
    assert_eq!(propagator.status(), after);
  }

  // #[test]
  // fn xlessy_propagate_test() {
  //   let empty = FDVar::new(0, Interval::empty());
  //   let var0_10 = FDVar::new(0, (0,10).to_interval());
  //   let var10_20 = FDVar::new(0, (10,20).to_interval());
  //   let var5_15 = FDVar::new(0, (5,15).to_interval());
  //   let var11_20 = FDVar::new(0, (11,20).to_interval());
  //   let var1_1 = FDVar::new(0, (1,1).to_interval());

  //   xlessy_propagate_test_one(var0_10, var10_20, Unknown, Unknown, vec![Nothing, Nothing]);
  //   xlessy_propagate_test_one(var5_15, var10_20, Unknown, Unknown, vec![Nothing, Nothing]);
  //   xlessy_propagate_test_one(var5_15, var0_10, Unknown, Unknown, vec![Bound, Bound]);
  //   xlessy_propagate_test_one(var0_10, var11_20, Entailed, Entailed, vec![Nothing, Nothing]);
  //   xlessy_propagate_test_one(var11_20, var0_10, Disentailed, Disentailed, vec![Failure, Failure]);
  // }
}
