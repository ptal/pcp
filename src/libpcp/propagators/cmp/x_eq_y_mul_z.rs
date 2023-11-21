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
use propagation::events::*;
use propagation::*;
use std::ops::*;
use trilean::SKleene;
use trilean::SKleene::*;

/// TODO see K. Apt Section 6.5.4 for a better implementation.

// x = y * z
#[derive(Debug)]
pub struct XEqYMulZ<VStore> {
    x: Var<VStore>,
    y: Var<VStore>,
    z: Var<VStore>,
}

impl<VStore> XEqYMulZ<VStore> {
    pub fn new(x: Var<VStore>, y: Var<VStore>, z: Var<VStore>) -> Self {
        XEqYMulZ { x, y, z }
    }
}

impl<VStore> Clone for XEqYMulZ<VStore>
where
    VStore: Collection,
{
    fn clone(&self) -> Self {
        XEqYMulZ::new(self.x.bclone(), self.y.bclone(), self.z.bclone())
    }
}

impl<VStore> DisplayStateful<Model> for XEqYMulZ<VStore> {
    fn display(&self, model: &Model) {
        self.x.display(model);
        print!(" = ");
        self.y.display(model);
        print!(" * ");
        self.z.display(model);
    }
}

impl<VStore> NotFormula<VStore> for XEqYMulZ<VStore> {
    fn not(&self) -> Formula<VStore> {
        unimplemented!();
    }
}

impl<VStore, Dom> Subsumption<VStore> for XEqYMulZ<VStore>
where
    VStore: Collection<Item = Dom>,
    Dom: Bounded + IsSingleton + Mul<Output = Dom> + Overlap,
{
    fn is_subsumed(&self, store: &VStore) -> SKleene {
        // False: x and y*z do not overlap.
        // True: x and y*z are singletons and equal.
        // Unknown: x and y*z overlap but are not singletons.
        let x = self.x.read(store);
        let y = self.y.read(store);
        let z = self.z.read(store);

        let yz = y * z;
        if yz.overlap(&x) {
            if yz.is_singleton() && x.is_singleton() {
                True
            } else {
                Unknown
            }
        } else {
            False
        }
    }
}

impl<VStore, Dom> Propagator<VStore> for XEqYMulZ<VStore>
where
    VStore: Collection<Item = Dom>,
    Dom: Bounded + Intersection<Output = Dom> + Mul<Output = Dom>,
{
    fn propagate(&mut self, store: &mut VStore) -> bool {
        let x = self.x.read(store);
        let y = self.y.read(store);
        let z = self.z.read(store);
        let yz = y * z;
        self.x.update(store, x.intersection(&yz))
    }
}

impl<VStore> PropagatorDependencies<FDEvent> for XEqYMulZ<VStore> {
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
    fn x_eq_y_mul_z_test() {
        let dom0_10 = (0, 10).to_interval();
        let dom10_20 = (10, 20).to_interval();
        let dom10_11 = (10, 11).to_interval();
        let dom5_15 = (5, 15).to_interval();
        let dom1_1 = (1, 1).to_interval();
        let dom1_2 = (1, 2).to_interval();

        x_eq_y_mul_z_test_one(1, dom0_10, dom0_10, dom0_10, Unknown, Unknown, vec![], true);
        x_eq_y_mul_z_test_one(2, dom10_11, dom5_15, dom5_15, False, False, vec![], false);
        x_eq_y_mul_z_test_one(3, dom10_20, dom1_1, dom1_1, False, False, vec![], false);
        x_eq_y_mul_z_test_one(4, dom1_1, dom1_1, dom1_1, True, True, vec![], true);
        x_eq_y_mul_z_test_one(
            5,
            dom1_2,
            dom1_1,
            dom1_1,
            Unknown,
            True,
            vec![(0, Assignment)],
            true,
        );
    }

    fn x_eq_y_mul_z_test_one(
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
            XEqYMulZ::new,
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
