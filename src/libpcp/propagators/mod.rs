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

//! Propagators are implementations of constraints, a single constraint can be realized by different propagators.
//!
//! We keep the propagator implementations generic over domains implementing specific operations (e.g. intersection or union). Propagators are also implemented to work on variable views, you can always obtain a view from a variable by using the `Identity` view.

pub mod all_equal;
pub mod cmp;
pub mod cumulative;
pub mod distinct;

pub use propagators::all_equal::*;
pub use propagators::cmp::*;
pub use propagators::distinct::*;

#[cfg(test)]
pub mod test {
    use concept::*;
    use gcollections::ops::*;
    use interval::interval::*;
    use propagation::events::*;
    use propagation::*;
    use trilean::SKleene;
    use variable::store::test::consume_delta;
    use variable::VStoreFD;

    // fn error_msg<T: Debug, VStore>(test_num: u32, msg: &str,
    //   prop: &Formula<VStore>, before: &T, after: &T) -> String
    // {
    //   format!("Test {}: {}\n\
    //            \tPropagator {:?}\n\
    //            \tBefore: {:?}\n\
    //            \tAfter: {:?}",
    //            test_num, msg, prop, before, after)
    // }

    // fn status_inclusion(s1: SKleene, s2: SKleene) -> bool {
    //   match s1 {
    //     True => s2 == True,
    //     False => s2 == False,
    //     Unknown => true
    //   }
    // }

    // /// contracting: p(d) ⊆ d for any domain d
    // fn contracting<VStore>(test_num: u32, vstore: &mut VStore, mut propagator: Formula<VStore>) where
    //  VStore: VStoreConcept + Clone + Subset
    // {
    //   let d1 = vstore.clone();
    //   let d1_status = propagator.is_subsumed(vstore);
    //   let prop_status = propagator.propagate(vstore);
    //   let d2_status = propagator.is_subsumed(vstore);
    //   let err1 = error_msg(test_num, "Propagator is not contracting.", &propagator, &d1, &vstore);
    //   assert!(vstore.is_subset(&d1), err1);
    //   let err2 = error_msg(test_num, "Propagator status is not monotonic.", &propagator, &d1_status, &d2_status);
    //   assert!(status_inclusion(d1_status, d2_status), err2);
    //   if prop_status == false {
    //     let err3 = error_msg(test_num, "Propagator is not monotonic: the propagation failed but the status is not `False`.",
    //       &propagator, &d1_status, &d2_status);
    //     assert!(d2_status == False, err3);
    //   }
    // }

    // /// A propagator p is idempotent if and only if for all domains d, p(p(d)) = p(d).
    // fn idempotent<VStore>(test_num: u32, vstore: &mut VStore, mut propagator: Formula<VStore>) where
    //  VStore: VStoreConcept + Clone + Eq
    // {
    //   let prop_status = propagator.propagate(vstore);
    //   let d1 = vstore.clone();
    //   let d1_status = propagator.is_subsumed(vstore);
    //   let prop_status2 = propagator.propagate(vstore);
    //   let d2_status = propagator.is_subsumed(vstore);
    //   let err1 = error_msg(test_num, "Propagator is not idempotent.", &propagator, &d1, &vstore);
    //   assert!(d1 == vstore.clone() && prop_status == prop_status2 && d1_status == d2_status, err1);
    // }

    // // /// It is monotonic if and only if for any two domains d1 and d2, d1 ⊆ d2 implies p(d1) ⊆ p(d2).
    // // fn monotonic<VStore>(test_num: u32, vstore1: &mut VStore, vstore2: &mut VStore,
    // //   mut propagator: Formula<VStore>) where
    // //  VStore: VStoreConcept + Clone
    // // {}

    // // /// sound: for any domain d ∈ Dom and any assignment a ∈ Asn, if {a} ⊆ d, then p({a}) ⊆ p(d)
    // // fn sound<VStore>(test_num: u32, assignment: &mut VStore, vstore: &mut VStore,
    // //   mut propagator: Formula<VStore>) where
    // //  VStore: VStoreConcept + Clone
    // // {}

