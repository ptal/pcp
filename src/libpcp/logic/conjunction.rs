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
use logic::{Disjunction, NotFormula};
use model::*;
use propagation::events::*;
use propagation::*;
use std::fmt::{Debug, Formatter, Result};
use trilean::SKleene;

pub struct Conjunction<VStore> {
    fs: Vec<Formula<VStore>>,
}

impl<VStore> Conjunction<VStore> {
    pub fn new(fs: Vec<Formula<VStore>>) -> Self {
        Conjunction { fs }
    }
}

impl<VStore> Debug for Conjunction<VStore> {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        fmt.debug_struct("Conjunction")
            .field("fs", &self.fs)
            .finish()
    }
}

impl<VStore> Clone for Conjunction<VStore>
where
    VStore: Collection,
{
    fn clone(&self) -> Self {
        Conjunction {
            fs: self.fs.iter().map(|f| f.bclone()).collect(),
        }
    }
}

impl<VStore> DisplayStateful<Model> for Conjunction<VStore> {
    fn display(&self, model: &Model) {
        let mut i = 0;
        while i < self.fs.len() - 1 {
            self.fs[i].display(model);
            print!(" /\\ ");
            i += 1;
        }
        self.fs[i].display(model);
    }
}

impl<VStore> NotFormula<VStore> for Conjunction<VStore>
where
    VStore: Collection + 'static,
{
    /// Apply De Morgan's laws.
    fn not(&self) -> Formula<VStore> {
        let fs = self.fs.iter().map(|f| f.not()).collect();
        Box::new(Disjunction::new(fs))
    }
}

impl<VStore> Subsumption<VStore> for Conjunction<VStore> {
    fn is_subsumed(&self, store: &VStore) -> SKleene {
        use trilean::SKleene::*;
        let mut all_entailed = true;
        for f in &self.fs {
            match f.is_subsumed(store) {
                False => return False,
                Unknown => all_entailed = false,
                _ => (),
            }
        }
        if all_entailed {
            True
        } else {
            Unknown
        }
    }
}

impl<VStore> Propagator<VStore> for Conjunction<VStore> {
    fn propagate(&mut self, store: &mut VStore) -> bool {
        for f in &mut self.fs {
            if !f.propagate(store) {
                return false;
            }
        }
        true
    }
}

impl<VStore> PropagatorDependencies<FDEvent> for Conjunction<VStore> {
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
