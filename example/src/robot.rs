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


#![allow(unused_variables, non_snake_case, dead_code)]

extern crate test;

//use env_logger;
use pcp::kernel::*;
use pcp::propagators::*;
use pcp::search::*;
use pcp::search::branching::*;
use pcp::search::engine::one_solution::*;
use pcp::search::debugger::*;
use pcp::search::propagation::*;
use gcollections::VectorStack;
use interval::interval_set::*;
use interval::ops::Range;
use gcollections::ops::*;
use pcp::term::*;
use pcp::term::ops::*;
use pcp::propagators::cumulative::Cumulative;
use std::fmt::{Formatter, Display, Error};

pub type Domain = IntervalSet<i32>;

pub struct RobotScheduling {
  pub num_robot: usize,
  pub max_time: usize,
  pub start: Vec<Identity<Domain>>,
  pub duration: Vec<Identity<Domain>>,
  pub pipeting_start: Vec<Box<Identity<Domain>>>,
  pub pipeting_duration: Vec<Box<Identity<Domain>>>,
  pub pipeting_resource: Vec<Box<Identity<Domain>>>,
  pub space: FDSpace,
  pub status: Status<FDSpace>,
}

static TASKS: usize = 5;
static L: usize = 0; // Loading
static T: usize = 1; // Take
static W: usize = 2; // Wait
static P: usize = 3; // Put
static E: usize = 4; // End, go to park

impl RobotScheduling
{
  pub fn new(num_robot: usize, max_time: usize) -> Self {
    let mut robot = RobotScheduling {
      num_robot: num_robot,
      max_time: max_time,
      start: vec![],
      duration: vec![],
      pipeting_start: vec![],
      pipeting_duration: vec![],
      pipeting_resource: vec![],
      space: FDSpace::empty(),
      status: Status::Unsatisfiable,
    };
    robot.initialize();
    robot
  }

  fn initialize(&mut self) {
    let one = IntervalSet::singleton(1);
    let time_dom = IntervalSet::new(1, self.max_time as i32);

    let DUR: Vec<Domain> = vec![
      (10, 15),   // Loading L duration between 10 and 15
      (25, 35),   // Take T duration between 25 and 35
      (240, 250), // Wait W duration between 100 and 250
      (25, 35)    // Put P duration between 25 and 35
    ].into_iter().map(|(b,e)| IntervalSet::new(b, e)).collect();


    // Start date for the different tasks.
    for i in 0..self.num_robot {
      for _ in 0..TASKS {
        self.start.push(self.space.vstore.alloc(time_dom.clone()));
      }
    }

    for i in 0..self.num_robot {
      for t in 0..TASKS-1 {
        self.duration.push(self.space.vstore.alloc(DUR[t].clone()));
      }
      self.pipeting_start.push(box self.start[i * TASKS + L].clone());
      self.pipeting_start.push(box self.start[i * TASKS + W].clone());

      self.pipeting_duration.push(box self.duration[i * 4 + L].clone());
      self.pipeting_duration.push(box self.duration[i * 4 + W].clone());

      self.pipeting_resource.push(box self.space.vstore.alloc(one.clone()));
      self.pipeting_resource.push(box self.space.vstore.alloc(one.clone()));

      // Ensure that every task starts after the end time of the previous task. (S' > S + D).
      for t in 0..TASKS-1 {
        self.space.cstore.alloc(
          box XGreaterYPlusZ::new(
            self.start[i * TASKS + t + 1].clone(),
            self.start[i * TASKS + t].clone(),
            self.duration[i * 4 + t].clone()));
      }
    }
    // Ls = 0 for the first robot to force it to start first
    self.space.cstore.alloc(box XEqY::new(self.start[0].clone(), Constant::new(1)));

    // for i in 0..num_robot*2 {
    //   self.pipeting_resource.push(box Constant::new(1));
    // }

    let cumulative_pipeting = Cumulative::new(
      self.pipeting_start.clone(),
      self.pipeting_duration.clone(),
      self.pipeting_resource.clone(),
      box self.space.vstore.alloc(one.clone())
    );
    cumulative_pipeting.join(&mut self.space.vstore, &mut self.space.cstore);
  }

  pub fn solve(mut self) -> Self {
    let search =
      OneSolution::<_, VectorStack<_>, FDSpace>::new(
      // Debugger::new(
      Propagation::new(
      Brancher::new(InputOrder, MinVal, Enumerate)));
    let mut search = Box::new(search);
    search.start(&self.space);
    let (frozen_space, status) = search.enter(self.space);
    self.space = frozen_space.unfreeze();
    self.status = status;
    self
  }

  fn start_at(&self, i: usize) -> i32 {
    self.start[i].read(&self.space.vstore).lower()
  }

  fn duration_at(&self, i: usize) -> i32 {
    self.duration[i].read(&self.space.vstore).lower()
  }
}

impl Display for RobotScheduling
{
  fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
    use pcp::search::search_tree_visitor::Status::*;
    match self.status {
      Unsatisfiable => fmt.write_fmt(format_args!("{}-analysis problem is unsatisfiable.", self.num_robot))?,
      EndOfSearch => fmt.write_str("Search terminated or was interrupted.")?,
      Unknown(_) => unreachable!(
        "After the search step, the problem instance should be either satisfiable or unsatisfiable."),
      Satisfiable => {
        fmt.write_fmt(format_args!("{}-robot scheduling is satisfiable. The first solution is:\n", self.num_robot))?;
        fmt.write_fmt(format_args!("tasks             : {:<8}{:<8}{:<8}{:<8}{:<8}\n", 'L', 'T', 'W', 'P', 'E'))?;
        for i in 0..self.num_robot {
          fmt.write_fmt(format_args!("start time robot {}: ", i+1))?;
          for j in 0..TASKS {
            fmt.write_fmt(format_args!("{:<8}", self.start_at(i+j)))?;
          }
          fmt.write_str("\n")?;
        }
        for i in 0..self.num_robot {
          fmt.write_fmt(format_args!("duration robot {}  : ", i+1))?;
          for j in 0..TASKS-1 {
            fmt.write_fmt(format_args!("{:<8}", self.duration_at(i+j)))?;
          }
        }
      }
    }
    Ok(())
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
