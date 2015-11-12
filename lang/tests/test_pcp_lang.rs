// Copyright 2014 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(plugin)]
#![plugin(pcp_lang)]

extern crate interval;
extern crate pcp;

#[cfg(test)]
mod test
{
  use interval::interval::*;
  use interval::ops::*;
  use pcp::propagation::events::*;
  use pcp::propagation::reactors::*;
  use pcp::propagation::schedulers::*;
  use pcp::propagation::store::*;
  use pcp::variable::delta_store::DeltaStore;
  use pcp::kernel::*;

  type VStore = DeltaStore<Interval<i32>, FDEvent>;
  type CStore = Store<VStore, FDEvent, IndexedDeps, RelaxedFifo>;

  #[test]
  fn test_nqueens()
  {
    pcp! {
      let mut variables: VStore = VStore::new();
      let mut constraints: CStore = CStore::new();
      let n = 10usize;
      let mut queens = vec![];
      for _ in 0..n {
        let n: i32 = n as i32;
        queens.push(#(variables <- 0..n));
      }
      for i in 0..n-1 {
        for j in i + 1..n {
          let queen_i = queens[i];
          let queen_j = queens[j];
          // #{
          //   constraints <- queen_i + i != queen_j + j;
          //   constraints <- queen_i - i != queen_j - j;
          // }
        }
      }
      // #{Distinct(queens)}
    }
    assert!(true);
  }
}
