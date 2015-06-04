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

use search::branching::*;
use search::search_tree_visitor::*;
use kernel::State;

pub struct Brancher<S,D>
{
  selector: S,
  distributor: D
}

impl<S,D> Brancher<S,D>
{
  pub fn new(selector: S, distributor: D) -> Brancher<S,D> {
    Brancher {
      selector: selector,
      distributor: distributor
    }
  }
}

impl<Space,S,D> SearchTreeVisitor<Space> for Brancher<S,D> where
  Space: State,
  S: VarSelection<Space>,
  D: Distributor<Space>
{
  fn enter(&mut self, current: Space) -> (Space, Status<Space>) {
    let var_idx = self.selector.select(&current);
    let branches = self.distributor.distribute(&current, var_idx);
    (current, Status::Unknown(branches))
  }
}
