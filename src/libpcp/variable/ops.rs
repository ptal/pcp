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

use vec_map::Drain;
use std::slice;

pub use term::ops::*;

pub trait DrainDelta<Event>
{
  fn drain_delta<'a>(&'a mut self) -> Drain<'a, Event>;
  fn has_changed(&self) -> bool;
}

pub trait Iterable
{
  type Value;

  fn iter<'a>(&'a self) -> slice::Iter<'a, Self::Value>;
}

pub trait VarIndex
{
  fn index(&self) -> usize;
}

pub trait Failure
{
  fn is_failed(&self) -> bool;
}

pub trait Update<Key, Value>
{
  // `None` if the update operation failed.
  // `Some(value)` if it succeeded, where `value` is the old one.
  fn update(&mut self, key: Key, value: Value) -> Option<Value>;
}

pub trait Freeze
{
  type FrozenState : Snapshot;
  fn freeze(self) -> Self::FrozenState;
}

pub trait Snapshot {
  type Label;
  type UnfrozenState;

  fn snapshot(&mut self) -> Self::Label;
  fn restore(self, label: Self::Label) -> Self::UnfrozenState;
}
