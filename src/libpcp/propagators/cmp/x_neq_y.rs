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
use propagators::cmp::x_eq_y::*;
use propagators::PropagatorKind;
use propagation::*;
use propagation::events::*;
use term::ops::*;
use gcollections::ops::*;
use std::fmt::{Formatter, Debug, Error};

#[derive(Clone, Copy)]
pub struct XNeqY<X, Y>
{
  x: X,
  y: Y
}

impl<X, Y> PropagatorKind for XNeqY<X, Y> {}

impl<X, Y> XNeqY<X, Y> {
  pub fn new(x: X, y: Y) -> XNeqY<X, Y> {
    XNeqY { x: x, y: y }
  }
}

impl<X, Y> Debug for XNeqY<X, Y> where
  X: Debug,
  Y: Debug
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("{:?} != {:?}", self.x, self.y))
  }
}

impl<Store, BX, BY, DomX, DomY, X, Y> Subsumption<Store> for XNeqY<X, Y> where
  X: StoreRead<Store, Value=DomX> + Clone,
  Y: StoreRead<Store, Value=DomY> + Clone,
  DomX: Bounded<Bound=BX> + Disjoint<DomY>,
  DomY: Bounded<Bound=BY>,
  BX: PartialOrd + PartialOrd<BY>,
  BY: PartialOrd
{
  fn is_subsumed(&self, store: &Store) -> Trilean {
    !XEqY::new(self.x.clone(), self.y.clone()).is_subsumed(store)
  }
}

impl<Store, BX, BY, DomX, DomY, X, Y> Propagator<Store> for XNeqY<X, Y> where
  X: StoreRead<Store, Value=DomX> + StoreMonotonicUpdate<Store, DomX>,
  Y: StoreRead<Store, Value=DomY> + StoreMonotonicUpdate<Store, DomY>,
  DomX: Bounded<Bound=BX> + Cardinality + Difference<BY, Output=DomX>,
  DomY: Bounded<Bound=BY> + Cardinality + Difference<BX, Output=DomY>,
  BX: PartialOrd,
  BY: PartialOrd
{
  fn propagate(&mut self, store: &mut Store) -> bool {
    let x = self.x.read(store);
    let y = self.y.read(store);

    if x.is_singleton() {
      self.y.update(store, y.difference(&x.lower()))
    }
    else if y.is_singleton() {
      self.x.update(store, x.difference(&y.lower()))
    }
    else {
      true
    }
  }
}

impl<X, Y> PropagatorDependencies<FDEvent> for XNeqY<X, Y> where
  X: ViewDependencies<FDEvent> + Clone,
  Y: ViewDependencies<FDEvent> + Clone
{
  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    XEqY::new(self.x.clone(), self.y.clone()).dependencies()
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
  fn x_neq_y_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom11_20 = (11,20).to_interval();
    let one = (1,1).to_interval();
    let zero = (0,0).to_interval();
    let ten = (10,10).to_interval();

    x_neq_y_test_one(1, dom0_10, dom0_10, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(2, dom0_10, dom10_20, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(3, dom5_15, dom10_20, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(4, dom0_10, dom11_20, True, True, vec![], true);
    x_neq_y_test_one(5, one, dom0_10, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(6, zero, dom0_10, Unknown, True, vec![(1, Bound)], true);
    x_neq_y_test_one(7, ten, dom0_10, Unknown, True, vec![(1, Bound)], true);
    x_neq_y_test_one(8, one, one, False, False, vec![], false);
    x_neq_y_test_one(9, zero, one, True, True, vec![], true);
  }

  fn x_neq_y_test_one(test_num: u32, x: Interval<i32>, y: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    binary_propagator_test(test_num, XNeqY::new, x, y, before, after, delta_expected, propagate_success);
  }
}