    // pub fn test_properties<VStore>(test_num: u32, vstore: &mut VStore, mut propagator: Formula<VStore>) where
    //  VStore: VStoreConcept + Clone
    // {
    //   contracting(test_num, &mut vstore.clone(), propagator.bclone());
    //   idempotent(test_num, &mut vstore.clone(), propagator.bclone());
    // }

    pub type FDVar = Var<VStoreFD>;

    pub fn test_propagation<P>(
        test_num: u32,
        mut prop: P,
        vstore: &mut VStoreFD,
        before: SKleene,
        after: SKleene,
        delta_expected: Vec<(usize, FDEvent)>,
        propagate_success: bool,
    ) where
        P: PropagatorConcept<VStoreFD, FDEvent>,
    {
        // test_properties(test_num, vstore, prop.bclone());
        println!("Test number {}", test_num);
        assert_eq!(prop.is_subsumed(vstore), before);
        assert_eq!(prop.propagate(vstore), propagate_success);
        if propagate_success {
            consume_delta(vstore, delta_expected);
        }
        assert_eq!(prop.is_subsumed(vstore), after);
    }

    pub fn binary_propagator_test<P, FnProp>(
        test_num: u32,
        make_prop: FnProp,
        x: Interval<i32>,
        y: Interval<i32>,
        before: SKleene,
        after: SKleene,
        delta_expected: Vec<(usize, FDEvent)>,
        propagate_success: bool,
    ) where
        P: PropagatorConcept<VStoreFD, FDEvent>,
        FnProp: FnOnce(FDVar, FDVar) -> P,
    {
        let mut vstore = VStoreFD::empty();
        let x = Box::new(vstore.alloc(x)) as Var<VStoreFD>;
        let y = Box::new(vstore.alloc(y)) as Var<VStoreFD>;
        let propagator = make_prop(x, y);
        test_propagation(
            test_num,
            propagator,
            &mut vstore,
            before,
            after,
            delta_expected,
            propagate_success,
        );
    }

    pub fn trinary_propagator_test<P, FnProp>(
        test_num: u32,
        make_prop: FnProp,
        x: Interval<i32>,
        y: Interval<i32>,
        z: Interval<i32>,
        before: SKleene,
        after: SKleene,
        delta_expected: Vec<(usize, FDEvent)>,
        propagate_success: bool,
    ) where
        P: PropagatorConcept<VStoreFD, FDEvent>,
        FnProp: FnOnce(FDVar, FDVar, FDVar) -> P,
    {
        let mut vstore = VStoreFD::empty();
        let x = Box::new(vstore.alloc(x)) as Var<VStoreFD>;
        let y = Box::new(vstore.alloc(y)) as Var<VStoreFD>;
        let z = Box::new(vstore.alloc(z)) as Var<VStoreFD>;
        let propagator = make_prop(x, y, z);
        test_propagation(
            test_num,
            propagator,
            &mut vstore,
            before,
            after,
            delta_expected,
            propagate_success,
        );
    }

    pub fn nary_propagator_test<P, FnProp>(
        test_num: u32,
        make_prop: FnProp,
        doms: Vec<Interval<i32>>,
        before: SKleene,
        after: SKleene,
        delta_expected: Vec<(usize, FDEvent)>,
        propagate_success: bool,
    ) where
        P: PropagatorConcept<VStoreFD, FDEvent>,
        FnProp: FnOnce(Vec<FDVar>) -> P,
    {
        let mut vstore = VStoreFD::empty();
        let vars = doms
            .into_iter()
            .map(|d| Box::new(vstore.alloc(d)) as Var<VStoreFD>)
            .collect();
        let propagator = make_prop(vars);
        test_propagation(
            test_num,
            propagator,
            &mut vstore,
            before,
            after,
            delta_expected,
            propagate_success,
        );
    }
}
