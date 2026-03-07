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
    UnterminatedString,
    InvalidNumber(String),
    InvalidConversion(&'static str, &'static str),
}

impl<'src> std::fmt::Display for LoxErrorKind<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedCharacter(c) => f.write_fmt(format_args!("Unexpected character '{c}'")),
            Self::UnexpectedEof => f.write_str("Unexpected end of file"),
            Self::UnexpectedToken(k) => f.write_fmt(format_args!("Unexpected token '{k}'")),
            Self::ExpectedToken(k) => f.write_fmt(format_args!("Expected '{k}'")),
            Self::ExpectedExpr => f.write_str("Expected expression"),
            Self::UnterminatedString { .. } => f.write_fmt(format_args!("Unterminated string")),
            Self::InvalidNumber(s) => f.write_fmt(format_args!("Invalid number: '{s}'")),
            Self::InvalidConversion(from, to) => {
                f.write_fmt(format_args!("Cannot convert {from} to {to}"))
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

    // pub fn with_location(mut self, location: Option<impl Into<String>>) -> Self {
    //     self.location = location.map(Into::into);
    //     self
    // }

    // pub fn with_path(self, location: Option<impl AsRef<Path>>) -> Self {
    //     self.with_location(location.map(|p| p.as_ref().to_string_lossy().into_owned()))
    // }

    // pub fn span(&self) -> RangeInclusive<usize> {
    //     match self.kind {
    //         LoxErrorKind::InvalidNumber { start, .. }
    //         | LoxErrorKind::UnterminatedString { start } => start..=self.column,
    //         LoxErrorKind::UnexpectedCharacter { .. }
    //         | LoxErrorKind::UnexpectedEof
    //         | LoxErrorKind::InvalidInput => self.column..=self.column,
    //     }
    // }
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

#[allow(unused)]
pub trait LoxResultIter<T>: Sized {
    fn handle_errors(self) -> impl Iterator<Item = T>;
    fn ignore_errors(self) -> impl Iterator<Item = T>;
    fn ignore_to_vec(self) -> Vec<T> {
        self.ignore_errors().collect()
    }
    fn handle_to_vec(self) -> Vec<T> {
        self.handle_errors().collect()
    }
}

impl<'src, T, I: Iterator<Item = LoxResult<'src, T>>> LoxResultIter<T> for I {
    fn handle_errors(self) -> impl Iterator<Item = T> {
        self.filter_map(|r| match r {
            Ok(value) => Some(value),
            Err(err) => {
                println!(
                    "{} {}\n  {}{}\n  {}",
                    "✗".red().bold(),
                    err.source
                        .script
                        .lines()
                        .nth(err.span.line)
                        .unwrap_or("<error>")
                        .italic(),
                    " ".repeat(err.span.char_start()),
                    "~".repeat(err.span.char_len() + 1).italic().yellow(),
                    err.to_string().red().bold(),
                );
                None
            }
        })
    }

    fn ignore_errors(self) -> impl Iterator<Item = T> {
        self.filter_map(|r| match r {
            Ok(value) => Some(value),
            Err(_err) => None,
        })
    }
}
