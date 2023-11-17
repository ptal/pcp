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

use variable::memory::trail::memory_cell::MemoryCell;

use gcollections::kind::*;
use gcollections::ops::*;
use std::fmt::{Display, Error, Formatter};
use variable::memory::ops::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SingleValueTrail<Domain> {
    trail: Vec<MemoryCell<Domain>>,
}

impl<Domain> Collection for SingleValueTrail<Domain> {
    type Item = Domain;
}

impl<Domain> AssociativeCollection for SingleValueTrail<Domain> {
    type Location = usize;
}

impl<Domain> TrailVariable for SingleValueTrail<Domain> {
    fn trail_variable(&mut self, loc: usize, value: Domain) {
        self.trail.push(MemoryCell::new(loc, value))
    }
}

impl<Domain> TrailRestoration for SingleValueTrail<Domain> {
    type Mark = usize;

    fn mark(&mut self) -> Self::Mark {
        self.trail.len()
    }

    fn undo(&mut self, mark: Self::Mark, memory: &mut Vec<Domain>) {
        while self.trail.len() != mark {
            let cell = self.trail.pop().unwrap();
            memory[cell.location] = cell.value;
        }
    }
}

impl<Domain> Display for SingleValueTrail<Domain>
where
    Domain: Display,
{
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        for cell in &self.trail {
            formatter.write_str(format!("{}\n", cell).as_str())?;
        }
        Ok(())
    }
}

impl<Domain> Empty for SingleValueTrail<Domain> {
    fn empty() -> SingleValueTrail<Domain> {
        SingleValueTrail { trail: vec![] }
    }
}
