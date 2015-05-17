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
use solver::fd::event::*;
use variable::ops::*;
use interval::ncollections::ops::*;

#[derive(Clone, Copy)]
pub struct XNeqY<X, Y>
{
  x: X,
  y: Y
}

impl<X, Y> XNeqY<X, Y> {
  pub fn new(x: X, y: Y) -> XNeqY<X, Y> {
    XNeqY { x: x, y: y }
  }
}

impl<Store, Domain, X, Y> Subsumption<Store> for XNeqY<X, Y> where
  X: StoreRead<Store, Value=Domain> + Copy,
  Y: StoreRead<Store, Value=Domain> + Copy,
  Domain: Bounded + Disjoint
{
  fn is_subsumed(&self, store: &Store) -> Trilean {
    !XEqY::new(self.x, self.y).is_subsumed(store)
  }
}

impl<Store, Domain, X, Y> Propagator<Store> for XNeqY<X, Y> where
  X: StoreRead<Store, Value=Domain> + StoreMonotonicUpdate<Store, Domain>,
  Y: StoreRead<Store, Value=Domain> + StoreMonotonicUpdate<Store, Domain>,
  Domain: Bounded + Cardinality,
  Domain: Difference<<Domain as Bounded>::Bound, Output=Domain>
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
  X: ViewDependencies<FDEvent> + Copy,
  Y: ViewDependencies<FDEvent> + Copy
{
  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    XEqY::new(self.x, self.y).dependencies()
  }
}

impl<X, Y> DeepClone for XNeqY<X, Y> where
  X: Copy,
  Y: Copy
{
  fn deep_clone(&self) -> XNeqY<X, Y> {
    *self
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
  fn x_neq_y_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom11_20 = (11,20).to_interval();
    let one = (1,1).to_interval();
    let zero = (0,0).to_interval();
    let ten = (10,10).to_interval();

    x_neq_y_test_one(dom0_10, dom0_10, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(dom0_10, dom10_20, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(dom5_15, dom10_20, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(dom0_10, dom11_20, True, True, vec![], true);
    x_neq_y_test_one(one, dom0_10, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(zero, dom0_10, Unknown, True, vec![(1, Bound)], true);
    x_neq_y_test_one(ten, dom0_10, Unknown, True, vec![(1, Bound)], true);
    x_neq_y_test_one(one, one, False, False, vec![], false);
  }

  fn x_neq_y_test_one(x: Interval<i32>, y: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    binary_propagator_test(XNeqY::new, x, y, before, after, delta_expected, propagate_success);
  }
}
