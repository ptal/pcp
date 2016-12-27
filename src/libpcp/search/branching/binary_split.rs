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
use search::space::*;
use variable::ops::*;
use term::*;
use propagators::cmp::*;
use propagation::concept::*;
use propagation::events::*;
use gcollections::ops::*;
use gcollections::*;
use std::ops::*;
use concept::*;

pub struct BinarySplit;

pub type XLessEqC<X, C> = XLessEqY<Identity<X>, Constant<C>, C>;
pub type XGreaterC<X, C> = XGreaterY<Identity<X>, Constant<C>>;

// See discussion about type bounds: https://github.com/ptal/pcp/issues/11
impl<VStore, CStore, R, Domain, Bound> Distributor<Space<VStore, CStore, R>> for BinarySplit where
  VStore: Freeze + Iterable<Item=Domain> + Index<usize, Output=Domain> + MonotonicUpdate,
  VStore: AssociativeCollection<Location=Identity<Domain>>,
  CStore: Freeze,
  CStore: Alloc + Collection<Item=Box<PropagatorConcept<VStore, FDEvent>>>,
  Domain: IntDomain<Item=Bound> + 'static,
  Bound: IntBound + 'static,
  R: FreezeSpace<VStore, CStore> + Snapshot<State=Space<VStore, CStore, R>>
{
  fn distribute(&mut self, space: Space<VStore, CStore, R>, var_idx: usize) ->
    (<Space<VStore, CStore, R> as Freeze>::FrozenState, Vec<Branch<Space<VStore, CStore, R>>>)
  {
    let dom = nth_dom(&space.vstore, var_idx);
    assert!(!dom.is_singleton() && !dom.is_empty(),
      "Can not distribute over assigned or failed variables.");
    let mid: Bound = (dom.lower() + dom.upper()) / (Bound::one() + Bound::one());
    let mid = Constant::new(mid);
    let x = Identity::<Domain>::new(var_idx);
    let x_less_mid = x_leq_y::<_,_,Bound>(x.clone(), mid.clone());
    let x_geq_mid = x_greater_y(x, mid);

    Branch::distribute(space,
      vec![
        Box::new(move |space: &mut Space<VStore, CStore, R>| {
          space.cstore.alloc(box x_less_mid);
        }),
        Box::new(move |space: &mut Space<VStore, CStore, R>| {
          space.cstore.alloc(box x_geq_mid);
        })
      ]
    )
  }
}

pub fn nth_dom<VStore, Domain>(vstore: &VStore, var_idx: usize) -> Domain where
  VStore: Iterable<Item=Domain>,
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
  use kernel::trilean::Trilean::*;
  use search::*;
  use interval::interval_set::*;
  use interval::ops::Range;

  fn test_distributor<D>(mut distributor: D, distribution_index: usize,
    root: Vec<(i32, i32)>, children: Vec<(i32, i32)>) where
   D: Distributor<FDSpace>
  {
    let mut space = FDSpace::empty();

    for (l,u) in root {
      space.vstore.alloc(IntervalSet::new(l,u));
    }

    let (mut immutable_state, branches) = distributor.distribute(space, distribution_index);

    assert_eq!(branches.len(), children.len());

    for (branch, (l,u)) in branches.into_iter().zip(children.into_iter()) {
      space = branch.commit(immutable_state);
      assert_eq!(space.consistency(), True);
      let split_dom = nth_dom(&space.vstore, distribution_index);
      assert_eq!(split_dom, IntervalSet::new(l,u));
      immutable_state = space.freeze();
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
