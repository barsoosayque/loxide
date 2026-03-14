use loxide::{
    ast::{Expr, ExprKind, Stmt, StmtKind},
    interpreter::{Interpreter, LoxValue},
    source::SourceSpan,
    token::TokenKind,
};

fn expr(kind: ExprKind<'static>) -> Expr<'static> {
    Expr::new(
        kind,
        SourceSpan {
            line: 0,
            char_range: 0..=0,
            bytes_range: 0..=0,
        },
    )
}

fn stmt(kind: StmtKind<'static>) -> Stmt<'static> {
    Stmt::new(
        kind,
        SourceSpan {
            line: 0,
            char_range: 0..=0,
            bytes_range: 0..=0,
        },
    )
}

fn num(n: f64) -> Expr<'static> {
    expr(ExprKind::LitNumber(n))
}

fn string(s: &'static str) -> Expr<'static> {
    expr(ExprKind::LitString(s))
}

fn boolean(b: bool) -> Expr<'static> {
    expr(ExprKind::LitBoolean(b))
}

fn nil() -> Expr<'static> {
    expr(ExprKind::LitNil)
}

fn unary(op: TokenKind<'static>, right: Expr<'static>) -> Expr<'static> {
    expr(ExprKind::Unary {
        op,
        right: Box::new(right),
    })
}

fn binary(left: Expr<'static>, op: TokenKind<'static>, right: Expr<'static>) -> Expr<'static> {
    expr(ExprKind::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

fn grouping(inner: Expr<'static>) -> Expr<'static> {
    expr(ExprKind::Grouping {
        inner: Box::new(inner),
    })
}

fn interpret_expr(expr: Expr<'static>) -> LoxValue<'static> {
    let stmt = stmt(StmtKind::ExprReturn(Box::new(expr)));
    Interpreter::execute_ast([stmt], "").unwrap()
}

#[test]
fn interpret_literals() {
    let value = interpret_expr(num(42.0));
    assert!(matches!(value, LoxValue::Number(42.0)));

    let value = interpret_expr(string("hello"));
    assert!(matches!(value, LoxValue::String(s) if s == "hello"));

    let value = interpret_expr(boolean(true));
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret_expr(boolean(false));
    assert!(matches!(value, LoxValue::Boolean(false)));

    let value = interpret_expr(nil());
    assert!(matches!(value, LoxValue::Nil));
}

#[test]
fn interpret_unary() {
    let value = interpret_expr(unary(TokenKind::Minus, num(5.0)));
    assert!(matches!(value, LoxValue::Number(-5.0)));

    let value = interpret_expr(unary(TokenKind::Bang, boolean(true)));
    assert!(matches!(value, LoxValue::Boolean(false)));

    let value = interpret_expr(unary(TokenKind::Bang, boolean(false)));
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret_expr(unary(TokenKind::Bang, nil()));
    assert!(matches!(value, LoxValue::Boolean(true)));
}

#[test]
fn interpret_binary_arithmetic() {
    let value = interpret_expr(binary(num(1.0), TokenKind::Plus, num(2.0)));
    assert!(matches!(value, LoxValue::Number(3.0)));

    let value = interpret_expr(binary(num(5.0), TokenKind::Minus, num(3.0)));
    assert!(matches!(value, LoxValue::Number(2.0)));

    let value = interpret_expr(binary(num(2.0), TokenKind::Star, num(3.0)));
    assert!(matches!(value, LoxValue::Number(6.0)));

    let value = interpret_expr(binary(num(6.0), TokenKind::Slash, num(2.0)));
    assert!(matches!(value, LoxValue::Number(3.0)));
}

#[test]
fn interpret_binary_comparison() {
    let value = interpret_expr(binary(num(5.0), TokenKind::Greater, num(3.0)));
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret_expr(binary(num(3.0), TokenKind::Greater, num(5.0)));
    assert!(matches!(value, LoxValue::Boolean(false)));

    let value = interpret_expr(binary(num(3.0), TokenKind::Less, num(5.0)));
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret_expr(binary(num(5.0), TokenKind::GreaterEqual, num(5.0)));
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret_expr(binary(num(5.0), TokenKind::LessEqual, num(5.0)));
    assert!(matches!(value, LoxValue::Boolean(true)));
}

#[test]
fn interpret_binary_equality() {
    let value = interpret_expr(binary(num(5.0), TokenKind::EqualEqual, num(5.0)));
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret_expr(binary(num(5.0), TokenKind::EqualEqual, num(3.0)));
    assert!(matches!(value, LoxValue::Boolean(false)));

    let value = interpret_expr(binary(num(5.0), TokenKind::BangEqual, num(3.0)));
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret_expr(binary(boolean(true), TokenKind::EqualEqual, boolean(true)));
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret_expr(binary(nil(), TokenKind::EqualEqual, nil()));
    assert!(matches!(value, LoxValue::Boolean(true)));
}

#[test]
fn interpret_string_concatenation() {
    let value = interpret_expr(binary(string("hello"), TokenKind::Plus, string("world")));
    assert!(matches!(value, LoxValue::String(s) if s == "helloworld"));
}

#[test]
fn interpret_chained_operations() {
    let value = interpret_expr(binary(
        num(1.0),
        TokenKind::Plus,
        binary(num(2.0), TokenKind::Star, num(3.0)),
    ));
    assert!(matches!(value, LoxValue::Number(7.0)));
}

#[test]
fn interpret_grouping() {
    let value = interpret_expr(grouping(binary(num(1.0), TokenKind::Plus, num(2.0))));
    assert!(matches!(value, LoxValue::Number(3.0)));

    let value = interpret_expr(binary(
        grouping(binary(num(1.0), TokenKind::Plus, num(2.0))),
        TokenKind::Star,
        num(3.0),
    ));
    assert!(matches!(value, LoxValue::Number(9.0)));
}
