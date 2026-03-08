use loxide::{
    ast::{Expr, ExprKind},
    interpreter::{Interpreter, LoxValue},
    source::SourceSpan,
    token::TokenKind,
};

fn expr(kind: ExprKind<'static>) -> Expr<'static> {
    Expr {
        kind,
        span: SourceSpan {
            line: 0,
            char_range: 0..=0,
            bytes_range: 0..=0,
        },
    }
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

fn interpret(exprs: Vec<Expr<'_>>) -> LoxValue<'_> {
    Interpreter::interpret(exprs, "").unwrap()
}

#[test]
fn interpret_literals() {
    let value = interpret(vec![num(42.0)]);
    assert!(matches!(value, LoxValue::Number(42.0)));

    let value = interpret(vec![string("hello")]);
    assert!(matches!(value, LoxValue::String(s) if s == "hello"));

    let value = interpret(vec![boolean(true)]);
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret(vec![boolean(false)]);
    assert!(matches!(value, LoxValue::Boolean(false)));

    let value = interpret(vec![nil()]);
    assert!(matches!(value, LoxValue::Nil));
}

#[test]
fn interpret_unary() {
    let value = interpret(vec![unary(TokenKind::Minus, num(5.0))]);
    assert!(matches!(value, LoxValue::Number(-5.0)));

    let value = interpret(vec![unary(TokenKind::Bang, boolean(true))]);
    assert!(matches!(value, LoxValue::Boolean(false)));

    let value = interpret(vec![unary(TokenKind::Bang, boolean(false))]);
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret(vec![unary(TokenKind::Bang, nil())]);
    assert!(matches!(value, LoxValue::Boolean(true)));
}

#[test]
fn interpret_binary_arithmetic() {
    let value = interpret(vec![binary(num(1.0), TokenKind::Plus, num(2.0))]);
    assert!(matches!(value, LoxValue::Number(3.0)));

    let value = interpret(vec![binary(num(5.0), TokenKind::Minus, num(3.0))]);
    assert!(matches!(value, LoxValue::Number(2.0)));

    let value = interpret(vec![binary(num(2.0), TokenKind::Star, num(3.0))]);
    assert!(matches!(value, LoxValue::Number(6.0)));

    let value = interpret(vec![binary(num(6.0), TokenKind::Slash, num(2.0))]);
    assert!(matches!(value, LoxValue::Number(3.0)));
}

#[test]
fn interpret_binary_comparison() {
    let value = interpret(vec![binary(num(5.0), TokenKind::Greater, num(3.0))]);
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret(vec![binary(num(3.0), TokenKind::Greater, num(5.0))]);
    assert!(matches!(value, LoxValue::Boolean(false)));

    let value = interpret(vec![binary(num(3.0), TokenKind::Less, num(5.0))]);
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret(vec![binary(num(5.0), TokenKind::GreaterEqual, num(5.0))]);
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret(vec![binary(num(5.0), TokenKind::LessEqual, num(5.0))]);
    assert!(matches!(value, LoxValue::Boolean(true)));
}

#[test]
fn interpret_binary_equality() {
    let value = interpret(vec![binary(num(5.0), TokenKind::EqualEqual, num(5.0))]);
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret(vec![binary(num(5.0), TokenKind::EqualEqual, num(3.0))]);
    assert!(matches!(value, LoxValue::Boolean(false)));

    let value = interpret(vec![binary(num(5.0), TokenKind::BangEqual, num(3.0))]);
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret(vec![binary(boolean(true), TokenKind::EqualEqual, boolean(true))]);
    assert!(matches!(value, LoxValue::Boolean(true)));

    let value = interpret(vec![binary(nil(), TokenKind::EqualEqual, nil())]);
    assert!(matches!(value, LoxValue::Boolean(true)));
}

#[test]
fn interpret_string_concatenation() {
    let value = interpret(vec![binary(string("hello"), TokenKind::Plus, string("world"))]);
    assert!(matches!(value, LoxValue::String(s) if s == "helloworld"));
}

#[test]
fn interpret_chained_operations() {
    let value = interpret(vec![binary(
        num(1.0),
        TokenKind::Plus,
        binary(num(2.0), TokenKind::Star, num(3.0)),
    )]);
    assert!(matches!(value, LoxValue::Number(7.0)));
}

#[test]
fn interpret_grouping() {
    let value = interpret(vec![grouping(binary(num(1.0), TokenKind::Plus, num(2.0)))]);
    assert!(matches!(value, LoxValue::Number(3.0)));

    let value = interpret(vec![binary(
        grouping(binary(num(1.0), TokenKind::Plus, num(2.0))),
        TokenKind::Star,
        num(3.0),
    )]);
    assert!(matches!(value, LoxValue::Number(9.0)));
}
