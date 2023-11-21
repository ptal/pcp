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

pub mod boolean;
pub mod boolean_neg;
pub mod conjunction;
pub mod disjunction;
pub mod ops;

pub use logic::boolean::*;
pub use logic::boolean_neg::*;
pub use logic::conjunction::*;
pub use logic::disjunction::*;
pub use logic::ops::*;

use concept::*;
use gcollections::*;

pub fn implication<VStore>(f: Formula<VStore>, g: Formula<VStore>) -> Formula<VStore>
where
    VStore: Collection + 'static,
{
    Box::new(Disjunction::new(vec![f, g.not()]))
}

pub fn equivalence<VStore>(f: Formula<VStore>, g: Formula<VStore>) -> Formula<VStore>
where
    VStore: Collection + 'static,
{
    Box::new(Conjunction::new(vec![
        implication(f.bclone(), g.bclone()),
        implication(g, f),
    ]))
}
