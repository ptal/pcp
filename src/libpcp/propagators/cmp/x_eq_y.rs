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
use propagators::PropagatorKind;
use propagation::*;
use propagation::events::*;
use term::ops::*;
use gcollections::ops::*;
use std::fmt::{Formatter, Debug, Error};

#[derive(Clone, Copy)]
pub struct XEqY<X, Y>
{
  x: X,
  y: Y
}

impl<X, Y> PropagatorKind for XEqY<X, Y> {}

impl<X, Y> XEqY<X, Y> {
  pub fn new(x: X, y: Y) -> XEqY<X, Y> {
    XEqY { x: x, y: y }
  }
}

impl<X, Y> Debug for XEqY<X, Y> where
  X: Debug,
  Y: Debug
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("{:?} = {:?}", self.x, self.y))
  }
}

impl<Store, BX, BY, DomX, DomY, X, Y> Subsumption<Store> for XEqY<X, Y> where
  X: StoreRead<Store, Value=DomX>,
  Y: StoreRead<Store, Value=DomY>,
  DomX: Bounded<Bound=BX> + Disjoint<DomY>,
  DomY: Bounded<Bound=BY>,
  BX: PartialOrd + PartialOrd<BY>,
  BY: PartialOrd
{
  fn is_subsumed(&self, store: &Store) -> Trilean {
    // False:
    // |--|
    //     |--|
    //
    // True:
    // |-|
    // |-|
    //
    // Unknown: Everything else.

    let x = self.x.read(store);
    let y = self.y.read(store);

    if x.lower() == y.upper() && x.upper() == y.lower() {
      True
    }
    else if x.is_disjoint(&y) {
      False
    }
    else {
      Unknown
    }
  }
}

impl<Store, DomX, DomY, X, Y> Propagator<Store> for XEqY<X, Y> where
  X: StoreRead<Store, Value=DomX> + StoreMonotonicUpdate<Store, DomX>,
  Y: StoreRead<Store, Value=DomY> + StoreMonotonicUpdate<Store, DomX>,
  DomX: Intersection<DomY, Output=DomX> + Clone,
{
  fn propagate(&mut self, store: &mut Store) -> bool {
    let x = self.x.read(store);
    let y = self.y.read(store);
    let new = x.intersection(&y);
    self.x.update(store, new.clone()) &&
    self.y.update(store, new)
  }
}

impl<X, Y> PropagatorDependencies<FDEvent> for XEqY<X, Y> where
  X: ViewDependencies<FDEvent>,
  Y: ViewDependencies<FDEvent>
{
  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    let mut deps = self.x.dependencies(FDEvent::Inner);
    deps.append(&mut self.y.dependencies(FDEvent::Inner));
    deps
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::*;
  use kernel::Trilean::*;
  use propagation::events::*;
  use propagation::events::FDEvent::*;
  use interval::interval::*;
  use propagators::test::*;

  #[test]
  fn x_eq_y_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom11_20 = (11,20).to_interval();
    let dom1_1 = (1,1).to_interval();

    x_eq_y_test_one(1, dom0_10, dom0_10, Unknown, Unknown, vec![], true);
    x_eq_y_test_one(2, dom0_10, dom10_20, Unknown, True, vec![(0, Assignment), (1, Assignment)], true);
    x_eq_y_test_one(3, dom5_15, dom10_20, Unknown, Unknown, vec![(0, Bound), (1, Bound)], true);
    x_eq_y_test_one(4, dom5_15, dom0_10, Unknown, Unknown, vec![(0, Bound), (1, Bound)], true);
    x_eq_y_test_one(5, dom0_10, dom11_20, False, False, vec![], false);
    x_eq_y_test_one(6, dom11_20, dom0_10, False, False, vec![], false);
    x_eq_y_test_one(7, dom1_1, dom0_10, Unknown, True, vec![(1, Assignment)], true);
  }

  fn x_eq_y_test_one(test_num: u32, x: Interval<i32>, y: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    binary_propagator_test(test_num, XEqY::new, x, y, before, after, delta_expected, propagate_success);
  }
}
