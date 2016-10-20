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

use term::ops::*;
use term::ExprInference;
use std::ops::*;
use std::fmt::{Formatter, Debug, Error};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Multiplication<X, V>
{
  x: X,
  v: V
}

impl<X, V, R> ExprInference for Multiplication<X, V> where
  X: Mul<V, Output=R>
{
  type Output = R;
}

impl<X, V> Multiplication<X, V> {
  pub fn new(x: X, v: V) -> Multiplication<X, V> {
    Multiplication {
      x: x,
      v: v
    }
  }
}

impl<X, V> Debug for Multiplication<X, V> where
  X: Debug,
  V: Debug
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("{:?} * {:?}", self.x, self.v))
  }
}

impl<X, V, Domain, Store> StoreMonotonicUpdate<Store, Domain> for Multiplication<X, V> where
  Domain: Sub<V, Output=Domain>,
  V: Clone,
  X: StoreMonotonicUpdate<Store, Domain>
{
  fn update(&self, store: &mut Store, value: Domain) -> bool {
    self.x.update(store, value - self.v.clone())
  }
}

impl<X, V, Domain, Store> StoreRead<Store> for Multiplication<X, V> where
  Domain: Mul<V, Output=Domain>,
  V: Clone,
  X: StoreRead<Store, Value=Domain>
{
  type Value = Domain;
  fn read(&self, store: &Store) -> Domain {
    self.x.read(store) * self.v.clone()
  }
}

impl<X, V, Event> ViewDependencies<Event> for Multiplication<X, V> where
  X: ViewDependencies<Event>
{
  fn dependencies(&self, event: Event) -> Vec<(usize, Event)> {
    self.x.dependencies(event)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use gcollections::ops::*;
  use kernel::*;
  use kernel::trilean::Trilean::*;
  use variable::VStoreFD;
  use propagation::events::FDEvent;
  use propagation::events::FDEvent::*;
  use propagators::test::*;
  use propagators::cmp::XLessY;
  use interval::interval::*;

  type Domain = Interval<i32>;
  type VStore = VStoreFD;

  #[test]
  fn x_less_y_mul_c() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom5_10 = (5,10).to_interval();
    let dom1_1 = (1,1).to_interval();

    // Same test as `x < y` but we add `c` to `y`.
    x_less_y_plus_c_test_one(1, dom0_10, dom5_15, 2, False, False, vec![], false);
    x_less_y_plus_c_test_one(2, dom0_10, dom0_10, 2, Unknown, Unknown, vec![], true);
    x_less_y_plus_c_test_one(3, dom5_15, dom5_15, 1, Unknown, Unknown, vec![], true);
    x_less_y_plus_c_test_one(4, dom10_20, dom5_10*2, -10, True, True, vec![], true);
    x_less_y_plus_c_test_one(7, dom5_15, dom1_1, 4, False, False, vec![], false);
    x_less_y_plus_c_test_one(7, dom5_15, dom1_1, 5, Unknown, True, vec![(1, Bound)], true);
    x_less_y_plus_c_test_one(7, dom5_15, dom1_1, 15, False, False, vec![], false);
  }

  fn x_less_y_mul_c_test_one(id: u32, x: Domain, y: Domain, c: i32,
    before: Trilean, after: Trilean, expected: Vec<(usize, FDEvent)>, update_success: bool)
  {
    let mut store = VStore::empty();
    let x = store.alloc(x);
    let y = store.alloc(y);
    let x_less_y_plus_c = XLessY::new(x, Multiplication::new(y, c));
    subsumption_propagate(id, x_less_y_plus_c, &mut store, before, after, expected, update_success);
  }
}
