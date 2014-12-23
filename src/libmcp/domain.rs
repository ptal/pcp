// Copyright 2014 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Code inspired by the monadiccp Haskell library.
// credits to Control/CP/FD/OvertonFD/Domain.hs

use std::collections::BTreeSet;
use domain::Domain::*;
use std::iter::range_inclusive;
use std::cmp::{min, max};

#[deriving(PartialEq, Eq, Clone)]
pub enum Domain {
  Set(BTreeSet<int>),
  Range(int, int)
}

impl Domain {
  pub fn size(&self) -> uint {
    match self {
      &Set(ref ts) => ts.len(),
      &Range(l, u) => (u - l + 1) as uint
    }
  }

  pub fn member(&self, n: int) -> bool {
    match self {
      &Set(ref ts) => ts.contains(&n),
      &Range(l, u) => n >= l && n <= u
    }
  }

  pub fn is_subset_of(&self, d: &Domain) -> bool {
    match (self, d) {
      (&Set(ref ts), &Set(ref ts2)) => ts.is_subset(ts2),
      (&Range(l, u), &Range(l2,u2)) => l >= l2 && u <= u2,
      (&Set(ref ts), r) => // (min, max) inside r.
        Range(ts.iter().next().unwrap().clone(),
              ts.iter().rev().next().unwrap().clone()).is_subset_of(r),
      (&Range(l, u), &Set(ref ts)) =>
        range_inclusive(l, u).all(|i| ts.contains(&i))
    }
  }

  pub fn intersection(&self, d: &Domain) -> Domain {
    match (self, d) {
      (&Set(ref ts), &Set(ref ts2)) => {
        let inter: BTreeSet<int> = ts.intersection(ts2).cloned().collect();
        inter.to_domain()
      }
      (&Range(l, u), &Range(l2, u2)) => Range(max(l, l2), min(u, u2)),
      (&Set(ref ts), ref r) => {
        let inter: BTreeSet<int> = ts.clone().into_iter()
          .filter(|&x| r.member(x)).collect();
        inter.to_domain()
      }
      (r, s) => s.intersection(r)
    }
  }
}

trait ToDomain {
  fn to_domain(self) -> Domain;
}

impl ToDomain for Domain {
  fn to_domain(self) -> Domain { self }
}

impl ToDomain for BTreeSet<int> {
  fn to_domain(self) -> Domain {
    Set(self)
  }
}

impl ToDomain for (int, int) {
  fn to_domain(self) -> Domain {
    let (a, b) = self;
    Range(a, b)
  }
}

impl ToDomain for int {
  fn to_domain(self) -> Domain {
    Range(self.clone(), self)
  }
}

impl ToDomain for Vec<int> {
  fn to_domain(self) -> Domain {
    Set(FromIterator::from_iter(self.into_iter()))
  }
}

#[test]
fn to_domain_id_test() {
  let d1 = Range(1, 2);
  let d1_id = d1.clone().to_domain();
  assert!(d1 == d1_id);
  assert!(d1 == Range(1, 2));
  assert!(d1.size() == 2);
}

#[test]
fn member_test() {
  let d1 = Range(5, 50);
  assert!(d1.member(5));
  assert!(d1.member(50));
  assert!(d1.member(30));

  let d2 = vec![1,4,8,19,25].to_domain();
  assert!(d2.member(1));
  assert!(d2.member(19));
}

#[test]
fn is_subset_of_test() {
  let d1 = Range(2, 10);
  let d2 = Range(2, 10);
  assert!(d1.is_subset_of(&d2));
  assert!(d2.is_subset_of(&d1));
  let d3 = Range(3, 10);
  assert!(d3.is_subset_of(&d1));
  assert!(!d2.is_subset_of(&d3));
  assert!(!Range(3,12).is_subset_of(&d1));
  assert!(!Range(0, 9).is_subset_of(&d1));

  let d4 = vec![4,9,3,2].to_domain();
  let d5 = d4.clone();
  assert!(d4.is_subset_of(&d5));
  assert!(d5.is_subset_of(&d4));

  assert!(d4.is_subset_of(&d1));
  assert!(!d1.is_subset_of(&d4));

  let d6 = Range(2,4);
  assert!(d6.is_subset_of(&d4));
  assert!(!d4.is_subset_of(&d6));

  let d7 = vec![2,9,3].to_domain();
  assert!(d7.is_subset_of(&d4));
  assert!(!d4.is_subset_of(&d7));
}

#[test]
fn intersection_test() {
  let d1 = Range(2, 10);
  let d2 = Range(2, 10);
  assert!(d1.intersection(&d2) == d1);
  let d3 = Range(3, 13);
  assert!(d3.intersection(&d1) == Range(3, 10));
  let d4 = Range(0, 9);
  assert!(d4.intersection(&d1) == Range(2, 9));

  let d5 = vec![3,4,7,8].to_domain();
  assert!(d5.intersection(&d5.clone()) == d5);

  let d6 = vec![1,2,3,10].to_domain();
  assert!(d5.intersection(&d6) == vec![3].to_domain());
  assert!(d6.intersection(&d5) == vec![3].to_domain());

  assert!(d1.intersection(&d5) == d5);
  assert!(d5.intersection(&d1) == d5);
  assert!(d6.intersection(&d4) == vec![1,2,3].to_domain());
  assert!(d4.intersection(&d6) == vec![1,2,3].to_domain());
}
