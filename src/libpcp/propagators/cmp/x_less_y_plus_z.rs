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
use propagators::x_geq_y_plus_z;
use trilean::SKleene;
use trilean::SKleene::*;

#[derive(Debug)]
pub struct XLessYPlusZ<VStore> {
    x: Var<VStore>,
    y: Var<VStore>,
    z: Var<VStore>,
}

impl<VStore> XLessYPlusZ<VStore> {
    pub fn new(x: Var<VStore>, y: Var<VStore>, z: Var<VStore>) -> Self {
        XLessYPlusZ { x, y, z }
    }
}

impl<VStore> Clone for XLessYPlusZ<VStore>
where
    VStore: Collection,
{
    fn clone(&self) -> Self {
        XLessYPlusZ::new(self.x.bclone(), self.y.bclone(), self.z.bclone())
    }
}

impl<VStore> DisplayStateful<Model> for XLessYPlusZ<VStore> {
    fn display(&self, model: &Model) {
        self.x.display(model);
        print!(" < ");
        self.y.display(model);
        print!(" + ");
        self.z.display(model);
    }
}

impl<VStore, Domain, Bound> NotFormula<VStore> for XLessYPlusZ<VStore>
where
    VStore: VStoreConcept<Item = Domain> + 'static,
    Domain: Collection<Item = Bound> + IntDomain,
    Bound: IntBound,
{
    fn not(&self) -> Formula<VStore> {
        Box::new(x_geq_y_plus_z(
            self.x.bclone(),
            self.y.bclone(),
            self.z.bclone(),
        ))
    }
}

impl<VStore, Dom, Bound> Subsumption<VStore> for XLessYPlusZ<VStore>
where
    VStore: Collection<Item = Dom>,
    Dom: Bounded<Item = Bound>,
    Bound: PartialOrd + Num,
{
    fn is_subsumed(&self, store: &VStore) -> SKleene {
        // False: min(X) >= max(Y) + max(Z)
        // True: max(X) < min(Y) + min(Z)
        // Unknown: Everything else.
        let x = self.x.read(store);
        let y = self.y.read(store);
        let z = self.z.read(store);

        if x.lower() >= y.upper() + z.upper() {
            False
        } else if x.upper() < y.lower() + z.lower() {
            True
        } else {
            Unknown
        }
    }
}

impl<VStore, Dom, Bound> Propagator<VStore> for XLessYPlusZ<VStore>
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
            .update(store, x.strict_shrink_right(y.upper() + z.upper()))
            && self
                .y
                .update(store, y.strict_shrink_left(x.lower() - z.upper()))
            && self
                .z
                .update(store, z.strict_shrink_left(x.lower() - y.upper()))
    }
}

impl<VStore> PropagatorDependencies<FDEvent> for XLessYPlusZ<VStore> {
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
    fn x_less_y_test() {
        let dom0_10 = (0, 10).to_interval();
        let dom10_20 = (10, 20).to_interval();
        let dom11_12 = (11, 12).to_interval();
        let dom0_6 = (0, 6).to_interval();
        let dom0_5 = (0, 5).to_interval();
        let dom0_1 = (0, 1).to_interval();
        let dom1_1 = (1, 1).to_interval();
        let dom2_2 = (2, 2).to_interval();

        x_less_y_plus_z_test_one(1, dom0_10, dom0_10, dom0_10, Unknown, Unknown, vec![], true);
        x_less_y_plus_z_test_one(
            2,
            dom11_12,
            dom0_6,
            dom0_6,
            Unknown,
            True,
            vec![(0, Assignment), (1, Assignment), (2, Assignment)],
            true,
        );
        x_less_y_plus_z_test_one(3, dom10_20, dom1_1, dom1_1, False, False, vec![], false);
        x_less_y_plus_z_test_one(4, dom2_2, dom1_1, dom1_1, False, False, vec![], false);
        x_less_y_plus_z_test_one(5, dom1_1, dom2_2, dom2_2, True, True, vec![], true);
        x_less_y_plus_z_test_one(
            6,
            dom0_6,
            dom0_5,
            dom0_1,
            Unknown,
            Unknown,
            vec![(0, Bound)],
            true,
        );
    }

    fn x_less_y_plus_z_test_one(
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
            XLessYPlusZ::new,
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
