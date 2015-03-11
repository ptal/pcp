// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use solver::propagator::Propagator;
use solver::propagator::Status as PStatus;
use solver::variable::Variable;
use solver::dependencies::VarEventDependencies;
use solver::agenda::Agenda;
use solver::event::VarEvent;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{Formatter, Display, Error};
use std::result::fold;
use std::iter::range;

#[derive(Copy, Debug, PartialEq, Eq)]
pub enum Status {
  Satisfiable,
  Unsatisfiable,
  Unknown
}

pub trait Space {
  type Constraint;
  type Variable : Clone;
  type Domain;
  type Label;

  fn newvar(&mut self, dom: Self::Domain) -> Self::Variable;
  fn add(&mut self, c: Self::Constraint);
  fn solve(&mut self) -> Status;
  fn mark(&self) -> Self::Label;
  fn goto(&self, label: Self::Label) -> Self;
}

pub struct Solver<V: Variable, D, A> {
  propagators: Vec<Box<Propagator<SharedVar=Rc<RefCell<V>>, Event=<V as Variable>::Event> + 'static>>,
  variables: Vec<Rc<RefCell<V>>>,
  deps: D,
  agenda: A
}

impl<V, D, A> Space for Solver<V, D, A> where
 V: Variable+Clone,
 D: VarEventDependencies,
 A: Agenda
{
  type Constraint = Box<Propagator<SharedVar=Rc<RefCell<V>>, Event=<V as Variable>::Event> + 'static>;
  type Variable = Rc<RefCell<V>>;
  type Domain = <V as Variable>::Domain;
  type Label = Solver<V, D, A>;

  fn newvar(&mut self, dom: <Solver<V, D, A> as Space>::Domain) ->
   <Solver<V, D, A> as Space>::Variable {
    let var_idx = self.variables.len();
    self.variables.push(Rc::new(RefCell::new(Variable::new(var_idx as u32, dom))));
    self.variables[var_idx].clone()
  }

  fn add(&mut self, p: <Solver<V, D, A> as Space>::Constraint) {
    self.propagators.push(p);
  }

  fn solve(&mut self) -> Status {
    self.prepare();
    self.propagation_loop()
  }

  fn mark(&self) -> <Solver<V, D, A> as Space>::Label {
    let mut solver = Solver::new();
    solver.variables = self.variables.iter()
      .map(|v| Rc::new(RefCell::new(v.borrow().clone())))
      .collect();
    solver.propagators = self.propagators.iter()
      .map(|p| p.deep_clone(&solver.variables))
      .collect();
    solver
  }

  fn goto(&self, label: <Solver<V, D, A> as Space>::Label) -> Self
  {
    label
  }
}

impl<V, D, A> Solver<V, D, A> where
 V: Variable+Clone,
 D: VarEventDependencies,
 A: Agenda
{
  pub fn new() -> Solver<V, D, A> {
    Solver {
      propagators: vec![],
      variables: vec![],
      deps: VarEventDependencies::new(0,0),
      agenda: Agenda::new(0)
    }
  }

  fn prepare(&mut self) {
    self.init_deps();
    self.init_agenda();
  }

  fn init_deps(&mut self) {
    self.deps = VarEventDependencies::new(self.variables.len(), <<V as Variable>::Event as VarEvent>::size());
    for (p_idx, p) in self.propagators.iter().enumerate() {
      let p_deps: Vec<(u32, _)> = p.dependencies();
      for (v, ev) in p_deps.into_iter() {
        self.deps.subscribe(v as usize, ev, p_idx);
      }
    }
  }

  fn init_agenda(&mut self) {
    let num_props = self.propagators.len();
    self.agenda = Agenda::new(num_props);
    for p_idx in range(0, num_props) {
      self.agenda.schedule(p_idx);
    }
  }

  fn propagation_loop(&mut self) -> Status {
    let mut unsatisfiable = false;
    while let Some(p_idx) = self.agenda.pop() {
      unsatisfiable = !self.propagate_one(p_idx);
      if unsatisfiable { break; }
    }
    if unsatisfiable { Status::Unsatisfiable }
    else if self.deps.is_empty() { Status::Satisfiable }
    else { Status::Unknown }
  }

  fn propagate_one(&mut self, p_idx: usize) -> bool {
    let (events, status) = {
      let mut prop = &mut self.propagators[p_idx];
      if let Some(events) = prop.propagate() {
        (events, prop.status())
      } else {
        return false
      }
    };
    match status {
      PStatus::Disentailed => return false,
      PStatus::Entailed => self.unlink_prop(p_idx),
      PStatus::Unknown => self.reschedule_prop(&events, p_idx)
    };
    self.react(events);
    true
  }

  fn reschedule_prop(&mut self, events: &Vec<(u32, <V as Variable>::Event)>, p_idx: usize) {
    if !events.is_empty() {
      self.agenda.schedule(p_idx);
    }
  }

  fn react(&mut self, events: Vec<(u32, <V as Variable>::Event)>) {
    for (v, ev) in events.into_iter() {
      let reactions = self.deps.react(v as usize, ev);
      for p in reactions.into_iter() {
        self.agenda.schedule(p);
      }
    }
  }

  fn unlink_prop(&mut self, p_idx: usize) {
    self.agenda.unschedule(p_idx);
    let deps: Vec<(u32, _)> = self.propagators[p_idx].dependencies();
    for &(var, ev) in deps.iter() {
      self.deps.unsubscribe(var as usize, ev, p_idx)
    }
  }
}

