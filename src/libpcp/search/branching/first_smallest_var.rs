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
use search::space::*;
use search::branching::*;
use variable::ops::Iterable;
use interval::ncollections::ops::*;
use num::traits::Unsigned;

pub struct FirstSmallestVar;

impl<VStore, CStore, Domain, Size> VarSelection<Space<VStore, CStore>> for FirstSmallestVar where
  VStore: Iterable<Value=Domain>,
  Domain: Cardinality<Size=Size>,
  Size: Ord + Unsigned
{
  fn select(&mut self, space: &Space<VStore, CStore>) -> usize {
    space.vstore.iter().enumerate()
      .filter(|&(_, v)| v.size() > Size::one())
      .min_by_key(|&(_, v)| v.size())
      .expect("Cannot select a variable in a space where all variables are assigned.")
      .0
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use interval::interval::*;
  use interval::ops::*;
  use kernel::*;
  use propagation::store::Store;
  use propagation::events::*;
  use propagation::reactors::*;
  use propagation::schedulers::*;
  use variable::delta_store::DeltaStore;
  use search::space::*;
  use search::branching::VarSelection;

  type VStore = DeltaStore<Interval<i32>, FDEvent>;
  type CStore = Store<VStore, FDEvent, IndexedDeps, RelaxedFifo>;
  type FDSpace = Space<VStore, CStore>;

  fn test_selector<S>(mut selector: S, vars: Vec<(i32, i32)>, expect: usize) where
    S: VarSelection<FDSpace>
  {
    let mut space = FDSpace::default();

    for (l,u) in vars {
      space.vstore.alloc(Interval::new(l,u));
    }

    assert_eq!(selector.select(&space), expect);
  }

  #[test]
  fn smallest_var_selection() {
    test_selector(FirstSmallestVar, vec![(1,10),(2,4),(1,1)], 1);
    test_selector(FirstSmallestVar, vec![(1,10),(2,4),(2,4)], 1);
    test_selector(FirstSmallestVar,
      vec![(1,1),(1,1),(1,10),(1,1),(2,4),(1,1),(1,1)], 4);
  }

  #[should_panic]
  #[test]
  fn smallest_var_selection_all_assigned() {
    test_selector(FirstSmallestVar, vec![(0, 0),(2,2),(1,1)], 0);
  }
}
