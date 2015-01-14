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

use std::collections::RingBuf;
use std::iter::repeat;
use std::iter::FromIterator;

pub trait Agenda {
  fn new(capacity: usize) -> Self;
  fn schedule(&mut self, idx: usize);
  fn unschedule(&mut self, idx: usize);
  fn pop(&mut self) -> Option<usize>;
  fn is_empty(&self) -> bool;
}

pub struct RelaxedFifoAgenda {
  inside_queue: Vec<bool>,
  queue: RingBuf<usize>
}

impl Agenda for RelaxedFifoAgenda {
  fn new(capacity: usize) -> RelaxedFifoAgenda {
    RelaxedFifoAgenda {
      inside_queue: FromIterator::from_iter(repeat(false).take(capacity)),
      queue: RingBuf::with_capacity(capacity)
    }
  }

  fn schedule(&mut self, idx: usize) {
    assert!((idx as usize) < self.inside_queue.len());
    if !self.inside_queue[idx] {
      self.inside_queue[idx] = true;
      self.queue.push_back(idx);
    }
  }

  fn unschedule(&mut self, idx: usize) {
    assert!((idx as usize) < self.inside_queue.len());
    if self.inside_queue[idx] {
      let queue_idx = self.queue.iter().position(|&e| e == idx);
      assert!(queue_idx.is_some());
      self.queue.swap_front_remove(queue_idx.unwrap());
      self.inside_queue[idx] = false;
    }
  }

  fn pop(&mut self) -> Option<usize> {
    let res = self.queue.pop_front();
    if res.is_some() { self.inside_queue[res.unwrap()] = false; }
    res
  }

  fn is_empty(&self) -> bool {
    self.queue.is_empty()
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn schedule_test() {
    let mut sched: RelaxedFifoAgenda = Agenda::new(3);
    sched_21(&mut sched);
    assert_eq!(sched.pop(), Some(2));
    pop_1(&mut sched);

    sched.schedule(1);
    sched.schedule(1);
    pop_1(&mut sched);
  }

  #[test]
  fn unschedule_test() {
    let mut sched: RelaxedFifoAgenda = Agenda::new(3);
    sched_21(&mut sched);
    sched.unschedule(1);
    assert_eq!(sched.pop(), Some(2));
    assert_eq!(sched.pop(), None);

    sched_21(&mut sched);
    sched.unschedule(2);
    pop_1(&mut sched);

    sched_21(&mut sched);
    sched.unschedule(2);
    sched.unschedule(2);
    pop_1(&mut sched);
  }

  fn sched_21(sched: &mut RelaxedFifoAgenda) {
    sched.schedule(2);
    sched.schedule(1);
  }

  fn pop_1(sched: &mut RelaxedFifoAgenda) {
    assert_eq!(sched.is_empty(), false);
    assert_eq!(sched.pop(), Some(1));
    assert_eq!(sched.pop(), None);
    assert_eq!(sched.is_empty(), true);
  }

  #[test]
  #[should_fail]
  fn schedule_outofbound() {
    let mut sched: RelaxedFifoAgenda = Agenda::new(3);
    sched.schedule(3);
  }

  #[test]
  #[should_fail]
  fn unschedule_outofbound() {
    let mut sched: RelaxedFifoAgenda = Agenda::new(3);
    sched.unschedule(3);
  }
}
