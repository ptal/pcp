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
use model::*;
use propagation::events::*;
use term::ops::*;
use gcollections::kind::*;
use std::ops::*;
use std::fmt::{Debug, Formatter, Result};
use concept::*;

pub struct Addition<VStore> where
 VStore: VStoreConcept,
 VStore::Item: Collection
{
  x: Var<VStore>,
  v: <VStore::Item as Collection>::Item
}

impl<VStore, Domain, Bound> Addition<VStore> where
  VStore: VStoreConcept<Item=Domain>,
  Domain: Collection<Item=Bound>
{
  pub fn new(x: Var<VStore>, v: Bound) -> Self {
    Addition {
      x: x,
      v: v
    }
  }
}

impl<VStore, Domain, Bound> Debug for Addition<VStore> where
  VStore: VStoreConcept<Item=Domain>,
  Domain: Collection<Item=Bound>,
  Bound: Debug
{
  fn fmt(&self, fmt: &mut Formatter) -> Result {
    fmt.debug_struct("Addition")
      .field("x", &self.x)
      .field("v", &self.v)
      .finish()
  }
}

impl<VStore, Domain, Bound> Clone for Addition<VStore> where
  VStore: VStoreConcept<Item=Domain>,
  Domain: Collection<Item=Bound>,
  Bound: Clone
{
  fn clone(&self) -> Self {
    Addition::new(self.x.bclone(), self.v.clone())
  }
}

impl<VStore, Domain, Bound> DisplayStateful<Model> for Addition<VStore> where
 VStore: VStoreConcept<Item=Domain>,
 Domain: Collection<Item=Bound>,
 Bound: Debug
{
  fn display(&self, model: &Model) {
    self.x.display(model);
    print!(" + {:?}", self.v);
  }
}

impl<VStore, Domain, Bound> StoreMonotonicUpdate<VStore> for Addition<VStore> where
 VStore: VStoreConcept<Item=Domain>,
 Domain: Collection<Item=Bound> + Sub<Bound, Output=Domain>,
 Bound: Clone
{
  fn update(&mut self, store: &mut VStore, value: Domain) -> bool {
    self.x.update(store, value - self.v.clone())
  }
}

impl<VStore, Domain, Bound> StoreRead<VStore> for Addition<VStore> where
 VStore: VStoreConcept<Item=Domain>,
 Domain: Collection<Item=Bound> +  Add<Bound, Output=Domain>,
 Bound: Clone
{
  fn read(&self, store: &VStore) -> VStore::Item {
    self.x.read(store) + self.v.clone()
  }
}

impl<VStore> ViewDependencies<FDEvent> for Addition<VStore> where
 VStore: VStoreConcept,
 VStore::Item: Collection
{
  fn dependencies(&self, event: FDEvent) -> Vec<(usize, FDEvent)> {
    self.x.dependencies(event)
  }
}

// #[cfg(test)]
// mod test {
//   use super::*;
//   use gcollections::ops::*;
//   use trilean::SKleene::*;
//   use variable::VStoreFD;
//   use propagation::events::FDEvent;
//   use propagation::events::FDEvent::*;
//   use propagators::test::*;
//   use propagators::cmp::XLessY;
//   use interval::interval::*;

//   type Domain = Interval<i32>;
//   type VStore = VStoreFD;

//   #[test]
//   fn x_less_y_plus_c() {
//     let dom0_10 = (0,10).to_interval();
//     let dom10_20 = (10,20).to_interval();
//     let dom5_15 = (5,15).to_interval();
//     let dom1_1 = (1,1).to_interval();

//     // Same test as `x < y` but we add `c` to `y`.
//     x_less_y_plus_c_test_one(1, dom0_10, dom5_15, -5, Unknown, Unknown, vec![(0, Bound), (1, Bound)], true);
//     x_less_y_plus_c_test_one(2, dom0_10, dom0_10, 10, Unknown, Unknown, vec![], true);
//     x_less_y_plus_c_test_one(3, dom5_15, dom5_15, 5, Unknown, Unknown, vec![], true);
//     x_less_y_plus_c_test_one(4, dom5_15, dom10_20, -10, Unknown, Unknown, vec![(0, Bound), (1, Bound)], true);
//     x_less_y_plus_c_test_one(5, dom0_10, dom0_10, 11, True, True, vec![], true);
//     x_less_y_plus_c_test_one(6, dom0_10, dom0_10, -11, False, False, vec![], false);
//     x_less_y_plus_c_test_one(7, dom1_1, dom5_15, -5, Unknown, True, vec![(1, Bound)], true);
//   }

//   fn x_less_y_plus_c_test_one(id: u32, x: Domain, y: Domain, c: i32,
//     before: SKleene, after: SKleene, expected: Vec<(usize, FDEvent)>, update_success: bool)
//   {
//     let mut store = VStore::empty();
//     let x = store.alloc(x);
//     let y = store.alloc(y);
//     let x_less_y_plus_c = XLessY::new(x, Addition::new(y, c));
//     subsumption_propagate(id, x_less_y_plus_c, &mut store, before, after, expected, update_success);
//   }
// }
