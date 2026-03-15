use std::borrow::Cow;

use crate::{
    ast::{Expr, ExprKind, Stmt, StmtKind},
    environment::Environment,
    error::{LoxError, LoxErrorKind, LoxResult},
    source::{IntoSource, Source, SourceSpan},
    token::TokenKind,
};

const NUMBER_KIND: &'static str = "number";
const STRING_KIND: &'static str = "string";
const BOOLEAN_KIND: &'static str = "boolean";
const NIL_KIND: &'static str = "nil";

#[derive(Debug, Default, Clone)]
pub enum LoxValue<'src> {
    #[default]
    Nil,
    Number(f64),
    String(Cow<'src, str>),
    Boolean(bool),
}

impl<'src> LoxValue<'src> {
    pub fn try_into_number(self) -> Option<Self> {
        match self {
            n @ LoxValue::Number(_) => Some(n),
            _ => None,
        }
    }

    pub fn try_into_string(self) -> Option<Self> {
        match self {
            s @ LoxValue::String(_) => Some(s),
            v => Some(LoxValue::String(v.to_string().into())),
        }
    }

    pub fn try_into_boolean(self) -> Option<Self> {
        match self {
            b @ LoxValue::Boolean(_) => Some(b),
            LoxValue::Nil => Some(LoxValue::Boolean(false)),
            _ => Some(LoxValue::Boolean(true)),
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::Number(_) => NUMBER_KIND,
            Self::String(_) => STRING_KIND,
            Self::Boolean(_) => BOOLEAN_KIND,
            Self::Nil => NIL_KIND,
        }
    }
}

impl PartialEq for LoxValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            _ => false,
        }
    }
}

