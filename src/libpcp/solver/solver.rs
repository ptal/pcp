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
use kernel::*;
use kernel::Trilean::*;
use variable::delta_store::DeltaStore;
use variable::ops::*;
use variable::arithmetics::identity::*;
use std::rc::{try_unwrap, Rc};
use std::fmt::{Formatter, Display, Error};
use std::result::fold;
use std::slice;

pub trait BoxedDeepClone<S, E>
{
  fn boxed_deep_clone(&self) -> Box<PropagatorErasure<S, E>>;
}

impl<S, E, R> BoxedDeepClone<S, E> for R where
  R: DeepClone,
  R: Propagator<S>,
  R: PropagatorDependencies<E>,
  R: Subsumption<S>,
  R: 'static
{
  fn boxed_deep_clone(&self) -> Box<PropagatorErasure<S, E>> {
    Box::new(self.deep_clone())
  }
}

pub trait PropagatorErasure<S, E> :
    Propagator<S>
  + PropagatorDependencies<E>
  + Subsumption<S>
  + BoxedDeepClone<S, E>
{}

impl<S, E, R> PropagatorErasure<S, E> for R where
 R: Propagator<S>,
 R: PropagatorDependencies<E>,
 R: Subsumption<S>,
 R: BoxedDeepClone<S, E>
{}

pub struct Solver<E, Dom, Deps, A> {
  propagators: Vec<Box<PropagatorErasure<DeltaStore<E, Dom>, E> + 'static>>,
  store: DeltaStore<E, Dom>,
  deps: Deps,
  agenda: A
}

impl<E, Dom, Deps, A> VariableIterator for Solver<E, Dom, Deps, A>
{
  type Variable = Dom;

  fn vars_iter<'a>(&'a self) -> slice::Iter<'a, Dom> {
    self.store.vars_iter()
  }
}

impl<E, Dom, Deps, A> Space for Solver<E, Dom, Deps, A> where
 Dom: Cardinality+Clone,
 E: EventIndex,
 Deps: VarEventDependencies,
 A: Agenda
{
  type Constraint = Box<PropagatorErasure<DeltaStore<E, Dom>, E> + 'static>;
  type Variable = Identity<Dom>;
  type Domain = Dom;

  fn newvar(&mut self, dom: Dom) -> Identity<Dom> {
    self.store.assign(dom)
  }

  fn add(&mut self, p: Self::Constraint) {
    self.propagators.push(p);
  }

  fn solve(&mut self) -> Trilean {
    self.prepare();
    self.propagation_loop()
  }
}

impl<E, Dom, Deps, A> State for Solver<E, Dom, Deps, A> where
 Dom: Cardinality+Clone,
 E: EventIndex,
 Deps: VarEventDependencies,
 A: Agenda
{
  type Label = Rc<Solver<E, Dom, Deps, A>>;

  fn mark(&self) -> Rc<Solver<E, Dom, Deps, A>> {
    Rc::new(self.deep_clone())
  }

  fn restore(self, label: Rc<Solver<E, Dom, Deps, A>>) -> Self {
    try_unwrap(label).unwrap_or_else(|l| l.deep_clone())
  }
}

impl<E, Dom, Deps, A> DeepClone for Solver<E, Dom, Deps, A> where
 Dom: Clone,
 E: EventIndex,
 Deps: VarEventDependencies,
 A: Agenda
{
  fn deep_clone(&self) -> Self {
    let mut solver = Solver::new();
    solver.store = self.store.deep_clone();
    solver.propagators = self.propagators.iter()
      .map(|p| p.boxed_deep_clone())
      .collect();
    solver
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
      store: DeltaStore::new(),
      deps: VarEventDependencies::new(0,0),
      agenda: Agenda::new(0)
    }
  }

  fn prepare(&mut self) {
    self.init_deps();
    self.init_agenda();
  }

  fn init_deps(&mut self) {
    self.deps = VarEventDependencies::new(self.store.size(), <E as EventIndex>::size());
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

  fn propagation_loop(&mut self) -> Trilean {
    let mut unsatisfiable = false;
    while let Some(p_idx) = self.agenda.pop() {
      unsatisfiable = !self.propagate_one(p_idx);
      if unsatisfiable { break; }
    }
    if unsatisfiable { False }
    else if self.deps.is_empty() { True }
    else { Unknown }
  }

  fn propagate_one(&mut self, p_idx: usize) -> bool {
    let status = {
      let mut prop = &mut self.propagators[p_idx];
      if prop.propagate(&mut self.store) {
        prop.is_subsumed(&self.store)
      } else {
        return false
      }
    };
    match status {
      False => return false,
      True => self.unlink_prop(p_idx),
      Unknown => self.reschedule_prop(p_idx)
    };
    self.react();
    true
  }

  fn reschedule_prop(&mut self, p_idx: usize) {
    if !self.store.drain_delta().peekable().is_empty() {
      self.agenda.schedule(p_idx);
    }
  }

  fn react(&mut self) {
    for (v, ev) in self.store.drain_delta() {
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
      self.store.vars_iter()
      .map(|v| formatter.write_fmt(format_args!("{}\n", v)));
    fold(format_vars, (), |a,_| a)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use solver::fd::event::FDEvent;
  use interval::interval::*;
  use interval::ops::*;
  use kernel::*;
  use kernel::Trilean::*;
  use propagators::cmp::*;
  use propagators::distinct::*;
  use variable::arithmetics::*;

  type FDSolver = Solver<FDEvent, Interval<i32>, VarEventDepsVector, RelaxedFifoAgenda>;

  #[test]
  fn basic_test() {
    let mut solver: FDSolver = Solver::new();

    assert_eq!(solver.solve(), True);

    let var1 = solver.newvar(Interval::new(1,4));
    let var2 = solver.newvar(Interval::new(1,4));
    let var3 = solver.newvar(Interval::new(1,1));

    assert_eq!(solver.solve(), True);

    solver.add(Box::new(XLessY::new(var1.clone(), var2)));
    assert_eq!(solver.solve(), Unknown);

    solver.add(Box::new(XEqY::new(var1, var3)));
    assert_eq!(solver.solve(), True);
  }

  fn chained_lt(n: usize, expect: Trilean) {
    // X1 < X2 < X3 < ... < XN, all in dom [1, 10]

    let mut solver: FDSolver = Solver::new();
    let mut vars = vec![];
    for _ in 0..n {
      vars.push(solver.newvar(Interval::new(1,10)));
    }
    for i in 0..n-1 {
      solver.add(Box::new(XLessY::new(vars[i].clone(), vars[i+1].clone())));
    }
    assert_eq!(solver.solve(), expect);
  }

  #[test]
  fn chained_lt_tests() {
    chained_lt(1, True);
    chained_lt(2, Unknown);
    chained_lt(5, Unknown);
    chained_lt(9, Unknown);
    chained_lt(10, True);
    chained_lt(11, False);
  }

  #[test]
  fn example_nqueens() {
    nqueens(1, True);
    nqueens(2, Unknown);
    nqueens(3, Unknown);
    nqueens(4, Unknown);
  }

  fn nqueens(n: usize, expect: Trilean) {
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
        solver.add(Box::new(XNeqY::new(queens[i].clone(), Addition::new(queens[j].clone(), q2 - q1))));
        // Xi - i != Xj - j
        solver.add(Box::new(XNeqY::new(queens[i].clone(), Addition::new(queens[j].clone(), -q2 + q1))));
      }
    }
    // 2 queens can't share the same column.
    solver.add(Box::new(Distinct::new(queens)));
    assert_eq!(solver.solve(), expect);
  }
}
