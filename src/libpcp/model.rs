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

use concept::*;
use term::identity::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Model {
  pub groups: Vec<(String, usize)>,
  pub var_names: HashMap<usize, String>
}

impl Model
{
  pub fn new() -> Self {
    Model {
      groups: vec![],
      var_names: HashMap::new()
    }
  }

  pub fn open_group(&mut self, name: &str) {
    self.groups.push((String::from(name), 1));
  }

  pub fn close_group(&mut self) {
    self.groups.pop().expect("Cannot close a non-opened group.");
  }

  pub fn alloc_var<Domain, VStore>(&mut self,
    vstore: &mut VStore, dom: VStore::Item) -> VStore::Location where
   VStore: IntVStore<Item=Domain, Location=Identity<Domain>>
  {
    let loc = vstore.alloc(dom);
    let name = self.make_name();
    self.register_var(loc.index(), name);
    loc
  }

  pub fn register_var(&mut self, var: usize, name: String) {
    self.var_names.insert(var, name);
  }

  pub fn var_name(&self, idx: usize) -> String {
    match self.var_names.get(&idx) {
      Some(name) => name.clone(),
      None => format!("_{}", idx)
    }
  }

  pub fn inc_group(&mut self) {
    self.groups.last_mut()
      .expect("Open a group with `open_group` before allocating a variable.").1 += 1;
  }

  fn make_name(&mut self) -> String {
    let mut name = String::new();
    for (g, i) in self.groups.clone() {
      name.push_str(&format!("{}{}", g, i));
    }
    self.inc_group();
    name
  }
}
