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

pub mod binary_split;
pub mod branch;
pub mod brancher;
pub mod enumerate;
pub mod first_smallest_var;
pub mod input_order;
pub mod middle_val;
pub mod min_val;

pub use search::branching::binary_split::*;
pub use search::branching::brancher::*;
pub use search::branching::enumerate::*;
pub use search::branching::first_smallest_var::*;
pub use search::branching::input_order::*;
pub use search::branching::middle_val::*;
pub use search::branching::min_val::*;

use gcollections::*;
use search::branching::branch::*;

use kernel::*;

pub trait VarSelection<Space> {
    // Precondition: `space` must have variables not assigned.
    // Returns the index of the variable selected in `space`.
    fn select(&mut self, space: &Space) -> usize;
}

pub trait ValSelection<Domain>
where
    Domain: Collection,
{
    fn select(&mut self, dom: Domain) -> Domain::Item;
}

pub trait Distributor<Space, Bound>
where
    Space: Freeze,
{
    // Postcondition: The union of the solutions of the child spaces must be equal to the solutions of the root space.
    fn distribute(
        &mut self,
        space: Space,
        var_idx: usize,
        val: Bound,
    ) -> (Space::FrozenState, Vec<Branch<Space>>);
}
