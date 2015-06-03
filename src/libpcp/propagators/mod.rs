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

pub mod cmp;
pub mod distinct;

#[cfg(test)]
pub mod test {
  use kernel::*;
  use propagation::event::*;
  use interval::interval::*;
  use variable::ops::*;
  use variable::arithmetics::identity::*;
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
    let mut store = FDStore::new();
    let x = store.assign(x);
    let y = store.assign(y);
    let propagator = make_prop(x, y);
    subsumption_propagate(test_num, propagator, &mut store, before, after, delta_expected, propagate_success);
  }

  pub fn nary_propagator_test<P, FnProp>(test_num: u32, make_prop: FnProp, doms: Vec<Interval<i32>>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool) where
   P: Propagator<FDStore> + Subsumption<FDStore>,
   FnProp: FnOnce(Vec<FDVar>) -> P
  {
    let mut store = FDStore::new();
    let vars = doms.into_iter().map(|d| store.assign(d)).collect();
    let propagator = make_prop(vars);
    subsumption_propagate(test_num, propagator, &mut store, before, after, delta_expected, propagate_success);
  }
}
