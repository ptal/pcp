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

//! The search explores a tree where nodes are a couple of variables and constraints store, called a *space*.

//! The tree is constructed during the search and backtracking occurs when a node is failed (it does not lead to a solution). The exploration of the tree can be customized by different heuristics combined with *search combinators* implemented with `SearchTreeVisitor`.

pub mod space;
pub mod recomputation;
pub mod branching;
pub mod search_tree_visitor;
pub mod propagation;
pub mod engine;
pub mod monitor;
pub mod statistics;
pub mod branch_and_bound;

pub use search::space::*;
pub use search::search_tree_visitor::*;

use propagation::CStoreFD;
use variable::VStoreFD;
use search::engine::one_solution::*;
use search::branching::*;
use search::propagation::*;
use gcollections::VectorStack;

pub type VStore = VStoreFD;
type CStore = CStoreFD<VStore>;
pub type FDSpace = Space<VStore, CStore, NoRecomputation<VStore, CStore>>;

pub fn one_solution_engine() -> Box<SearchTreeVisitor<FDSpace>> {
  let search =
    OneSolution::<_, VectorStack<_>, FDSpace>::new(
    Propagation::new(
    Brancher::new(FirstSmallestVar, BinarySplit)));
  Box::new(search)
}

#[cfg(test)]
mod test {
  pub use super::*;
  use propagators::cmp::*;
  use propagators::distinct::*;
  use term::*;
  use gcollections::ops::*;
  use interval::interval::*;

  pub fn nqueens(n: usize, space: &mut FDSpace) {
    let mut queens = vec![];
    // 2 queens can't share the same line.
    for _ in 0..n {
      queens.push(space.vstore.alloc((1, n as i32).to_interval()));
    }
    for i in 0..n-1 {
      for j in i + 1..n {
        // 2 queens can't share the same diagonal.
        let q1 = (i + 1) as i32;
        let q2 = (j + 1) as i32;
        // Xi + i != Xj + j
        space.cstore.alloc(box XNeqY::new(queens[i].clone(), Addition::new(queens[j].clone(), q2 - q1)));
        // Xi - i != Xj - j
        space.cstore.alloc(box XNeqY::new(queens[i].clone(), Addition::new(queens[j].clone(), -q2 + q1)));
      }
    }
    // 2 queens can't share the same column.
    space.cstore.alloc(box Distinct::new(queens));
  }
}