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


// TODO: Redo unnecessary? By recomputation?
//  Investigate why it became slow again...
//  Turn TrailingStore into a store and not a memory.
//  Add different store for NonFailure/StrictMonotonicity/... : First implement Derive(...) and see the inheritance system.

use kernel::*;
use variable::concept::*;
use variable::ops::*;
use variable::memory::copy::*;
use gcollections::ops::constructor::*;
use gcollections::ops::cardinality::*;
use gcollections::ops::sequence::*;
use gcollections::ops::sequence::ordering::*;
use vec_map::VecMap;
use std::slice;
use std::ops::{Index, Deref, DerefMut};
use std::fmt::{Formatter, Display, Error};
use std::rc::*;

pub struct TrailedStore<Domain>
{
  variables: CopyMemory<Domain>,
  parent_trail: Rc<Trail<Domain>>,
  trail: VecMap<Domain>
}

impl<Domain> MemoryConcept<Domain> for TrailedStore<Domain> where
 Domain: DomainConcept
{}

impl<Domain> ImmutableMemoryConcept<Domain> for TrailedStore<Domain> where
 Domain: DomainConcept
{}

impl<Domain> TrailedStore<Domain> where
 Domain: DomainConcept
{
  /// We only trail the first change of the domain during the current instant (before a freeze).
  fn trail_variable(&mut self, key: usize, dom: Domain) {
    self.trail.insert(key, dom);
    // self.trail.entry(key).or_insert(dom);
  }
}

impl<Domain> Empty for TrailedStore<Domain>
{
  fn empty() -> TrailedStore<Domain> {
    TrailedStore {
      variables: CopyMemory::empty(),
      parent_trail: Rc::new(Trail::empty()),
      trail: VecMap::new()
    }
  }
}

impl<Domain> Cardinality for TrailedStore<Domain>
{
  type Size = usize;

  fn size(&self) -> usize {
    self.variables.size()
  }
}

impl<Domain> Iterable for TrailedStore<Domain>
{
  type Item = Domain;

  fn iter<'a>(&'a self) -> slice::Iter<'a, Self::Item> {
    self.variables.iter()
  }
}

impl<Domain> Push<Back, Domain> for TrailedStore<Domain> where
 Domain: DomainConcept
{
  fn push(&mut self, dom: Domain) {
    let dom_location = self.variables.size();
    self.trail_variable(dom_location, dom.clone());
    self.variables.push(dom);
  }
}

impl<Domain> Replace<usize, Domain> for TrailedStore<Domain> where
 Domain: DomainConcept
{
  fn replace(&mut self, key: usize, dom: Domain) -> Domain
  {
    let dom = self.variables.replace(key, dom);
    self.trail_variable(key, dom.clone());
    dom
  }
}

impl<Domain> Index<usize> for TrailedStore<Domain>
{
  type Output = Domain;
  fn index<'a>(&'a self, index: usize) -> &'a Domain {
    &self.variables[index]
  }
}

impl<Domain> Display for TrailedStore<Domain> where
 Domain: Display
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    self.variables.fmt(formatter)
  }
}

impl<Domain> Freeze for TrailedStore<Domain> where
 Domain: DomainConcept
{
  type FrozenState = FrozenTrailedStore<Domain>;
  fn freeze(self) -> Self::FrozenState
  {
    FrozenTrailedStore::new(self)
  }
}

pub struct FrozenTrailedStore<Domain>
{
  store: TrailedStore<Domain>
}

impl<Domain> FrozenTrailedStore<Domain> where
 Domain: DomainConcept
{
  fn new(mut store: TrailedStore<Domain>) -> FrozenTrailedStore<Domain> {
    let parent_trail = Trail::new(store.parent_trail, store.variables.size(), store.trail);
    store.parent_trail = parent_trail;
    store.trail = VecMap::with_capacity(store.variables.size());
    FrozenTrailedStore {
      store: store
    }
  }
}

impl<Domain> Snapshot for FrozenTrailedStore<Domain> where
 Domain: DomainConcept
{
  type Label = Rc<Trail<Domain>>;
  type State = TrailedStore<Domain>;

  fn label(&mut self) -> Self::Label {
    println!("Labelling: \n{}\n", self.store.parent_trail);
    self.store.parent_trail.clone()
  }

  fn restore(mut self, label: Self::Label) -> Self::State {
    println!("Restoring:");
    println!("  From (parent_trail):\n{}\n", self.store.parent_trail);
    println!("  To (label):\n{}\n", label);
    if !rc_eq(&self.store.parent_trail, &label) {
      let mut redo_delta: VecMap<Domain> = VecMap::with_capacity(self.store.size());
      let mut undo_delta: VecMap<Domain> = VecMap::with_capacity(self.store.size());
      let mut redo = label.clone();
      let mut undo = self.store.parent_trail;

      while redo.depth > undo.depth {
        redo_delta_from_trail(&redo, &mut redo_delta);
        redo = redo.ancestor();
      }
      while undo.depth > redo.depth {
        undo_delta_from_trail(&undo, &mut undo_delta);
        undo = undo.ancestor();
      }
      while !rc_eq(&redo, &undo) {
        redo_delta_from_trail(&redo, &mut redo_delta);
        undo_delta_from_trail(&undo, &mut undo_delta);
        redo = redo.ancestor();
        undo = undo.ancestor();
      }
      let common_ancestor_vars = redo.num_vars;
      self.store.variables.truncate(common_ancestor_vars);
      undo_redo_node(&mut self.store.variables, undo_delta, redo_delta);
    }
    self.store.parent_trail = label.clone();
    self.store
  }
}

