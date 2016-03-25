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

use variable::memory::ops::*;
use std::ops::{DerefMut, Deref};
use std::rc::*;

pub struct CopyStore<Domain>
{
  variables: Vec<Domain>
}

impl<Domain> CopyStore<Domain>
{
  pub fn new() -> CopyStore<Domain> {
    CopyStore {
      variables: vec![]
    }
  }

  fn restore(variables: Vec<Domain>) -> CopyStore<Domain> {
    CopyStore {
      variables: variables
    }
  }
}

impl<Domain> Freeze for CopyStore<Domain> where
 Domain: Clone
{
  type FrozenState = FrozenCopyStore<Domain>;
  fn freeze(self) -> FrozenCopyStore<Domain>
  {
    FrozenCopyStore::new(self)
  }
}

pub struct FrozenCopyStore<Domain>
{
  variables: Rc<Vec<Domain>>
}

impl<Domain> FrozenCopyStore<Domain>
{
  fn new(store: CopyStore<Domain>) -> FrozenCopyStore<Domain> {
    FrozenCopyStore {
      variables: Rc::new(store.variables)
    }
  }
}

impl<Domain> Snapshot for FrozenCopyStore<Domain> where
 Domain: Clone
{
  type Label = Rc<Vec<Domain>>;
  type UnfrozenState = CopyStore<Domain>;

  fn snapshot(&mut self) -> Rc<Vec<Domain>> {
    self.variables.clone()
  }

  fn restore(self, label: Rc<Vec<Domain>>) -> CopyStore<Domain> {
    let variables = Rc::try_unwrap(label).unwrap_or_else(|l| l.deref().clone());
    CopyStore::restore(variables)
  }
}

// #[cfg(test)]
// mod test {
//   use super::*;
//   use variable::memory::ops::*;
//   use std::ops::Deref;

//   fn make_frozen_store(initial_data: Vec<u32>) -> FrozenCopyStore<u32> {
//     let mut copy_store = CopyStore::new();
//     copy_store.extend(initial_data);
//     copy_store.freeze()
//   }

//   #[test]
//   fn save_and_restore_identity() {
//     let initial_data = vec![1,2,3];
//     let mut frozen = make_frozen_store(initial_data.clone());
//     let snapshots: Vec<_> = (1..10).map(|_| frozen.snapshot()).collect();

//     for snapshot in snapshots {
//       let copy_store = frozen.restore(snapshot);
//       assert_eq!(initial_data.clone(), copy_store.deref().clone());
//       frozen = copy_store.freeze();
//     }
//   }
// }
