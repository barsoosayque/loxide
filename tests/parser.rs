use loxide::{
    ast::{Expr, ExprKind},
    error::HandleLoxResultIter,
    parser::Parser,
    source::SourceSpan,
    token::{Token, TokenKind},
};

fn t(kind: TokenKind<'static>) -> Token<'static> {
    Token {
        kind,
        span: SourceSpan {
            line: 0,
            char_range: 0..=0,
            bytes_range: 0..=0,
        },
    }
}

fn parse(tokens: Vec<Token<'_>>) -> Vec<Expr<'_>> {
    Parser::parse(tokens, "").process_silent().0
}

fn parse_single(tokens: Vec<Token<'_>>) -> Expr<'_> {
    parse(tokens).into_iter().next().unwrap()
}

#[test]
fn parse_literals() {
    let expr = parse_single(vec![t(TokenKind::Number(42.0)), t(TokenKind::Eof)]);
    assert!(matches!(expr.kind, ExprKind::LitNumber(42.0)));

    let expr = parse_single(vec![t(TokenKind::String("hello")), t(TokenKind::Eof)]);
    assert!(matches!(expr.kind, ExprKind::LitString("hello")));

    let expr = parse_single(vec![t(TokenKind::True), t(TokenKind::Eof)]);
    assert!(matches!(expr.kind, ExprKind::LitBoolean(true)));

    let expr = parse_single(vec![t(TokenKind::False), t(TokenKind::Eof)]);
    assert!(matches!(expr.kind, ExprKind::LitBoolean(false)));

    let expr = parse_single(vec![t(TokenKind::Nil), t(TokenKind::Eof)]);
    assert!(matches!(expr.kind, ExprKind::LitNil));
}

#[test]
fn parse_grouping() {
    let expr = parse_single(vec![
        t(TokenKind::LeftParen),
        t(TokenKind::Number(42.0)),
        t(TokenKind::RightParen),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Grouping { inner } => {
            assert!(matches!(inner.kind, ExprKind::LitNumber(42.0)));
        }
        _ => panic!("expected Grouping, got {:?}", expr.kind),
    }
}

#[test]
fn parse_unary() {
    let expr = parse_single(vec![
        t(TokenKind::Minus),
        t(TokenKind::Number(5.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Unary { op, right } => {
            assert!(matches!(op, TokenKind::Minus));
            assert!(matches!(right.kind, ExprKind::LitNumber(5.0)));
        }
        _ => panic!("expected Unary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Bang),
        t(TokenKind::True),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Unary { op, right } => {
            assert!(matches!(op, TokenKind::Bang));
            assert!(matches!(right.kind, ExprKind::LitBoolean(true)));
        }
        _ => panic!("expected Unary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Minus),
        t(TokenKind::Minus),
        t(TokenKind::Number(5.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Unary { right, .. } => {
            assert!(matches!(right.kind, ExprKind::Unary { .. }));
        }
        _ => panic!("expected Unary, got {:?}", expr.kind),
    }
}

#[test]
fn parse_binary() {
    let expr = parse_single(vec![
        t(TokenKind::Number(1.0)),
        t(TokenKind::Plus),
        t(TokenKind::Number(2.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { left, op, right } => {
            assert!(matches!(left.kind, ExprKind::LitNumber(1.0)));
            assert!(matches!(op, TokenKind::Plus));
            assert!(matches!(right.kind, ExprKind::LitNumber(2.0)));
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Number(5.0)),
        t(TokenKind::Minus),
        t(TokenKind::Number(3.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { op, .. } => {
            assert!(matches!(op, TokenKind::Minus));
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Number(2.0)),
        t(TokenKind::Star),
        t(TokenKind::Number(3.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { op, .. } => {
            assert!(matches!(op, TokenKind::Star));
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Number(6.0)),
        t(TokenKind::Slash),
        t(TokenKind::Number(2.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { op, .. } => {
            assert!(matches!(op, TokenKind::Slash));
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Number(5.0)),
        t(TokenKind::Greater),
        t(TokenKind::Number(3.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { op, .. } => {
            assert!(matches!(op, TokenKind::Greater));
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Number(3.0)),
        t(TokenKind::Less),
        t(TokenKind::Number(5.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { op, .. } => {
            assert!(matches!(op, TokenKind::Less));
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Number(5.0)),
        t(TokenKind::GreaterEqual),
        t(TokenKind::Number(5.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { op, .. } => {
            assert!(matches!(op, TokenKind::GreaterEqual));
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Number(3.0)),
        t(TokenKind::LessEqual),
        t(TokenKind::Number(3.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { op, .. } => {
            assert!(matches!(op, TokenKind::LessEqual));
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Number(5.0)),
        t(TokenKind::EqualEqual),
        t(TokenKind::Number(5.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { op, .. } => {
            assert!(matches!(op, TokenKind::EqualEqual));
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Number(5.0)),
        t(TokenKind::BangEqual),
        t(TokenKind::Number(3.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { op, .. } => {
            assert!(matches!(op, TokenKind::BangEqual));
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::Number(1.0)),
        t(TokenKind::Plus),
        t(TokenKind::Number(2.0)),
        t(TokenKind::Star),
        t(TokenKind::Number(3.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { left, op, right } => {
            assert!(matches!(left.kind, ExprKind::LitNumber(1.0)));
            assert!(matches!(op, TokenKind::Plus));
            match right.kind {
                ExprKind::Binary { left, op, right } => {
                    assert!(matches!(left.kind, ExprKind::LitNumber(2.0)));
                    assert!(matches!(op, TokenKind::Star));
                    assert!(matches!(right.kind, ExprKind::LitNumber(3.0)));
                }
                _ => panic!("expected nested Binary, got {:?}", right.kind),
            }
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }

    let expr = parse_single(vec![
        t(TokenKind::LeftParen),
        t(TokenKind::Number(1.0)),
        t(TokenKind::Plus),
        t(TokenKind::Number(2.0)),
        t(TokenKind::RightParen),
        t(TokenKind::Star),
        t(TokenKind::Number(3.0)),
        t(TokenKind::Eof),
    ]);
    match expr.kind {
        ExprKind::Binary { left, op, .. } => {
            assert!(matches!(op, TokenKind::Star));
            match left.kind {
                ExprKind::Grouping { inner } => {
                    assert!(matches!(inner.kind, ExprKind::Binary { .. }));
                }
                _ => panic!("expected Grouping, got {:?}", left.kind),
            }
        }
        _ => panic!("expected Binary, got {:?}", expr.kind),
    }
}

#[test]
fn parse_expressions() {
    let exprs = parse(vec![
        t(TokenKind::Number(1.0)),
        t(TokenKind::Number(2.0)),
        t(TokenKind::Number(3.0)),
        t(TokenKind::Eof),
    ]);
    assert_eq!(exprs.len(), 3);
    assert!(matches!(exprs[0].kind, ExprKind::LitNumber(1.0)));
    assert!(matches!(exprs[1].kind, ExprKind::LitNumber(2.0)));
    assert!(matches!(exprs[2].kind, ExprKind::LitNumber(3.0)));
}
