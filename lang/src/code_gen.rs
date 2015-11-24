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
use rust::AstBuilder;
use grammar::*;
use oak_runtime::*;
use ama::compiler::*;

type RBlock = rust::P<rust::Block>;
type RStmt = rust::P<rust::Stmt>;
type RExpr = rust::P<rust::Expr>;

pub struct CodeGenerator<'cx>
{
  cx: &'cx rust::ExtCtxt<'cx>
}

impl<'cx> Compiler for CodeGenerator<'cx>
{
  fn compile_expr(&mut self, unquote: Unquote) -> RExpr {
    let state = pcp::parse_expression(unquote.code.stream());
    match state.into_result() {
      Ok((success, _)) => {
        self.gen_placement_store(&unquote, success.data)
      }
      Err(error) => {
        self.cx.span_err(unquote.span, format!("{}", error).as_str());
        quote_expr!(self.cx, ())
      }
    }
  }

  fn compile_block(&mut self, unquote: Unquote) -> RBlock {
    let state = pcp::parse_program(unquote.code.stream());
    match state.into_result() {
      Ok((success, _)) => {
        self.gen_statements(&unquote, success.data)
      }
      Err(error) => {
        self.cx.span_err(unquote.span, format!("{}", error).as_str());
        self.cx.block(unquote.span, vec![], None)
      }
    }
  }
}

impl<'cx> CodeGenerator<'cx>
{
  pub fn new(cx: &'cx rust::ExtCtxt) -> CodeGenerator<'cx> {
    CodeGenerator {
      cx: cx
    }
  }

  fn gen_placement_store(&self, unquote: &Unquote, store: pcp::StorePlacement) -> RExpr {
    let store_name = unquote.text_to_ident[&store.store_name];
    match store.expr {
      pcp::StoreExpression::Domain(range) => {
        let domain = self.gen_domain(unquote, range);
        quote_expr!(self.cx, $store_name.alloc($domain))
      }
      pcp::StoreExpression::Constraint(constraint) => {
        let constraint = self.gen_constraint(unquote, constraint);
        quote_expr!(self.cx, $store_name.alloc($constraint))
      }
    }
  }

  fn gen_domain(&self, unquote: &Unquote, range: pcp::Range) -> RExpr {
    let min = self.gen_arith_expr(unquote, range.min);
    let max = self.gen_arith_expr(unquote, range.max);
    quote_expr!(self.cx, Interval::new($min, $max))
  }

  fn gen_arith_expr(&self, unquote: &Unquote, arith_expr: pcp::AExpr) -> RExpr {
    match *arith_expr {
      pcp::ArithExpr::Variable(var) => self.gen_ident(unquote, var),
      pcp::ArithExpr::Number(n) => self.cx.expr_lit(unquote.span, n),
      x => panic!(format!("gen_arith_expr: {:?}: Not implemented", x))
    }
  }

  fn gen_ident(&self, unquote: &Unquote, var: String) -> RExpr {
    let var = unquote.text_to_ident[&var];
    quote_expr!(self.cx, $var)
  }

  fn gen_statements(&self, unquote: &Unquote, statements: Vec<pcp::Statement>) -> RBlock {
    let rust_stmt = statements.into_iter()
      .map(|stmt| self.gen_statement(unquote, stmt)).collect();
    self.cx.block(unquote.span, rust_stmt, None)
  }

  fn gen_statement(&self, unquote: &Unquote, statement: pcp::Statement) -> RStmt {
    use grammar::pcp::Statement::*;
    match statement {
      Local(_) => panic!("gen_statement: Let binding: Not implemented"),
      Tell(store) => {
        let expr = self.gen_placement_store(unquote, store);
        self.cx.stmt_expr(expr)
      }
    }
  }

  fn gen_constraint(&self, unquote: &Unquote, constraint: pcp::Constraint) -> RExpr {
    use grammar::pcp::Constraint::*;
    match constraint {
      Binary(constraint) => {
        self.gen_binary_constraint(unquote, constraint)
      }
      Nary(constraint) => {
        self.gen_nary_constraint(unquote, constraint)
      }
    }
  }

  fn gen_binary_constraint(&self, unquote: &Unquote, constraint: pcp::BinaryConstraint) -> RExpr {
    use grammar::pcp::RelationalOp::*;
    let x = self.gen_var_view(unquote, constraint.left);
    let y = self.gen_var_view(unquote, constraint.right);
    match constraint.rel_op {
      Lt => quote_expr!(self.cx, XLessY::new($x, $y)),
      Le => quote_expr!(self.cx, x_leq_y($x, $y)),
      Gt => quote_expr!(self.cx, x_greater_y($x, $y)),
      Ge => quote_expr!(self.cx, x_geq_y($x, $y)),
      Eq => quote_expr!(self.cx, XEqY::new($x, $y)),
      Neq => quote_expr!(self.cx, XNeqY::new($x, $y)),
    }
  }

  fn gen_var_view(&self, unquote: &Unquote, arith_expr: pcp::AExpr) -> RExpr {
    use grammar::pcp::ArithExpr::*;
    match *arith_expr {
      Variable(var) => self.gen_ident(unquote, var),
      Number(n) => {
        let lit = self.cx.expr_lit(unquote.span, n);
        quote_expr!(self.cx, Constant::new($lit))
      },
      SignedArithExpr(..) => {
        panic!("gen_var_view: SignedArithExpr: unimplemented.")
      }
      BinaryArithExpr(op, ref x, ref y) => {
        self.gen_bin_arith_expr(unquote, op, x.clone(), y.clone())
      }
    }
  }

  fn gen_bin_arith_expr(&self, unquote: &Unquote, op: pcp::BinArithOp,
    x: pcp::AExpr, y: pcp::AExpr) -> RExpr
  {
    use grammar::pcp::BinArithOp::*;
    let x = self.gen_var_view(unquote, x);
    let y = self.gen_var_view(unquote, y);
    match op {
      Add => quote_expr!(self.cx, Addition::new($x, $y)),
      Sub => panic!("gen_bin_arith_expr: Sub: unimplemented."),
      Mul => panic!("gen_bin_arith_expr: Mul: unimplemented.")
    }
  }

  fn gen_nary_constraint(&self, unquote: &Unquote, constraint: pcp::NaryConstraint) -> RExpr {
    let fun_name = self.cx.ident_of(constraint.name.as_str());
    let args: Vec<RExpr> = constraint.args.into_iter()
      .map(|arg| self.gen_var_view(unquote, arg))
      .collect();
    quote_expr!(self.cx, $fun_name::new($args))
  }
}
