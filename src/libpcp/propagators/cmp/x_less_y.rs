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
use gcollections::ops::*;
use gcollections::*;
use kernel::*;
use logic::*;
use model::*;
use propagation::events::*;
use propagation::*;
use propagators::x_geq_y;
use trilean::SKleene;
use trilean::SKleene::*;

#[derive(Debug)]
pub struct XLessY<VStore> {
    x: Var<VStore>,
    y: Var<VStore>,
}

impl<VStore> XLessY<VStore> {
    pub fn new(x: Var<VStore>, y: Var<VStore>) -> XLessY<VStore> {
        XLessY { x, y }
    }
}

impl<VStore> Clone for XLessY<VStore>
where
    VStore: Collection,
{
    fn clone(&self) -> Self {
        XLessY::new(self.x.bclone(), self.y.bclone())
    }
}

impl<VStore> DisplayStateful<Model> for XLessY<VStore> {
    fn display(&self, model: &Model) {
        self.x.display(model);
        print!(" < ");
        self.y.display(model);
    }
}

impl<VStore, Domain, Bound> NotFormula<VStore> for XLessY<VStore>
where
    VStore: VStoreConcept<Item = Domain> + 'static,
    Domain: Collection<Item = Bound> + IntDomain,
    Bound: IntBound,
{
    fn not(&self) -> Formula<VStore> {
        Box::new(x_geq_y(self.x.bclone(), self.y.bclone()))
    }
}

impl<VStore, Dom, Bound> Subsumption<VStore> for XLessY<VStore>
where
    VStore: Collection<Item = Dom>,
    Dom: Bounded<Item = Bound>,
    Bound: PartialOrd,
{
    fn is_subsumed(&self, store: &VStore) -> SKleene {
        // False:
        // x:    |--|
        // y: |--|
        //
        // True:
        // x: |--|
        // y:     |--|
        //
        // Unknown: Everything else.

        let x = self.x.read(store);
        let y = self.y.read(store);

        if x.lower() >= y.upper() {
            False
        } else if x.upper() < y.lower() {
            True
        } else {
            Unknown
        }
    }
}

impl<VStore, Dom, Bound> Propagator<VStore> for XLessY<VStore>
where
    VStore: Collection<Item = Dom>,
    Dom: Bounded<Item = Bound> + StrictShrinkLeft + StrictShrinkRight,
    Bound: PartialOrd,
{
    fn propagate(&mut self, store: &mut VStore) -> bool {
        let x = self.x.read(store);
        let y = self.y.read(store);
        self.x.update(store, x.strict_shrink_right(y.upper()))
            && self.y.update(store, y.strict_shrink_left(x.lower()))
    }
}

impl<VStore> PropagatorDependencies<FDEvent> for XLessY<VStore> {
    fn dependencies(&self) -> Vec<(usize, FDEvent)> {
        let mut deps = self.x.dependencies(FDEvent::Bound);
        deps.append(&mut self.y.dependencies(FDEvent::Bound));
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
        let dom10_11 = (10, 11).to_interval();
        let dom5_15 = (5, 15).to_interval();
        let dom11_20 = (11, 20).to_interval();
        let dom1_1 = (1, 1).to_interval();

        x_less_y_test_one(
            1,
            dom0_10,
            dom0_10,
            Unknown,
            Unknown,
            vec![(0, Bound), (1, Bound)],
            true,
        );
        x_less_y_test_one(2, dom0_10, dom10_20, Unknown, Unknown, vec![], true);
        x_less_y_test_one(
            3,
            dom10_11,
            dom10_11,
            Unknown,
            True,
            vec![(0, Assignment), (1, Assignment)],
            true,
        );
        x_less_y_test_one(4, dom5_15, dom10_20, Unknown, Unknown, vec![], true);
        x_less_y_test_one(
            5,
            dom5_15,
            dom0_10,
            Unknown,
            Unknown,
            vec![(0, Bound), (1, Bound)],
            true,
        );
        x_less_y_test_one(6, dom0_10, dom11_20, True, True, vec![], true);
        x_less_y_test_one(7, dom11_20, dom0_10, False, False, vec![], false);
        x_less_y_test_one(8, dom1_1, dom0_10, Unknown, True, vec![(1, Bound)], true);
    }

    fn x_less_y_test_one(
        test_num: u32,
        x: Interval<i32>,
        y: Interval<i32>,
        before: SKleene,
        after: SKleene,
        delta_expected: Vec<(usize, FDEvent)>,
        propagate_success: bool,
    ) {
        binary_propagator_test(
            test_num,
            XLessY::new,
            x,
            y,
            before,
            after,
            delta_expected,
            propagate_success,
        );
    }
}
