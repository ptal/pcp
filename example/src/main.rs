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
mod robot2;
mod robot3;

use nqueens::nqueens;


fn main() {
  // nqueens(60);
  let robot = robot::RobotScheduling::new(2, 500).solve();
  println!("{}", robot);

// test en changeant le domaine pour voir la diff de perf.
  let robot2 = robot2::RobotScheduling::new(2, 400, 10, 25, 240, 25, 5).solve();
  println!("{}", robot2);
// toutes les taches sont à la suites. Pendant le Wait du robot 1 le robot 2 attend alors qu'il peut faire le Put.
  let robot2 = robot2::RobotScheduling::new(2, 33, 1, 3, 24, 3, 1).solve();
  println!("{}", robot2);
//Quand la contrainte de fin est forcée, le robot s'exéxute pendant le Wait. Bonne solution.
  let robot2 = robot2::RobotScheduling::new(2, 32, 1, 3, 24, 3, 1).solve();
  println!("{}", robot2);

//simplification du problem avec des durées fixes
//J'ai m'impression qu'il y a un pb dans les contraintes cumulatives:
//  assigned: 
//  not assigned: r1s1   = [1..33]    r1s2   = [1..33]    r1s3   = [1..33]    r1s4   = [1..33]    r1s5   = [1..33]    t0_0   = [0..1]     t1_0   = [0..1]     
// r1s1 + 1 < r1s2 /\ r1s2 + 3 < r1s3 /\ r1s3 + 24 < r1s4 /\ r1s4 + 3 < r1s5 /\ r1s1 = 1 /\ 
// Contrainte cumulative: t0_0 = bool2int(r1s4 < r1s1 + 1 /\ r1s1 < r1s4 + r1s4)(<- bizarre ??) * 1 /\ 1 + 1 > 1 + sum(t0_0) /\ t1_0 = bool2int(r1s1 < r1s4 + 1 /\ r1s4 < r1s1 + r1s2) * 1 /\ 1 + 1 > 1 + sum(t1_0).
  let robot3 = robot3::RobotScheduling::new(1, 40, 1, 3, 24, 3).solve();
  println!("{}", robot3);

  //erreur durant la résolution.
  let robot3 = robot3::RobotScheduling::new(2, 40, 1, 3, 24, 3).solve();
  println!("{}", robot3);

}
