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

use rust;
use grammar::*;
use oak_runtime::*;

pub type RTy = rust::P<rust::Ty>;
pub type RExpr = rust::P<rust::Expr>;
pub type RItem = rust::P<rust::Item>;

pub struct CodeGenerator<'cx>
{
  cx: &'cx rust::ExtCtxt<'cx>
}

impl<'cx> CodeGenerator<'cx>
{
  pub fn new(cx: &'cx rust::ExtCtxt) -> CodeGenerator<'cx> {
    CodeGenerator {
      cx: cx
    }
  }

  pub fn generate_expr(&self, code: String) -> RExpr {
    let state = pcp::parse_store_placement(code.stream());
    quote_expr!(self.cx, {})
  }
}
