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

use kernel::trilean::Trilean::*;
use std::ops::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trilean
{
  False = 0,
  True = 1,
  Unknown = 2
}

impl Trilean {
  pub fn and(self, other: Trilean)-> Trilean {
    match (self, other) {
      (True, True) => True,
      (False, _) => False,
      (_, False) => False,
      _ => Unknown
    }
  }
}

impl Not for Trilean {
  type Output = Trilean;

  fn not(self) -> Trilean {
    match self {
      False => True,
      True => False,
      Unknown => Unknown
    }
  }
}
