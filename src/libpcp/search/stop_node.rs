// Copyright 2017 Pierre Talbot (IRCAM)

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
use kernel::*;
use search::search_tree_visitor::*;
use search::space::*;

pub struct StopNode<C> {
    child: C,
    limit: usize,
    nodes_explored: usize,
}

impl<C> StopNode<C> {
    pub fn new(limit: usize, child: C) -> StopNode<C> {
        StopNode {
            child,
            limit,
            nodes_explored: 0,
        }
    }
}

impl<VStore, CStore, R, C> SearchTreeVisitor<Space<VStore, CStore, R>> for StopNode<C>
where
    VStore: VStoreConcept,
    CStore: IntCStore<VStore>,
    C: SearchTreeVisitor<Space<VStore, CStore, R>>,
    R: FreezeSpace<VStore, CStore> + Snapshot<State = Space<VStore, CStore, R>>,
{
    fn start(&mut self, root: &Space<VStore, CStore, R>) {
        self.child.start(root);
    }

    fn enter(
        &mut self,
        current: Space<VStore, CStore, R>,
    ) -> (
        <Space<VStore, CStore, R> as Freeze>::FrozenState,
        Status<Space<VStore, CStore, R>>,
    ) {
        let (space, status) = self.child.enter(current);
        self.nodes_explored += 1;
        // If we reached the limit, we stop the search by changing the status.
        if self.nodes_explored >= self.limit {
            (space, Status::EndOfSearch)
        } else {
            (space, status)
        }
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
    use search::engine::all_solution::*;
    use search::engine::one_solution::*;
    use search::monitor::*;
    use search::propagation::*;
    use search::statistics::*;
    use search::test::*;
    use search::FDSpace;

    #[test]
    fn test_stop() {
        let n = 6;
        let mut space = FDSpace::empty();
        nqueens(n, &mut space);

        let nodes_limit = 10;
        let mut statistics = Statistics::new();
        {
            let mut search: AllSolution<OneSolution<_, VectorStack<_>, FDSpace>> =
                AllSolution::new(OneSolution::new(Monitor::new(
                    &mut statistics,
                    StopNode::new(
                        nodes_limit,
                        Propagation::new(Brancher::new(FirstSmallestVar, MiddleVal, BinarySplit)),
                    ),
                )));
            search.start(&space);
            let (_, status) = search.enter(space);
            assert_eq!(status, Status::EndOfSearch);
        }
        assert_eq!(statistics.num_nodes, nodes_limit);
    }
}
