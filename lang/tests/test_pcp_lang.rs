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



#[test]
fn test_nqueens()
{
  let n = 10usize;
  pcp! {
    let mut queens: Vec<usize> = vec![];
    for _ in 0..n {
      assert!(true);
      queens.push(n);
      // queens.push(#{1 .. n});
    }
    for i in 0..n-1 {
      for j in i + 1..n {
        let queen_i = queens[i];
        let queen_j = queens[j];
        #{
          queen_i + i != queen_j + j;
          queen_i - i != queen_j - j;
        }
      }
    }
    #{Distinct(queens)}
  }


  assert!(true);
}
