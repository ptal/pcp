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

//! Constraint programming is a declarative programming paradigm mainly used to solve combinatorial problems where you state which constraints a solution must respect instead of explaining how to solve it.

#![crate_name = "pcp"]

#![feature(alloc, unboxed_closures, box_syntax, fn_traits, fnbox)]
#![feature(test)]

extern crate test;
extern crate interval;
extern crate gcollections;
extern crate num;
extern crate alloc;
extern crate vec_map;

pub mod kernel;
pub mod propagation;
pub mod propagators;
pub mod term;
pub mod variable;
pub mod search;
pub mod concept;
