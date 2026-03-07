use display_tree::{AsTree, DisplayTree};

use crate::token::TokenKind;

// `Box<Expr>` ??? In my epic blazingly fast AST ???
// Surely rustc *has* something better:
// https://github.com/rust-lang/rust/blob/765fd2d8c77a570e7069d9f30bb6d3d8fe437f9e/compiler/rustc_ast/src/ast.rs#L1739
// Oh... oh, ok.
#[derive(DisplayTree, Debug, Clone)]
pub enum Expr<'src> {
    Binary {
        #[tree]
        left: Box<Expr<'src>>,
        #[node_label]
        op: TokenKind<'src>,
        #[tree]
        right: Box<Expr<'src>>,
    },
    Unary {
        #[node_label]
        op: TokenKind<'src>,
        #[tree]
        right: Box<Expr<'src>>,
    },
    Grouping {
        #[tree]
        inner: Box<Expr<'src>>,
    },
    Literal(ExprLiteral<'src>),
}

impl std::fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        AsTree::new(self).fmt(f)
    }
}

#[derive(Debug, Clone)]
pub enum ExprLiteral<'src> {
    String(&'src str),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl<'src> std::fmt::Display for ExprLiteral<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => f.write_fmt(format_args!("\"{s}\"")),
            Self::Number(n) => f.write_fmt(format_args!("{n}")),
            Self::Boolean(b) => f.write_fmt(format_args!("{b}")),
            Self::Nil => f.write_str("nil"),
        }
    }
}
