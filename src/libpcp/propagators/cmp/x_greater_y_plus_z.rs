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
use gcollections::ops::*;
use gcollections::*;
use kernel::*;
use logic::*;
use model::*;
use num::traits::Num;
use propagation::events::*;
use propagation::*;
use propagators::x_leq_y_plus_z;
use trilean::SKleene;
use trilean::SKleene::*;

#[derive(Debug)]
pub struct XGreaterYPlusZ<VStore> {
    pub x: Var<VStore>,
    pub y: Var<VStore>,
    pub z: Var<VStore>,
}

impl<VStore> XGreaterYPlusZ<VStore> {
    pub fn new(x: Var<VStore>, y: Var<VStore>, z: Var<VStore>) -> Self {
        XGreaterYPlusZ { x, y, z }
    }
}

impl<VStore> Clone for XGreaterYPlusZ<VStore>
where
    VStore: Collection,
{
    fn clone(&self) -> Self {
        XGreaterYPlusZ::new(self.x.bclone(), self.y.bclone(), self.z.bclone())
    }
}

impl<VStore> DisplayStateful<Model> for XGreaterYPlusZ<VStore> {
    fn display(&self, model: &Model) {
        self.x.display(model);
        print!(" > ");
        self.y.display(model);
        print!(" + ");
        self.z.display(model);
    }
}

impl<VStore, Domain, Bound> NotFormula<VStore> for XGreaterYPlusZ<VStore>
where
    VStore: VStoreConcept<Item = Domain> + 'static,
    Domain: Collection<Item = Bound> + IntDomain,
    Bound: IntBound,
{
    fn not(&self) -> Formula<VStore> {
        Box::new(x_leq_y_plus_z(
            self.x.bclone(),
            self.y.bclone(),
            self.z.bclone(),
        ))
    }
}

impl<VStore, Dom, Bound> Subsumption<VStore> for XGreaterYPlusZ<VStore>
where
    VStore: Collection<Item = Dom>,
    Dom: Bounded<Item = Bound>,
    Bound: PartialOrd + Num,
{
    fn is_subsumed(&self, store: &VStore) -> SKleene {
        // False: max(X) <= min(Y) + min(Z)
        // True: min(X) > max(Y) + max(Z)
        // Unknown: Everything else.

        let x = self.x.read(store);
        let y = self.y.read(store);
        let z = self.z.read(store);

        if x.upper() <= y.lower() + z.lower() {
            False
        } else if x.lower() > y.upper() + z.upper() {
            True
        } else {
            Unknown
        }
    }
}

impl<VStore, Dom, Bound> Propagator<VStore> for XGreaterYPlusZ<VStore>
where
    VStore: Collection<Item = Dom>,
    Dom: Bounded<Item = Bound> + StrictShrinkRight + StrictShrinkLeft,
    Bound: PartialOrd + Num,
{
    fn propagate(&mut self, store: &mut VStore) -> bool {
        let x = self.x.read(store);
        let y = self.y.read(store);
        let z = self.z.read(store);
        self.x
            .update(store, x.strict_shrink_left(y.lower() + z.lower()))
            && self
                .y
                .update(store, y.strict_shrink_right(x.upper() - z.lower()))
            && self
                .z
                .update(store, z.strict_shrink_right(x.upper() - y.lower()))
    }
}

impl<VStore> PropagatorDependencies<FDEvent> for XGreaterYPlusZ<VStore> {
    fn dependencies(&self) -> Vec<(usize, FDEvent)> {
        let mut deps = self.x.dependencies(FDEvent::Bound);
        deps.append(&mut self.y.dependencies(FDEvent::Bound));
        deps.append(&mut self.z.dependencies(FDEvent::Bound));
        deps
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use interval::interval::*;
    use propagation::events::FDEvent::*;
    use propagators::test::*;

    #[test]
    fn x_greater_y_plus_z_test() {
        let dom0_10 = (0, 10).to_interval();
        let dom10_20 = (10, 20).to_interval();
        let dom10_11 = (10, 11).to_interval();
        let dom5_15 = (5, 15).to_interval();
        let dom1_10 = (1, 10).to_interval();
        let dom5_10 = (5, 10).to_interval();
        let dom6_10 = (6, 10).to_interval();
        let dom1_1 = (1, 1).to_interval();
        let dom2_2 = (2, 2).to_interval();

        x_greater_y_plus_z_test_one(
            1,
            dom0_10,
            dom0_10,
            dom0_10,
            Unknown,
            Unknown,
            vec![(0, Bound), (1, Bound), (2, Bound)],
            true,
        );
        x_greater_y_plus_z_test_one(
            2,
            dom10_11,
            dom5_15,
            dom5_15,
            Unknown,
            True,
            vec![(0, Assignment), (1, Assignment), (2, Assignment)],
            true,
        );
        x_greater_y_plus_z_test_one(3, dom10_20, dom1_1, dom1_1, True, True, vec![], true);
        x_greater_y_plus_z_test_one(4, dom1_1, dom1_1, dom1_1, False, False, vec![], false);
        x_greater_y_plus_z_test_one(5, dom2_2, dom1_1, dom1_1, False, False, vec![], false);
        x_greater_y_plus_z_test_one(
            6,
            dom6_10,
            dom5_10,
            dom1_10,
            Unknown,
            Unknown,
            vec![(0, Bound), (1, Bound), (2, Bound)],
            true,
        );
    }

    fn x_greater_y_plus_z_test_one(
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
            XGreaterYPlusZ::new,
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
