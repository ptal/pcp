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

use gcollections::*;
use gcollections::ops::*;
use interval::ops::Range;
use num::{Signed, Integer};
use std::ops::*;
use std::fmt::Debug;


pub trait IntBound:
  Integer + Clone + Debug
  + Signed // Due to the lack of Subtraction in term/
{}

impl<R> IntBound for R where
  R: Integer + Clone + Debug
  + Signed
{}

pub trait IntDomain:
  Bounded + Cardinality + Empty + IsEmpty + Singleton + IsSingleton + Range +
  ShrinkLeft + ShrinkRight + StrictShrinkLeft + StrictShrinkRight +
  Difference<<Self as Collection>::Item, Output=Self> +
  Intersection<Output=Self> + Overlap + Subset + Disjoint +
  Add<<Self as Collection>::Item, Output=Self> + Sub<<Self as Collection>::Item, Output=Self> +
  Add<Output=Self> + Sub<Output=Self> + Mul<Output=Self> +
  Clone + Debug
where
  <Self as Collection>::Item: IntBound
{}

impl<R> IntDomain for R where
  R: Bounded + Cardinality + Empty + IsEmpty + Singleton + IsSingleton + Range,
  R: ShrinkLeft + ShrinkRight + StrictShrinkLeft + StrictShrinkRight,
  R: Difference<<R as Collection>::Item, Output=R>,
  R: Intersection<Output=R> + Overlap + Subset + Disjoint,
  R: Add<<R as Collection>::Item, Output=R> + Sub<<R as Collection>::Item, Output=R>,
  R: Add<Output=R> + Sub<Output=R> + Mul<Output=R>,
  R: Clone + Debug,
  <R as Collection>::Item: IntBound
{}
