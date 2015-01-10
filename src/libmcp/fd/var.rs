// Copyright 2014 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use interval::interval::*;
use self::VarEvent::*;
use std::cmp::min;
use std::num::FromPrimitive;

// we try to take the most precise first
// which is failure, then Ass, then Bound,...

// the union between two events is min(e1, e2)
#[derive(Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
pub enum VarEvent {
  Failure,
  Assignment,
  Bound,
  Inner,
  Nothing
}

impl VarEvent {
  pub fn merge(self, other: VarEvent) -> VarEvent {
    FromPrimitive::from_int(min(self as int, other as int)).unwrap()
  }
}

#[derive(Copy, PartialEq, Eq)]
pub struct Var {
  id: uint,
  dom: Interval
}

impl Var {
  pub fn new(id: uint, dom: Interval) -> Var {
    Var {
      id: id,
      dom: dom
    }
  }

  pub fn update(&mut self, dom: Interval) -> VarEvent {
    let old = self.dom;
    self.dom = dom;
    if dom.is_empty() { Failure }
    else if dom.is_singleton() { Assignment }
    else if dom.lower() != old.lower() ||
            dom.upper() != old.upper() { Bound }
    else if dom != old { Inner }
    else { Nothing }
  }

  pub fn id(&self) -> uint { self.id }
}
