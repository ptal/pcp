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

/// The `AllSolution` combinator continuously calls its child until it returns `EndOfSearch`. You should use it with the `OneSolution` combinator.
use kernel::*;
use search::search_tree_visitor::Status::*;
use search::search_tree_visitor::*;

pub struct AllSolution<C> {
    pub child: C,
}

impl<C> AllSolution<C> {
    pub fn new(child: C) -> Self {
        AllSolution { child }
    }
}

impl<C, Space> SearchTreeVisitor<Space> for AllSolution<C>
where
    Space: Freeze,
    C: SearchTreeVisitor<Space>,
{
    fn start(&mut self, root: &Space) {
        self.child.start(root);
    }

    fn enter(&mut self, root: Space) -> (Space::FrozenState, Status<Space>) {
        let (mut immutable_state, mut status) = self.child.enter(root);
        while status != EndOfSearch {
            let state = immutable_state.unfreeze();
            let frozen_state = self.child.enter(state);
            immutable_state = frozen_state.0;
            status = frozen_state.1;
        }
        (immutable_state, status)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use gcollections::ops::*;
    use gcollections::VectorStack;
    use search::branching::binary_split::*;
    use search::branching::brancher::*;
    use search::branching::first_smallest_var::*;
    use search::branching::middle_val::*;
    use search::engine::one_solution::*;
    use search::monitor::*;
    use search::propagation::*;
    use search::statistics::*;
    use search::test::*;
    use search::FDSpace;

    #[test]
    fn example_nqueens() {
        // Data from Wikipedia.
        let nqueens_solution = vec![1, 0, 0, 2, 10, 4, 40, 92, 352];

        for (n, sol) in nqueens_solution.into_iter().enumerate() {
            test_nqueens(n + 1, sol, EndOfSearch);
        }
    }

    fn test_nqueens(n: usize, sol_expected: usize, expect: Status<FDSpace>) {
        let mut space = FDSpace::empty();
        nqueens(n, &mut space);

        let mut statistics = Statistics::new();
        {
            let mut search: AllSolution<
                Monitor<Statistics, OneSolution<_, VectorStack<_>, FDSpace>>,
            > = AllSolution::new(Monitor::new(
                &mut statistics,
                OneSolution::new(Propagation::new(Brancher::new(
                    FirstSmallestVar,
                    MiddleVal,
                    BinarySplit,
                ))),
            ));
            search.start(&space);
            let (_, status) = search.enter(space);
            assert_eq!(status, expect);
        }
        assert_eq!(statistics.num_solution, sol_expected);
    }
}
