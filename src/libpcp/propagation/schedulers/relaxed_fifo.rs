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

use propagation::Scheduler;
use std::collections::VecDeque;
use std::iter::repeat;
use std::iter::FromIterator;

// It is a "relaxed FIFO" because the unschedule operation
// might not preserve the ordering. However, this operation
// is not call as often as pop, hence the ordering shouldn't
// be too much modified. Anyways, we don't care, the unschedule
// operation is always a good news, it is called when a
// propagator is entailed, so the problem is less important.

pub struct RelaxedFifo {
  inside_queue: Vec<bool>,
  queue: VecDeque<usize>
}

impl Scheduler for RelaxedFifo {
  fn new(capacity: usize) -> RelaxedFifo {
    RelaxedFifo {
      inside_queue: FromIterator::from_iter(repeat(false).take(capacity)),
      queue: VecDeque::with_capacity(capacity)
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
      self.queue.swap_remove_front(queue_idx.unwrap());
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
  use propagation::Scheduler;

  #[test]
  fn schedule_test() {
    let mut scheduler: RelaxedFifo = Scheduler::new(3);
    schedule_21(&mut scheduler);
    assert_eq!(scheduler.pop(), Some(2));
    pop_1(&mut scheduler);

    scheduler.schedule(1);
    scheduler.schedule(1);
    pop_1(&mut scheduler);
  }

  #[test]
  fn unschedule_test() {
    let mut scheduler: RelaxedFifo = Scheduler::new(3);
    schedule_21(&mut scheduler);
    scheduler.unschedule(1);
    assert_eq!(scheduler.pop(), Some(2));
    assert_eq!(scheduler.pop(), None);

    schedule_21(&mut scheduler);
    scheduler.unschedule(2);
    pop_1(&mut scheduler);

    schedule_21(&mut scheduler);
    scheduler.unschedule(2);
    scheduler.unschedule(2);
    pop_1(&mut scheduler);
  }

  fn schedule_21(scheduler: &mut RelaxedFifo) {
    scheduler.schedule(2);
    scheduler.schedule(1);
  }

  fn pop_1(scheduler: &mut RelaxedFifo) {
    assert_eq!(scheduler.is_empty(), false);
    assert_eq!(scheduler.pop(), Some(1));
    assert_eq!(scheduler.pop(), None);
    assert_eq!(scheduler.is_empty(), true);
  }

  #[test]
  #[should_panic]
  fn schedule_outofbound() {
    let mut scheduler: RelaxedFifo = Scheduler::new(3);
    scheduler.schedule(3);
  }

  #[test]
  #[should_panic]
  fn unschedule_outofbound() {
    let mut scheduler: RelaxedFifo = Scheduler::new(3);
    scheduler.unschedule(3);
  }
}
