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

pub struct Trail<Domain>
{
  depth: usize,
  num_vars: usize,
  trail: Vec<MemoryCell<Domain>>,
  previous: Option<Rc<Trail<Domain>>>
}

impl<Domain> Trail<Domain>
{
  fn from_parent(parent: Rc<Trail<Domain>>) -> Rc<Trail<Domain>> {
    Rc::new(
      Trail {
        depth: parent.depth + 1,
        num_vars: parent.num_vars,
        trail: vec![],
        previous: Some(parent)
      }
    )
  }

  fn ancestor(&self) -> Rc<Trail<Domain>> {
    assert!(self.depth > 0, "Only trails with depth > 0 have an ancestor.");
    self.previous.clone().expect("Trail with a depth > 0 must have a parent trail.")
  }

  fn trail_update(&mut self, key: usize, dom: Domain) {
    self.trail.push(MemoryCell::new(key, dom));
  }

  fn update_num_vars(&mut self, n: usize) {
    debug_assert!(n >= self.num_vars, "The number of variables trailed can only increase.");
    self.num_vars = n;
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

pub struct TrailedStore<Domain>
{
  variables: CopyStore<Domain>,
  trail: Rc<Trail<Domain>>
}

impl<Domain> MemoryConcept<Domain> for TrailedStore<Domain> where
 Domain: DomainConcept
{}

impl<Domain> ImmutableMemoryConcept<Domain> for TrailedStore<Domain> where
 Domain: DomainConcept
{}

impl<Domain> TrailedStore<Domain>
{
  fn update_num_vars(&mut self) {
    Rc::get_mut(&mut self.trail)
      .expect("Cannot update the trail if it is not unique.")
      .update_num_vars(self.variables.size());
  }
}

impl<Domain> Empty for TrailedStore<Domain>
{
  fn empty() -> TrailedStore<Domain> {
    TrailedStore {
      variables: CopyStore::empty(),
      trail: Rc::new(Trail::empty())
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

impl<Domain> Push<Back, Domain> for TrailedStore<Domain>
{
  fn push(&mut self, value: Domain) {
    self.variables.push(value);
  }
}

impl<Domain> Update<usize, Domain> for TrailedStore<Domain> where
 Domain: DomainConcept
{
  fn update(&mut self, key: usize, dom: Domain) -> Option<Domain>
  {
    self.variables.update(key, dom).map(|dom| {
      Rc::get_mut(&mut self.trail)
        .expect("The trail must be a leaf of the tree when updated. Therefore, it must only have one strong reference.")
        .trail_update(key, dom.clone());
      dom
    })
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
  type ImmutableState = ImmutableTrailedStore<Domain>;
  fn freeze(self) -> Self::ImmutableState
  {
    ImmutableTrailedStore::new(self)
  }
}

pub struct ImmutableTrailedStore<Domain>
{
  store: TrailedStore<Domain>
}

impl<Domain> ImmutableTrailedStore<Domain>
{
  fn new(mut store: TrailedStore<Domain>) -> ImmutableTrailedStore<Domain> {
    store.update_num_vars();
    ImmutableTrailedStore {
      store: store
    }
  }
}

fn rc_eq<T>(a: &Rc<T>, b: &Rc<T>) -> bool
{
  a.deref() as *const T == b.deref() as *const T
}


fn redo_delta_from_trail<Domain>(node: &Rc<Trail<Domain>>, delta: &mut VecMap<Domain>) where
 Domain: DomainConcept
{
  for cell in node.trail.iter().rev() {
    delta.entry(cell.location).or_insert(cell.value.clone());
  }
}

fn undo_delta_from_trail<Domain>(node: &Rc<Trail<Domain>>, delta: &mut VecMap<Domain>) where
 Domain: DomainConcept
{
  for cell in node.trail.iter().rev() {
    delta.insert(cell.location, cell.value.clone());
  }
}

fn undo_redo_node<Domain>(node: &mut CopyStore<Domain>,
  undo_delta: VecMap<Domain>, redo_delta: VecMap<Domain>)
{
  for (loc, value) in undo_delta {
    if loc > node.size() { break; }
    node.deref_mut()[loc] = value;
  }
  let mut redo_delta = redo_delta.into_iter();
  while let Some((loc, value)) = redo_delta.next() {
    debug_assert!(loc <= node.size(),
      "Every variable must be recorded.");
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

impl<Domain> Snapshot for ImmutableTrailedStore<Domain> where
 Domain: DomainConcept
{
  type Label = Rc<Trail<Domain>>;
  type MutableState = TrailedStore<Domain>;

  fn label(&mut self) -> Self::Label {
    self.store.trail.clone()
  }

  fn restore(mut self, label: Self::Label) -> Self::MutableState {
    let mut redo_delta: VecMap<Domain> = VecMap::with_capacity(self.store.size());
    let mut undo_delta: VecMap<Domain> = VecMap::with_capacity(self.store.size());
    let mut redo = label.clone();
    let mut undo = self.store.trail;

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
    self.store.trail = Trail::from_parent(label.clone());
    self.store
  }
}
