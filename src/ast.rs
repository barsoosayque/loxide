use display_tree::{AsTree, DisplayTree, to_display_tree_ref::ToDisplayTreeRef};

use crate::{source::SourceSpan, token::TokenKind};

#[derive(Debug, Clone)]
pub struct Expr<'src> {
    pub kind: ExprKind<'src>,
    pub span: SourceSpan,
}

impl<'src> Expr<'src> {
    pub fn new(kind: ExprKind<'src>, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}

impl std::fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Expr")?;
        if self.span.is_char() {
            f.write_fmt(format_args!("@{}", self.span.char_start()))?;
        } else {
            f.write_fmt(format_args!(
                "@{}..{}",
                self.span.char_start(),
                self.span.char_end()
            ))?;
        }

        f.write_fmt(format_args!("\n{}", self.kind))?;

        Ok(())
    }
}

impl<'src> ToDisplayTreeRef<ExprKind<'src>> for Box<Expr<'src>> {
    fn to_display_tree(&self) -> &ExprKind<'src> {
        &self.kind
    }
}

// `Box<Expr>` ??? In my epic blazingly fast AST ???
// Surely rustc *has* something better:
// https://github.com/rust-lang/rust/blob/765fd2d8c77a570e7069d9f30bb6d3d8fe437f9e/compiler/rustc_ast/src/ast.rs#L1739
// Oh... oh, ok.
#[derive(DisplayTree, Debug, Clone)]
pub enum ExprKind<'src> {
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
    LitString(&'src str),
    LitNumber(f64),
    LitBoolean(bool),
    LitNil,
}

impl std::fmt::Display for ExprKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        AsTree::new(self).fmt(f)
    }
}
