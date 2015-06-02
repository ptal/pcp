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

use kernel::*;
use kernel::Trilean::*;
use solver::fd::event::*;
use variable::ops::*;
use interval::ncollections::ops::*;

#[derive(Clone, Copy)]
pub struct XLessY<X, Y>
{
  x: X,
  y: Y
}

impl<X, Y> PropagatorKind for XLessY<X, Y> {}

impl<X, Y> XLessY<X, Y> {
  pub fn new(x: X, y: Y) -> XLessY<X, Y> {
    XLessY { x: x, y: y }
  }
}

impl<Store, BX, BY, DomX, DomY, X, Y> Subsumption<Store> for XLessY<X, Y> where
  X: StoreRead<Store, Value=DomX>,
  Y: StoreRead<Store, Value=DomY>,
  DomX: Bounded<Bound=BX>,
  DomY: Bounded<Bound=BY>,
  BX: PartialOrd + PartialOrd<BY>,
  BY: PartialOrd + PartialOrd<BX>
{
  fn is_subsumed(&self, store: &Store) -> Trilean {
    // False:
    // x:    |--|
    // y: |--|
    //
    // True:
    // x: |--|
    // y:     |--|
    //
    // Unknown: Everything else.

    let x = self.x.read(store);
    let y = self.y.read(store);

    if x.lower() >= y.upper() {
      False
    }
    else if x.upper() < y.lower() {
      True
    }
    else {
      Unknown
    }
  }
}

impl<Store, BX, BY, DomX, DomY, X, Y> Propagator<Store> for XLessY<X, Y> where
  X: StoreRead<Store, Value=DomX> + StoreMonotonicUpdate<Store, DomX>,
  Y: StoreRead<Store, Value=DomY> + StoreMonotonicUpdate<Store, DomY>,
  DomX: Bounded<Bound=BX> + StrictShrinkLeft<BY> + StrictShrinkRight<BY>,
  DomY: Bounded<Bound=BY> + StrictShrinkLeft<BX> + StrictShrinkRight<BX>,
  BX: PartialOrd,
  BY: PartialOrd
{
  fn propagate(&mut self, store: &mut Store) -> bool {
    let x = self.x.read(store);
    let y = self.y.read(store);
    self.x.update(store, x.strict_shrink_right(y.upper())) &&
    self.y.update(store, y.strict_shrink_left(x.lower()))
  }
}

impl<X, Y> PropagatorDependencies<FDEvent> for XLessY<X, Y> where
  X: ViewDependencies<FDEvent>,
  Y: ViewDependencies<FDEvent>
{
  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    let mut deps = self.x.dependencies(FDEvent::Bound);
    deps.append(&mut self.y.dependencies(FDEvent::Bound));
    deps
  }
}

impl<X, Y> DeepClone for XLessY<X, Y> where
  X: Clone,
  Y: Clone
{
  fn deep_clone(&self) -> XLessY<X, Y> {
    XLessY::new(self.x.clone(), self.y.clone())
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::*;
  use kernel::Trilean::*;
  use solver::fd::event::*;
  use solver::fd::event::FDEvent::*;
  use interval::interval::*;
  use propagators::test::*;

  #[test]
  fn x_less_y_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom10_11 = (10,11).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom11_20 = (11,20).to_interval();
    let dom1_1 = (1,1).to_interval();

    x_less_y_test_one(1, dom0_10, dom0_10, Unknown, Unknown, vec![(0, Bound), (1, Bound)], true);
    x_less_y_test_one(2, dom0_10, dom10_20, Unknown, Unknown, vec![], true);
    x_less_y_test_one(3, dom10_11, dom10_11, Unknown, True, vec![(0, Assignment), (1, Assignment)], true);
    x_less_y_test_one(4, dom5_15, dom10_20, Unknown, Unknown, vec![], true);
    x_less_y_test_one(5, dom5_15, dom0_10, Unknown, Unknown, vec![(0, Bound), (1, Bound)], true);
    x_less_y_test_one(6, dom0_10, dom11_20, True, True, vec![], true);
    x_less_y_test_one(7, dom11_20, dom0_10, False, False, vec![], false);
    x_less_y_test_one(8, dom1_1, dom0_10, Unknown, True, vec![(1, Bound)], true);
  }

  fn x_less_y_test_one(test_num: u32, x: Interval<i32>, y: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    binary_propagator_test(test_num, XLessY::new, x, y, before, after, delta_expected, propagate_success);
  }
}
