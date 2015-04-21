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

use solver::space::Space;
use search::branching::branch::*;

// This is inspired by the paper:
//   Search Combinators
//   Authors: Tom Schrijvers, Guido Tack, Pieter Wuille, Horst Samulowitz, Peter J. Stuckey

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
  Satisfiable,
  Unsatisfiable,
  Pruned
}

pub trait SearchTreeVisitor<S: Space> {
  fn front_edge(current: &S, branch: &Branch<S>) {}
  fn back_edge(current: &S, exit_status: Status) {}
  fn start(root: &S) {}
  fn enter(current: &S) {}
}
