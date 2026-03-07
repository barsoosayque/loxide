use crate::{
    ast::{Expr, ExprKind},
    error::LoxResult,
    source::{IntoSource, Source},
};

#[derive(Debug, Clone)]
pub enum LoxValue {
    Number(f32),
    String(String),
    Nil,
}

impl LoxValue {
    pub fn try_into_number(self) -> Option<Self> {
        match self {
            n @ LoxValue::Number(_) => Some(n),
            _ => None,
        }
    }

    pub fn try_into_string(self) -> Option<Self> {
        match self {
            s @ LoxValue::String(_) => Some(s),
            v => Some(LoxValue::String(v.to_string())),
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Nil => "nil",
        }
    }
}

impl std::fmt::Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) if n.fract() >= f32::EPSILON => write!(f, "{n}"),
            Self::Number(n) => write!(f, "{n:.0}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Nil => write!(f, "nil"),
        }
    }
}

pub struct Interpreter<'src> {
    source: Source<'src>,
}

impl<'src> Interpreter<'src> {
    pub fn interpret<T: IntoIterator<Item = Expr<'src>>>(
        ast: T,
        source: impl IntoSource<'src>,
    ) -> LoxResult<'src, LoxValue> {
        let mut interpreter = Self {
            source: source.into_source(),
        };

        // TODO: currently only the last expr is interpreted
        let Some(expr) = ast.into_iter().last() else {
            return Ok(LoxValue::Nil);
        };

        interpreter.eval(&expr)
    }

    fn eval(&mut self, expr: &Expr<'src>) -> LoxResult<'src, LoxValue> {
        // match expr {
        //     ExprKind::Binary { left, op, right } => {
        //         let right = self.eval(right)?;

        //         match op {
        //             TokenKind::Minus => return Ok(LoxValue::Number(())),
        //         }
        //     }
        //     ExprKind::Unary { op, right } => todo!(),
        //     ExprKind::Grouping { inner } => todo!(),
        //     ExprKind::Literal(expr_literal) => todo!(),
        // }

        Ok(LoxValue::Nil)
    }

    // fn cast_number(&self, value: LoxValue) -> LoxResult<'src, LoxValue> {}
}
