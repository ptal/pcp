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

use gcollections::kind::*;
use std::ops::{Deref, DerefMut};

pub trait StoreMonotonicUpdate<Store: Collection> {
    fn update(&mut self, store: &mut Store, value: Store::Item) -> bool;
}

pub trait StoreRead<Store: Collection> {
    fn read(&self, store: &Store) -> Store::Item;
}

pub trait ViewDependencies<Event> {
    fn dependencies(&self, event: Event) -> Vec<(usize, Event)>;
}

impl<Store, R> StoreMonotonicUpdate<Store> for Box<R>
where
    R: StoreMonotonicUpdate<Store>,
    Store: Collection,
{
    fn update(&mut self, store: &mut Store, value: Store::Item) -> bool {
        self.deref_mut().update(store, value)
    }
}

impl<Store, R> StoreRead<Store> for Box<R>
where
    R: StoreRead<Store>,
    Store: Collection,
{
    fn read(&self, store: &Store) -> Store::Item {
        self.deref().read(store)
    }
}

impl<Event, R> ViewDependencies<Event> for Box<R>
where
    R: ViewDependencies<Event>,
{
    fn dependencies(&self, event: Event) -> Vec<(usize, Event)> {
        self.deref().dependencies(event)
    }
}
