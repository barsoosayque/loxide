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
const NATIVE_FUN_KIND: &'static str = "native-fun";
const FUN_KIND: &'static str = "fun";

pub trait LoxCall<'src> {
    fn arity(&self) -> usize;

    fn call(
        &self,
        interpreter: &mut Interpreter<'_, 'src>,
        args: Vec<LoxValue<'src>>,
    ) -> LoxResult<'src, LoxValue<'src>>;
}

#[derive(Debug, Clone)]
pub struct LoxNativeFun<'src> {
    f: fn(args: Vec<LoxValue<'src>>) -> LoxResult<'src, LoxValue<'src>>,
    arity: usize,
}

impl<'src> LoxNativeFun<'src> {
    pub fn new(
        f: fn(args: Vec<LoxValue<'src>>) -> LoxResult<'src, LoxValue<'src>>,
        arity: usize,
    ) -> Self {
        Self { f, arity }
    }
}

impl<'src> LoxCall<'src> for LoxNativeFun<'src> {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        args: Vec<LoxValue<'src>>,
    ) -> LoxResult<'src, LoxValue<'src>> {
        (self.f)(args)
    }
}

#[derive(Debug, Clone)]
pub struct LoxFun<'src> {
    name: &'src str,
    params: Vec<&'src str>,
    body: Box<Stmt<'src>>,
}

impl<'src> LoxFun<'src> {
    pub fn new(name: &'src str, params: Vec<&'src str>, body: Box<Stmt<'src>>) -> Self {
        Self { name, params, body }
    }
}

impl<'src> LoxCall<'src> for LoxFun<'src> {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter<'_, 'src>,
        args: Vec<LoxValue<'src>>,
    ) -> LoxResult<'src, LoxValue<'src>> {
        interpreter.env.push_scope();
        for (id, value) in self.params.iter().zip(args.iter()) {
            interpreter.env.define(*id, value.clone());
        }
        let result = interpreter.execute(&self.body);
        interpreter.env.pop_scope();
        result
    }
}