impl std::fmt::Display for LoxValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) if n.fract() >= f64::EPSILON => write!(f, "{n}"),
            Self::Number(n) => write!(f, "{n:.0}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

macro_rules! cast {
    ($name:ident => $out:ty, $f:ident in $pat:pat => $val:expr, as $kind:expr) => {
        fn $name(&self, value: LoxValue<'src>, span: SourceSpan) -> LoxResult<'src, $out> {
            let kind = value.kind();
            match value.$f() {
                Some($pat) => Ok($val),
                None => Err(LoxError {
                    kind: LoxErrorKind::InvalidConversion(kind, $kind),
                    source: self.source.clone(),
                    span,
                }),
                Some(_) => Err(LoxError {
                    kind: LoxErrorKind::Unreachable,
                    source: self.source.clone(),
                    span,
                }),
            }
        }
    };
}

#[derive(Debug)]
pub struct Interpreter<'env, 'src> {
    source: Source<'src>,
    env: &'env mut Environment<'src>,
}

impl<'env, 'src> Interpreter<'env, 'src> {
    pub fn execute_many<T>(
        ast: T,
        source: impl IntoSource<'src>,
        env: &'env mut Environment<'src>,
    ) -> LoxResult<'src, LoxValue<'src>>
    where
        T: IntoIterator<Item = Stmt<'src>>,
    {
        let mut int = Self {
            source: source.into_source(),
            env,
        };

        let mut value = LoxValue::Nil;
        for stmt in ast {
            value = int.execute(&stmt)?;
        }

        Ok(value)
    }

    fn execute(&mut self, stmt: &Stmt<'src>) -> LoxResult<'src, LoxValue<'src>> {
        match &stmt.kind {
            StmtKind::VariableDecl { id, init } => {
                let value = init
                    .as_ref()
                    .map(|init| self.eval(&init))
                    .transpose()?
                    .unwrap_or_default();

                self.env.define(*id, value);
            }
            StmtKind::Block(stmts) => {
                self.env.push_scope();
                for stmt in stmts {
                    self.execute(stmt)?;
                }
                self.env.pop_scope();
            }
            StmtKind::Expr(expr) => {
                let _value = self.eval(expr)?;
            }
            StmtKind::Print(expr) => {
                let value = self.eval(expr)?;
                println!("{value}");
            }
            StmtKind::ExprReturn(expr) => return self.eval(&expr),
            StmtKind::Conditional {
                condition,
                then,
                or_else,
            } => {
                let condition_value = self.eval(&condition)?;
                let condition = self.cast_boolean(condition_value, condition.span.clone())?;
                if condition {
                    let then_value = self.execute(&then)?;
                    return Ok(then_value);
                } else if let Some(or_else) = or_else {
                    let or_else_value = self.execute(&or_else)?;
                    return Ok(or_else_value);
                }
            }
        }
        Ok(LoxValue::Nil)
    }

    fn eval(&mut self, expr: &Expr<'src>) -> LoxResult<'src, LoxValue<'src>> {
        match &expr.kind {
            ExprKind::Binary { left, op, right } => {
                let left_value = self.eval(&left)?;
                let right_value = self.eval(&right)?;

                match op {
                    TokenKind::Greater => {
                        let left_n = self.cast_number(left_value, left.span.clone())?;
                        let right_n = self.cast_number(right_value, right.span.clone())?;
                        return Ok(LoxValue::Boolean(left_n > right_n));
                    }
                    TokenKind::GreaterEqual => {
                        let left_n = self.cast_number(left_value, left.span.clone())?;
                        let right_n = self.cast_number(right_value, right.span.clone())?;
                        return Ok(LoxValue::Boolean(left_n >= right_n));
                    }
                    TokenKind::Less => {
                        let left_n = self.cast_number(left_value, left.span.clone())?;
                        let right_n = self.cast_number(right_value, right.span.clone())?;
                        return Ok(LoxValue::Boolean(left_n < right_n));
                    }
                    TokenKind::LessEqual => {
                        let left_n = self.cast_number(left_value, left.span.clone())?;
                        let right_n = self.cast_number(right_value, right.span.clone())?;
                        return Ok(LoxValue::Boolean(left_n <= right_n));
                    }
                    TokenKind::EqualEqual => {
                        return Ok(LoxValue::Boolean(left_value == right_value));
                    }
                    TokenKind::BangEqual => {
                        return Ok(LoxValue::Boolean(left_value != right_value));
                    }
                    TokenKind::Minus => {
                        let left_n = self.cast_number(left_value, left.span.clone())?;
                        let right_n = self.cast_number(right_value, right.span.clone())?;
                        return Ok(LoxValue::Number(left_n - right_n));
                    }
                    TokenKind::Slash => {
                        let left_n = self.cast_number(left_value, left.span.clone())?;
                        let right_n = self.cast_number(right_value, right.span.clone())?;
                        return Ok(LoxValue::Number(left_n / right_n));
                    }
                    TokenKind::Star => {
                        let left_n = self.cast_number(left_value, left.span.clone())?;
                        let right_n = self.cast_number(right_value, right.span.clone())?;
                        return Ok(LoxValue::Number(left_n * right_n));
                    }
                    TokenKind::Plus => match (left_value, right_value) {
                        (LoxValue::Number(left_n), LoxValue::Number(right_n)) => {
                            return Ok(LoxValue::Number(left_n + right_n));
                        }
                        (LoxValue::String(left_s), LoxValue::String(right_s)) => {
                            return Ok(LoxValue::String(format!("{left_s}{right_s}").into()));
                        }
                        _ => {
                            return Err(LoxError {
                                kind: LoxErrorKind::ExpectedValues(&[NUMBER_KIND, STRING_KIND]),
                                source: self.source.clone(),
                                span: expr.span.clone(),
                            });
                        }
                    },
                    _ => {}
                }
            }
            ExprKind::Unary { op, right } => {
                let right_value = self.eval(&right)?;

                match op {
                    TokenKind::Minus => {
                        let n = self.cast_number(right_value, right.span.clone())?;
                        return Ok(LoxValue::Number(-n));
                    }
                    TokenKind::Bang => {
                        let b = self.cast_boolean(right_value, right.span.clone())?;
                        return Ok(LoxValue::Boolean(!b));
                    }
                    _ => {}
                }
            }
            ExprKind::Grouping { inner } => {
                return self.eval(&inner);
            }
            ExprKind::Assign { id, value } => {
                let value = self.eval(value)?;
                if !self.env.set(id, value.clone()) {
                    return Err(LoxError {
                        kind: LoxErrorKind::UndefinedVariable((*id).into()),
                        source: self.source.clone(),
                        span: expr.span.clone(),
                    });
                };
                return Ok(value);
            }
            ExprKind::Var(id) => return self.get_var(id, expr.span.clone()),
            &ExprKind::LitString(s) => return Ok(LoxValue::String(s.into())),
            &ExprKind::LitNumber(n) => return Ok(LoxValue::Number(n)),
            &ExprKind::LitBoolean(b) => return Ok(LoxValue::Boolean(b)),
            ExprKind::LitNil => return Ok(LoxValue::Nil),
        }

        Err(LoxError {
            kind: LoxErrorKind::Unreachable,
            source: self.source.clone(),
            span: expr.span.clone(),
        })
    }

    cast!(cast_number => f64, try_into_number in LoxValue::Number(v) => v, as NUMBER_KIND);
    cast!(cast_boolean => bool, try_into_boolean in LoxValue::Boolean(v) => v, as BOOLEAN_KIND);
    // cast!(cast_string => Cow<'src, str>, try_into_string in LoxValue::String(v) => v, as STRING_KIND);

    fn get_var(&self, id: &'src str, span: SourceSpan) -> LoxResult<'src, LoxValue<'src>> {
        self.env.get(id).cloned().ok_or_else(|| {
            LoxError::new(
                LoxErrorKind::UndefinedVariable(id.into()),
                self.source.clone(),
                span,
            )
        })
    }
}
