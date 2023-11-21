// Copyright 2016 Pierre Talbot (IRCAM)

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
use propagators::cmp::{x_geq_y_plus_z, x_leq_y_plus_z, XGreaterEqYPlusZ, XLessEqYPlusZ};
use trilean::SKleene;

#[derive(Clone, Debug)]
pub struct XEqYPlusZ<VStore: Collection> {
    geq: XGreaterEqYPlusZ<VStore>,
    leq: XLessEqYPlusZ<VStore>,
}

impl<VStore, Domain, Bound> XEqYPlusZ<VStore>
where
    VStore: VStoreConcept<Item = Domain> + 'static,
    Domain: Collection<Item = Bound> + IntDomain,
    Bound: IntBound,
{
    pub fn new(x: Var<VStore>, y: Var<VStore>, z: Var<VStore>) -> Self {
        XEqYPlusZ {
            geq: x_geq_y_plus_z(x.bclone(), y.bclone(), z.bclone()),
            leq: x_leq_y_plus_z(x, y, z),
        }
    }
}

impl<VStore> DisplayStateful<Model> for XEqYPlusZ<VStore>
where
    VStore: Collection,
{
    fn display(&self, model: &Model) {
        self.geq.x.display(model);
        print!(" = ");
        self.geq.y.display(model);
        print!(" + ");
        self.geq.z.display(model);
        print!(" (decomposed)");
    }
}

impl<VStore> Subsumption<VStore> for XEqYPlusZ<VStore>
where
    VStore: Collection,
    XGreaterEqYPlusZ<VStore>: Subsumption<VStore>,
    XLessEqYPlusZ<VStore>: Subsumption<VStore>,
{
    fn is_subsumed(&self, store: &VStore) -> SKleene {
        self.geq.is_subsumed(store).and(self.leq.is_subsumed(store))
    }
}

impl<VStore> NotFormula<VStore> for XEqYPlusZ<VStore>
where
    VStore: Collection,
{
    fn not(&self) -> Formula<VStore> {
        unimplemented!()
    }
}

impl<VStore> Propagator<VStore> for XEqYPlusZ<VStore>
where
    VStore: Collection,
    XGreaterEqYPlusZ<VStore>: Propagator<VStore>,
    XLessEqYPlusZ<VStore>: Propagator<VStore>,
{
    fn propagate(&mut self, store: &mut VStore) -> bool {
        self.geq.propagate(store) && self.leq.propagate(store)
    }
}

impl<VStore> PropagatorDependencies<FDEvent> for XEqYPlusZ<VStore>
where
    VStore: Collection,
    XGreaterEqYPlusZ<VStore>: PropagatorDependencies<FDEvent>,
    XLessEqYPlusZ<VStore>: PropagatorDependencies<FDEvent>,
{
    fn dependencies(&self) -> Vec<(usize, FDEvent)> {
        let geq_deps = self.geq.dependencies();
        let leq_deps = self.leq.dependencies();
        assert_eq!(
            geq_deps, leq_deps,
            "This function assumed both dependencies of X >= Y + Z and X <= Y + Z are equals."
        );
        geq_deps
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
    fn x_eq_y_plus_z_test() {
        let dom0_10 = (0, 10).to_interval();
        let dom10_20 = (10, 20).to_interval();
        let dom12_12 = (12, 12).to_interval();
        let dom0_6 = (0, 6).to_interval();
        let dom0_5 = (0, 5).to_interval();
        let dom0_1 = (0, 1).to_interval();
        let dom1_1 = (1, 1).to_interval();
        let dom2_2 = (2, 2).to_interval();

        x_eq_y_plus_z_test_one(1, dom0_10, dom0_10, dom0_10, Unknown, Unknown, vec![], true);
        x_eq_y_plus_z_test_one(
            2,
            dom12_12,
            dom0_6,
            dom0_6,
            Unknown,
            True,
            vec![(1, Assignment), (2, Assignment)],
            true,
        );
        x_eq_y_plus_z_test_one(3, dom10_20, dom1_1, dom1_1, False, False, vec![], false);
        x_eq_y_plus_z_test_one(4, dom2_2, dom1_1, dom1_1, True, True, vec![], true);
        x_eq_y_plus_z_test_one(5, dom1_1, dom2_2, dom2_2, False, False, vec![], false);
        x_eq_y_plus_z_test_one(6, dom0_6, dom0_5, dom0_1, Unknown, Unknown, vec![], true);
    }

    fn x_eq_y_plus_z_test_one(
        test_num: u32,
        x: Interval<i32>,
        y: Interval<i32>,
        z: Interval<i32>,
        before: SKleene,
        after: SKleene,
        delta_expected: Vec<(usize, FDEvent)>,
        propagate_success: bool,
    ) {
        trinary_propagator_test(
            test_num,
            XEqYPlusZ::new,
            x,
            y,
            z,
            before,
            after,
            delta_expected,
            propagate_success,
        );
    }
}
