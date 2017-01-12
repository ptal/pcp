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
use model::*;
use num::Integer;
use term::ops::*;
use propagation::ops::*;
use gcollections::ops::*;
use interval::ops::Range;
use concept::*;

/// bool2int(c), read-only view.
#[derive(Clone, Debug)]
pub struct Bool2Int<P>
{
  p: P
}

impl<P> Bool2Int<P> {
  pub fn new(p: P) -> Self {
    Bool2Int {
      p: p
    }
  }
}

impl<P> DisplayStateful<Model> for Bool2Int<P> where
 P: DisplayStateful<Model>
{
  fn display(&self, model: &Model) {
    print!("bool2int(");
    self.p.display(model);
    print!(")");
  }
}

impl<VStore, P> StoreMonotonicUpdate<VStore> for Bool2Int<P> where
 VStore: VStoreConcept
{
  fn update(&mut self, _store: &mut VStore, _value: VStore::Item) -> bool {
    panic!("Cannot update a bool2int view.");
  }
}

impl<VStore, Domain, Bound, P> StoreRead<VStore> for Bool2Int<P> where
 VStore: VStoreConcept<Item=Domain>,
 Domain: Bounded<Item=Bound> + Range + Singleton,
 Bound: Integer,
 P: Subsumption<VStore>
{
  fn read(&self, store: &VStore) -> Domain {
    use kernel::trilean::Trilean::*;
    match self.p.is_subsumed(store) {
      True => Domain::singleton(Bound::one()),
      False => Domain::singleton(Bound::zero()),
      Unknown => Domain::new(Bound::zero(), Bound::one())
    }
  }
}

impl<P, Event> ViewDependencies<Event> for Bool2Int<P> where
  P: PropagatorDependencies<Event>
{
  fn dependencies(&self, _event: Event) -> Vec<(usize, Event)> {
    self.p.dependencies()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use propagators::XEqY;
  use variable::VStoreFD;
  use interval::interval::*;

  type VStore = VStoreFD;

  fn bool2int_read_x_eq_y(narrow_x: Interval<i32>, narrow_y: Interval<i32>, expected: Interval<i32>) {
    let dom0_10 = (0,10).to_interval();
    let dom0_1 = (0,1).to_interval();
    let mut store = VStore::empty();
    let mut x = store.alloc(dom0_10);
    let mut y = store.alloc(dom0_10);

    // z = bool2int(x == y)
    let z = Bool2Int::new(XEqY::new(x, y));
    assert_eq!(z.read(&store), dom0_1);

    x.update(&mut store, narrow_x);
    y.update(&mut store, narrow_y);
    assert_eq!(z.read(&store), expected);
  }

  #[test]
  fn bool2int_read() {
    let dom10_10 = (10,10).to_interval();
    let dom9_9 = (9,9).to_interval();
    let dom9_10 = (9,10).to_interval();
    let dom0_0 = (0,0).to_interval();
    let dom1_1 = (1,1).to_interval();
    let dom0_1 = (0,1).to_interval();

    bool2int_read_x_eq_y(dom10_10, dom10_10, dom1_1);
    bool2int_read_x_eq_y(dom9_9, dom10_10, dom0_0);
    bool2int_read_x_eq_y(dom9_10, dom9_10, dom0_1);
  }
}
