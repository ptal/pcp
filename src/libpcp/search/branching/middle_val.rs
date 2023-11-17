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
use search::branching::*;

pub struct MiddleVal;

impl<Domain, Bound> ValSelection<Domain> for MiddleVal
where
    Domain: IntDomain<Item = Bound>,
    Bound: IntBound,
{
    fn select(&mut self, dom: Domain) -> Bound {
        (dom.lower() + dom.upper()) / (Bound::one() + Bound::one())
    }
}
