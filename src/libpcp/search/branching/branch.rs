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
use alloc::boxed::FnBox;
use std::rc::Rc;

// A branch represents an edge between two distinct nodes in the search tree.
// Each branch store a copy of the label of the current node.
// However, depending on the `clone` method of the `Space::Label` type,
// several branches can share the same data until calling `goto`.
// We don't store the propagators but instead a closure that
// add the propagator(s) to the new space, when available.

pub struct Branch<Space> where
  Space: State
{
  label: Rc<Space::Label>,
  alternative: Box<FnBox(&mut Space)>
}

impl<Space, L> Branch<Space> where
  Space: State<Label=L>,
  L: Clone
{
  pub fn distribute(space: &Space, alternatives: Vec<Box<FnBox(&mut Space)>>) -> Vec<Branch<Space>> {
    let label = shared_mark(space);
    alternatives.into_iter().map(|alt|
      Branch {
        label: label.clone(),
        alternative: alt
      }
    ).collect()
  }

  pub fn commit(self, space_from: Space) -> Space {
    let mut new = shared_restore(space_from, self.label);
    self.alternative.call_once((&mut new,));
    new
  }
}
