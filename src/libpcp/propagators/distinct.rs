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
use gcollections::*;
use kernel::*;
use logic::*;
use model::*;
use propagation::events::*;
use propagation::*;
use propagators::cmp::x_neq_y::*;
use trilean::SKleene;

/// Precondition: `vars.len() > 1`.
pub fn join_distinct<VStore, CStore, Domain, Bound>(
    _vstore: &mut VStore,
    cstore: &mut CStore,
    vars: Vec<Var<VStore>>,
) where
    VStore: VStoreConcept<Item = Domain> + 'static,
    Domain: IntDomain<Item = Bound> + 'static,
    Bound: IntBound + 'static,
    CStore: IntCStore<VStore> + 'static,
{
    assert!(
        vars.len() > 0,
        "Variable array in `Distinct` must be non-empty."
    );
    for i in 0..vars.len() - 1 {
        for j in i + 1..vars.len() {
            cstore.alloc(Box::new(XNeqY::new(vars[i].bclone(), vars[j].bclone())));
        }
    }
}

#[derive(Debug)]
pub struct Distinct<VStore> {
    conj: Conjunction<VStore>,
    vars: Vec<Var<VStore>>,
}

impl<VStore> NotFormula<VStore> for Distinct<VStore>
where
    VStore: Collection + 'static,
{
    fn not(&self) -> Formula<VStore> {
        self.conj.not()
    }
}

impl<VStore, Domain, Bound> Distinct<VStore>
where
    VStore: VStoreConcept<Item = Domain> + 'static,
    Domain: IntDomain<Item = Bound> + 'static,
    Bound: IntBound + 'static,
{
    /// Precondition: `vars.len() > 1`.
    pub fn new(vars: Vec<Var<VStore>>) -> Self {
        assert!(
            vars.len() > 0,
            "Variable array in `Distinct` must be non-empty."
        );
        let mut props = vec![];
        for i in 0..vars.len() - 1 {
            for j in i + 1..vars.len() {
                let i_neq_j =
                    Box::new(XNeqY::new(vars[i].bclone(), vars[j].bclone())) as Formula<VStore>;
                props.push(i_neq_j);
            }
        }
        Distinct {
            conj: Conjunction::new(props),
            vars,
        }
    }
}

impl<VStore> Clone for Distinct<VStore>
where
    VStore: Collection,
{
    fn clone(&self) -> Self {
        Distinct {
            conj: self.conj.clone(),
            vars: self.vars.iter().map(|v| v.bclone()).collect(),
        }
    }
}

impl<VStore> DisplayStateful<Model> for Distinct<VStore> {
    fn display(&self, model: &Model) {
        model.display_global("distinct", &self.vars);
    }
}

impl<VStore> Subsumption<VStore> for Distinct<VStore> {
    fn is_subsumed(&self, vstore: &VStore) -> SKleene {
        self.conj.is_subsumed(vstore)
    }
}

impl<VStore> Propagator<VStore> for Distinct<VStore> {
    fn propagate(&mut self, vstore: &mut VStore) -> bool {
        self.conj.propagate(vstore)
    }
}

impl<VStore> PropagatorDependencies<FDEvent> for Distinct<VStore> {
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
    fn distinct_test() {
        let zero = (0, 0).to_interval();
        let one = (1, 1).to_interval();
        let two = (2, 2).to_interval();
        let dom0_1 = (0, 1).to_interval();
        let dom0_2 = (0, 2).to_interval();
        let dom0_3 = (0, 3).to_interval();

        distinct_test_one(1, vec![zero, one, two], True, True, vec![], true);
        distinct_test_one(2, vec![zero, zero, two], False, False, vec![], false);
        distinct_test_one(
            3,
            vec![zero, one, dom0_3],
            Unknown,
            True,
            vec![(2, Bound)],
            true,
        );
        distinct_test_one(
            4,
            vec![zero, one, dom0_2],
            Unknown,
            True,
            vec![(2, Assignment)],
            true,
        );
        distinct_test_one(5, vec![zero, one, dom0_1], Unknown, False, vec![], false);
        distinct_test_one(
            6,
            vec![zero, dom0_3, dom0_3],
            Unknown,
            Unknown,
            vec![(1, Bound), (2, Bound)],
            true,
        );
        distinct_test_one(7, vec![dom0_3], True, True, vec![], true);
    }

    fn distinct_test_one(
        test_num: u32,
        doms: Vec<Interval<i32>>,
        before: SKleene,
        after: SKleene,
        delta_expected: Vec<(usize, FDEvent)>,
        propagate_success: bool,
    ) {
        nary_propagator_test(
            test_num,
            Distinct::new,
            doms,
            before,
            after,
            delta_expected,
            propagate_success,
        );
    }
}
