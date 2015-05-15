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

use variable::ops::*;
use variable::arithmetics::identity::*;

pub struct Store<Domain> {
  variables: Vec<Domain>
}

impl<Domain> Store<Domain>
{
  pub fn new() -> Store<Domain> {
    Store {
      variables: vec![]
    }
  }
}

impl<Domain> Assign<Domain> for Store<Domain> where
  Domain: VarDomain
{
  type Variable = Identity<Domain>;

  fn assign(&mut self, dom: Domain) -> Identity<Domain> {
    assert!(!dom.is_empty());
    let var_idx = self.variables.len();
    self.variables.push(dom);
    Identity::new(var_idx)
  }
}

impl<Domain> MonotonicUpdate<usize, Domain> for Store<Domain> where
  Domain: VarDomain
{
  fn update(&mut self, key: usize, dom: Domain) -> bool {
    assert!(dom.is_subset(&self.variables[key]), "Domain update must be monotonic.");
    if dom.is_empty() {
      false
    }
    else {
      self.variables[key] = dom;
      true
    }
  }
}

impl<Domain> Read<usize> for Store<Domain> where
  Domain: Clone
{
  type Value = Domain;
  fn read(&self, key: usize) -> Domain {
    assert!(key < self.variables.len(), "Variable not registered in the store. Variable index must be obtained with `assign`.");
    self.variables[key].clone()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use variable::ops::*;
  use variable::arithmetics::identity::*;
  use interval::interval::*;
  use interval::ncollections::ops::*;

  #[test]
  fn ordered_assign_10_vars() {
    let dom0_10 = (0, 10).to_interval();
    let mut store = Store::new();

    for i in 0..10 {
      assert_eq!(store.assign(dom0_10), Identity::new(i));
    }
  }

  #[test]
  fn valid_read_update() {
    let dom0_10 = (0, 10).to_interval();
    let dom5_5 = (5, 5).to_interval();
    let mut store = Store::new();

    let vars: Vec<_> = (0..10).map(|_| store.assign(dom0_10)).collect();
    for var in vars {
      assert_eq!(var.read(&store), dom0_10);
      assert_eq!(var.update(&mut store, dom5_5), true);
      assert_eq!(var.read(&store), dom5_5);
    }
  }

  #[test]
  fn empty_update() {
    let mut store = Store::new();
    let dom5_5 = (5, 5).to_interval();

    let var = store.assign(dom5_5);
    assert_eq!(var.update(&mut store, Interval::empty()), false);
  }

  #[test]
  #[should_panic]
  fn empty_assign() {
    let mut store = Store::new();
    store.assign(Interval::<i32>::empty());
  }

  #[test]
  #[should_panic]
  fn non_monotonic_update_singleton() {
    let dom0_10 = (0,10).to_interval();
    let dom11_11 = 11.to_interval();

    let mut store = Store::new();
    let var = store.assign(dom0_10);
    var.update(&mut store, dom11_11);
  }

  #[test]
  #[should_panic]
  fn non_monotonic_update_widen() {
    let dom0_10 = (0,10).to_interval();
    let domm5_15 = (-5, 15).to_interval();

    let mut store = Store::new();
    let var = store.assign(dom0_10);
    var.update(&mut store, domm5_15);
  }
}