impl<V, D, A> Display for Solver<V, D, A> where
 V: Variable+Display
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    let format_vars =
      self.variables.iter()
      .map(|v| v.borrow())
      .map(|v| formatter.write_fmt(format_args!("{}\n", *v)));
    fold(format_vars, (), |a,_| a)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use solver::fd::var::*;
  use solver::fd::propagator::*;
  use solver::agenda::RelaxedFifoAgenda;
  use solver::dependencies::VarEventDepsVector;

  type FDSolver = Solver<FDVar, VarEventDepsVector, RelaxedFifoAgenda>;

  #[test]
  fn basic_test() {
    let mut solver: FDSolver = Solver::new();

    assert_eq!(solver.solve(), Status::Satisfiable);

    let var1 = solver.newvar(Interval::new(1,4));
    let var2 = solver.newvar(Interval::new(1,4));
    let var3 = solver.newvar(Interval::new(1,1));

    assert_eq!(solver.solve(), Status::Satisfiable);

    solver.add(Box::new(XLessThanY::new(var1.clone(), var2)));
    assert_eq!(solver.solve(), Status::Unknown);

    solver.add(Box::new(XEqualY::new(var1, var3)));
    assert_eq!(solver.solve(), Status::Satisfiable);
  }

  fn chained_lt(n: usize, expect: Status) {
    // X1 < X2 < X3 < ... < XN, all in dom [1, 10]

    let mut solver: FDSolver = Solver::new();
    let mut vars = vec![];
    for _ in range(0,n) {
      vars.push(solver.newvar(Interval::new(1,10)));
    }
    for i in range(0,n-1) {
      solver.add(Box::new(XLessThanY::new(vars[i].clone(), vars[i+1].clone())));
    }
    assert_eq!(solver.solve(), expect);
  }

  #[test]
  fn chained_lt_tests() {
    chained_lt(1, Status::Satisfiable);
    chained_lt(2, Status::Unknown);
    chained_lt(5, Status::Unknown);
    chained_lt(9, Status::Unknown);
    chained_lt(10, Status::Satisfiable);
    chained_lt(11, Status::Unsatisfiable);
  }

  #[test]
  fn example_nqueens() {
    nqueens(1, Status::Satisfiable);
    nqueens(2, Status::Unknown);
    nqueens(3, Status::Unknown);
    nqueens(4, Status::Unknown);
  }

  fn nqueens(n: usize, expect: Status) {
    let mut solver: FDSolver = Solver::new();
    let mut queens = vec![];
    // 2 queens can't share the same line.
    for _ in range(0,n) {
      queens.push(solver.newvar((1, n as i32).to_interval()));
    }
    for i in range(0usize,n-1) {
      for j in range(i + 1, n) {
        // 2 queens can't share the same diagonal.
        let q1 = (i + 1) as i32;
        let q2 = (j + 1) as i32;
        // Xi + i != Xj + j
        solver.add(Box::new(XNotEqualYPlusC::new(queens[i].clone(), queens[j].clone(), q2 - q1)));
        // Xi - i != Xj - j
        solver.add(Box::new(XNotEqualYPlusC::new(queens[i].clone(), queens[j].clone(), -q2 + q1)));
      }
    }
    // 2 queens can't share the same column.
    solver.add(Box::new(Distinct::new(queens)));
    assert_eq!(solver.solve(), expect);
  }
}
