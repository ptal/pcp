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

use gcollections::ops::*;
use kernel::event::*;
use propagation::Reactor;
use std::fmt::{Debug, Error, Formatter};
use std::iter::{repeat, FromIterator};

/// `deps[num_events*v + e]` contains the propagators dependent to the event `e` on the variable `v`.
#[derive(Clone)]
pub struct IndexedDeps {
    num_events: usize,
    num_subscriptions: usize,
    deps: Vec<Vec<usize>>,
}

impl IndexedDeps {
    fn index_of<E>(&self, var: usize, ev: E) -> usize
    where
        E: EventIndex,
    {
        self.num_events * var + ev.to_index()
    }

    fn deps_of_mut<E>(&mut self, var: usize, ev: E) -> &mut Vec<usize>
    where
        E: EventIndex,
    {
        let idx = self.index_of(var, ev);
        &mut self.deps[idx]
    }

    fn num_vars(&self) -> usize {
        self.deps.len() / self.num_events
    }

    fn assert_var_idx(&self, var: usize, op: &str) {
        assert!(var < self.num_vars(),
      "Reactor IndexedDeps has been initialized for {} variables but `{}` of the variable {} was requested.",
        self.num_vars(), op, var);
    }
}

impl Reactor for IndexedDeps {
    fn new(num_vars: usize, num_events: usize) -> IndexedDeps {
        IndexedDeps {
            num_events,
            num_subscriptions: 0,
            deps: FromIterator::from_iter(repeat(vec![]).take(num_vars * num_events)),
        }
    }

    fn subscribe<E>(&mut self, var: usize, ev: E, prop: usize)
    where
        E: EventIndex,
    {
        assert!(
            self.deps
                .iter()
                .skip(var * self.num_events)
                .take(self.num_events)
                .flat_map(|x| x.iter())
                .all(|&x| x != prop),
            "propagator already subscribed to this variable"
        );
        self.assert_var_idx(var, "subscription");
        self.num_subscriptions += 1;
        let props = self.deps_of_mut(var, ev);
        props.push(prop);
    }

    fn unsubscribe<E>(&mut self, var: usize, ev: E, prop: usize)
    where
        E: EventIndex,
    {
        self.assert_var_idx(var, "unsubscription");
        self.num_subscriptions -= 1;
        let props = self.deps_of_mut(var, ev);
        let idx = props.iter().position(|&v| v == prop);
        assert!(
            idx.is_some(),
            "cannot unsubscribe propagator not registered."
        );
        props.swap_remove(idx.unwrap());
    }

    fn react<E>(&self, var: usize, ev: E) -> Vec<usize>
    where
        E: EventIndex,
    {
        self.assert_var_idx(var, "react");
        let from = self.index_of(var, ev);
        let len = self.num_events - ev.to_index();
        self.deps
            .iter()
            .skip(from)
            .take(len)
            .flat_map(|x| x.iter())
            .cloned()
            .collect()
    }
}

impl Cardinality for IndexedDeps {
    type Size = usize;
    fn size(&self) -> usize {
        self.num_subscriptions
    }
}

impl Debug for IndexedDeps {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        for prop in self.deps.iter().flat_map(|props| props.iter()) {
            formatter.write_fmt(format_args!("{} ", prop))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use kernel::event::EventIndex;
    use propagation::events::FDEvent;
    use propagation::events::FDEvent::*;
    use propagation::Reactor;

    fn make_reactor() -> IndexedDeps {
        Reactor::new(3, <FDEvent as EventIndex>::size())
    }

    fn iter_deps(reactor: &mut IndexedDeps, var: usize, ev: FDEvent, expect: Vec<usize>) {
        let mut it = reactor.react(var, ev).into_iter();
        let mut it_e = expect.into_iter();
        loop {
            let next = it.next();
            let next_e = it_e.next();
            if next.is_none() || next_e.is_none() {
                break;
            }
            assert_eq!(next.unwrap(), next_e.unwrap());
        }
        assert_eq!(it.next(), None);
        assert_eq!(it_e.next(), None);
    }

    #[test]
    fn subscribe_test() {
        let reactor = &mut make_reactor();
        assert!(reactor.is_empty());
        reactor.subscribe(0, Assignment, 4);
        assert!(!reactor.is_empty());

        iter_deps(reactor, 0, Assignment, vec![4]);

        // Assignment is more precise than Inner, so we don't care.
        iter_deps(reactor, 0, Inner, vec![]);

        // If we subscribe to Inner, we must react on Inner
        // or more precise events.
        reactor.subscribe(0, Inner, 5);
        iter_deps(reactor, 0, Inner, vec![5]);
        iter_deps(reactor, 0, Assignment, vec![4, 5]);

        reactor.subscribe(1, Bound, 6);
        reactor.subscribe(1, Assignment, 7);
        iter_deps(reactor, 1, Inner, vec![]);
        iter_deps(reactor, 1, Bound, vec![6]);
        iter_deps(reactor, 1, Assignment, vec![7, 6]);

        iter_deps(reactor, 2, Assignment, vec![]);

        reactor.subscribe(2, Assignment, 8);
        iter_deps(reactor, 1, Inner, vec![]);
    }

    #[test]
    fn unsubscribe_test() {
        let reactor = &mut make_reactor();
        reactor.subscribe(0, Assignment, 4);
        reactor.subscribe(0, Bound, 5);

        iter_deps(reactor, 0, Assignment, vec![4, 5]);

        reactor.unsubscribe(0, Bound, 5);
        iter_deps(reactor, 0, Assignment, vec![4]);

        reactor.unsubscribe(0, Assignment, 4);
        iter_deps(reactor, 0, Assignment, vec![]);

        reactor.subscribe(0, Assignment, 4);
        iter_deps(reactor, 0, Assignment, vec![4]);
    }

    #[test]
    #[should_panic]
    fn subscribe_fail_test() {
        let mut reactor = make_reactor();

        reactor.subscribe(0, Assignment, 0);
        reactor.subscribe(0, Assignment, 0);
    }

    #[test]
    #[should_panic]
    fn unsubscribe_fail_test() {
        let mut reactor = make_reactor();

        reactor.unsubscribe(0, Assignment, 0);
    }

    #[test]
    #[should_panic]
    fn subscribe_two_events_fail_test() {
        let mut reactor = make_reactor();

        reactor.subscribe(0, Assignment, 0);
        reactor.subscribe(0, Bound, 0); // already subscribed to this variable
    }
}