#[derive(Debug, Default, Clone)]
pub enum LoxValue<'src> {
    #[default]
    Nil,
    Number(f64),
    String(Cow<'src, str>),
    Boolean(bool),
    NativeFun(LoxNativeFun<'src>),
    Fun(LoxFun<'src>),
}

impl<'src> LoxValue<'src> {
    pub fn try_as_number(&self) -> Option<Self> {
        match self {
            &Self::Number(n) => Some(Self::Number(n)),
            _ => None,
        }
    }

    pub fn try_as_string(&self) -> Option<Self> {
        match self {
            Self::String(s) => Some(Self::String(s.clone())),
            v => Some(LoxValue::String(v.to_string().into())),
        }
    }

    pub fn try_as_boolean(&self) -> Option<Self> {
        match self {
            &Self::Boolean(b) => Some(Self::Boolean(b)),
            Self::Nil => Some(Self::Boolean(false)),
            _ => Some(Self::Boolean(true)),
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::Number(_) => NUMBER_KIND,
            Self::String(_) => STRING_KIND,
            Self::Boolean(_) => BOOLEAN_KIND,
            Self::Nil => NIL_KIND,
            Self::NativeFun { .. } => NATIVE_FUN_KIND,
            Self::Fun { .. } => FUN_KIND,
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
            Self::NativeFun(fun) => write!(f, "native-fun {:?}", fun.f),
            Self::Fun(fun) => write!(f, "fun {}({})", fun.name, fun.params.join(",")),
        }
    }
}

macro_rules! cast {
    ($name:ident => $out:ty, $f:ident in $pat:pat => $val:expr, as $kind:expr) => {
        fn $name(&self, value: &LoxValue<'src>, span: SourceSpan) -> LoxResult<'src, $out> {
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
                let condition = self.cast_boolean(&condition_value, condition.span.clone())?;
                if condition {
                    let then_value = self.execute(&then)?;
                    return Ok(then_value);
                } else if let Some(or_else) = or_else {
                    let or_else_value = self.execute(&or_else)?;
                    return Ok(or_else_value);
                }
            }
            StmtKind::While { condition, body } => {
                while self
                    .eval(&condition)
                    .and_then(|v| self.cast_boolean(&v, condition.span.clone()))?
                {
                    self.execute(&body)?;
                }
            }
            StmtKind::Function { name, params, body } => {
                self.env.define(
                    *name,
                    LoxValue::Fun(LoxFun::new(name, params.clone(), body.clone())),
                );
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
                        let left_n = self.cast_number(&left_value, left.span.clone())?;
                        let right_n = self.cast_number(&right_value, right.span.clone())?;
                        return Ok(LoxValue::Boolean(left_n > right_n));
                    }
                    TokenKind::GreaterEqual => {
                        let left_n = self.cast_number(&left_value, left.span.clone())?;
                        let right_n = self.cast_number(&right_value, right.span.clone())?;
                        return Ok(LoxValue::Boolean(left_n >= right_n));
                    }
                    TokenKind::Less => {
                        let left_n = self.cast_number(&left_value, left.span.clone())?;
                        let right_n = self.cast_number(&right_value, right.span.clone())?;
                        return Ok(LoxValue::Boolean(left_n < right_n));
                    }
                    TokenKind::LessEqual => {
                        let left_n = self.cast_number(&left_value, left.span.clone())?;
                        let right_n = self.cast_number(&right_value, right.span.clone())?;
                        return Ok(LoxValue::Boolean(left_n <= right_n));
                    }
                    TokenKind::EqualEqual => {
                        return Ok(LoxValue::Boolean(left_value == right_value));
                    }
                    TokenKind::BangEqual => {
                        return Ok(LoxValue::Boolean(left_value != right_value));
                    }
                    TokenKind::Minus => {
                        let left_n = self.cast_number(&left_value, left.span.clone())?;
                        let right_n = self.cast_number(&right_value, right.span.clone())?;
                        return Ok(LoxValue::Number(left_n - right_n));
                    }
                    TokenKind::Slash => {
                        let left_n = self.cast_number(&left_value, left.span.clone())?;
                        let right_n = self.cast_number(&right_value, right.span.clone())?;
                        return Ok(LoxValue::Number(left_n / right_n));
                    }
                    TokenKind::Star => {
                        let left_n = self.cast_number(&left_value, left.span.clone())?;
                        let right_n = self.cast_number(&right_value, right.span.clone())?;
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

            ExprKind::Logic { left, op, right } => {
                let left_value = self.eval(&left)?;
                let left = self.cast_boolean(&left_value, left.span.clone())?;

                match op {
                    TokenKind::Or if left => {
                        return Ok(left_value);
                    }
                    TokenKind::And if !left => {
                        return Ok(left_value);
                    }
                    TokenKind::Or | TokenKind::And => {
                        return self.eval(&right);
                    }
                    _ => {
                        return Err(LoxError {
                            kind: LoxErrorKind::Expected("'or' | 'and' operators"),
                            source: self.source.clone(),
                            span: expr.span.clone(),
                        });
                    }
                }
            }
            ExprKind::Unary { op, right } => {
                let right_value = self.eval(&right)?;

                match op {
                    TokenKind::Minus => {
                        let n = self.cast_number(&right_value, right.span.clone())?;
                        return Ok(LoxValue::Number(-n));
                    }
                    TokenKind::Bang => {
                        let b = self.cast_boolean(&right_value, right.span.clone())?;
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
            ExprKind::Call { callee, args } => {
                let callee = self.eval(callee)?;
                let args = args
                    .iter()
                    .map(|arg| self.eval(arg))
                    .collect::<LoxResult<Vec<LoxValue>>>()?;
                return self.call(callee, args, expr.span.clone());
            }
        }

        Err(LoxError {
            kind: LoxErrorKind::Unreachable,
            source: self.source.clone(),
            span: expr.span.clone(),
        })
    }

    fn call(
        &mut self,
        callee: LoxValue<'src>,
        args: Vec<LoxValue<'src>>,
        span: SourceSpan,
    ) -> LoxResult<'src, LoxValue<'src>> {
        fn check_arity<'src, C: LoxCall<'src>>(
            callable: &C,
            provided: usize,
            source: &Source<'src>,
            span: &SourceSpan,
        ) -> LoxResult<'src, bool> {
            if callable.arity() != provided {
                return Err(LoxError::new(
                    LoxErrorKind::InvalidArity(provided, callable.arity()),
                    source.clone(),
                    span.clone(),
                ));
            }
            Ok(true)
        }

        match callee {
            LoxValue::NativeFun(fun) if check_arity(&fun, args.len(), &self.source, &span)? => {
                fun.call(self, args)
            }
            LoxValue::Fun(fun) if check_arity(&fun, args.len(), &self.source, &span)? => {
                fun.call(self, args)
            }
            _ => Err(LoxError::new(
                LoxErrorKind::InvalidCallee,
                self.source.clone(),
                span.clone(),
            )),
        }
    }

    cast!(cast_number => f64, try_as_number in LoxValue::Number(v) => v, as NUMBER_KIND);
    cast!(cast_boolean => bool, try_as_boolean in LoxValue::Boolean(v) => v, as BOOLEAN_KIND);
    // cast!(cast_string => Cow<'src, str>, try_as_string in LoxValue::String(v) => v, as STRING_KIND);

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
