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

pub mod expr_inference;
pub mod identity;
pub mod constant;
pub mod addition;
pub mod ops;

pub use variable::arithmetics::expr_inference::ExprInference;
pub use variable::arithmetics::identity::Identity;
pub use variable::arithmetics::constant::Constant;
pub use variable::arithmetics::addition::Addition;
