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
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone)]
pub struct Model {
    pub groups: Vec<(String, usize)>,
    pub var_names: HashMap<usize, String>,
}

impl Model {
    pub fn new() -> Self {
        Model {
            groups: vec![],
            var_names: HashMap::new(),
        }
    }

    pub fn open_group(&mut self, name: &str) {
        self.groups.push((String::from(name), 1));
    }

    pub fn close_group(&mut self) {
        self.groups.pop().expect("Cannot close a non-opened group.");
    }

    pub fn alloc_var<VStore, Domain>(&mut self, vstore: &mut VStore, dom: Domain) -> Var<VStore>
    where
        VStore: VStoreConcept<Item = Domain>,
        Domain: Clone + Debug + 'static,
    {
        let name = self.make_name();
        self.alloc_var_with_name(vstore, dom, name)
    }

    pub fn alloc_var_with_name<VStore, Domain>(
        &mut self,
        vstore: &mut VStore,
        dom: Domain,
        name: String,
    ) -> Var<VStore>
    where
        VStore: VStoreConcept<Item = Domain>,
        Domain: Clone + Debug + 'static,
    {
        let loc = vstore.alloc(dom);
        self.register_var(loc.index(), name);
        Box::new(loc)
    }

    pub fn register_var(&mut self, var: usize, name: String) {
        self.var_names.insert(var, name);
    }

    pub fn var_name(&self, idx: usize) -> String {
        match self.var_names.get(&idx) {
            Some(name) => name.clone(),
            None => format!("_{}", idx),
        }
    }

    pub fn inc_group(&mut self) {
        self.groups
            .last_mut()
            .expect("Open a group with `open_group` before allocating a variable.")
            .1 += 1;
    }

    fn make_name(&mut self) -> String {
        let mut name = String::new();
        for (g, i) in self.groups.clone() {
            name.push_str(&format!("{}{}", g, i));
        }
        self.inc_group();
        name
    }

    pub fn display_global<VStore>(&self, name: &str, args: &Vec<Var<VStore>>) {
        print!("{}(", name);
        let mut i = 0;
        while i < args.len() - 1 {
            args[i].display(self);
            print!(", ");
            i += 1;
        }
        args[i].display(self);
        print!(") (decomposed)");
    }
}
