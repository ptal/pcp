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

use kernel::event::*;

pub trait Reactor {
    fn new(num_vars: usize, num_events: usize) -> Self;
    fn subscribe<E>(&mut self, var: usize, ev: E, prop: usize)
    where
        E: EventIndex;
    fn unsubscribe<E>(&mut self, var: usize, ev: E, prop: usize)
    where
        E: EventIndex;
    fn react<E>(&self, var: usize, ev: E) -> Vec<usize>
    where
        E: EventIndex;
}
