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
use rust::{Ident, Span, AstBuilder};
use grammar::*;
use oak_runtime::*;

use std::collections::hash_map::HashMap;

pub type RTy = rust::P<rust::Ty>;
pub type RExpr = rust::P<rust::Expr>;
pub type RItem = rust::P<rust::Item>;

pub struct CodeGenerator<'cx>
{
  cx: &'cx rust::ExtCtxt<'cx>,
  text_to_ident: HashMap<String, Ident>,
  span: Span
}

impl<'cx> CodeGenerator<'cx>
{
  pub fn new(cx: &'cx rust::ExtCtxt, text_to_ident: HashMap<String, Ident>, span: Span)
    -> CodeGenerator<'cx>
  {
    CodeGenerator {
      cx: cx,
      text_to_ident: text_to_ident,
      span: span
    }
  }

  pub fn generate_expr(&self, code: String) -> RExpr {
    let state = pcp::parse_expression(code.stream());
    match state.into_result() {
      Ok((success, _)) => {
        self.generate_placement_store(success.data)
      }
      Err(error) => {
        self.cx.span_err(self.span, format!("{}", error).as_str());
        quote_expr!(self.cx, ())
      }
    }
  }

  fn generate_placement_store(&self, store: pcp::StorePlacement) -> RExpr {
    let store_name = self.text_to_ident[&store.store_name];
    match store.expr {
      pcp::StoreExpression::Domain(range) => {
        let min = self.generate_arith_expr(range.min);
        let max = self.generate_arith_expr(range.max);
        quote_expr!(self.cx, $store_name.alloc(Interval::new($min, $max)))
      }
      x => panic!(format!("generate_placement_store: {:?}: Not implemented", x))
    }
  }

  fn generate_arith_expr(&self, arith_expr: pcp::AExpr) -> RExpr {
    match *arith_expr {
      pcp::ArithExpr::Variable(var) => {
        let var = self.text_to_ident[&var];
        quote_expr!(self.cx, $var)
      }
      pcp::ArithExpr::Number(n) => {
        self.cx.expr_lit(self.span, n)
      }
      x => panic!(format!("generate_arith_expr: {:?}: Not implemented", x))
    }
  }
}
