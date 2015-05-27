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

use kernel::trilean::*;

pub trait Space {
  type Constraint;
  type Variable : Clone;
  type Domain;
  type Label : Clone;

  fn newvar(&mut self, dom: Self::Domain) -> Self::Variable;
  fn add(&mut self, c: Self::Constraint);
  fn solve(&mut self) -> Trilean;
  fn mark(&self) -> Self::Label;
  fn goto(self, label: Self::Label) -> Self;
}
