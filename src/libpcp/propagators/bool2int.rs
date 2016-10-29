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

use term::ops::*;
use term::ExprInference;
use std::ops::*;
use std::fmt::{Formatter, Debug, Error};
use propagation::ops::Subsumption;
use kernel::trilean::Trilean;
use search::VStore;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bool2Int
{
  conjunction: Vec<Box<Subsumption<VStore>>>,
}

impl Bool2Int {
  pub fn new() -> Bool2Int {
    Bool2Int {
      conjunction: vec!(),
    }
  }
}

impl Debug for Bool2Int
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("Conjunction"))
  }
}

impl<Store> Subsumption<Store> for Bool2Int
{
  fn is_subsumed(&self, store: &Store) -> Trilean {
    return Trilean::Unknown;
  }
}


#[cfg(test)]
mod test {
  use super::*;
  use gcollections::ops::*;
  use kernel::*;
  use kernel::trilean::Trilean::*;
  use variable::VStoreFD;
  use propagation::events::FDEvent;
  use propagation::events::FDEvent::*;
  use propagators::test::*;
  use propagators::cmp::XLessY;
  use interval::interval::*;

  type Domain = Interval<i32>;
  type VStore = VStoreFD;

  #[test]
  fn x_less_y_mul_c() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom5_10 = (5,10).to_interval();
    let dom1_1 = (1,1).to_interval();

    // Same test as `x < y` but we add `c` to `y`.
    x_less_y_plus_c_test_one(1, dom0_10, dom5_15, 2, False, False, vec![], false);
    x_less_y_plus_c_test_one(2, dom0_10, dom0_10, 2, Unknown, Unknown, vec![], true);
    x_less_y_plus_c_test_one(3, dom5_15, dom5_15, 1, Unknown, Unknown, vec![], true);
    x_less_y_plus_c_test_one(4, dom10_20, dom5_10*2, -10, True, True, vec![], true);
    x_less_y_plus_c_test_one(7, dom5_15, dom1_1, 4, False, False, vec![], false);
    x_less_y_plus_c_test_one(7, dom5_15, dom1_1, 5, Unknown, True, vec![(1, Bound)], true);
    x_less_y_plus_c_test_one(7, dom5_15, dom1_1, 15, False, False, vec![], false);
  }

  fn x_less_y_mul_c_test_one(id: u32, x: Domain, y: Domain, c: i32,
    before: Trilean, after: Trilean, expected: Vec<(usize, FDEvent)>, update_success: bool)
  {
    let mut store = VStore::empty();
    let x = store.alloc(x);
    let y = store.alloc(y);
    let x_less_y_plus_c = XLessY::new(x, Multiplication::new(y, c));
    subsumption_propagate(id, x_less_y_plus_c, &mut store, before, after, expected, update_success);
  }
}
