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

use kernel::*;
use search::branching::branch::*;
use search::search_tree_visitor::Status::*;
use std::cmp::PartialEq;
use std::fmt::{Debug, Error, Formatter};

pub enum Status<Space>
where
    Space: Freeze,
{
    Satisfiable,
    Unsatisfiable,
    Unknown(Vec<Branch<Space>>),
    EndOfSearch,
}

impl<Space> Status<Space>
where
    Space: Freeze,
{
    pub fn pruned() -> Status<Space> {
        Unknown(vec![])
    }
}

impl<Space> Debug for Status<Space>
where
    Space: Freeze,
{
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let name = match self {
            &Satisfiable => "Satisfiable",
            &Unsatisfiable => "Unsatisfiable",
            Unknown(branches) if branches.is_empty() => "Pruned",
            &Unknown(_) => "Unknown",
            &EndOfSearch => "End of search",
        };
        formatter.write_str(name)
    }
}

impl<Space> PartialEq for Status<Space>
where
    Space: Freeze,
{
    fn eq(&self, other: &Status<Space>) -> bool {
        match (self, other) {
            (&Satisfiable, &Satisfiable) => true,
            (&Unsatisfiable, &Unsatisfiable) => true,
            (Unknown(b1), Unknown(b2)) if b1.is_empty() && b2.is_empty() => true,
            (&Unknown(_), &Unknown(_)) => panic!("Cannot compare unknown status."),
            (&EndOfSearch, &EndOfSearch) => true,
            (_, _) => false,
        }
    }
}

pub trait SearchTreeVisitor<Space>
where
    Space: Freeze,
{
    fn start(&mut self, _space: &Space) {}
    fn enter(&mut self, space: Space) -> (Space::FrozenState, Status<Space>);
}
