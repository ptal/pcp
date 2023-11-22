// Copyright 2018 Pierre Talbot (IRCAM)

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
use gcollections::*;
use kernel::*;
use logic::*;
use model::*;
use propagation::events::*;
use propagation::*;
use propagators::cmp::x_eq_y::*;
use trilean::SKleene;

#[derive(Debug)]
pub struct AllEqual<VStore> {
    conj: Conjunction<VStore>,
    vars: Vec<Var<VStore>>,
}

impl<VStore> NotFormula<VStore> for AllEqual<VStore>
where
    VStore: Collection + 'static,
{
    fn not(&self) -> Formula<VStore> {
        self.conj.not()
    }
}

impl<VStore, Domain, Bound> AllEqual<VStore>
where
    VStore: VStoreConcept<Item = Domain> + 'static,
    Domain: IntDomain<Item = Bound> + 'static,
    Bound: IntBound + 'static,
{
    /// Precondition: `vars.len() > 1`.
    pub fn new(vars: Vec<Var<VStore>>) -> Self {
        assert!(
            vars.len() > 0,
            "Variable array in `AllEqual` must be non-empty."
        );
        let mut props = vec![];
        for i in 0..vars.len() - 1 {
            let i_eq_j =
                Box::new(XEqY::new(vars[i].bclone(), vars[i + 1].bclone())) as Formula<VStore>;
            props.push(i_eq_j);
        }
        AllEqual {
            conj: Conjunction::new(props),
            vars,
        }
    }
}

impl<VStore> Clone for AllEqual<VStore>
where
    VStore: Collection,
{
    fn clone(&self) -> Self {
        AllEqual {
            conj: self.conj.clone(),
            vars: self.vars.iter().map(|v| v.bclone()).collect(),
        }
    }
}

impl<VStore> DisplayStateful<Model> for AllEqual<VStore> {
    fn display(&self, model: &Model) {
        model.display_global("all_equal", &self.vars);
    }
}

impl<VStore> Subsumption<VStore> for AllEqual<VStore> {
    fn is_subsumed(&self, vstore: &VStore) -> SKleene {
        self.conj.is_subsumed(vstore)
    }
}

impl<VStore> Propagator<VStore> for AllEqual<VStore> {
    fn propagate(&mut self, vstore: &mut VStore) -> bool {
        self.conj.propagate(vstore)
    }
}

impl<VStore> PropagatorDependencies<FDEvent> for AllEqual<VStore> {
    fn dependencies(&self) -> Vec<(usize, FDEvent)> {
        self.vars
            .iter()
            .flat_map(|v| v.dependencies(FDEvent::Inner))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use interval::interval::*;
    use propagation::events::FDEvent::*;
    use propagators::test::*;
    use trilean::SKleene::*;

    #[test]
    fn all_equal_test() {
        let zero = (0, 0).to_interval();
        let one = (1, 1).to_interval();
        let two = (2, 2).to_interval();
        let dom0_1 = (0, 1).to_interval();
        let dom0_2 = (0, 2).to_interval();
        let dom0_3 = (0, 3).to_interval();

        all_equal_test_one(1, vec![zero, one, two], False, False, vec![], false);
        all_equal_test_one(2, vec![zero, zero, two], False, False, vec![], false);
        all_equal_test_one(3, vec![zero, zero, zero], True, True, vec![], true);
        all_equal_test_one(
            4,
            vec![zero, dom0_3, dom0_3],
            Unknown,
            True,
            vec![(1, Assignment), (2, Assignment)],
            true,
        );
        all_equal_test_one(
            5,
            vec![dom0_1, dom0_3, dom0_3],
            Unknown,
            Unknown,
            vec![(1, Bound), (2, Bound)],
            true,
        );
        all_equal_test_one(6, vec![zero, one, dom0_2], False, False, vec![], false);
        all_equal_test_one(
            7,
            vec![dom0_1, one, dom0_1],
            Unknown,
            True,
            vec![(0, Assignment), (2, Assignment)],
            true,
        );
        all_equal_test_one(8, vec![dom0_3], True, True, vec![], true);
        all_equal_test_one(9, vec![one], True, True, vec![], true);
    }

    fn all_equal_test_one(
        test_num: u32,
        doms: Vec<Interval<i32>>,
        before: SKleene,
        after: SKleene,
        delta_expected: Vec<(usize, FDEvent)>,
        propagate_success: bool,
    ) {
        nary_propagator_test(
            test_num,
            AllEqual::new,
            doms,
            before,
            after,
            delta_expected,
            propagate_success,
        );
    }
}
