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

use variable::ops::*;
use term::ExprInference;
use gcollections::ops::*;
use gcollections::wrappers::*;
use std::fmt::{Formatter, Debug, Error};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Constant<V>
{
  value: V
}

impl<V> ExprInference for Constant<V>
{
  type Output = V;
}

impl<V> Constant<V>
{
  pub fn new(value: V) -> Constant<V> {
    Constant {
      value: value
    }
  }
}

impl<V> Debug for Constant<V> where
  V: Debug
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("{:?}", self.value))
  }
}


impl<V, Domain, Store> StoreMonotonicUpdate<Store, Domain> for Constant<V> where
  Domain: Cardinality
{
  fn update(&self, _store: &mut Store, value: Domain) -> bool {
    !value.is_empty()
  }
}

impl<V, Store> StoreRead<Store> for Constant<V> where
  V: Clone
{
  type Value = Optional<V>;
  fn read(&self, _store: &Store) -> Optional<V> {
    Optional::singleton(self.value.clone())
  }
}

impl<V, Event> ViewDependencies<Event> for Constant<V>
{
  fn dependencies(&self, _event: Event) -> Vec<(usize, Event)> {
    vec![]
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use gcollections::ops::*;
  use kernel::*;
  use kernel::trilean::Trilean::*;
  use propagation::*;
  use propagation::events::FDEvent;
  use propagation::events::FDEvent::*;
  use variable::delta_store::*;
  use variable::ops::*;
  use propagators::test::*;
  use propagators::cmp::*;
  use interval::interval::*;

  #[test]
  fn x_less_constant() {
    let dom0_10 = (0,10).to_interval();
    let dom0_4 = (0,4).to_interval();
    let mut store: FDStore = DeltaStore::empty();
    let x = store.alloc(dom0_10);
    let c: Constant<i32> = Constant::new(5);

    let x_less_c = XLessY::new(x, c);
    subsumption_propagate(1, x_less_c, &mut store, Unknown, True, vec![(0, Bound)], true);
    assert_eq!(x.read(&store), dom0_4);
  }

  #[test]
  fn unary_propagator_test() {
    let dom0_10 = (0,10).to_interval();
    let dom0_0 = (0,0).to_interval();

    unary_propagator_test_one(1, dom0_10, 0, XLessY::new, False, False, vec![], false);
    unary_propagator_test_one(2, dom0_10, 11, XLessY::new, True, True, vec![], true);
    unary_propagator_test_one(3, dom0_10, 10, XLessY::new, Unknown, True, vec![(0, Bound)], true);

    unary_propagator_test_one(4, dom0_10, -1, x_leq_y, False, False, vec![], false);
    unary_propagator_test_one(5, dom0_10, 10, x_leq_y, True, True, vec![], true);
    unary_propagator_test_one(6, dom0_10, 9, x_leq_y, Unknown, True, vec![(0, Bound)], true);

    unary_propagator_test_one(7, dom0_10, 10, x_greater_y, False, False, vec![], false);
    unary_propagator_test_one(8, dom0_10, -1, x_greater_y, True, True, vec![], true);
    unary_propagator_test_one(9, dom0_10, 0, x_greater_y, Unknown, True, vec![(0, Bound)], true);

    unary_propagator_test_one(10, dom0_10, 11, x_geq_y, False, False, vec![], false);
    unary_propagator_test_one(11, dom0_10, 0, x_geq_y, True, True, vec![], true);
    unary_propagator_test_one(12, dom0_10, 1, x_geq_y, Unknown, True, vec![(0, Bound)], true);

    unary_propagator_test_one(13, dom0_0, 0, XNeqY::new, False, False, vec![], false);
    unary_propagator_test_one(14, dom0_10, 5, XNeqY::new, Unknown, Unknown, vec![], true);
    unary_propagator_test_one(15, dom0_10, 0, XNeqY::new, Unknown, True, vec![(0, Bound)], true);
    unary_propagator_test_one(16, dom0_10, 10, XNeqY::new, Unknown, True, vec![(0, Bound)], true);
  }

  fn unary_propagator_test_one<P, R>(id: u32, x: Interval<i32>, c: i32, make_prop: P,
    before: Trilean, after: Trilean, expected: Vec<(usize, FDEvent)>, update_success: bool) where
   P: FnOnce(FDVar, Constant<i32>) -> R,
   R: Propagator<FDStore> + Subsumption<FDStore>
  {
    let mut store: FDStore = DeltaStore::empty();
    let x = store.alloc(x);
    let propagator = make_prop(x, Constant::new(c));
    subsumption_propagate(id, propagator, &mut store, before, after, expected, update_success);
  }
}
