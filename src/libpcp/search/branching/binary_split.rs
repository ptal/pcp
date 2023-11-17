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

use concept::*;
use kernel::*;
use propagators::cmp::*;
use search::branching::branch::*;
use search::branching::*;
use search::space::*;
use term::*;

pub struct BinarySplit;

impl<VStore, CStore, R, Domain, Bound> Distributor<Space<VStore, CStore, R>, Bound> for BinarySplit
where
    VStore: VStoreConcept<Item = Domain, Location = Identity<Domain>, Output = Domain> + 'static,
    CStore: IntCStore<VStore>,
    Domain: IntDomain<Item = Bound> + 'static,
    Bound: IntBound + Copy + 'static,
    R: FreezeSpace<VStore, CStore> + Snapshot<State = Space<VStore, CStore, R>>,
{
    fn distribute(
        &mut self,
        space: Space<VStore, CStore, R>,
        var_idx: usize,
        val: Bound,
    ) -> (
        <Space<VStore, CStore, R> as Freeze>::FrozenState,
        Vec<Branch<Space<VStore, CStore, R>>>,
    ) {
        // See notes in Enumerate::distribute.
        Branch::distribute(
            space,
            vec![
                Box::new(move |space: &mut Space<VStore, CStore, R>| {
                    let x = Box::new(Identity::<Domain>::new(var_idx)) as Var<VStore>;
                    let v = Box::new(Constant::new(val)) as Var<VStore>;
                    let x_less_v = x_leq_y::<_, _, Bound>(x.bclone(), v.bclone());
                    space.cstore.alloc(Box::new(x_less_v));
                }),
                Box::new(move |space: &mut Space<VStore, CStore, R>| {
                    let x = Box::new(Identity::<Domain>::new(var_idx)) as Var<VStore>;
                    let v = Box::new(Constant::new(val)) as Var<VStore>;
                    let x_geq_v = x_greater_y(x, v);
                    space.cstore.alloc(Box::new(x_geq_v));
                }),
            ],
        )
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use gcollections::ops::*;
    use interval::interval_set::*;
    use interval::ops::Range;
    use search::branching::Distributor;
    use search::branching::MiddleVal;
    use search::*;
    use term::ops::*;
    use trilean::SKleene::*;

    type Domain = IntervalSet<i32>;

    pub fn test_distributor<D, Val>(
        mut distributor: D,
        mut val_selection: Val,
        distribution_index: usize,
        root: Vec<(i32, i32)>,
        children: Vec<(i32, i32)>,
    ) where
        Val: ValSelection<Domain>,
        D: Distributor<FDSpace, i32>,
    {
        let mut space = FDSpace::empty();

        for (l, u) in root {
            space.vstore.alloc(IntervalSet::new(l, u));
        }

        let x = Identity::<Domain>::new(distribution_index);
        let d = x.read(&space.vstore);
        let b = val_selection.select(d);

        let (mut immutable_state, branches) = distributor.distribute(space, distribution_index, b);

        assert_eq!(branches.len(), children.len());

        for (branch, (l, u)) in branches.into_iter().zip(children.into_iter()) {
            space = branch.commit(immutable_state);
            assert_eq!(space.consistency(), True);
            let split_dom = x.read(&space.vstore);
            assert_eq!(split_dom, IntervalSet::new(l, u));
            immutable_state = space.freeze();
        }
    }

    #[test]
    fn binary_split_distribution() {
        let vars = vec![(1, 10), (2, 4), (1, 2)];
        test_distributor(
            BinarySplit,
            MiddleVal,
            0,
            vars.clone(),
            vec![(1, 5), (6, 10)],
        );
        test_distributor(
            BinarySplit,
            MiddleVal,
            1,
            vars.clone(),
            vec![(2, 3), (4, 4)],
        );
        test_distributor(
            BinarySplit,
            MiddleVal,
            2,
            vars.clone(),
            vec![(1, 1), (2, 2)],
        );
    }

    #[test]
    #[should_panic]
    fn binary_split_impossible_distribution() {
        test_distributor(BinarySplit, MiddleVal, 0, vec![(1, 1)], vec![]);
    }

    #[test]
    #[should_panic]
    fn binary_split_impossible_distribution_2() {
        test_distributor(
            BinarySplit,
            MiddleVal,
            2,
            vec![(1, 5), (2, 4), (4, 4)],
            vec![],
        );
    }
}
