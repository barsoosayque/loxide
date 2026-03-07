use crate::scanner::Token;
use crate::token::Token;

// `Box<Expr>` ??? In my epic blazingly fast AST ???
// Surely rustc *has* something better:
// https://github.com/rust-lang/rust/blob/765fd2d8c77a570e7069d9f30bb6d3d8fe437f9e/compiler/rustc_ast/src/ast.rs#L1739
// Oh... oh, ok.
#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Binary {
        left: Box<Expr<'src>>,
        op: Token<'src>,
        right: Box<Expr<'src>>,
    },
    Unary {
        op: Token<'src>,
        right: Box<Expr<'src>>,
    },
    Grouping {
        inner: Box<Expr<'src>>,
    },
    Literal(ExprLiteral<'src>),
}

#[derive(Debug, Clone)]
pub enum ExprLiteral<'src> {
    String(&'src str),
    Number(f64),
}
