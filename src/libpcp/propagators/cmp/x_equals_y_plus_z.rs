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
use propagators::PropagatorKind;
use propagators::cmp::{XLessYPlusZ, XLessEqYPlusZ, XGreaterYPlusZ, XGreaterEqYPlusZ, x_geq_y_plus_z, x_leq_y_plus_z};
use propagation::*;
use propagation::events::*;
use term::ops::*;
use gcollections::ops::*;
use std::fmt::{Formatter, Debug, Error};
use num::traits::Num;
use term::addition::Addition;
use num::PrimInt;
use term::expr_inference::ExprInference;

#[derive(Clone, Copy)]
pub struct XEqualsYPlusZ<X, Y, Z, BX> 
{
  greater: XGreaterYPlusZ<Addition<X, BX>, Y, Z>,
  less: XLessYPlusZ<Addition<X, BX>, Y, Z>,
  x: X,
  y: Y,
  z: Z,
}

impl<X, Y, Z, BX> PropagatorKind for XEqualsYPlusZ<X, Y, Z, BX> {}

impl<X, Y, Z, BX> XEqualsYPlusZ<X, Y, Z, BX> where
  X: Clone,
  Y: Clone,
  Z: Clone,
  BX: PrimInt {
  pub fn new(x: X, y: Y, z: Z) -> XEqualsYPlusZ<X, Y, Z, BX> {
    XEqualsYPlusZ { 
      greater: XGreaterYPlusZ::new(Addition::new(x.clone(), BX::one()), y.clone(), z.clone()),
      less: XLessYPlusZ::new(Addition::new(x.clone(), BX::one()), y.clone(), z.clone()),
      x: x,
      y: y,
      z: z,

    }
  }
}

impl<X, Y, Z, BX> Debug for XEqualsYPlusZ<X, Y, Z, BX> where
  X: Debug,
  Y: Debug,
  Z: Debug,
  BX: PrimInt + Debug
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("Great:{:?} and less {:?}", self.greater, self.less))
  }
}

impl<Store, B, DomX, DomY, DomZ, X, Y, Z> Subsumption<Store> for XEqualsYPlusZ<X, Y, Z, B> where
  X: ExprInference<Output=DomX> + StoreRead<Store, Value=DomX> + Debug,
  Y: StoreRead<Store, Value=DomY> + Debug,
  Z: StoreRead<Store, Value=DomZ> + Debug,
  DomX: Bounded<Bound=B> + Debug,
  DomY: Bounded<Bound=B> + Debug,
  DomZ: Bounded<Bound=B> + Debug,
  B: PartialOrd + Num + Debug{
  fn is_subsumed(&self, store: &Store) -> Trilean {
    //debug!("PCP XEqualsYPlusZ is_subsumed SELF:{:?}", self);
    self.greater.is_subsumed(store).add(self.less.is_subsumed(store))
  }
}

impl<Store, B, DomX, DomY, DomZ, X, Y, Z> Propagator<Store> for XEqualsYPlusZ<X, Y, Z, B> where
  X: StoreRead<Store, Value=DomX> + StoreMonotonicUpdate<Store, DomX> + Debug,
  Y: StoreRead<Store, Value=DomY> + StoreMonotonicUpdate<Store, DomY> + Debug,
  Z: StoreRead<Store, Value=DomZ> + StoreMonotonicUpdate<Store, DomZ> + Debug,
  DomX: Bounded<Bound=B> + StrictShrinkRight<B> + StrictShrinkLeft<B> + Debug,
  DomY: Bounded<Bound=B> + StrictShrinkRight<B> + StrictShrinkLeft<B> + Debug,
  DomZ: Bounded<Bound=B> + StrictShrinkRight<B> + StrictShrinkLeft<B> + Debug,
  B: PartialOrd + Num + Debug,
{
  fn propagate(&mut self, store: &mut Store) -> bool {
    self.greater.propagate(store) &&
    self.less.propagate(store)
  }
}

impl<X, Y, Z, BX> PropagatorDependencies<FDEvent> for XEqualsYPlusZ<X, Y, Z, BX> where
  X: ViewDependencies<FDEvent>,
  Y: ViewDependencies<FDEvent>,
  Z: ViewDependencies<FDEvent>, 
  BX: PrimInt
{
  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    let mut deps = self.x.dependencies(FDEvent::Bound);
    deps.append(&mut self.y.dependencies(FDEvent::Bound));
    deps.append(&mut self.z.dependencies(FDEvent::Bound));
    deps
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::*;
  use kernel::Trilean::*;
  use propagation::events::*;
  use propagation::events::FDEvent::*;
  use interval::interval::*;
  use propagators::test::*;

  #[test]
  fn x_less_y_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom11_12 = (11,12).to_interval();
    let dom0_6 = (0,6).to_interval();
    let dom0_5 = (0,5).to_interval();
    let dom0_1 = (0,1).to_interval();
    let dom1_10 = (1,10).to_interval();
    let dom5_10 = (5,10).to_interval();
    let dom6_10 = (6,10).to_interval();
    let dom1_1 = (1,1).to_interval();
    let dom2_2 = (2,2).to_interval();

    x_eqauls_y_plus_z_test_one(1, dom0_10, dom0_10, dom0_10, Unknown, Unknown, vec![], true);
    x_eqauls_y_plus_z_test_one(2, dom11_12, dom0_6, dom0_6, Unknown, True, vec![(0, Assignment), (1, Assignment), (2, Assignment)], true);
    x_eqauls_y_plus_z_test_one(3, dom10_20, dom1_1, dom1_1, False, False, vec![], false);
    x_eqauls_y_plus_z_test_one(4, dom2_2, dom1_1, dom1_1, False, False, vec![], false);
    x_eqauls_y_plus_z_test_one(5, dom1_1, dom2_2, dom2_2, True, True, vec![], true);
    x_eqauls_y_plus_z_test_one(6, dom0_6, dom0_5, dom0_1, Unknown, Unknown, vec![(0, Bound)], true);
  }

  fn x_eqauls_y_plus_z_test_one(test_num: u32,
    x: Interval<i32>, y: Interval<i32>, z: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    trinary_propagator_test(test_num, XEqualsYPlusZ::new, x, y, z, before, after, delta_expected, propagate_success);
  }
}
