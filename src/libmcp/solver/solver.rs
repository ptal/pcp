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

use interval::ncollections::ops::*;
use solver::propagator::{BoxedDeepClone, Propagator, PropagatorErasure};
use solver::entailment::Status as EStatus;
use solver::entailment::Entailment;
use solver::variable::{Variable, SharedVar};
use solver::dependencies::VarEventDependencies;
use solver::agenda::Agenda;
use solver::event::EventIndex;
use solver::space::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::{Formatter, Display, Error};
use std::result::fold;

pub struct Solver<E, Dom, Deps, A> {
  propagators: Vec<Box<PropagatorErasure<E, Dom> + 'static>>,
  variables: Vec<SharedVar<Dom>>,
  deps: Deps,
  agenda: A
}

impl<E, Dom, Deps, A> Space for Solver<E, Dom, Deps, A> where
 Dom: Cardinality+Clone,
 E: EventIndex,
 Deps: VarEventDependencies,
 A: Agenda
{
  type Constraint = Box<PropagatorErasure<E, Dom> + 'static>;
  type Variable = SharedVar<Dom>;
  type Domain = Dom;
  type Label = Solver<E, Dom, Deps, A>;

  fn newvar(&mut self, dom: Dom) -> SharedVar<Dom> {
    let var_idx = self.variables.len();
    self.variables.push(Rc::new(RefCell::new(Variable::new(var_idx, dom))));
    self.variables[var_idx].clone()
  }

  fn add(&mut self, p: <Solver<E, Dom, Deps, A> as Space>::Constraint) {
    self.propagators.push(p);
  }

  fn solve(&mut self) -> Status {
    self.prepare();
    self.propagation_loop()
  }

  fn mark(&self) -> Solver<E, Dom, Deps, A> {
    let mut solver = Solver::new();
    solver.variables = self.variables.iter()
      .map(|v| Rc::new(RefCell::new(v.borrow().clone())))
      .collect();
    solver.propagators = self.propagators.iter()
      .map(|p| p.boxed_deep_clone(&solver.variables))
      .collect();
    solver
  }

  fn goto(&self, label: Solver<E, Dom, Deps, A>) -> Self
  {
    label
  }
}

impl<E, Dom, Deps, A> Solver<E, Dom, Deps, A> where
 E: EventIndex,
 Deps: VarEventDependencies,
 A: Agenda
{
  pub fn new() -> Solver<E, Dom, Deps, A> {
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
    self.deps = VarEventDependencies::new(self.variables.len(), <E as EventIndex>::size());
    for (p_idx, p) in self.propagators.iter().enumerate() {
      let p_deps = p.dependencies();
      for (v, ev) in p_deps.into_iter() {
        self.deps.subscribe(v as usize, ev, p_idx);
      }
    }
  }

  fn init_agenda(&mut self) {
    let num_props = self.propagators.len();
    self.agenda = Agenda::new(num_props);
    for p_idx in 0..num_props {
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
    let mut events = vec![];
    let status = {
      let mut prop = &mut self.propagators[p_idx];
      if prop.propagate(&mut events) {
        prop.is_entailed()
      } else {
        return false
      }
    };
    match status {
      EStatus::Disentailed => return false,
      EStatus::Entailed => self.unlink_prop(p_idx),
      EStatus::Unknown => self.reschedule_prop(&events, p_idx)
    };
    self.react(events);
    true
  }

  fn reschedule_prop(&mut self, events: &Vec<(usize, E)>, p_idx: usize) {
    if !events.is_empty() {
      self.agenda.schedule(p_idx);
    }
  }

  fn react(&mut self, events: Vec<(usize, E)>) {
    for (v, ev) in events.into_iter() {
      let reactions = self.deps.react(v, ev);
      for p in reactions.into_iter() {
        self.agenda.schedule(p);
      }
    }
  }

  fn unlink_prop(&mut self, p_idx: usize) {
    self.agenda.unschedule(p_idx);
    let deps = self.propagators[p_idx].dependencies();
    for &(var, ev) in deps.iter() {
      self.deps.unsubscribe(var, ev, p_idx)
    }
  }
}

impl<E, Dom, Deps, A> Display for Solver<E, Dom, Deps, A> where
 E: EventIndex,
 Dom: Display
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
  use interval::interval::*;
  use interval::ops::*;
  use solver::space::*;
  use solver::fd::event::*;
  use solver::fd::propagator::*;
  use solver::agenda::RelaxedFifoAgenda;
  use solver::dependencies::VarEventDepsVector;

  type FDSolver = Solver<FDEvent, Interval<i32>, VarEventDepsVector, RelaxedFifoAgenda>;

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
    for _ in 0..n {
      vars.push(solver.newvar(Interval::new(1,10)));
    }
    for i in 0..n-1 {
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
    for _ in 0..n {
      queens.push(solver.newvar((1, n as i32).to_interval()));
    }
    for i in 0..n-1 {
      for j in i + 1..n {
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
