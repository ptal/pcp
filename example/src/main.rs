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
// to test : cargo test --features="nightly" -- --nocapture
// to bench: cargo bench --features="nightly"

#![cfg_attr(feature = "nightly", feature(test))]

extern crate pcp;
extern crate interval;
extern crate gcollections;

#[cfg(feature = "nightly")] extern crate test;

mod nqueens;
mod robot;
mod robot2;
// mod robot3;

use nqueens::nqueens;

fn main() {
   nqueens(8);
//  let robot = robot::RobotScheduling::new(3, 500).solve();
//  println!("{}", robot);

// !!! Ok with 2 robot. doesn't stop with 3 robot.
   println!("Solve robot2 domaine 400");
   let test1 = robot2::RobotScheduling::new_test1(2, 400, 10, 25, 240, 25).solve();
   println!("{}", test1);

//   //ok bonne réponses. Le wait est bien intercallé
   println!("Solve robot2 domaine 400");
   let test2 = robot2::RobotScheduling::new_test2(2, 400, 10, 25, 240, 25, 5).solve();
   println!("{}", test2);

// Non satisfi
   println!("Solve robot1 domaine 36");
   let test1 = robot2::RobotScheduling::new_test1(2, 37, 1, 3, 24, 3).solve();  //non satisfauisable
   println!("{}", test1);

//   //Ok bonne solution
   println!("Solve robot2 domaine 38");
   let test2 = robot2::RobotScheduling::new_test2(2, 38, 1, 3, 24, 3, 1).solve(); //durée des varibales est une variables.
   println!("{}", test2);

   println!("Solve robot1 domaine 38");
   let test1 = robot2::RobotScheduling::new_test1(2, 38, 1, 3, 24, 3).solve(); //ne rend pas la main trop long alors que le problème
   //est plus simple que test précédent. Dans cette version la durée des taches est fixée
   println!("{}", test1);


  //erreur durant la résolution.
//  let robot3 = robot3::RobotScheduling::new(2, 40, 1, 3, 24, 3).solve();
//  println!("{}", robot3);

}
