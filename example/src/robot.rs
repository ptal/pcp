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

extern crate test;

//use env_logger;
use pcp::kernel::*;
use pcp::propagators::*;
use pcp::variable::ops::*;
use pcp::search::search_tree_visitor::Status::*;
use pcp::search::*;
use interval::interval::*;
use gcollections::ops::*;
use pcp::term::constant::Constant;
use pcp::term::Addition;

//build the contraint for num_robot with max-time indicating that all robot must finish before.
pub fn build_store(num_robot: usize, max_time: usize) -> FDSpace {
  let mut space = FDSpace::empty();
  let mut task = vec![];
  let mut last = vec![];

  let time_dom = (1, max_time as i32).to_interval();
 
  //add task start variable
  for i in 0..num_robot { 
	  task.push(space.vstore.alloc(time_dom)); //Loading L start
	  last.push(space.vstore.alloc((10, 15 as i32).to_interval())); //Loading L Duration
	  task.push(space.vstore.alloc(time_dom)); //Take T start
	  last.push(space.vstore.alloc((25, 35 as i32).to_interval())); //Take T Duration
	  task.push(space.vstore.alloc(time_dom)); //Wait W start
	  last.push(space.vstore.alloc((100, 250 as i32).to_interval())); //Wait W Duration
	  task.push(space.vstore.alloc(time_dom)); //Put P start
	  last.push(space.vstore.alloc((25, 35 as i32).to_interval())); //Put P Duration
	  task.push(space.vstore.alloc(time_dom)); //Park End E start
 
	  //contraints definition
	  if i == 0 {space.cstore.alloc(XEqY::new(task[0], Constant::new(3)));} // Ls = 0 first robot start condition

    space.cstore.alloc(x_greater_y(task[i * 5 + 1], Addition::new(task[i * 5], 15)));
    space.cstore.alloc(x_greater_y(task[i * 5 + 2], Addition::new(task[i * 5 + 1], 15)));
    space.cstore.alloc(x_greater_y(task[i * 5 + 3], Addition::new(task[i * 5 + 2], 15)));
    space.cstore.alloc(x_greater_y(task[i * 5 + 4], Addition::new(task[i * 5 + 3], 15)));
	  
/*	  space.cstore.alloc(XGreaterYPlusZ::new(task[i * 5 + 1], task[i * 5], last[i * 4])); // Ts > Ls + Ld 
	  space.cstore.alloc(XGreaterYPlusZ::new(task[i * 5 + 2], task[i * 5 + 1], last[i * 4 + 1])); // Ws > Ts + Td 
	  space.cstore.alloc(XGreaterYPlusZ::new(task[i * 5 + 3], task[i * 5 + 2], last[i * 4 + 2])); // Ps > Ws + Wd 
	  space.cstore.alloc(XGreaterYPlusZ::new(task[i * 5 + 4], task[i * 5 + 3], last[i * 4 + 3])); // Es > Ps + Pd 
*/
  }
  space
}

pub fn solve_schedule(num_robot: usize, space: FDSpace, show_trace: bool) {
  // Search step.
  let mut search = one_solution_engine();
  search.start(&space);
  let (mut space, status) = search.enter(space);
  let label = space.label();
  let space = space.restore(label);

  if show_trace {
  	// Print result.
  	match status {
  	Satisfiable => {
      println!("{}-robot scheduling is satisfiable. The first solution is:", num_robot);
      println!("task              : L,  T ,  W  , P , E ");
      let mut robot = 1;
      print!("start time robot {}: ", robot);
      let mut num = 0;
      for dom in space.vstore.iter() {
        // At this stage, dom.lower() == dom.upper().
        if num % 2 == 0 {print!("{}, ", dom.lower())};
        num+=1;
        if num == 9 {
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
      for dom in space.vstore.iter() {
        // At this stage, dom.lower() == dom.upper().
        if num % 2 == 1  {print!("{}, ", dom.lower())};
        num+=1;
        if num == 9 {
        	println!("");
	        robot += 1;
	        print!("duration robot {}: ", robot);
        	num=0;
        }
      }
      println!("");
  	}
  	Unsatisfiable => println!("{}-analysis problem is unsatisfiable.", num_robot),
  	Unknown(_) => unreachable!(
  	  "After the search step, the problem instance should be either satisfiable or unsatisfiable.")
    }
  }
}


 #[cfg(test)]
mod tests {
    use test::Bencher;

    #[test]
    fn test_shedule() {
        super::solve_schedule(4, super::build_store(4, 500), true);
    }

    #[bench]
    fn bench_schedule_2(b: &mut Bencher) {
    	b.iter(|| super::solve_schedule(2, super::build_store(2, 500), false));
    }

    #[bench]
    fn bench_schedule_4(b: &mut Bencher) {
    	b.iter(|| super::solve_schedule(4, super::build_store(4, 500), false));
    }

    #[bench]
    fn bench_schedule_8(b: &mut Bencher) {
    	b.iter(|| super::solve_schedule(8, super::build_store(8, 500), false));
    }

    #[bench]
    fn bench_schedule_16(b: &mut Bencher) {
    	b.iter(|| super::solve_schedule(16, super::build_store(16, 500), false));
    }
}
