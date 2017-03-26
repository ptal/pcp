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
// mod robot3;

use nqueens::nqueens;


fn main() {
//  nqueens(100);
//  let robot = robot::RobotScheduling::new(3, 500).solve();
//  println!("{}", robot);

// !!! le solveur plante avec une erreur 'index out of bounds sur un vecteur
   println!("Solve robot2 domaine 400");
   let test1 = robot2::RobotScheduling::new_test1(2, 400, 10, 25, 240, 25).solve();
   println!("{}", test1);

//   //ok bonne réponses. Le wait est bien intercallé
   println!("Solve robot2 domaine 400");
   let test2 = robot2::RobotScheduling::new_test2(2, 400, 10, 25, 240, 25, 5).solve();
   println!("{}", test2);

// // !!! Non satisfaisable alors que robot 2 l'est avec les même données.
   println!("Solve robot1 domaine 34");
   let test1 = robot2::RobotScheduling::new_test1(2, 34, 1, 3, 24, 3).solve(); // dure et ne rend pas le mains
   println!("{}", test1);

//   // !!! dure et ne rend pas le mains pourtant le problème est plus simple qu'avec le robot2 après
   println!("Solve robot1 domaine 38");
   let test1 = robot2::RobotScheduling::new_test1(2, 38, 1, 3, 24, 3).solve();
   println!("{}", test1);

//   //Ok bonne solution
   println!("Solve robot2 domaine 38");
   let test2 = robot2::RobotScheduling::new_test2(2, 38, 1, 3, 24, 3, 1).solve();
   println!("{}", test2);


  //erreur durant la résolution.
//  let robot3 = robot3::RobotScheduling::new(2, 40, 1, 3, 24, 3).solve();
//  println!("{}", robot3);

}
