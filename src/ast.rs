use std::marker::PhantomData;

use crate::{source::SourceSpan, token::TokenKind};

pub type Expr<'src> = AstPart<'src, ExprKind<'src>>;
pub type Stmt<'src> = AstPart<'src, StmtKind<'src>>;

#[derive(Debug, Clone)]
pub struct AstPart<'src, T> {
    pub kind: T,
    pub span: SourceSpan,
    _phantom: PhantomData<&'src ()>,
}

impl<T> AstPart<'_, T> {
    pub fn new(kind: T, span: SourceSpan) -> Self {
        Self {
            kind,
            span,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind<'src> {
    Binary {
        left: Box<Expr<'src>>,
        op: TokenKind<'src>,
        right: Box<Expr<'src>>,
    },
    Unary {
        op: TokenKind<'src>,
        right: Box<Expr<'src>>,
    },
    Grouping {
        inner: Box<Expr<'src>>,
    },
    Assign {
        id: &'src str,
        value: Box<Expr<'src>>,
    },
    Var(&'src str),
    LitString(&'src str),
    LitNumber(f64),
    LitBoolean(bool),
    LitNil,
}

#[derive(Debug, Clone)]
pub enum StmtKind<'src> {
    VariableDecl {
        id: &'src str,
        init: Option<Box<Expr<'src>>>,
    },
    Block(Vec<Box<Stmt<'src>>>),
    Expr(Box<Expr<'src>>),
    ExprReturn(Box<Expr<'src>>),
    Print(Box<Expr<'src>>),
    Conditional {
        condition: Box<Expr<'src>>,
        then: Box<Stmt<'src>>,
        or_else: Option<Box<Stmt<'src>>>,
    },
}

pub trait DisplayTree {
    fn format_tree(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result;
}

impl std::fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_tree(f, 0)
    }
}

impl std::fmt::Display for Stmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_tree(f, 0)
    }
}

impl DisplayTree for Expr<'_> {
    fn format_tree(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        let t = "  ".repeat(indent);
        let span = if self.span.is_char() {
            format!("@{}", self.span.char_start())
        } else {
            format!("@{}..{}", self.span.char_start(), self.span.char_end())
        };
        match &self.kind {
            ExprKind::Binary { left, op, right } => {
                write!(f, "Binary{span} {op}")?;
                write!(f, "\n{t}├── ")?;
                left.format_tree(f, indent + 1)?;
                write!(f, "\n{t}└── ")?;
                right.format_tree(f, indent + 1)?;
                Ok(())
            }
            ExprKind::Unary { op, right } => {
                write!(f, "Unary{span} {op}")?;
                write!(f, "\n{t}└── ")?;
                right.format_tree(f, indent + 1)?;
                Ok(())
            }
            ExprKind::Grouping { inner } => {
                write!(f, "Grouping{span}: ")?;
                inner.format_tree(f, indent + 1)?;
                Ok(())
            }
            ExprKind::Assign { id, value } => {
                write!(f, "Assign{span} {id}")?;
                write!(f, "\n{t}└── ")?;
                value.format_tree(f, indent + 1)?;
                Ok(())
            }
            ExprKind::LitString(s) => {
                write!(f, "\"{s}\"{span}")?;
                Ok(())
            }
            ExprKind::LitNumber(n) => {
                write!(f, "{n}{span}")?;
                Ok(())
            }
            ExprKind::LitBoolean(b) => {
                write!(f, "{b}{span}")?;
                Ok(())
            }
            ExprKind::LitNil => {
                write!(f, "nil{span}")?;
                Ok(())
            }
            ExprKind::Var(id) => {
                write!(f, "Var({id}){span}")?;
                Ok(())
            }
        }
    }
}

impl DisplayTree for Stmt<'_> {
    fn format_tree(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        let t = "  ".repeat(indent);
        let span = if self.span.is_char() {
            format!("@{}", self.span.char_start())
        } else {
            format!("@{}..{}", self.span.char_start(), self.span.char_end())
        };
        match &self.kind {
            StmtKind::VariableDecl { id, init } => {
                write!(f, "VariableDecl{span}: {id}")?;
                if let Some(init) = init {
                    write!(f, "\n{t}└── ")?;
                    init.format_tree(f, indent + 1)?;
                }
                Ok(())
            }
            StmtKind::Expr(expr) => {
                write!(f, "Expr{span}: ")?;
                write!(f, "\n{t}└── ")?;
                expr.format_tree(f, indent + 1)?;
                Ok(())
            }
            StmtKind::ExprReturn(expr) => {
                write!(f, "ExprReturn{span}: ")?;
                write!(f, "\n{t}└── ")?;
                expr.format_tree(f, indent + 1)?;
                Ok(())
            }
            StmtKind::Print(expr) => {
                write!(f, "Print{span}: ")?;
                write!(f, "\n{t}└── ")?;
                expr.format_tree(f, indent + 1)?;
                Ok(())
            }
            StmtKind::Block(stmts) => {
                write!(f, "Block{span}")?;
                if stmts.is_empty() {
                    write!(f, "\n{t}└── <empty>")?;
                    return Ok(());
                }
                if stmts.len() > 1 {
                    for (i, stmt) in stmts.iter().enumerate() {
                        if i == (stmts.len() - 1) {
                            continue;
                        }
                        write!(f, "\n{t}├── ")?;
                        stmt.format_tree(f, indent + 1)?;
                    }
                }
                write!(f, "\n{t}└── ")?;
                stmts.last().unwrap().format_tree(f, indent + 1)?;
                Ok(())
            }
            StmtKind::Conditional {
                condition,
                then,
                or_else,
            } => {
                write!(f, "Conditional{span}")?;
                write!(f, "\n{t}├── if ")?;
                condition.format_tree(f, indent + 1)?;
                if let Some(or_else) = or_else {
                    write!(f, "\n{t}├── then ")?;
                    then.format_tree(f, indent + 1)?;
                    write!(f, "\n{t}└── else")?;
                    or_else.format_tree(f, indent + 1)?;
                } else {
                    write!(f, "\n{t}└── then")?;
                    then.format_tree(f, indent + 1)?;
                }
                Ok(())
            }
        }
    }
}
