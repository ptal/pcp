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

// This example example of PCP use.
//
// to run all example : cargo run.
// to test : cargo test -- --nocapture
// to bench: cargo bench

#![feature(test, box_syntax)]
extern crate pcp;
extern crate interval;
extern crate gcollections;
extern crate test;

mod nqueens;
mod robot;

use nqueens::nqueens;

fn main() {
  // nqueens(4);
  robot::solve_schedule(2, robot::build_store(2, 500), true);
}
