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

use kernel::*;

// A branch represents an edge between two distinct nodes in the search tree.
// Each branch store a copy of the label of the current node.
// However, depending on the `clone` method of the `Space::Label` type,
// several branches can share the same data until calling `restore`.
// We don't store the propagators but instead a closure that
// add the propagator(s) to the new space, when available.

pub struct Branch<Space>
where
    Space: Freeze,
{
    label: <Space::FrozenState as Snapshot>::Label,
    alternative: Box<dyn Fn(&mut Space)>,
}

impl<Space> Branch<Space>
where
    Space: Freeze,
{
    pub fn distribute(
        space: Space,
        alternatives: Vec<Box<dyn Fn(&mut Space)>>,
    ) -> (Space::FrozenState, Vec<Branch<Space>>) {
        let mut immutable_space = space.freeze();
        let branches = alternatives
            .into_iter()
            .map(|alt| Branch {
                label: immutable_space.label(),
                alternative: alt,
            })
            .collect();
        (immutable_space, branches)
    }

    pub fn commit(self, space_from: Space::FrozenState) -> Space {
        let mut new = space_from.restore(self.label);
        (self.alternative)(&mut new);
        new
    }
}
