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

pub mod branch;
pub mod brancher;
pub mod first_smallest_var;
pub mod binary_split;

use search::branching::branch::*;

pub trait VarSelection<S> {
  // Precondition: `space` must have variables not assigned.
  // Returns the index of the variable selected in `space`.
  fn select(&mut self, space: &S) -> usize;
}

pub trait Distributor<S> {
  // Postcondition: The union of the solutions of the child spaces must be equal to the solutions of the root space.
  fn distribute(&mut self, space: &S, var_idx: usize) -> Vec<Branch<S>>;
}
