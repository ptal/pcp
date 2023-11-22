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
use logic::*;
use propagators::*;
use term::*;

pub struct Cumulative<VStore> {
    starts: Vec<Var<VStore>>,
    durations: Vec<Var<VStore>>,
    resources: Vec<Var<VStore>>,
    capacity: Var<VStore>,
    intermediate: Vec<Vec<usize>>, // Given intermediate[j][i], if i left-overlap j, then it contains the number of resources used by i.
}

impl<VStore> Cumulative<VStore> {
    pub fn new(
        starts: Vec<Var<VStore>>,
        durations: Vec<Var<VStore>>,
        resources: Vec<Var<VStore>>,
        capacity: Var<VStore>,
    ) -> Self {
        let tasks = starts.len();
        assert_eq!(tasks, durations.len());
        assert_eq!(tasks, resources.len());
        Cumulative {
            starts,
            durations,
            resources,
            capacity,
            intermediate: vec![],
        }
    }
}

impl<VStore, Domain, Bound> Cumulative<VStore>
where
    VStore: VStoreConcept<Item = Domain> + 'static,
    Domain: IntDomain<Item = Bound> + 'static,
    Bound: IntBound + 'static,
{
    // Decomposition described in `Why cumulative decomposition is not as bad as it sounds`, Schutt and al., 2009.
    // Intuitively, it says that for each task j, the sum of the resources used by the other tasks overlapping with j must not exceed the capacity.
    // forall( j in tasks ) (
    //   c >= r[j] + sum( i in tasks where i != j ) (
    //     bool2int( s[i] <= s[j] /\ s[j] < s[i] + d[i] ) * r[i]));
    pub fn join<CStore>(&mut self, vstore: &mut VStore, cstore: &mut CStore)
    where
        CStore: IntCStore<VStore> + 'static,
    {
        let tasks = self.starts.len();
        // Special case where only one task needs to be scheduled.
        if tasks == 1 {
            // c >= r[j]
            cstore.alloc(Box::new(x_geq_y(self.capacity_var(), self.resource_at(0))));
        } else {
            // forall( j in tasks ) (...)
            for j in 0..tasks {
                let mut resource_vars = vec![];
                self.intermediate.push(vec![]);
                for i in 0..tasks {
                    if i != j {
                        // conj <-> s[i] <= s[j] /\ s[j] < s[i] + d[i]
                        let conj = Box::new(Conjunction::new(vec![
                            // s[i] <= s[j]
                            Box::new(x_leq_y(self.start_at(i), self.start_at(j))),
                            // s[j] < s[i] + d[i]
                            Box::new(XLessYPlusZ::new(
                                self.start_at(j),
                                self.start_at(i),
                                self.duration_at(i),
                            )),
                        ]));

                        // bi <-> conj
                        let bi = Boolean::new(vstore);
                        let equiv = equivalence(Box::new(bi.clone()), conj);
                        cstore.alloc(equiv);

                        // r = bi * r[i]
                        let ri = self.resource_at(i);
                        let ri_ub = ri.read(vstore).upper();
                        let r_dom = Domain::new(Bound::zero(), ri_ub);
                        // let hole = Domain::new(Bound::one(), ri_ub.clone() - Bound::one());
                        let r = vstore.alloc(r_dom);
                        self.intermediate.last_mut().unwrap().push(r.index());
                        let r = Box::new(r) as Var<VStore>;
                        cstore.alloc(Box::new(XEqYMulZ::new(r.bclone(), Box::new(bi), ri)));
                        resource_vars.push(r);
                    }
                }
                //  sum( i in tasks where i != j )(...)
                let sum = Box::new(Sum::new(resource_vars));
                // c >= r[j] + sum
                cstore.alloc(Box::new(x_geq_y_plus_z(
                    self.capacity_var(),
                    self.resource_at(j),
                    sum,
                )));
            }
        }
    }

    pub fn intermediate_vars(&self) -> Vec<Vec<usize>> {
        self.intermediate.clone()
    }

    fn start_at(&self, i: usize) -> Var<VStore> {
        self.starts[i].bclone()
    }
    fn duration_at(&self, i: usize) -> Var<VStore> {
        self.durations[i].bclone()
    }
    fn resource_at(&self, i: usize) -> Var<VStore> {
        self.resources[i].bclone()
    }
    fn capacity_var(&self) -> Var<VStore> {
        self.capacity.bclone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use gcollections::ops::*;
    use interval::interval::*;
    use interval::ops::Range;
    use kernel::*;
    use model::*;
    use propagation::ops::Subsumption;
    use propagation::CStoreFD;
    use trilean::SKleene;
    use trilean::SKleene::*;
    use variable::VStoreCopy;

    type Dom = Interval<i32>;
    type VStoreFD = VStoreCopy<Dom>;

    struct CumulativeTest {
        starts: Vec<Interval<i32>>,
        durations: Vec<Interval<i32>>,
        resources: Vec<Interval<i32>>,
        capacity: Interval<i32>,
    }

    impl CumulativeTest {
        fn new(
            starts: Vec<Interval<i32>>,
            durations: Vec<Interval<i32>>,
            resources: Vec<Interval<i32>>,
            capacity: Interval<i32>,
        ) -> Self {
            CumulativeTest {
                starts,
                durations,
                resources,
                capacity,
            }
        }

        fn new_assignment(
            starts: Vec<i32>,
            durations: Vec<i32>,
            resources: Vec<i32>,
            capacity: i32,
        ) -> Self {
            CumulativeTest::new(
                starts.into_iter().map(|s| Interval::new(s, s)).collect(),
                durations.into_iter().map(|d| Interval::new(d, d)).collect(),
                resources.into_iter().map(|r| Interval::new(r, r)).collect(),
                Interval::new(capacity, capacity),
            )
        }

        fn create_var(
            dom: Interval<i32>,
            model: &mut Model,
            vstore: &mut VStoreFD,
            constant: bool,
        ) -> Var<VStoreFD> {
            if dom.is_singleton() && constant {
                Box::new(Constant::new(dom.lower()))
            } else {
                model.alloc_var(vstore, dom)
            }
        }

        fn instantiate(
            self,
            model: &mut Model,
            vstore: &mut VStoreFD,
            cstore: &mut CStoreFD<VStoreFD>,
            constant: bool,
        ) {
            model.open_group("s");
            let starts = self
                .starts
                .into_iter()
                .map(|s| Self::create_var(s, model, vstore, constant))
                .collect();
            model.close_group();
            model.open_group("d");
            let durations = self
                .durations
                .into_iter()
                .map(|d| Self::create_var(d, model, vstore, constant))
                .collect();
            model.close_group();
            model.open_group("r");
            let resources = self
                .resources
                .into_iter()
                .map(|r| Self::create_var(r, model, vstore, constant))
                .collect();
            model.close_group();
            let capacity = Box::new(vstore.alloc(self.capacity));
            model.register_var(capacity.index(), String::from("c"));

            let mut cumulative = Cumulative::new(starts, durations, resources, capacity);
            cumulative.join(vstore, cstore);
        }

        // The boolean "constant" indicates if we transform the singleton domains into constant terms or not.
        fn test(self, test_num: usize, before: SKleene, after: SKleene, constant: bool) {
            println!("Test number {}", test_num);
            let mut vstore = VStoreFD::empty();
            let mut cstore = CStoreFD::empty();
            let mut model = Model::new();
            self.instantiate(&mut model, &mut vstore, &mut cstore, constant);
            cstore.display(&(model, vstore.clone()));
            assert_eq!(cstore.is_subsumed(&vstore), before);
            assert_eq!(cstore.consistency(&mut vstore), after);
            assert_eq!(cstore.is_subsumed(&vstore), after);
        }

        fn test_assignment(self, test_num: usize, expected: SKleene, constant: bool) {
            // Unknown because cumulative introduces new variables not fixed.
            self.test(test_num, Unknown, expected, constant);
        }
    }

    #[test]
    fn disjunctive_test() {
        CumulativeTest::new_assignment(vec![0, 0], vec![0, 0], vec![1, 1], 1)
            .test_assignment(1, True, false);
    }

    #[test]
    fn singleton_task() {
        CumulativeTest::new_assignment(vec![0], vec![0], vec![1], 1).test(1, True, True, false);

        CumulativeTest::new_assignment(vec![0], vec![0], vec![1], 1).test(1, True, True, true);
    }

    #[test]
    fn cumulative_assignment_test() {
        let constant = false;
        // The task 2 and 3 overlaps and consume 4 resources altogether.
        let test = CumulativeTest::new_assignment(vec![0, 1, 4], vec![3, 4, 2], vec![1, 2, 2], 3);
        test.test(1, Unknown, False, constant);

        // We can delay the task 3 to fix the problem.
        let test = CumulativeTest::new_assignment(vec![0, 1, 5], vec![3, 4, 2], vec![1, 2, 2], 3);
        test.test(2, Unknown, True, constant);

        // Another possibility is to reduce the resource of task 3.
        let test = CumulativeTest::new_assignment(vec![0, 1, 4], vec![3, 4, 2], vec![1, 2, 1], 3);
        test.test(3, Unknown, True, constant);

        // Or augment the total amount of resources available.
        let test = CumulativeTest::new_assignment(vec![0, 1, 4], vec![3, 4, 2], vec![1, 2, 2], 4);
        test.test(4, Unknown, True, constant);

        // Or reduce the duration of task 2.
        let test = CumulativeTest::new_assignment(vec![0, 1, 4], vec![3, 3, 2], vec![1, 2, 2], 3);
        test.test(4, Unknown, True, constant);
    }

    #[test]
    fn cumulative_test_constant() {
        cumulative_test_param(true);
    }

    #[test]
    fn cumulative_test_variable() {
        cumulative_test_param(false);
    }

    fn cumulative_test_param(constant: bool) {
        let mut test =
            CumulativeTest::new_assignment(vec![0, 1, 4], vec![3, 4, 2], vec![1, 2, 2], 3);
        // Widden the start date of task 1, should fail anyway.
        test.starts[0] = Interval::new(0, 4);
        test.test(1, Unknown, False, constant);

        let mut test =
            CumulativeTest::new_assignment(vec![0, 1, 4], vec![3, 4, 2], vec![1, 2, 2], 3);
        // Widden the start date of task 2, succeed when schedule at start=0.
        test.starts[1] = Interval::new(0, 1);
        test.test(2, Unknown, Unknown, constant);

        let mut test =
            CumulativeTest::new_assignment(vec![0, 1, 4], vec![3, 4, 2], vec![1, 2, 2], 3);
        // Widden the start date of task 3, succeed when schedule at start=5.
        test.starts[2] = Interval::new(4, 5);
        test.test(3, Unknown, Unknown, constant);
    }
}
