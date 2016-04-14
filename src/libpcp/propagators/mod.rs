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

//! Propagators are implementations of constraints, a single constraint can be realized by different propagators.
//!
//! We keep the propagator implementations generic over domains implementing specific operations (e.g. intersection or union). Propagators are also implemented to work on variable views, you can always obtain a view from a variable by using the `Identity` view.

pub mod cmp;
pub mod distinct;

use kernel::trilean::Trilean;
use kernel::trilean::Trilean::*;
use kernel::Consistency;
use propagation::ops::*;

// FIXME: Without this trait, `Consistency` impl for the Propagator store conflicts with the following one.
pub trait PropagatorKind {}

impl<VStore, R> Consistency<VStore> for R where
  R: Subsumption<VStore>,
  R: Propagator<VStore>,
  R: PropagatorKind
{
  fn consistency(&mut self, store: &mut VStore) -> Trilean {
    if self.propagate(store) {
      self.is_subsumed(store)
    } else {
      False
    }
  }
}

#[cfg(test)]
pub mod test {
  use gcollections::ops::*;
  use kernel::*;
  use propagation::*;
  use propagation::events::*;
  use interval::interval::*;
  use term::identity::*;
  use variable::delta_store::test::consume_delta;
  pub use variable::delta_store::test::FDStore;

  pub type FDVar = Identity<Interval<i32>>;

  pub fn subsumption_propagate<P>(test_num: u32, mut prop: P, store: &mut FDStore,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool) where
   P: Propagator<FDStore> + Subsumption<FDStore>
  {
    println!("Test number {}", test_num);
    assert_eq!(prop.is_subsumed(store), before);
    assert_eq!(prop.propagate(store), propagate_success);
    if propagate_success {
      consume_delta(store, delta_expected);
    }
    assert_eq!(prop.is_subsumed(store), after);
  }

  pub fn binary_propagator_test<P, FnProp>(test_num: u32, make_prop: FnProp, x: Interval<i32>, y: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool) where
   P: Propagator<FDStore> + Subsumption<FDStore>,
   FnProp: FnOnce(FDVar, FDVar) -> P
  {
    let mut store = FDStore::empty();
    let x = store.alloc(x);
    let y = store.alloc(y);
    let propagator = make_prop(x, y);
    subsumption_propagate(test_num, propagator, &mut store, before, after, delta_expected, propagate_success);
  }

  pub fn nary_propagator_test<P, FnProp>(test_num: u32, make_prop: FnProp, doms: Vec<Interval<i32>>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool) where
   P: Propagator<FDStore> + Subsumption<FDStore>,
   FnProp: FnOnce(Vec<FDVar>) -> P
  {
    let mut store = FDStore::empty();
    let vars = doms.into_iter().map(|d| store.alloc(d)).collect();
    let propagator = make_prop(vars);
    subsumption_propagate(test_num, propagator, &mut store, before, after, delta_expected, propagate_success);
  }
}
