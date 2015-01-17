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

use std::iter::repeat;
use propagator::Propagator;
use variable::Variable;
use dependencies::VarEventDependencies;
use agenda::Agenda;
use event::VarEvent;
use std::rc::Rc;
use std::cell::RefCell;

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

impl<'d, V, D, A> Solver<V, D, A> where
 V: Variable,
 D: VarEventDependencies<'d>,
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
    Status::Satisfiable
  }

  fn prepare(&mut self) {
    self.init_deps();
    self.init_agenda();
  }

  fn init_deps(&mut self) {
    let dummy: Option<<V as Variable>::Event> = None;
    self.deps = VarEventDependencies::new(self.variables.len(), VarEvent::size(dummy));
    for (p_idx, p) in self.propagators.iter().enumerate() {
      for (v, ev) in p.dependencies().into_iter() {
        self.deps.subscribe(v as usize, ev, p_idx);
      }
    }
  }

  fn init_agenda(&mut self) {
    self.agenda = Agenda::new(self.propagators.len());
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use fd::var::*;
  use fd::propagator::*;
  use propagator::Propagator;
  use agenda::RelaxedFifoAgenda;
  use dependencies::VarEventDepsVector;

  type FDSolver = Solver<FDVar, VarEventDepsVector, RelaxedFifoAgenda>;

  #[test]
  fn newvar_test() {
    let mut solver: FDSolver = Solver::new();
    let var1 = solver.newvar(Interval::new(1,4));
    let var2 = solver.newvar(Interval::new(1,4));

    let prop: Box<Propagator<Event=FDEvent>> = Box::new(XLessThanY::new(var1, var2));
    solver.add(prop);
  }
}
