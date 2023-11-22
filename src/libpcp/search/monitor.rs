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

use kernel::*;
use search::search_tree_visitor::Status::*;
use search::search_tree_visitor::*;

pub trait SearchMonitor<Space: Freeze> {
    fn on_node(&mut self, space: &Space, status: &Status<Space>) {
        self.dispatch_node(space, status)
    }

    fn dispatch_node(&mut self, space: &Space, status: &Status<Space>) {
        match status {
            &Satisfiable => self.on_solution(space),
            &Unsatisfiable => self.on_failure(space),
            &EndOfSearch => self.on_end_of_search(space),
            Unknown(b) if b.is_empty() => self.on_prune(space),
            &Unknown(_) => self.on_unknown(space),
        }
    }

    fn on_solution(&mut self, _space: &Space) {}
    fn on_failure(&mut self, _space: &Space) {}
    fn on_end_of_search(&mut self, _space: &Space) {}
    fn on_prune(&mut self, _space: &Space) {}
    fn on_unknown(&mut self, _space: &Space) {}
}

pub struct Monitor<'a, M: 'a, C> {
    monitor: &'a mut M,
    child: C,
}

impl<'a, M, C> Monitor<'a, M, C> {
    pub fn new(monitor: &'a mut M, child: C) -> Monitor<'a, M, C> {
        Monitor { monitor, child }
    }
}

impl<'a, M, C, Space> SearchTreeVisitor<Space> for Monitor<'a, M, C>
where
    Space: Freeze,
    C: SearchTreeVisitor<Space>,
    M: SearchMonitor<Space>,
{
    fn start(&mut self, root: &Space) {
        self.child.start(root);
    }

    fn enter(&mut self, current: Space) -> (Space::FrozenState, Status<Space>) {
        let (immutable_state, status) = self.child.enter(current);
        let space = immutable_state.unfreeze();
        self.monitor.on_node(&space, &status);
        (space.freeze(), status)
    }
}
