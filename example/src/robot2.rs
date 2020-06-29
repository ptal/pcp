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

//use env_logger;
use pcp::kernel::*;
use pcp::propagators::*;
use pcp::search::*;
use pcp::search::branching::*;
use pcp::search::engine::one_solution::*;
// use pcp::search::debugger::*;
use pcp::search::propagation::*;
use gcollections::VectorStack;
use interval::interval_set::*;
use interval::ops::Range;
use gcollections::ops::*;
use pcp::term::*;
use pcp::propagators::cumulative::Cumulative;
use pcp::model::*;
use pcp::concept::*;
use std::fmt::{Formatter, Display, Error};

pub type Bound = i32;
pub type Domain = IntervalSet<Bound>;

//static TASKS: usize = 5;
//static DTASKS: usize = 4;
static L: usize = 0; // Loading
static T: usize = 1; // Take
static W: usize = 2; // Wait
static P: usize = 3; // Put
static E: usize = 4; // End, go to park

pub enum RobotType {
  Simple,  //task has a fixed duration
  Duration{vdurations: Vec<Var<VStore>>, variation: usize}, //task duration is a variable
}

impl Clone for RobotType {
    fn clone(&self) -> Self {
      match self {
        &RobotType::Simple => RobotType::Simple,  //task has a fixed duration
        &RobotType::Duration{ref vdurations, variation} => RobotType::Duration{vdurations: vdurations.iter().map(|v| v.bclone()).collect(), variation: variation}, //task duration is a variable
      }
    }
}
pub struct Robot {
  robot_type : RobotType,
  start: Vec<Var<VStore>>, //start task variable
  tasks: Vec<usize>, //task list of the robot
  durations: Vec<usize>, //task duration value
  cumultasks: Vec<(usize, usize)>  //define task that participe to the first cumulative constraint. (Task_index, Duration_index)
}

impl Robot {
  //first test alternate with robot that has a Wait task and robot without. No duration variable
  pub fn create_robots(robot_type: RobotType, num_robot: usize, Lduration: usize, Tduration: usize, Wduration: usize, Pduration: usize) -> Vec<Robot> {
    let mut robotschel = vec!();
    for i in 0 .. num_robot {
      let robot = if i % 2 == 0 { //robot with wait
        Robot {
          robot_type : robot_type.clone(),  //RobotType::simple,
          start: vec!(),
          tasks: vec!(L, T, W, P, E),
          durations: vec!(Lduration, Tduration, Wduration, Pduration), //Fixed duration variable
          cumultasks: vec!((0,0), (3,3)),
        }

      } else {
        Robot {
          robot_type : robot_type.clone(),
          start: vec!(),
          tasks: vec!(L, T, P, E),
          durations: vec!(Lduration, Tduration, Pduration), //fixed duration variable
          cumultasks: vec!((0,0), (2,2)),
        }

      };
      robotschel.push(robot)
    }
    robotschel
  }

  pub fn get_pipeting_duration_at_rank(&self, i: usize) -> Var<VStore> {
    match self.robot_type {
      RobotType::Simple => {
        Box::new(Identity::new(self.durations[i]))
      },
      RobotType::Duration{ref vdurations, ..} => {
        vdurations[i].bclone()
      },
    }

  }

  pub fn add_robot_duration_variables(&mut self, space: &mut FDSpace, model: &mut Model) {
    if let RobotType::Duration{ref mut vdurations, variation} = self.robot_type {
      self.durations.iter()
      .for_each(|duration|
          vdurations.push(model.alloc_var(&mut space.vstore, IntervalSet::new(*duration as i32, (*duration + variation) as i32)))
      )
    }
  }


  pub fn add_robot_task_sequencing_variable(&self, space: &mut FDSpace) {
    // Ensure that every task starts after the end time of the previous task. (S' >= S + D).
    match self.robot_type {
      RobotType::Simple => {
        for t in 1..self.tasks.len() {
          space.cstore.alloc(
            Box::new(x_greater_y(
              self.start[t].bclone(),
              Box::new(Addition::new(self.start[t - 1].bclone(), self.durations[t - 1] as i32)))));
        }
      },
      RobotType::Duration{ref vdurations, ..} => {
        for t in 1..self.tasks.len() {
          space.cstore.alloc(
            Box::new(x_geq_y_plus_z(
              self.start[t].bclone(),
              self.start[t - 1].bclone(),
              vdurations[t - 1].bclone())));
        }
      },
    }
  }

  pub fn fmt_start(&self, fmt: &mut Formatter, space: &FDSpace) -> Result<(), Error> {
    for task in self.start.iter() {
      fmt.write_fmt(format_args!("{:<8}", task.read(&space.vstore).lower()))?;
    }
    Ok(())
  }

