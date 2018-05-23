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
use pcp::model::*;
use std::fmt::{Formatter, Display, Error};

pub type Bound = i32;
pub type Domain = IntervalSet<Bound>;

pub struct Robot {
  tasks: Vec<usize>,
  cumultasks: Vec<(usize, usize)>
}

pub struct RobotScheduling {
  pub robots: Vec<Robot>,
  pub max_time: usize,
  pub start: Vec<Identity<Domain>>,
  pub durations: Vec<usize>,
  pub pipeting_start: Vec<Box<Identity<Domain>>>,
  pub pipeting_duration: Vec<Box<Identity<Domain>>>,
  pub pipeting_resource: Vec<Box<Constant<Bound>>>,
  pub model: Model,
  pub space: FDSpace,
  pub status: Status<FDSpace>,
}

//static TASKS: usize = 5;
//static DTASKS: usize = 4;
static L: usize = 0; // Loading
static T: usize = 1; // Take
static W: usize = 2; // Wait
static P: usize = 3; // Put
static E: usize = 4; // End, go to park

impl RobotScheduling
{
  pub fn new(num_robot: usize, max_time: usize, Lduration: usize, Tduration: usize, Wduration: usize, Pduration: usize) -> Self {
    let mut robotschel = RobotScheduling {
      robots: vec!(),
      max_time: max_time,
      start: vec![],
      durations: vec!(Lduration, Tduration, Wduration, Pduration),
      pipeting_start: vec![],
      pipeting_duration: vec![],
      pipeting_resource: vec![],
      model: Model::new(),
      space: FDSpace::empty(),
      status: Status::Unsatisfiable,
    };

    //create robot
    for i in 0 .. num_robot {
      let robot = if i % 2 == 0 { //robot with wait
        Robot {
          tasks: vec!(L, T, W, P, E),
          cumultasks: vec!((0,0), (3,3)),
        }

      } else {
        Robot {
          tasks: vec!(L, T, P, E),
          cumultasks: vec!((0,0), (2,2)),
        }

      };
      robotschel.robots.push(robot)
    }

    robotschel.initialize();
    robotschel
  }

  fn initialize(&mut self) {
    let time_dom = IntervalSet::new(1, self.max_time as i32);
    let cumul_tasks = vec![L, P];

    // Start date for the different tasks.
    self.model.open_group("r");
    let mut task_counter = 0;

    for (i, robot) in self.robots.iter().enumerate() {
      self.model.open_group("s");
      for _ in 0..robot.tasks.len() {
        self.start.push(self.model.alloc_var(&mut self.space.vstore, time_dom.clone()));
      }
      self.model.close_group();

      for &(t,d) in robot.cumultasks.iter() {
        self.pipeting_start.push(Box::new(self.start[task_counter + t].clone()));
        self.pipeting_duration.push(Box::new(Identity::new(self.durations[d])));
      }
      // Ensure that every task starts after the end time of the previous task. (S' >= S + D).
      self.model.inc_group();

      task_counter += robot.tasks.len();
    }
    self.model.close_group();
    // Ls = 0 for the first robot to force it to start first
    self.space.cstore.alloc(Box::new(XEqY::new(self.start[0].clone(), Constant::new(1))));

    for i in 0..self.robots.len()*2 {
      self.pipeting_resource.push(Box::new(Constant::new(1)));
    }

    let mut cumulative_pipeting = Cumulative::new(
      self.pipeting_start.clone(),
      self.pipeting_duration.clone(),
      self.pipeting_resource.clone(),
      Box::new(Constant::new(1))
    );
    cumulative_pipeting.join(&mut self.space.vstore, &mut self.space.cstore);

    let inter_tasks = cumulative_pipeting.intermediate_vars();
    for (nti, ti) in inter_tasks.into_iter().enumerate() {
      for (ntj, var) in ti.into_iter().enumerate() {
        self.model.register_var(
          var,
          format!("t{}_{}", nti, ntj));
      }
    }
    self.space.vstore.display(&self.model);
    self.space.cstore.display(&self.model);
    println!(" fin \n");
  }

  pub fn solve(mut self) -> Self {
    println!(" solve deb \n");
    let search =
      OneSolution::<_, VectorStack<_>, FDSpace>::new(
      // Debugger::new(self.model.clone(),
      Propagation::new(
      Brancher::new(InputOrder, MinVal, Enumerate)));
    let mut search = Box::new(search);
    search.start(&self.space);
    let (frozen_space, status) = search.enter(self.space);
    self.space = frozen_space.unfreeze();
    self.status = status;
    println!(" solve fin \n");
    self
  }

  fn start_at(&self, task: usize) -> i32 {
    self.start[task].read(&self.space.vstore).lower()
  }

}

impl Display for RobotScheduling
{
  fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
    use pcp::search::search_tree_visitor::Status::*;
    match self.status {
      Unsatisfiable => fmt.write_fmt(format_args!("{}-analysis problem is unsatisfiable.", self.robots.len()))?,
      EndOfSearch => fmt.write_str("Search terminated or was interrupted.")?,
      Unknown(_) => unreachable!(
        "After the search step, the problem instance should be either satisfiable or unsatisfiable."),
      Satisfiable => {

        fmt.write_fmt(format_args!("{}-robot scheduling is satisfiable. The first solution is:\n", self.robots.len()))?;
        fmt.write_fmt(format_args!("tasks             : {:<8}{:<8}{:<8}{:<8}{:<8}\n", 'L', 'T', 'W', 'P', 'E'))?;

        let mut task_counter = 0;

        for (i, robot) in self.robots.iter().enumerate() {
          fmt.write_fmt(format_args!("start time robot {}: ", i+1))?;
          for j in 0 .. robot.tasks.len() {
            fmt.write_fmt(format_args!("{:<8}", self.start_at(j + task_counter)))?;
          }
          fmt.write_str("\n")?;
          task_counter += robot.tasks.len();
        }

        for (i, robot) in self.robots.iter().enumerate() {
          fmt.write_fmt(format_args!("duration robot {}  : ", i+1))?;
          for dur in self.durations.iter() {
            fmt.write_fmt(format_args!("{:<8}", dur))?;
          }
          fmt.write_str("\n")?;
        }
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test::Bencher;

  #[bench]
  fn bench_schedule_2(b: &mut Bencher) {
    b.iter(|| RobotScheduling::new(2, 500).solve());
  }
}
