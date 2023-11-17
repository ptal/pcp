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

pub trait Freeze: Sized {
    type FrozenState: Snapshot<State = Self>;
    fn freeze(self) -> Self::FrozenState;
}

pub trait Snapshot: Sized {
    type Label;
    type State: Freeze<FrozenState = Self>;

    fn label(&mut self) -> Self::Label;
    fn restore(self, label: Self::Label) -> Self::State;
    fn unfreeze(mut self) -> Self::State {
        let label = self.label();
        self.restore(label)
    }
}
