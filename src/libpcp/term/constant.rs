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

use gcollections::ops::*;
use gcollections::*;
use kernel::*;
use model::*;
use propagation::events::*;
use std::fmt::Debug;
use term::ops::*;

#[derive(Clone, Debug)]
pub struct Constant<V> {
    value: V,
}

impl<V> Constant<V> {
    pub fn new(value: V) -> Constant<V> {
        Constant { value }
    }
}

impl<V> DisplayStateful<Model> for Constant<V>
where
    V: Debug,
{
    fn display(&self, _model: &Model) {
        print!("{:?}", self.value);
    }
}

impl<V, Domain, VStore> StoreMonotonicUpdate<VStore> for Constant<V>
where
    VStore: Collection<Item = Domain>,
    Domain: Collection<Item = V> + Cardinality + Contains,
{
    fn update(&mut self, _store: &mut VStore, value: VStore::Item) -> bool {
        !value.is_empty() && value.contains(&self.value)
    }
}

impl<V, Domain, VStore> StoreRead<VStore> for Constant<V>
where
    VStore: Collection<Item = Domain>,
    Domain: Collection<Item = V> + Singleton,
    V: Clone,
{
    fn read(&self, _store: &VStore) -> Domain {
        Domain::singleton(self.value.clone())
    }
}

impl<V> ViewDependencies<FDEvent> for Constant<V> {
    fn dependencies(&self, _event: FDEvent) -> Vec<(usize, FDEvent)> {
        vec![]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use concept::*;
    use interval::interval::*;
    use propagation::events::FDEvent;
    use propagation::events::FDEvent::*;
    use propagation::*;
    use propagators::cmp::*;
    use propagators::test::*;
    use trilean::SKleene;
    use trilean::SKleene::*;
    use variable::VStoreFD;

    type VStore = VStoreFD;

    #[test]
    fn x_less_constant() {
        let dom0_10 = (0, 10).to_interval();
        let dom0_4 = (0, 4).to_interval();
        let mut store = VStore::empty();
        let x = Box::new(store.alloc(dom0_10)) as Var<VStore>;
        let c = Box::new(Constant::new(5_i32)) as Var<VStore>;

        let x_less_c = XLessY::new(x.bclone(), c);
        test_propagation(
            1,
            x_less_c,
            &mut store,
            Unknown,
            True,
            vec![(0, Bound)],
            true,
        );
        assert_eq!(x.read(&store), dom0_4);
    }

    #[test]
    fn unary_propagator_test() {
        let dom0_10 = (0, 10).to_interval();
        let dom0_0 = (0, 0).to_interval();

        unary_propagator_test_one(1, dom0_10, 0, XLessY::new, False, False, vec![], false);
        unary_propagator_test_one(2, dom0_10, 11, XLessY::new, True, True, vec![], true);
        unary_propagator_test_one(
            3,
            dom0_10,
            10,
            XLessY::new,
            Unknown,
            True,
            vec![(0, Bound)],
            true,
        );

        unary_propagator_test_one(4, dom0_10, -1, x_leq_y, False, False, vec![], false);
        unary_propagator_test_one(5, dom0_10, 10, x_leq_y, True, True, vec![], true);
        unary_propagator_test_one(
            6,
            dom0_10,
            9,
            x_leq_y,
            Unknown,
            True,
            vec![(0, Bound)],
            true,
        );

        unary_propagator_test_one(7, dom0_10, 10, x_greater_y, False, False, vec![], false);
        unary_propagator_test_one(8, dom0_10, -1, x_greater_y, True, True, vec![], true);
        unary_propagator_test_one(
            9,
            dom0_10,
            0,
            x_greater_y,
            Unknown,
            True,
            vec![(0, Bound)],
            true,
        );

        unary_propagator_test_one(10, dom0_10, 11, x_geq_y, False, False, vec![], false);
        unary_propagator_test_one(11, dom0_10, 0, x_geq_y, True, True, vec![], true);
        unary_propagator_test_one(
            12,
            dom0_10,
            1,
            x_geq_y,
            Unknown,
            True,
            vec![(0, Bound)],
            true,
        );

        unary_propagator_test_one(13, dom0_0, 0, XNeqY::new, False, False, vec![], false);
        unary_propagator_test_one(14, dom0_10, 5, XNeqY::new, Unknown, Unknown, vec![], true);
        unary_propagator_test_one(
            15,
            dom0_10,
            0,
            XNeqY::new,
            Unknown,
            True,
            vec![(0, Bound)],
            true,
        );
        unary_propagator_test_one(
            16,
            dom0_10,
            10,
            XNeqY::new,
            Unknown,
            True,
            vec![(0, Bound)],
            true,
        );
    }

    fn unary_propagator_test_one<P, R>(
        id: u32,
        x: Interval<i32>,
        c: i32,
        make_prop: P,
        before: SKleene,
        after: SKleene,
        expected: Vec<(usize, FDEvent)>,
        propagate_success: bool,
    ) where
        P: FnOnce(FDVar, FDVar) -> R,
        R: PropagatorConcept<VStoreFD, FDEvent>,
    {
        let mut store = VStore::empty();
        let x = Box::new(store.alloc(x)) as Var<VStore>;
        let propagator = make_prop(x, Box::new(Constant::new(c)) as Var<VStore>);
        test_propagation(
            id,
            propagator,
            &mut store,
            before,
            after,
            expected,
            propagate_success,
        );
    }
}
