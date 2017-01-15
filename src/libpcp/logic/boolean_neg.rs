// Copyright 2017 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// This class implements the negation of boolean value (not arbritrary formula, for which the negation can be obtained with `f.not()`).

use kernel::*;
use model::*;
use logic::boolean::*;
use propagators::PropagatorKind;
use propagation::*;
use propagation::events::*;
use term::ops::*;
use gcollections::kind::*;
use gcollections::ops::*;
use interval::ops::Range;
use num::traits::Num;
use std::fmt::{Debug, Formatter, Result};
use concept::*;

pub struct BooleanNeg<VStore> {
  b: Boolean<VStore>
}

impl<VStore> PropagatorKind for BooleanNeg<VStore> {}

impl<VStore, Domain, Bound> BooleanNeg<VStore> where
 VStore: VStoreConcept<Item=Domain>,
 Domain: Range + Collection<Item=Bound> + Clone + Debug + 'static,
 Bound: Num
{
  pub fn new(b: Boolean<VStore>) -> Self {
    BooleanNeg {
      b: b
    }
  }
}

impl<VStore> Debug for BooleanNeg<VStore>
{
  fn fmt(&self, fmt: &mut Formatter) -> Result {
    fmt.debug_struct("BooleanNeg")
      .field("b", &self.b)
      .finish()
  }
}

impl<VStore> Clone for BooleanNeg<VStore> where
 VStore: Collection
{
  fn clone(&self) -> Self {
    BooleanNeg {
      b: self.b.clone()
    }
  }
}

impl<VStore> DisplayStateful<Model> for BooleanNeg<VStore>
{
  fn display(&self, model: &Model) {
    print!("not ");
    self.b.display(model);
  }
}

impl<VStore, Dom, Bound> Subsumption<VStore> for BooleanNeg<VStore> where
  VStore: Collection<Item=Dom>,
  Dom: Bounded<Item=Bound> + IsSingleton,
  Bound: Num
{
  fn is_subsumed(&self, vstore: &VStore) -> Trilean {
    !self.b.is_subsumed(vstore)
  }
}

impl<VStore, Dom, Bound> Propagator<VStore> for BooleanNeg<VStore> where
  VStore: VStoreConcept<Item=Dom>,
  Dom: Collection<Item=Bound> + Singleton,
  Bound: Num
{
  fn propagate(&mut self, vstore: &mut VStore) -> bool {
    self.b.update(vstore, Dom::singleton(Bound::zero()))
  }
}

impl<VStore> PropagatorDependencies<FDEvent> for BooleanNeg<VStore>
{
  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    PropagatorDependencies::dependencies(&self.b)
  }
}
