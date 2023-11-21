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

use gcollections::kind::*;

pub trait TrailVariable: AssociativeCollection {
    fn trail_variable(&mut self, loc: Self::Location, value: Self::Item);
}

pub trait TrailRestoration: Collection {
    type Mark;
    fn commit(&mut self) {}
    fn mark(&mut self) -> Self::Mark;
    fn undo(&mut self, mark: Self::Mark, memory: &mut Vec<Self::Item>);
}