  pub fn fmt_duration(&self, fmt: &mut Formatter, space: &FDSpace) -> Result<(), Error> {
    match self.robot_type {
      RobotType::Simple => {
        for dur in self.durations.iter() {
          fmt.write_fmt(format_args!("{:<8}", dur))?;
        }
       },
      RobotType::Duration{ref vdurations, ..} => {
        for dur in vdurations {
          fmt.write_fmt(format_args!("{:<8}", dur.read(&space.vstore).lower()))?;
        }
      },
    }
    Ok(())
  }

}

pub struct RobotScheduling {
  pub robots: Vec<Robot>,
  pub max_time: usize,
  pub pipeting_start: Vec<Var<VStore>>,
  pub pipeting_duration: Vec<Var<VStore>>,
  pub pipeting_resource: Vec<Box<Constant<Bound>>>,
  pub model: Model,
  pub space: FDSpace,
  pub status: Status<FDSpace>,
}

impl RobotScheduling
{
  /// test with robot type simple. Task duration is fixed and isn't a variable
  pub fn new_test1(num_robot: usize, max_time: usize, Lduration: usize, Tduration: usize, Wduration: usize, Pduration: usize) -> Self {
    let mut robotschel = RobotScheduling {
      robots: Robot::create_robots(RobotType::Simple, num_robot, Lduration, Tduration, Wduration, Pduration),
      max_time: max_time,
      pipeting_start: vec![],
      pipeting_duration: vec![],
      pipeting_resource: vec![],
      model: Model::new(),
      space: FDSpace::empty(),
      status: Status::Unsatisfiable,
    };
    //construct robots task;

    robotschel.initialize();
    robotschel
  }
  /// test with robot type Duration. Task duration is a variable bounded.
  pub fn new_test2(num_robot: usize, max_time: usize, Lduration: usize, Tduration: usize, Wduration: usize, Pduration: usize, duration_variation: usize) -> Self {
    let mut robotschel = RobotScheduling {
      robots: Robot::create_robots(RobotType::Duration{vdurations: vec!(), variation: duration_variation}, num_robot, Lduration, Tduration, Wduration, Pduration),
      max_time: max_time,
      pipeting_start: vec![],
      pipeting_duration: vec![],
      pipeting_resource: vec![],
      model: Model::new(),
      space: FDSpace::empty(),
      status: Status::Unsatisfiable,
    };
    //construct robots task;

    robotschel.initialize();
    robotschel
  }

  fn initialize(&mut self) {
    let time_dom = IntervalSet::new(1, self.max_time as i32);
    let cumul_tasks = vec![L, P];

    // Start date for the different tasks.
    self.model.open_group("r");

    for (i, robot) in self.robots.iter_mut().enumerate() {

       //create task start variables.
      self.model.open_group("s");
      for _ in 0..robot.tasks.len() {
        robot.start.push(self.model.alloc_var(&mut self.space.vstore, time_dom.clone()));
      }
      self.model.close_group();

      //create duration variable if needed.
      self.model.open_group("d");
      robot.add_robot_duration_variables(&mut self.space, &mut self.model);
      self.model.close_group();

      // Ensure that every task starts after the end time of the previous task. (S' >= S + D).
      robot.add_robot_task_sequencing_variable(&mut self.space);

      //Add cumulative info.
      for &(t,d) in robot.cumultasks.iter() {
        self.pipeting_start.push(robot.start[t].bclone());
        self.pipeting_duration.push(robot.get_pipeting_duration_at_rank(d).bclone());
      }

      self.model.inc_group();
    }
    self.model.close_group();
    // Ls = 0 for the first robot to force it to start first
    self.space.cstore.alloc(Box::new(XEqY::new(self.robots[0].start[0].bclone(), Box::new(Constant::new(1)))));

    for i in 0..self.robots.len()*2 {
      self.pipeting_resource.push(Box::new(Constant::new(1)));
    }

    let mut cumulative_pipeting = Cumulative::new(
      self.pipeting_start.iter().map(|v| v.bclone()).collect(),
      self.pipeting_duration.iter().map(|v| v.bclone()).collect(),
      self.pipeting_resource.iter().map(|v| v.bclone()).collect(),
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
//    self.space.vstore.display(&self.model);
//    self.space.cstore.display(&self.model);
    println!("\n");
  }

  pub fn solve(mut self) -> Self {
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
    self
  }

/*  fn start_at(&self, task: usize) -> i32 {
    self.start[task].read(&self.space.vstore).lower()
  } */

/*  fn duration_at(&self, duration: usize) -> i32 {
    self.duration[duration].read(&self.space.vstore).lower()
  } */
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
          robot.fmt_start(fmt, &self.space)?;
          fmt.write_str("\n")?;
          task_counter += robot.tasks.len();
        }

        let mut duration_counter = 0;
        for (i, robot) in self.robots.iter().enumerate() {
          fmt.write_fmt(format_args!("duration robot {}  : ", i+1))?;
          robot.fmt_duration(fmt, &self.space)?;
          fmt.write_str("\n")?;
          duration_counter += robot.durations.len();
        }
      }
    }
    Ok(())
  }
}
