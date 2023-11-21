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
use num::traits::Unsigned;
use num::Integer;
use search::branching::*;
use search::space::*;
use variable::ops::Iterable;

pub struct InputOrder;

impl<VStore, CStore, R, Domain, Size> VarSelection<Space<VStore, CStore, R>> for InputOrder
where
    VStore: Iterable<Item = Domain>,
    Domain: Cardinality<Size = Size>,
    Size: Ord + Unsigned + Integer,
{
    fn select(&mut self, space: &Space<VStore, CStore, R>) -> usize {
        space
            .vstore
            .iter()
            .enumerate()
            .filter(|&(_, v)| v.size() > Size::one())
            .next()
            .expect("Cannot select a variable in a space where all variables are assigned.")
            .0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use search::branching::first_smallest_var::test::test_selector;

    #[test]
    fn smallest_var_selection() {
        test_selector(InputOrder, vec![(1, 10), (2, 4), (1, 1)], 0);
        test_selector(InputOrder, vec![(1, 10), (2, 4), (2, 4)], 0);
        test_selector(
            InputOrder,
            vec![(1, 1), (1, 1), (1, 10), (1, 1), (2, 4), (1, 1), (1, 1)],
            2,
        );
    }

    #[should_panic]
    #[test]
    fn smallest_var_selection_all_assigned() {
        test_selector(InputOrder, vec![(0, 0), (2, 2), (1, 1)], 0);
    }
}
