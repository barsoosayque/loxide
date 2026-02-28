use std::{ops::RangeInclusive, path::Path};

use yansi::Paint;

pub type LoxResult<T> = Result<T, LoxError>;

#[derive(Debug)]
pub enum LoxErrorKind {
    UnexpectedCharacter { c: char },
    UnterminatedString { start: usize },
    InvalidNumber { start: usize, s: String },
    UnexpectedEof,
    InvalidInput,
}

impl std::fmt::Display for LoxErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedCharacter { c } => {
                f.write_fmt(format_args!("Unexpected character '{c}'"))
            }
            Self::UnterminatedString { .. } => f.write_fmt(format_args!("Unterminated string")),
            Self::InvalidNumber { s, .. } => f.write_fmt(format_args!("Invalid number: '{s}'")),
            Self::UnexpectedEof => f.write_str("Unexpected end of file"),
            Self::InvalidInput => f.write_str("Invalid input, damn"),
        }
    }
}

#[derive(Debug)]
pub struct LoxError {
    pub kind: LoxErrorKind,
    pub location: Option<String>,
    pub line: usize,
    pub column: usize,
}

impl LoxError {
    pub fn new(kind: LoxErrorKind, line: usize, column: usize) -> Self {
        Self {
            kind,
            line,
            column,
            location: None,
        }
    }

    pub fn with_location(mut self, location: Option<impl Into<String>>) -> Self {
        self.location = location.map(Into::into);
        self
    }

    pub fn with_path(self, location: Option<impl AsRef<Path>>) -> Self {
        self.with_location(location.map(|p| p.as_ref().to_string_lossy().into_owned()))
    }

    pub fn span(&self) -> RangeInclusive<usize> {
        match self.kind {
            LoxErrorKind::InvalidNumber { start, .. }
            | LoxErrorKind::UnterminatedString { start } => start..=self.column,
            LoxErrorKind::UnexpectedCharacter { .. }
            | LoxErrorKind::UnexpectedEof
            | LoxErrorKind::InvalidInput => self.column..=self.column,
        }
    }
}

impl std::error::Error for LoxError {}

impl std::fmt::Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            location,
            kind,
            line,
            column,
        } = self;

        if let Some(location) = location {
            f.write_fmt(format_args!("[{location} at {line}:{column}] {kind}"))
        } else {
            f.write_fmt(format_args!("[{line}:{column}] {kind}"))
        }
    }
}

#[allow(unused)]
pub trait LoxResultIter<T> {
    fn handle_errors(self, source: &str) -> impl Iterator<Item = T>;
    fn ignore_errors(self) -> impl Iterator<Item = T>;
}

impl<T, I: Iterator<Item = LoxResult<T>>> LoxResultIter<T> for I {
    fn handle_errors(self, source: &str) -> impl Iterator<Item = T> {
        self.filter_map(|r| match r {
            Ok(value) => Some(value),
            Err(err) => {
                println!(
                    "{} {}\n  {}{}\n  {}",
                    "✗".red().bold(),
                    source.lines().nth(err.line).unwrap_or("<error>").italic(),
                    " ".repeat(*err.span().start()),
                    "~".repeat(err.span().end() - err.span().start() + 1)
                        .italic()
                        .yellow(),
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
