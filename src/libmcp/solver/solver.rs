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

#[derive(Copy, Show, PartialEq, Eq)]
pub enum Status {
  Satisfiable,
  Unsatisfiable,
  Unknown
}

pub struct Solver<V: Variable, D, A> {
  propagators: Vec<Box<Propagator<Event=V::Event> + 'static>>,
  variables: Vec<Rc<RefCell<V>>>,
  deps: D,
  agenda: A
}

impl<V, D, A> Solver<V, D, A> where
 V: Variable,
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

  pub fn newvar(&mut self, dom: <V as Variable>::Domain) -> Rc<RefCell<V>> {
    let var_idx = self.variables.len();
    self.variables.push(Rc::new(RefCell::new(Variable::new(var_idx as u32, dom))));
    self.variables[var_idx].clone()
  }

  pub fn add(&mut self, p: Box<Propagator<Event=<V as Variable>::Event> + 'static>) {
    self.propagators.push(p);
  }

  pub fn solve(&mut self) -> Status {
    self.prepare();
    self.propagation_loop()
  }

  fn prepare(&mut self) {
    self.init_deps();
    self.init_agenda();
  }

  fn init_deps(&mut self) {
    self.deps = VarEventDependencies::new(self.variables.len(), <<V as Variable>::Event as VarEvent>::size());
    for (p_idx, p) in self.propagators.iter().enumerate() {
      for (v, ev) in p.dependencies().into_iter() {
        self.deps.subscribe(v as usize, ev, p_idx);
      }
    }
  }

  fn init_agenda(&mut self) {
    self.agenda = Agenda::new(self.propagators.len());
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
      let mut reactions = self.deps.react(v as usize, ev);
      for &p in *reactions {
        self.agenda.schedule(p);
      }
    }
  }

  fn unlink_prop(&mut self, p_idx: usize) {
    self.agenda.unschedule(p_idx);
    let deps = self.propagators[p_idx].dependencies();
    for &(var, ev) in deps.iter() {
      self.deps.unsubscribe(var as usize, ev, p_idx)
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use solver::fd::var::*;
  use solver::fd::propagator::*;
  use solver::propagator::Propagator;
  use solver::agenda::RelaxedFifoAgenda;
  use solver::dependencies::VarEventDepsVector;
  use std::iter::range;

  type FDSolver = Solver<FDVar, VarEventDepsVector, RelaxedFifoAgenda>;

  #[test]
  fn basic_test() {
    let mut solver: FDSolver = Solver::new();
    let var1 = solver.newvar(Interval::new(1,4));
    let var2 = solver.newvar(Interval::new(1,4));
    let var3 = solver.newvar(Interval::new(1,1));

    solver.add(Box::new(XLessThanY::new(var1.clone(), var2)));
    assert_eq!(solver.solve(), Status::Unknown);

    solver.add(Box::new(XEqualY::new(var1, var3)));
    assert_eq!(solver.solve(), Status::Satisfiable);
  }
}
