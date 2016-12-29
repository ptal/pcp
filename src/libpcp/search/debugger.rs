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

use kernel::*;
use variable::ops::*;
use gcollections::ops::*;
use search::space::*;
use search::search_tree_visitor::*;
use std::fmt::{Debug, Display};
use std::io::{self};

pub struct Debugger<C> {
  child: C
}

impl<C> Debugger<C> {
  pub fn new(child: C) -> Debugger<C> {
    Debugger {
      child: child
    }
  }
}

impl<VStore, CStore, Domain, R, C> SearchTreeVisitor<Space<VStore, CStore, R>> for Debugger<C> where
  VStore: Freeze + Display + Iterable<Item=Domain> + Cardinality<Size=usize>,
  CStore: Freeze + Consistency<VStore> + Debug,
  C: SearchTreeVisitor<Space<VStore, CStore, R>>,
  Domain: IsSingleton,
  R: FreezeSpace<VStore, CStore> + Snapshot<State=Space<VStore, CStore, R>>
{
  fn start(&mut self, root: &Space<VStore, CStore, R>) {
    self.child.start(root);
  }

  fn enter(&mut self, current: Space<VStore, CStore, R>)
    -> (<Space<VStore, CStore, R> as Freeze>::FrozenState, Status<Space<VStore, CStore, R>>)
  {
    println!("Enter debugging:");
    println!("  Variable store:\n{}\n", current.vstore);
    println!("  Number of variables: {}\n", current.vstore.size());
    println!("  Number of variables assigned: {}\n", current.vstore.iter().filter(|v| v.is_singleton()).count());
    // println!("  Constraint store:\n{:?}\n", current.cstore);
    let (frozen, status) = self.child.enter(current);
    println!("Exit debugging with status {:?}", status);
    println!("Press enter to continue...");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    (frozen, status)
  }
}
