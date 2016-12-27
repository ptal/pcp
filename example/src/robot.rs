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

// this is en example of scheduling several robot that do the same tasks.
// schedule num_robot to do some tasks. //
// Each robot with take something, wait some time, put the thing somewhere and go to park.
// Robot task : Loading (L), Take (T), Wait (W), Put (P), go to park end (E)
// Contraint for one robot :
// for each task start and duraton variable are defined
// start task are in domaine time[0, max_time]
//constraint:
// Ls = 0 //loading first robot start at 0.
// Ld element [10, 15] //loading take from 10 to 15 second
// Ts > Ls + Ld // take start after that loading has finished (Ls + Ld)
// Td element [25,35]
// Ws > Ts + Td
// Wd element [100, 250]
// P > Ws + Wd
// Pd element [25, 35]
// Es > Ps + Pd
// Es end before max_time.
//
//cumulative constraints
//add cumulative constraints
//Two ressouces with capacity of one:
//    * Pipeting unit use by Loading and Put operations
//    * Mixing unit use by Take and Put operations


#![allow(unused_variables, non_snake_case)]

extern crate test;

//use env_logger;
use pcp::kernel::*;
use pcp::propagators::*;
use pcp::variable::ops::*;
use pcp::search::search_tree_visitor::Status::*;
use pcp::search::*;
use interval::interval::*;
use interval::ops::Range;
use gcollections::ops::*;
use pcp::term::constant::Constant;
use pcp::term::Addition;
use pcp::propagators::cumulative::Cumulative;

//build the contraint for num_robot with max-time indicating that all robot must finish before.
pub fn build_store(num_robot: usize, max_time: usize) -> FDSpace {
  let TASKS = 5;
  let L = 0; // Loading
  let T = 1; // Take
  let W = 2; // Wait
  let P = 3; // Put
  let E = 4; // End, go to park

  let mut space = FDSpace::empty();
  let mut start = vec![];
  let mut duration = vec![];

  let time_dom = (1, max_time as i32).to_interval();

  let mut pipeting_start = vec!();
  let mut pipeting_duration = vec!();
  let mut pipeting_resource = vec!();

  let DUR: Vec<Interval<i32>> = vec![
    (10, 15),   // Loading L duration between 10 and 15
    (25, 35),   // Take T duration between 25 and 35
    (240, 250), // Wait W duration between 100 and 250
    (25, 35)    // Put P duration between 25 and 35
  ].into_iter().map(|d| d.to_interval()).collect();

  for i in 0..num_robot {
    // Start date for the different tasks.
    for _ in 0..TASKS {
      start.push(space.vstore.alloc(time_dom));
    }
    for t in 0..TASKS-1 {
      duration.push(space.vstore.alloc(DUR[t]));
    }
    pipeting_start.push(box start[i * TASKS + L]);
    pipeting_start.push(box start[i * TASKS + W]);

    pipeting_duration.push(box duration[i * 4 + L]);
    pipeting_duration.push(box duration[i * 4 + W]);

    pipeting_resource.push(box space.vstore.alloc(Interval::singleton(1)));
    pipeting_resource.push(box space.vstore.alloc(Interval::singleton(1)));

    // Ensure that every task starts after the end time of the previous task. (S' > S + D).
    for t in 0..TASKS-1 {
      space.cstore.alloc(
        box XGreaterYPlusZ::new(start[i * TASKS + t + 1], start[i * TASKS + t], duration[i * 4 + t]));
    }
  }
  // Ls = 0 for the first robot to force it to start first
  space.cstore.alloc(box XEqY::new(start[0], Constant::new(1)));

  // for i in 0..num_robot*2 {
  //   pipeting_resource.push(box Constant::new(1));
  // }

  // let cumulative_pipeting = Cumulative::new(
  //   pipeting_start,
  //   pipeting_duration,
  //   pipeting_resource,
  //   box space.vstore.alloc(Interval::new(1,1))
  // );
  // cumulative_pipeting.join(&mut space.vstore, &mut space.cstore);
  space
}

pub fn solve_schedule(num_robot: usize, space: FDSpace, show_trace: bool) {
  // Search step.
  let mut search = one_solution_engine();
  search.start(&space);
  let (space, status) = search.enter(space);
  let space = space.unfreeze();

  if show_trace {
    // Print result.
    match status {
    Satisfiable => {
      //result are formated has follow foe each robot:
      // Start_L, Duration_L, Pipeting_Cumum_Res_L, Start_T, Duration_T, Start_W, Duration_W, Start_P, Duration_P, Pipeting_Cumum_Res_P, Start_E
      let mut start_list = vec!();
      let mut duration_list = vec!();

      let mut iter = space.vstore.iter();

      for _ in 0 .. num_robot {
        start_list.push(iter.next().unwrap().clone());
        duration_list.push(iter.next().unwrap().clone());
        iter.next(); //skip resource
        start_list.push(iter.next().unwrap().clone());
        duration_list.push(iter.next().unwrap().clone());
        start_list.push(iter.next().unwrap().clone());
        duration_list.push(iter.next().unwrap().clone());
        start_list.push(iter.next().unwrap().clone());
        duration_list.push(iter.next().unwrap().clone());
        iter.next(); //skip resource
        start_list.push(iter.next().unwrap().clone());
      }


      println!("{}-robot scheduling is satisfiable. The first solution is:", num_robot);
      println!("task              : L,  T ,  W  , P , E ");
      let mut robot = 1;
      print!("start time robot {}: ", robot);
      let mut num = 0;
      for start in start_list.iter() {
        // At this stage, dom.lower() == dom.upper().
        print!("{}, ", start.lower());
        num+=1;
        if num == 5 {
          println!("");
          robot += 1;
            print!("start time robot {}: ", robot);
          num=0;
        }
      }
      println!("");
      robot = 1;
      print!("duration robot {}: ", robot);
      let mut num = 0;
      for dur in duration_list.iter() {
        // At this stage, dom.lower() == dom.upper().
        print!("{}, ", dur.lower());
        num+=1;
        if num == 4 {
          println!("");
          robot += 1;
          print!("duration robot {}: ", robot);
          num=0;
        }
      }
      println!("");
    }
    Unsatisfiable => println!("{}-analysis problem is unsatisfiable.", num_robot),
    EndOfSearch => println!("Search terminated or was interrupted."),
    Unknown(_) => unreachable!(
      "After the search step, the problem instance should be either satisfiable or unsatisfiable.")
    }
  }
}

 #[cfg(test)]
mod tests {
  use test::Bencher;

  // #[test]
  // fn test_shedule() {
  //     super::solve_schedule(4, super::build_store(4, 500), true);
  // }

  #[bench]
  fn bench_schedule_2(b: &mut Bencher) {
    b.iter(|| super::solve_schedule(2, super::build_store(2, 500), false));
  }

  // #[bench]
  // fn bench_schedule_4(b: &mut Bencher) {
  //   b.iter(|| super::solve_schedule(4, super::build_store(4, 500), false));
  // }

  // #[bench]
  // fn bench_schedule_8(b: &mut Bencher) {
  //   b.iter(|| super::solve_schedule(8, super::build_store(8, 500), false));
  // }

  // #[bench]
  // fn bench_schedule_16(b: &mut Bencher) {
  //   b.iter(|| super::solve_schedule(16, super::build_store(16, 500), false));
  // }
}
