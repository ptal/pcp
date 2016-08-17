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

#![allow(dead_code)]
grammar! pcp {
  // #![show_api]

  program = spacing statement_list eof

  expression = spacing store_placement eof

  statement_list = (statement semi_colon)+

  statement
    = store_let_binding > make_let_binding_stmt
    / store_placement > make_store_placement_stmt

  store_let_binding
    = let_kw identifier bind_op store_placement > make_let_binding

  store_placement = identifier left_arrow store_expression > make_store_placement

  store_expression
    = range > make_store_domain
    / constraint > make_store_constraint

  range = arith_expr dotdot arith_expr > make_pcp_range

  constraint
    = bin_constraint > make_store_bin_constraint
    / nary_constraint > make_store_nary_constraint

  nary_constraint = identifier lparen list_arith_expr rparen > make_nary_constraint

  bin_constraint = arith_expr cmp_op arith_expr > make_bin_constraint

  list_arith_expr = arith_expr (comma arith_expr)* > arith_expr_list

  arith_expr
    = term (term_op term)* > fold_left

  term
    = factor (factor_op factor)* > fold_left

  factor
    = integer > number_arith_expr
    / indexed_expr
    / identifier > variable_arith_expr
    / unary_arith_expr
    / lparen arith_expr rparen

  indexed_expr = identifier lbracket arith_expr rbracket > make_indexed_expr

  unary_arith_expr
    = unary_neg_op factor > make_unary_expr

  term_op
    = add_op > add_bin_op
    / sub_op > sub_bin_op

  factor_op
    = mul_op > mul_bin_op

  cmp_op
    = lt > lt_bin_op
    / le > le_bin_op
    / gt > gt_bin_op
    / ge > ge_bin_op
    / eq > eq_bin_op
    / neq > neq_bin_op

  identifier = !digit !keyword ident_char+ spacing > to_string
  ident_char = ["a-zA-Z0-9_"]

  keyword = "let"
  kw_tail = !ident_char spacing
  let_kw = "let" kw_tail

  underscore = "_"
  dotdot = ".." spacing
  semi_colon = ";" spacing
  comma = "," spacing
  left_arrow = "<-" spacing
  bind_op = "=" spacing
  add_op = "+" spacing
  sub_op = "-" spacing
  mul_op = "*" spacing
  lparen = "(" spacing
  rparen = ")" spacing
  lbracket = "[" spacing
  rbracket = "]" spacing

  lt = "<" spacing
  le = "<=" spacing
  gt = ">" spacing
  ge = ">=" spacing
  eq = "==" spacing
  neq = "!=" spacing

  spacing = [" \n\r\t"]* -> (^)
  eof = !.

  integer
    = number integer_suffix? spacing > make_integer

  unary_neg_op = sub_op > make_neg_unary_op

  number = digits > make_number
  digits = digit+ (underscore* digit)* > concat
  digit = ["0-9"]

  integer_suffix
    = "u8" > make_u8
    / "u16" > make_u16
    / "u32" > make_u32
    / "u64" > make_u64
    / "usize" > make_usize
    / "i8" > make_i8
    / "i16" > make_i16
    / "i32" > make_i32
    / "i64" > make_i64
    / "isize" > make_isize

  pub use syntax::ast::*;
  use std::str::FromStr;
  use self::ArithExpr::*;
  use self::BinArithOp::*;

  #[derive(Debug, Clone)]
  pub enum Statement {
    Local(LetBinding),
    Tell(StorePlacement)
  }

  #[derive(Debug, Clone)]
  pub struct LetBinding {
    pub var_name: String,
    pub store_placement: StorePlacement
  }

  #[derive(Debug, Clone)]
  pub struct StorePlacement {
    pub store_name: String,
    pub expr: StoreExpression
  }

  #[derive(Debug, Copy, Clone)]
  pub enum BinArithOp {
    Add, Sub, Mul
  }

  #[derive(Debug, Clone, Copy)]
  pub enum RelationalOp {
    Lt, Le, Gt, Ge, Eq, Neq
  }

  #[derive(Debug, Clone)]
  pub enum StoreExpression {
    Domain(Range),
    Constraint(Constraint)
  }

  #[derive(Debug, Clone)]
  pub enum Constraint {
    Binary(BinaryConstraint),
    Nary(NaryConstraint)
  }

  #[derive(Debug, Clone)]
  pub struct Range {
    pub min: AExpr,
    pub max: AExpr
  }

  #[derive(Debug, Clone)]
  pub struct NaryConstraint {
    pub name: String,
    pub args: Vec<AExpr>
  }

  #[derive(Debug, Clone)]
  pub struct BinaryConstraint {
    pub rel_op: RelationalOp,
    pub left: AExpr,
    pub right: AExpr
  }

  #[derive(Debug, Clone)]
  pub enum ArithExpr {
    Variable(String),
    Number(LitKind),
    IndexedExpr(String, AExpr),
    UnaryArithExpr(UnOp, AExpr),
    BinaryArithExpr(BinArithOp, AExpr, AExpr)
  }

  pub type AExpr = Box<ArithExpr>;

  fn number_arith_expr(value: LitKind) -> AExpr {
    Box::new(Number(value))
  }

  fn variable_arith_expr(ident: String) -> AExpr {
    Box::new(Variable(ident))
  }

  fn make_indexed_expr(ident: String, index: AExpr) -> AExpr {
    Box::new(IndexedExpr(ident, index))
  }

  fn fold_left(head: AExpr, rest: Vec<(BinArithOp, AExpr)>) -> AExpr {
    rest.into_iter().fold(head,
      |accu, (op, expr)| Box::new(BinaryArithExpr(op, accu, expr)))
  }

  fn fold_right(front: Vec<(AExpr, BinArithOp)>, last: AExpr) -> AExpr {
    front.into_iter().rev().fold(last,
      |accu, (expr, op)| Box::new(BinaryArithExpr(op, expr, accu)))
  }

  fn add_bin_op() -> BinArithOp { Add }
  fn sub_bin_op() -> BinArithOp { Sub }
  fn mul_bin_op() -> BinArithOp { Mul }

  fn concat(mut x: Vec<char>, y: Vec<char>) -> Vec<char> {
    x.extend(y.into_iter());
    x
  }

  fn to_string(raw_text: Vec<char>) -> String {
    raw_text.into_iter().collect()
  }

  fn make_u8() -> LitIntType { LitIntType::Unsigned(UintTy::U8) }
  fn make_u16() -> LitIntType { LitIntType::Unsigned(UintTy::U16) }
  fn make_u32() -> LitIntType { LitIntType::Unsigned(UintTy::U32) }
  fn make_u64() -> LitIntType { LitIntType::Unsigned(UintTy::U64) }
  fn make_usize() -> LitIntType { LitIntType::Unsigned(UintTy::Us) }
  fn make_i8() -> LitIntType { LitIntType::Signed(IntTy::I8) }
  fn make_i16() -> LitIntType { LitIntType::Signed(IntTy::I16) }
  fn make_i32() -> LitIntType { LitIntType::Signed(IntTy::I32) }
  fn make_i64() -> LitIntType { LitIntType::Signed(IntTy::I64) }
  fn make_isize() -> LitIntType { LitIntType::Signed(IntTy::Is) }

  fn make_neg_unary_op() -> UnOp { UnOp::Neg }

  fn make_integer(number: u64, suffix: Option<LitIntType>) -> LitKind {
    let ty = match suffix {
      None => LitIntType::Unsuffixed,
      Some(x) => x
    };
    LitKind::Int(number, ty)
  }

  fn make_number(raw_number: Vec<char>) -> u64 {
    match u64::from_str(&*to_string(raw_number)).ok() {
      Some(x) => x,
      None => panic!("int literal is too large")
    }
  }

  fn make_unary_expr(op: UnOp, expr: AExpr) -> AExpr {
    Box::new(UnaryArithExpr(op, expr))
  }

  fn make_pcp_range(min_bound: AExpr, max_bound: AExpr) -> Range {
    Range {
      min: min_bound,
      max: max_bound
    }
  }

  fn lt_bin_op() -> RelationalOp { RelationalOp::Lt }
  fn le_bin_op() -> RelationalOp { RelationalOp::Le }
  fn gt_bin_op() -> RelationalOp { RelationalOp::Gt }
  fn ge_bin_op() -> RelationalOp { RelationalOp::Ge }
  fn eq_bin_op() -> RelationalOp { RelationalOp::Eq }
  fn neq_bin_op() -> RelationalOp { RelationalOp::Neq }

  fn make_let_binding_stmt(let_binding: LetBinding) -> Statement {
    Statement::Local(let_binding)
  }

  fn make_store_placement_stmt(store: StorePlacement) -> Statement {
    Statement::Tell(store)
  }

  fn make_let_binding(var_name: String, store_placement: StorePlacement) -> LetBinding {
    LetBinding {
      var_name: var_name,
      store_placement: store_placement
    }
  }

  fn make_store_placement(store_name: String, expr: StoreExpression) -> StorePlacement {
    StorePlacement {
      store_name: store_name,
      expr: expr
    }
  }

  fn make_store_domain(range: Range) -> StoreExpression {
    StoreExpression::Domain(range)
  }

  fn make_store_constraint(constraint: Constraint) -> StoreExpression {
    StoreExpression::Constraint(constraint)
  }

  fn make_store_bin_constraint(bin_constraint: BinaryConstraint) -> Constraint {
    Constraint::Binary(bin_constraint)
  }

  fn make_store_nary_constraint(nary_constraint: NaryConstraint) -> Constraint {
    Constraint::Nary(nary_constraint)
  }

  fn make_bin_constraint(left: AExpr, rel_op: RelationalOp, right: AExpr) -> BinaryConstraint {
    BinaryConstraint {
      rel_op: rel_op,
      left: left,
      right: right
    }
  }

  fn arith_expr_list(x: AExpr, mut list: Vec<AExpr>) -> Vec<AExpr> {
    list.push(x);
    list
  }

  fn make_nary_constraint(name: String, args: Vec<AExpr>) -> NaryConstraint {
    NaryConstraint {
      name: name,
      args: args
    }
  }
}

#[cfg(test)]
mod test
{
  use oak_runtime::*;
  use oak_runtime::ParseResult::*;
  use super::*;

  #[test]
  fn test_grammar()
  {
    let state = pcp::recognize_program(
      "let x = variables <- 9i32 .. 100;
      constraints <- x*1 > y + (z - 9);
      let y = variables <- 0..0;".into_state());
    match state.into_result() {
      Success(_) => (),
      Partial(_, _)
    | Failure(_) => assert!(false)
    };
  }
}
