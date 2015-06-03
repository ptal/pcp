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

use std::rc::*;
use std::ops::Deref;

pub trait State {
  type Label;

  fn mark(&self) -> Self::Label;
  fn restore(self, label: Self::Label) -> Self;
}

impl<A, B> State for (A, B) where
 A: State,
 B: State
{
  type Label = (A::Label, B::Label);

  fn mark(&self) -> (A::Label, B::Label) {
    (self.0.mark(), self.1.mark())
  }

  fn restore(self, label: (A::Label, B::Label)) -> (A, B) {
    (self.0.restore(label.0), self.1.restore(label.1))
  }
}

pub fn shared_mark<S>(state: &S) -> Rc<S::Label> where
 S: State
{
  Rc::new(state.mark())
}

pub fn shared_restore<S, L>(state: S, label: Rc<S::Label>) -> S where
 S: State<Label=L>,
 L: Clone
{
  state.restore(try_unwrap(label).unwrap_or_else(|l| l.deref().clone()))
}
