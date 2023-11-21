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
use search::monitor::*;
use search::search_tree_visitor::Status;

pub struct Statistics {
    pub num_solution: usize,
    pub num_failed_node: usize,
    pub num_prune: usize,
    pub num_nodes: usize,
}

impl Statistics {
    pub fn new() -> Self {
        Statistics {
            num_solution: 0,
            num_failed_node: 0,
            num_prune: 0,
            num_nodes: 0,
        }
    }
}

impl<Space: Freeze> SearchMonitor<Space> for Statistics {
    fn on_node(&mut self, space: &Space, status: &Status<Space>) {
        println!("\n\nLLLL\n\n");
        self.num_nodes += 1;
        self.dispatch_node(space, status)
    }

    fn on_solution(&mut self, _space: &Space) {
        self.num_solution += 1;
    }
    fn on_failure(&mut self, _space: &Space) {
        self.num_failed_node += 1;
    }
    fn on_prune(&mut self, _space: &Space) {
        self.num_prune += 1;
    }
}
