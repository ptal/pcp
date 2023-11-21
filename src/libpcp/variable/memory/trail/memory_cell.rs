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

use std::fmt::{Display, Error, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryCell<Domain> {
    pub location: usize,
    pub value: Domain,
}

impl<Domain> MemoryCell<Domain> {
    pub fn new(location: usize, value: Domain) -> MemoryCell<Domain> {
        MemoryCell { location, value }
    }
}

impl<Domain> Display for MemoryCell<Domain>
where
    Domain: Display,
{
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        formatter.write_str(format!("{}: {}", self.location, self.value).as_str())
    }
}
