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

//! Constraint programming is a declarative programming paradigm mainly used to solve combinatorial problems where you state what constraints a solution must fulfill instead of explaining how to solve it (see README.md).
//! A very classic introductory problem to constraint programming is the [N-Queens puzzle](https://en.wikipedia.org/wiki/Eight_queens_puzzle): the goal is to align N queens on a chessboard of size N*N such that no queen can attack each other.
//! In PCP, we can solve this problem as follows:
//! ```rust
//! extern crate pcp;
//! extern crate gcollections;
//! extern crate interval;
//! use pcp::kernel::*;
//! use pcp::propagators::*;
//! use pcp::variable::ops::*;
//! use pcp::term::*;
//! use pcp::search::search_tree_visitor::Status::*;
//! use pcp::search::*;
//! use pcp::concept::*;
//! use interval::ops::Range;
//! use interval::interval_set::*;
//! use gcollections::ops::*;
//!
//! pub fn nqueens(n: usize) {
//!   let mut space = FDSpace::empty();
//!
//!   let mut queens = vec![];
//!   // 2 queens can't share the same line.
//!   for _ in 0..n {
//!     queens.push(Box::new(space.vstore.alloc(IntervalSet::new(1, n as i32))) as Var<VStore>);
//!   }
//!   for i in 0..n-1 {
//!     for j in i + 1..n {
//!       // 2 queens can't share the same diagonal.
//!       let q1 = (i + 1) as i32;
//!       let q2 = (j + 1) as i32;
//!       // Xi + i != Xj + j reformulated as: Xi != Xj + j - i
//!       space.cstore.alloc(Box::new(XNeqY::new(
//!         queens[i].bclone(), Box::new(Addition::new(queens[j].bclone(), q2 - q1)) as Var<VStore>)));
//!       // Xi - i != Xj - j reformulated as: Xi != Xj - j + i
//!       space.cstore.alloc(Box::new(XNeqY::new(
//!         queens[i].bclone(), Box::new(Addition::new(queens[j].bclone(), -q2 + q1)) as Var<VStore>)));
//!     }
//!   }
//!   // 2 queens can't share the same column.
//!   // join_distinct(&mut space.vstore, &mut space.cstore, queens);
//!   space.cstore.alloc(Box::new(Distinct::new(queens)));
//!
//!   // Search step.
//!   let mut search = one_solution_engine();
//!   search.start(&space);
//!   let (frozen_space, status) = search.enter(space);
//!   let space = frozen_space.unfreeze();
//!
//!   // Print result.
//!   match status {
//!     Satisfiable => {
//!       print!("{}-queens problem is satisfiable. The first solution is:\n[", n);
//!       for dom in space.vstore.iter() {
//!         // At this stage, dom.lower() == dom.upper().
//!         print!("{}, ", dom.lower());
//!       }
//!       println!("]");
//!     }
//!     Unsatisfiable => println!("{}-queens problem is unsatisfiable.", n),
//!     EndOfSearch => println!("Search terminated or was interrupted."),
//!     Unknown(_) => unreachable!(
//!       "After the search step, the problem instance should be either satisfiable or unsatisfiable.")
//!   }
//! }
//!
//! fn main() {
//!  nqueens(8);
//! }
//! ```

extern crate bit_set;
extern crate gcollections;
extern crate interval;
extern crate num;
extern crate trilean;
extern crate vec_map;

pub mod concept;
pub mod kernel;
pub mod logic;
pub mod model;
pub mod propagation;
pub mod propagators;
pub mod search;
pub mod term;
pub mod variable;
