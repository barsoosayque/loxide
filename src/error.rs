use yansi::Paint;

use crate::{
    source::{IntoSource, Source, SourceSpan},
    token::TokenKind,
};

pub type LoxResult<'src, T> = Result<T, LoxError<'src>>;

#[derive(Debug)]
pub enum LoxErrorKind<'src> {
    UnexpectedCharacter(char),
    UnexpectedEof,
    UnexpectedToken(TokenKind<'src>),
    ExpectedToken(TokenKind<'src>),
    ExpectedExpr,
    ExpectedValues(&'static [&'static str]),
    UnterminatedString,
    InvalidNumber(String),
    InvalidConversion(&'static str, &'static str),
    Unreachable,
}

impl<'src> std::fmt::Display for LoxErrorKind<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedCharacter(c) => f.write_fmt(format_args!("Unexpected character '{c}'")),
            Self::UnexpectedEof => f.write_str("Unexpected end of file"),
            Self::UnexpectedToken(k) => f.write_fmt(format_args!("Unexpected token '{k}'")),
            Self::ExpectedToken(k) => f.write_fmt(format_args!("Expected '{k}'")),
            Self::ExpectedExpr => f.write_str("Expected expression"),
            Self::ExpectedValues(vs) => {
                f.write_fmt(format_args!("Expected values of types: {}", vs.join(", ")))
            }
            Self::UnterminatedString { .. } => f.write_fmt(format_args!("Unterminated string")),
            Self::InvalidNumber(s) => f.write_fmt(format_args!("Invalid number: '{s}'")),
            Self::InvalidConversion(from, to) => {
                f.write_fmt(format_args!("Cannot convert {from} to {to}"))
            }
            Self::Unreachable => {
                f.write_str("Unreachable state reached, this is a bug. Damn.. good job")
            }
        }
    }
}

#[derive(Debug)]
pub struct LoxError<'src> {
    pub kind: LoxErrorKind<'src>,
    pub source: Source<'src>,
    pub span: SourceSpan,
}

impl<'src> LoxError<'src> {
    pub fn new(kind: LoxErrorKind<'src>, source: impl IntoSource<'src>, span: SourceSpan) -> Self {
        Self {
            kind,
            source: source.into_source(),
            span,
        }
    }
}

impl std::error::Error for LoxError<'_> {}

impl std::fmt::Display for LoxError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            kind,
            source: Source { location, .. },
            span,
        } = self;

        f.write_str("[")?;
        if let Some(location) = location {
            f.write_str(location.to_string_lossy().as_ref())?;
            f.write_str(" ")?;
        }
        span.line.fmt(f)?;
        f.write_str(":")?;
        if span.is_char() {
            span.char_start().fmt(f)?;
        } else {
            f.write_fmt(format_args!("{}..{}", span.char_start(), span.char_end()))?;
        }
        f.write_str("] ")?;
        kind.fmt(f)?;

        Ok(())
    }
}

pub trait HandleLoxResult<T>: Sized {
    fn report_err(self) -> Option<T>;
}

impl<'src, T> HandleLoxResult<T> for LoxResult<'src, T> {
    fn report_err(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(err) => {
                println!(
                    "{} {}\n{} {}{}\n{} {}",
                    "▓".red().bold(),
                    err.source
                        .script
                        .lines()
                        .nth(err.span.line)
                        .unwrap_or("<error>")
                        .italic(),
                    "░".red().bold(),
                    " ".repeat(err.span.char_start()),
                    "~".repeat(err.span.char_len() + 1).italic().yellow(),
                    "░".red().bold(),
                    err.to_string().red().bold(),
                );
                None
            }
        }
    }
}

pub trait HandleLoxResultIter<T>: Sized {
    fn report_err(self) -> impl Iterator<Item = T>;
    fn ignore_err(self) -> impl Iterator<Item = T>;
    fn process_silent(self) -> (Vec<T>, usize);
    fn process(self) -> (Vec<T>, usize);
}

impl<'src, T, I: Iterator<Item = LoxResult<'src, T>>> HandleLoxResultIter<T> for I {
    fn report_err(self) -> impl Iterator<Item = T> {
        self.filter_map(HandleLoxResult::report_err)
    }

    fn ignore_err(self) -> impl Iterator<Item = T> {
        self.filter_map(|r| match r {
            Ok(value) => Some(value),
            Err(_err) => None,
        })
    }

    fn process_silent(self) -> (Vec<T>, usize) {
        let mut errors = 0_usize;

        let v = self
            .filter_map(|r| {
                errors += if r.is_err() { 1 } else { 0 };
                r.ok()
            })
            .collect();

        (v, errors)
    }

    fn process(self) -> (Vec<T>, usize) {
        let mut errors = 0_usize;

        let v = self
            .filter_map(|r| {
                errors += if r.is_err() { 1 } else { 0 };
                HandleLoxResult::report_err(r)
            })
            .collect();

        (v, errors)
    }
}
