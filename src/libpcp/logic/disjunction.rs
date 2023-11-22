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

use concept::*;
use gcollections::kind::*;
use kernel::*;
use logic::{Conjunction, NotFormula};
use model::*;
use propagation::events::*;
use propagation::*;
use std::fmt::{Debug, Formatter, Result};
use trilean::SKleene;

pub struct Disjunction<VStore> {
    fs: Vec<Formula<VStore>>,
}

impl<VStore> Disjunction<VStore> {
    pub fn new(fs: Vec<Formula<VStore>>) -> Self {
        Disjunction { fs }
    }
}

impl<VStore> Debug for Disjunction<VStore> {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        fmt.debug_struct("Disjunction")
            .field("fs", &self.fs)
            .finish()
    }
}

impl<VStore> Clone for Disjunction<VStore>
where
    VStore: Collection,
{
    fn clone(&self) -> Self {
        Disjunction {
            fs: self.fs.iter().map(|f| f.bclone()).collect(),
        }
    }
}

impl<VStore> DisplayStateful<Model> for Disjunction<VStore> {
    fn display(&self, model: &Model) {
        let mut i = 0;
        while i < self.fs.len() - 1 {
            self.fs[i].display(model);
            print!(" \\/ ");
            i += 1;
        }
        self.fs[i].display(model);
    }
}

impl<VStore> NotFormula<VStore> for Disjunction<VStore>
where
    VStore: Collection + 'static,
{
    /// Apply De Morgan's laws.
    fn not(&self) -> Formula<VStore> {
        let fs = self.fs.iter().map(|f| f.not()).collect();
        Box::new(Conjunction::new(fs))
    }
}

impl<VStore> Subsumption<VStore> for Disjunction<VStore> {
    fn is_subsumed(&self, vstore: &VStore) -> SKleene {
        use trilean::SKleene::*;
        let mut all_disentailed = true;
        for f in &self.fs {
            match f.is_subsumed(vstore) {
                True => return True,
                Unknown => all_disentailed = false,
                _ => (),
            }
        }
        if all_disentailed {
            False
        } else {
            Unknown
        }
    }
}

impl<VStore> Propagator<VStore> for Disjunction<VStore> {
    fn propagate(&mut self, vstore: &mut VStore) -> bool {
        use trilean::SKleene::*;
        let mut num_disentailed = 0;
        let mut unknown_formula = 0;
        for (i, f) in self.fs.iter().enumerate() {
            match f.is_subsumed(vstore) {
                True => return true,
                False => num_disentailed += 1,
                Unknown => unknown_formula = i,
            }
        }
        if num_disentailed == self.fs.len() - 1 {
            self.fs[unknown_formula].propagate(vstore)
        } else if num_disentailed == self.fs.len() {
            false
        } else {
            true
        }
    }
}

impl<VStore> PropagatorDependencies<FDEvent> for Disjunction<VStore> {
    fn dependencies(&self) -> Vec<(usize, FDEvent)> {
        let mut deps: Vec<_> = self
            .fs
            .iter()
            .map(|f| f.dependencies())
            .flat_map(|deps| deps.into_iter())
            .collect();
        deps.sort();
        deps.dedup();
        deps
    }
}