fn rc_eq<T>(a: &Rc<T>, b: &Rc<T>) -> bool
{
  a.deref() as *const T == b.deref() as *const T
}


fn redo_delta_from_trail<Domain>(node: &Rc<Trail<Domain>>, delta: &mut VecMap<Domain>) where
 Domain: DomainConcept
{
  for cell in node.trail.iter() {
    delta.entry(cell.location).or_insert(cell.value.clone());
  }
}

fn undo_delta_from_trail<Domain>(node: &Rc<Trail<Domain>>, delta: &mut VecMap<Domain>) where
 Domain: DomainConcept
{
  for cell in node.trail.iter() {
    delta.insert(cell.location, cell.value.clone());
  }
}

fn undo_redo_node<Domain>(node: &mut CopyMemory<Domain>,
  undo_delta: VecMap<Domain>, redo_delta: VecMap<Domain>)
{
  for (loc, value) in undo_delta {
    debug_assert!(loc <= node.size(), "All variables must be recorded.");
    if loc == node.size() { break; }
    node.deref_mut()[loc] = value;
  }
  let mut redo_delta = redo_delta.into_iter();
  while let Some((loc, value)) = redo_delta.next() {
    debug_assert!(loc <= node.size(), "All variables must be recorded.");
    if loc == node.size() {
      node.push(value);
      break;
    }
    node.deref_mut()[loc] = value;
  }

  for (loc, value) in redo_delta {
    node.push(value);
    debug_assert!(node.size() == loc,
      "From a node A (with n variables) to a node B (with m variables), some variables between n to m-1 were not recorded.");
  }
}

struct MemoryCell<Domain>
{
  location: usize,
  value: Domain
}

impl<Domain> MemoryCell<Domain>
{
  fn new(location: usize, value: Domain) -> MemoryCell<Domain> {
    MemoryCell {
      location: location,
      value: value
    }
  }
}

impl<Domain> Display for MemoryCell<Domain> where
 Domain: Display
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_str(format!(
      "{}: {}", self.location, self.value
    ).as_str())
  }
}

pub struct Trail<Domain>
{
  depth: usize,
  num_vars: usize,
  trail: Vec<MemoryCell<Domain>>,
  previous: Option<Rc<Trail<Domain>>>
}

impl<Domain> Display for Trail<Domain> where
 Domain: Display
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_str("Trail information: \n")?;
    formatter.write_str(format!("  depth: {}\n", self.depth).as_str())?;
    formatter.write_str(format!("  num_vars: {}\n", self.num_vars).as_str())?;
    formatter.write_str("  trail:\n")?;
    for cell in &self.trail {
      formatter.write_str(format!("    {}\n", cell).as_str())?;
    }
    Ok(())
  }
}


impl<Domain> Trail<Domain> where
 Domain: DomainConcept
{
  fn new(parent: Rc<Trail<Domain>>, num_vars: usize, trail: VecMap<Domain>) -> Rc<Trail<Domain>> {
    debug_assert!(parent.num_vars <= num_vars, "The number of trailed variables can only increase.");
    Rc::new(
      Trail {
        depth: parent.depth + 1,
        num_vars: num_vars,
        trail: Self::compress_trail(trail),
        previous: Some(parent)
      }
    )
  }

  fn compress_trail(trail: VecMap<Domain>) -> Vec<MemoryCell<Domain>> {
    let mut compressed_trail = Vec::with_capacity(trail.len());
    for (loc, value) in trail {
      compressed_trail.push(MemoryCell::new(loc, value));
    }
    compressed_trail
  }

  fn ancestor(&self) -> Rc<Trail<Domain>> {
    assert!(self.depth > 0, "Only trails with a depth > 0 have an ancestor.");
    self.previous.clone().expect("Trail with a depth > 0 must have a parent trail.")
  }
}

impl<Domain> Empty for Trail<Domain>
{
  fn empty() -> Trail<Domain> {
    Trail {
      depth: 0,
      num_vars: 0,
      trail: vec![],
      previous: None
    }
  }
}

