use std::{fmt::Write, path::Path, vec::IntoIter};

#[derive(Debug)]
pub enum LoxError {
    UnexpectedCharacter { c: char, n: usize },
    UnterminatedString { start: usize },
    InvalidNumber { s: String },
}

impl std::error::Error for LoxError {}

impl std::fmt::Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxError::UnexpectedCharacter { c, n } => {
                f.write_fmt(format_args!("Unexpected character '{c}' at {n}"))
            }
            LoxError::UnterminatedString { start } => {
                f.write_fmt(format_args!("Unterminated string that starts at {start}"))
            }
            LoxError::InvalidNumber { s } => f.write_fmt(format_args!("Invalid number: '{s}'")),
        }
    }
}

#[derive(Debug, Default)]
pub struct LoxErrorAcc {
    errors: Vec<(LoxError, Option<String>, usize)>,
}

impl std::error::Error for LoxErrorAcc {}

impl std::fmt::Display for LoxErrorAcc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (error, location, line) in &self.errors {
            if let Some(location) = location {
                f.write_fmt(format_args!("[{location}:{line}] {error}"))?;
            } else {
                f.write_fmt(format_args!("[{line}] {error}"))?;
            }
            f.write_char('\n')?;
        }

        Ok(())
    }
}

impl Extend<(LoxError, Option<String>, usize)> for LoxErrorAcc {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (LoxError, Option<String>, usize)>,
    {
        self.errors.extend(iter);
    }
}

impl IntoIterator for LoxErrorAcc {
    type Item = (LoxError, Option<String>, usize);
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl LoxErrorAcc {
    pub fn push(&mut self, error: LoxError, location: Option<&Path>, line: usize) {
        self.errors.push((
            error,
            location.map(|s| s.to_string_lossy().into_owned()),
            line,
        ));
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}
