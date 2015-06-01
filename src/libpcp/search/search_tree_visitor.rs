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

use kernel::State;
use search::search_tree_visitor::Status::*;
use search::branching::branch::*;
use std::cmp::PartialEq;
use std::fmt::{Debug, Formatter, Error};

pub enum Status<S: State> {
  Satisfiable,
  Pruned,
  Unsatisfiable,
  Unknown(Vec<Branch<S>>)
}

impl<S: State> Debug for Status<S> {
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    let name = match self {
      &Satisfiable => "Satisfiable",
      &Unsatisfiable => "Unsatisfiable",
      &Pruned => "Pruned",
      &Unknown(_) => "Unknown"
    };
    formatter.write_str(name)
  }
}

impl<S: State> PartialEq for Status<S> {
  fn eq(&self, other: &Status<S>) -> bool {
    match (self, other) {
      (&Satisfiable, &Satisfiable) => true,
      (&Unsatisfiable, &Unsatisfiable) => true,
      (&Pruned, &Pruned) => true,
      (&Unknown(_), &Unknown(_)) => panic!("Cannot compare unknown status."),
      (_, _) => false,
    }
  }
}

impl<S: State> Status<S> {
  // Promote `self` to `Pruned` or `Satisfiable` depending on `status`.
  pub fn or(self, status: &Status<S>) -> Self {
    match (self, status) {
      (_, &Satisfiable) => Satisfiable,
      (Unsatisfiable, &Pruned) => Pruned,
      (s, _) => s,
    }
  }
}

pub trait SearchTreeVisitor<S: State> {
  fn start(&mut self, _root: &S) {}
  fn enter(&mut self, current: S) -> (S, Status<S>);
}
