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

use gcollections::ops::multiset::*;
use gcollections::*;
/// OneSolution combinator is a generator over the solution. It returns from `enter` each time it found a solution (with `Satisfiable`) or when no more node can be explored (with `EndOfSearch`). Its method `enter` can be safely call several time to generate more than one solution.
use kernel::*;
use search::branching::branch::*;
use search::search_tree_visitor::Status::*;
use search::search_tree_visitor::*;
use std::marker::PhantomData;

pub struct OneSolution<C, Q, Space> {
    pub child: C,
    queue: Q,
    started_exploration: bool,
    phantom_space: PhantomData<Space>,
}

impl<C, Q, Space> OneSolution<C, Q, Space>
where
    Space: Freeze,
    C: SearchTreeVisitor<Space>,
    Q: Multiset + Collection<Item = Branch<Space>>,
{
    pub fn new(child: C) -> OneSolution<C, Q, Space> {
        OneSolution {
            queue: Q::empty(),
            child,
            started_exploration: false,
            phantom_space: PhantomData,
        }
    }

    fn push_branches(&mut self, branches: Vec<Branch<Space>>) {
        // For traversing the tree from left to right.
        for branch in branches.into_iter().rev() {
            self.queue.insert(branch);
        }
    }

    fn enter_child(&mut self, current: Space, status: &mut Status<Space>) -> Space::FrozenState {
        let (immutable_state, child_status) = self.child.enter(current);
        match child_status {
            Unknown(ref branches) if branches.is_empty() => *status = Status::pruned(),
            Unknown(branches) => self.push_branches(branches),
            Satisfiable => *status = Satisfiable,
            EndOfSearch => *status = EndOfSearch,
            _ => (),
        }
        immutable_state
    }

    fn fully_explored(&self) -> bool {
        self.queue.is_empty() && self.started_exploration
    }

    // Only visit the root if we didn't visit it before (based on the queue emptiness).
    fn enter_root(&mut self, root: Space, status: &mut Status<Space>) -> Space::FrozenState {
        if self.queue.is_empty() && !self.started_exploration {
            self.started_exploration = true;
            self.enter_child(root, status)
        } else {
            root.freeze()
        }
    }
}

impl<C, Q, Space> SearchTreeVisitor<Space> for OneSolution<C, Q, Space>
where
    Space: Freeze,
    C: SearchTreeVisitor<Space>,
    Q: Multiset + Collection<Item = Branch<Space>>,
{
    fn start(&mut self, root: &Space) {
        self.queue = Q::empty();
        self.started_exploration = false;
        self.child.start(root);
    }

    fn enter(&mut self, root: Space) -> (Space::FrozenState, Status<Space>) {
        if self.fully_explored() {
            return (root.freeze(), EndOfSearch);
        }

        let mut status = Unsatisfiable;
        let mut immutable_state = self.enter_root(root, &mut status);
        while status != EndOfSearch && status != Satisfiable && !self.queue.is_empty() {
            let branch = self.queue.extract().unwrap();
            let child = branch.commit(immutable_state);
            immutable_state = self.enter_child(child, &mut status);
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
    use search::propagation::*;
    use search::test::*;

    #[test]
    fn example_nqueens() {
        test_nqueens(1, Satisfiable);
        test_nqueens(2, Unsatisfiable);
        test_nqueens(3, Unsatisfiable);
        for i in 4..12 {
            test_nqueens(i, Satisfiable);
        }
    }

    fn test_nqueens(n: usize, expect: Status<FDSpace>) {
        let mut space = FDSpace::empty();
        nqueens(n, &mut space);

        let mut search: OneSolution<_, VectorStack<_>, FDSpace> = OneSolution::new(
            Propagation::new(Brancher::new(FirstSmallestVar, MiddleVal, BinarySplit)),
        );
        search.start(&space);
        let (_, status) = search.enter(space);
        assert_eq!(status, expect);
    }
}
