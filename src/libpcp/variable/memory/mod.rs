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

pub mod copy;
pub mod trailed;

pub use variable::memory::copy::*;
pub use variable::memory::trailed::*;

#[cfg(test)]
mod test {
  use super::*;
  use kernel::*;
  use variable::test::DomainI32;
  use variable::concept::*;
  use variable::store::*;
  use variable::ops::*;
  use propagation::events::*;
  use interval::interval::*;
  use gcollections::ops::*;

  pub type StoreI32<M> = Store<M, DomainI32, FDEvent>;

  #[test]
  fn identity_restoration() {
    type MCopy = CopyMemory<DomainI32>;
    type MTrailed = TrailedStore<DomainI32>;
    identity_restoration_mem::<MCopy>();
    identity_restoration_mem::<MTrailed>();
  }

  fn identity_restoration_mem<M>() where
   M: MemoryConcept<DomainI32>
  {
    let domains_set = vec![
      vec![],
      vec![(0,0)],
      vec![(0,10), (5,5)]
    ];

    for domains in domains_set {
      let doms: Vec<_> = domains.into_iter()
        .map(|d| d.to_interval())
        .collect();
      for i in 1..4 {
        identity_restoration_n::<M>(i, doms.clone());
      }
    }
  }

  fn identity_restoration_n<M>(num_labels: usize, domains: Vec<DomainI32>) where
   M: MemoryConcept<DomainI32>
  {
    let mut store: StoreI32<M> = Store::empty();

    for dom in domains.clone() {
      store.alloc(dom);
    }

    let mut frozen = store.freeze();
    let labels: Vec<_> = (0..num_labels)
      .map(|_| frozen.label())
      .collect();

    for label in labels {
      store = frozen.restore(label);

      let store_doms: Vec<_> = store.iter().cloned().collect();
      assert_eq!(domains, store_doms);

      frozen = store.freeze();
    }
  }

}
