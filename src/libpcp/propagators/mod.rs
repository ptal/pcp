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

#[cfg(test)]
pub mod test {
  use kernel::*;
  use solver::fd::event::*;
  use interval::interval::*;
  use variable::ops::*;
  use variable::arithmetics::identity::*;
  use variable::delta_store::*;
  use variable::delta_store::test::*;

  pub type FDStore = DeltaStore<FDEvent, Interval<i32>>;
  pub type FDVar = Identity<Interval<i32>>;

  pub fn subsumption_propagate<P>(mut prop: P, mut store: FDStore,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool) where
   P: Propagator<FDStore> + Subsumption<FDStore>
  {
    assert_eq!(prop.is_subsumed(&store), before);
    assert_eq!(prop.propagate(&mut store), propagate_success);
    if propagate_success {
      consume_delta(&mut store, delta_expected);
    }
    assert_eq!(prop.is_subsumed(&store), after);
  }

  pub fn binary_propagator_test<P, FnProp>(make_prop: FnProp, x: Interval<i32>, y: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool) where
   P: Propagator<FDStore> + Subsumption<FDStore>,
   FnProp: FnOnce(FDVar, FDVar) -> P
  {
    let mut store = FDStore::new();
    let x = store.assign(x);
    let y = store.assign(y);
    let propagator = make_prop(x, y);
    subsumption_propagate(propagator, store, before, after, delta_expected, propagate_success);
  }
}
