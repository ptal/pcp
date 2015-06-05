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
use search::branching::*;
use search::branching::branch::*;
use variable::ops::*;
use variable::arithmetics::*;
use propagators::cmp::*;
use interval::ncollections::ops::*;
use num::traits::Num;
use num::PrimInt;

pub struct BinarySplit;

pub type XLessEqC<X, C> = XLessEqY<Identity<X>, Constant<C>, C>;
pub type XGreaterC<X, C> = XGreaterY<Identity<X>, Constant<C>>;

// See discussion about type bounds: https://github.com/ptal/pcp/issues/11
impl<VStore, CStore, VLabel, CLabel, Domain, Bound> Distributor<(VStore, CStore)> for BinarySplit where
  VStore: State<Label = VLabel> + Iterable<Value=Domain>,
  CStore: State<Label = CLabel>,
  CStore: Assign<XLessEqC<Domain, Bound>>,
  CStore: Assign<XGreaterC<Domain, Bound>>,
  VLabel: Clone,
  CLabel: Clone,
  Domain: Clone + Cardinality + Bounded<Bound=Bound> + 'static,
  Bound: PrimInt + Num + PartialOrd + Clone + Bounded<Bound=Bound> + 'static
{
  fn distribute(&mut self, space: &(VStore, CStore), var_idx: usize) -> Vec<Branch<(VStore, CStore)>> {
    let dom = nth_dom(&space.0, var_idx);
    assert!(!dom.is_singleton() && !dom.is_empty(),
      "Can not distribute over assigned or failed variables.");
    let mid = (dom.lower() + dom.upper()) / (Bound::one() + Bound::one());
    let mid = Constant::new(mid);
    let x = Identity::<Domain>::new(var_idx);
    let x_less_mid = x_leq_y(x.clone(), mid.clone());
    let x_geq_mid = x_greater_y(x, mid);

    Branch::distribute(space,
      vec![
        Box::new(move |space: &mut (VStore, CStore)| {
          space.1.assign(x_less_mid);
        }),
        Box::new(move |space: &mut (VStore, CStore)| {
          space.1.assign(x_geq_mid);
        })
      ]
    )
  }
}

pub fn nth_dom<VStore, Domain>(vstore: &VStore, var_idx: usize) -> Domain where
  VStore: Iterable<Value=Domain>,
  Domain: Clone
{
  vstore.iter()
  .nth(var_idx)
  .expect("Number of variable in a space can not decrease.")
  .clone()
}

#[cfg(test)]
mod test {
  use super::*;
  use search::branching::Distributor;
  use interval::interval::*;
  use interval::ops::*;
  use kernel::*;
  use kernel::trilean::Trilean::*;
  use propagation::store::Store;
  use propagation::events::*;
  use propagation::reactors::*;
  use propagation::schedulers::*;
  use variable::ops::*;
  use variable::delta_store::DeltaStore;

  type VStore = DeltaStore<Interval<i32>, FDEvent>;
  type CStore = Store<VStore, FDEvent, IndexedDeps, RelaxedFifo>;

  fn test_distributor<D>(mut distributor: D, distribution_index: usize,
    root: Vec<(i32, i32)>, children: Vec<(i32, i32)>) where
   D: Distributor<(VStore, CStore)>
  {
    let mut space = (VStore::new(), CStore::new());

    for (l,u) in root {
      space.0.assign(Interval::new(l,u));
    }

    let branches = distributor.distribute(&space, distribution_index);

    assert_eq!(branches.len(), children.len());

    for (branch, (l,u)) in branches.into_iter().zip(children.into_iter()) {
      space = branch.commit(space);
      assert_eq!(space.1.consistency(&mut space.0), True);
      let split_dom = nth_dom(&space.0, distribution_index);
      assert_eq!(split_dom, Interval::new(l,u));
    }
  }

  #[test]
  fn binary_split_distribution() {
    let vars = vec![(1,10),(2,4),(1,2)];
    test_distributor(BinarySplit, 0,
      vars.clone(),
      vec![(1,5),(6,10)]
    );
    test_distributor(BinarySplit, 1,
      vars.clone(),
      vec![(2,3),(4,4)]
    );
    test_distributor(BinarySplit, 2,
      vars.clone(),
      vec![(1,1),(2,2)]
    );
  }

  #[test]
  #[should_panic]
  fn binary_split_impossible_distribution() {
    test_distributor(BinarySplit, 0,
      vec![(1,1)],
      vec![]
    );
  }

  #[test]
  #[should_panic]
  fn binary_split_impossible_distribution_2() {
    test_distributor(BinarySplit, 2,
      vec![(1,5),(2,4),(4,4)],
      vec![]
    );
  }
}
