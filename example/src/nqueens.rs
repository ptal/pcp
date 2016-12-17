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

// This example models the n-queens problem using interval domain (with `i32` bounds).

use pcp::kernel::*;
use pcp::propagators::*;
use pcp::variable::ops::*;
use pcp::term::*;
use pcp::search::search_tree_visitor::Status::*;
use pcp::search::*;
use interval::interval::*;
use gcollections::ops::*;

use pcp::propagation::CStoreFD;
use pcp::propagation::events::*;
use pcp::variable::memory::*;
use pcp::variable::store::*;
use pcp::search::engine::one_solution::*;
use pcp::search::branching::*;
use pcp::search::propagation::*;
use gcollections::VectorStack;

type TVStore = Store<TimestampTrailMemory<Interval<i32>>, FDEvent>;

type TCStore = CStoreFD<TVStore>;
type TFDSpace = Space<TVStore, TCStore>;

fn t_one_solution_engine() -> Box<SearchTreeVisitor<TFDSpace>> {
  let search =
    OneSolution::<_, VectorStack<_>, TFDSpace>::new(
    Propagation::new(
    Brancher::new(FirstSmallestVar, BinarySplit)));
  Box::new(search)
}


pub fn nqueens(n: usize) {
  let mut space = TFDSpace::empty();
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
      // Xi + i != Xj + j reformulated as: Xi != Xj + j - i
      space.cstore.alloc(box XNeqY::new(
        queens[i], Addition::new(queens[j], q2 - q1)));
      // Xi - i != Xj - j reformulated as: Xi != Xj - j + i
      space.cstore.alloc(box XNeqY::new(
        queens[i], Addition::new(queens[j], -q2 + q1)));
    }
  }
  // 2 queens can't share the same column.
  space.cstore.alloc(box Distinct::new(queens));

  // Search step.
  let mut search = t_one_solution_engine();
  search.start(&space);
  let (frozen_space, status) = search.enter(space);
  let space = frozen_space.unfreeze();

  // Print result.
  match status {
    Satisfiable => {
      print!("{}-queens problem is satisfiable. The first solution is:\n[", n);
      for dom in space.vstore.iter() {
        // At this stage, dom.lower() == dom.upper().
        print!("{}, ", dom.lower());
      }
      println!("]");
    }
    Unsatisfiable => println!("{}-queens problem is unsatisfiable.", n),
    EndOfSearch => println!("Search terminated or was interrupted."),
    Unknown(_) => unreachable!(
      "After the search step, the problem instance should be either satisfiable or unsatisfiable.")
  }
}