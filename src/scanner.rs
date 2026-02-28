use std::{ops::RangeInclusive, path::Path, str::CharIndices};

use crate::error::{LoxError, LoxErrorKind, LoxResult};

#[derive(Debug, PartialEq)]
pub enum TokenKind<'src> {
    // Single characters: brackers
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    // Single characters: other
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two characters
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier(&'src str),
    String(&'src str),
    Number(f64),

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, PartialEq)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub span: RangeInclusive<usize>,
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Token::{:?}", self.kind,))?;

        if self.span.start().abs_diff(*self.span.end()) == 0 {
            f.write_fmt(format_args!("@{}", self.span.start()))?;
        } else {
            f.write_fmt(format_args!("@{}..{}", self.span.start(), self.span.end()))?;
        }

        Ok(())
    }
}

impl<'src> Token<'src> {
    pub fn empty(kind: TokenKind<'src>, span: RangeInclusive<usize>) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug)]
pub struct ScannerIter<'scanner, 'src> {
    scanner: &'scanner Scanner<'src>,
    iter: CharIndices<'src>,
    current_line: usize,
    current_char: usize,
    current_byte: usize,
    is_terminated: bool,
}

impl<'scanner, 'src> ScannerIter<'scanner, 'src> {
    pub fn new(scanner: &'scanner Scanner<'src>) -> Self {
        Self {
            scanner,
            iter: scanner.source.char_indices(),
            current_line: 0,
            current_char: 0,
            current_byte: 0,
            is_terminated: false,
        }
    }

    fn next_token(&mut self) -> LoxResult<Token<'src>> {
        let lexeme_start = self.current_char;

        macro_rules! token {
            ($kind:expr) => {
                Ok(Token::empty($kind, lexeme_start..=(self.current_char - 1)))
            };
            ($char:expr => $kind:expr, else => $other:expr) => {
                if self.find($char) {
                    token!($kind)
                } else {
                    token!($other)
                }
            };
        }

        match self.advance()? {
            '(' => token!(TokenKind::LeftParen),
            ')' => token!(TokenKind::RightParen),
            '{' => token!(TokenKind::LeftBrace),
            '}' => token!(TokenKind::RightBrace),
            ',' => token!(TokenKind::Comma),
            '.' => token!(TokenKind::Dot),
            '-' => token!(TokenKind::Minus),
            '+' => token!(TokenKind::Plus),
            ';' => token!(TokenKind::Semicolon),
            '*' => token!(TokenKind::Star),
            '!' => token!('=' => TokenKind::BangEqual, else => TokenKind::Bang),
            '=' => token!('=' => TokenKind::EqualEqual, else => TokenKind::Equal),
            '<' => token!('=' => TokenKind::LessEqual, else => TokenKind::Less),
            '>' => token!('=' => TokenKind::GreaterEqual, else => TokenKind::Greater),
            '/' => {
                if self.find('/') {
                    // Skip comments and try to scan again
                    self.consume_until('\n')?;
                    self.next_token()
                } else {
                    token!(TokenKind::Slash)
                }
            }
            ' ' | '\r' | '\t' => self.next_token(),
            '\n' => {
                self.current_line += 1;
                self.next_token()
            }
            '"' => self.string(lexeme_start),
            '0'..='9' => self.number(lexeme_start),
            'a'..='z' | 'A'..='Z' => self.ident(lexeme_start),
            c => Err(self.error(LoxErrorKind::UnexpectedCharacter { c })),
        }
    }

    fn try_next_token(&mut self) -> LoxResult<Option<Token<'src>>> {
        match self.next_token() {
            Ok(token) => Ok(Some(token)),
            Err(LoxError {
                kind: LoxErrorKind::UnexpectedEof,
                ..
            }) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn string(&mut self, start: usize) -> LoxResult<Token<'src>> {
        let start_byte = self.current_byte + 1;

        if !self.consume_until('"')? {
            return Err(self.error(LoxErrorKind::UnterminatedString { start }));
        }

        let end_byte = self.current_byte;
        self.advance()?;

        let s = self.source_slice(start_byte..=end_byte)?;
        Ok(Token {
            kind: TokenKind::String(s),
            // lexeme: self.source.get(start_byte..=end_byte),
            span: start..=(self.current_char - 1),
        })
    }

    fn number(&mut self, start: usize) -> LoxResult<Token<'src>> {
        let start_byte = self.current_byte;

        while matches!(self.peek(), Some('0'..='9')) {
            self.advance()?;
        }

        match (self.peek(), self.peek_nth(1)) {
            (Some('.'), Some('0'..='9')) => {
                self.advance()?;
                while matches!(self.peek(), Some('0'..='9')) {
                    self.advance()?;
                }
            }
            _ => {}
        }

        let end_byte = self.current_byte;
        let lexeme = self.source_slice(start_byte..=end_byte)?;
        let n = lexeme
            .parse::<f64>()
            .or_else(|_| lexeme.parse::<u64>().map(|n| n as f64))
            .map_err(|_err| {
                self.error(LoxErrorKind::InvalidNumber {
                    s: lexeme.to_string(),
                    start,
                })
            })?;

        Ok(Token {
            kind: TokenKind::Number(n),
            // lexeme: Some(lexeme),
            span: start..=(self.current_char - 1),
        })
    }

    fn ident(&mut self, start: usize) -> LoxResult<Token<'src>> {
        let start_byte = self.current_byte;

        while matches!(
            self.peek(),
            Some('0'..='9') | Some('a'..='z') | Some('A'..='Z')
        ) {
            self.advance()?;
        }
        let end_byte = self.current_byte;

        let lexeme = self.source_slice(start_byte..=end_byte)?;
        let kind = match lexeme {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "fun" => TokenKind::Fun,
            "for" => TokenKind::For,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            lexeme => TokenKind::Identifier(lexeme),
        };

        Ok(Token {
            kind,
            // lexeme: Some(lexeme),
            span: start..=(self.current_char - 1),
        })
    }

    fn error(&self, kind: LoxErrorKind) -> LoxError {
        LoxError::new(kind, self.current_line, self.current_char.saturating_sub(1))
            .with_path(self.scanner.location)
    }

    fn source_slice(&self, range: RangeInclusive<usize>) -> LoxResult<&'src str> {
        self.scanner
            .source
            .get(range)
            .ok_or_else(|| self.error(LoxErrorKind::InvalidInput))
    }

    fn advance(&mut self) -> LoxResult<char> {
        let (idx, char) = self
            .iter
            .next()
            .ok_or_else(|| self.error(LoxErrorKind::UnexpectedEof))?;
        self.current_char += 1;
        self.current_byte = idx;
        Ok(char)
    }

    fn peek(&self) -> Option<char> {
        self.iter.clone().peekable().next().map(|(_, c)| c)
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        self.iter.clone().peekable().nth(n).map(|(_, c)| c)
    }

    fn find(&mut self, expected: char) -> bool {
        match self.peek() {
            Some(c) if c == expected => {
                let _ = self.advance();
                true
            }
            _ => false,
        }
    }

    fn consume_until(&mut self, expected: char) -> LoxResult<bool> {
        while let Some(peek) = self.peek() {
            if peek == expected {
                return Ok(true);
            } else {
                match self.advance() {
                    Ok('\n') => {
                        self.current_line += 1;
                    }
                    Ok(_) => {}
                    Err(err) => return Err(err),
                }
            }
        }
        Ok(false)
    }

    fn is_end(&self) -> bool {
        self.iter.clone().peekable().next().is_none()
    }
}

impl<'scanner, 'src> Iterator for ScannerIter<'scanner, 'src> {
    type Item = LoxResult<Token<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_end() {
            if self.is_terminated {
                return None;
            } else {
                self.is_terminated = true;
                return Some(Ok(Token::empty(
                    TokenKind::Eof,
                    self.scanner.source.len()..=self.scanner.source.len(),
                )));
            }
        }
        self.try_next_token().transpose()
    }
}

#[derive(Debug)]
pub struct Scanner<'src> {
    source: &'src str,
    location: Option<&'src Path>,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str, location: Option<&'src Path>) -> Self {
        Self { source, location }
    }

    pub fn scan(&self) -> impl Iterator<Item = Result<Token<'src>, LoxError>> {
        ScannerIter::new(&self)
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;
    use crate::error::LoxResultIter;

    #[test]
    fn scan_string() {
        let tokens = Scanner::new(r#""string""#, None)
            .scan()
            .ignore_errors()
            .collect::<Vec<_>>();

        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::String("string"),
                    span: 0..=7,
                },
                Token {
                    kind: TokenKind::Eof,
                    span: 8..=8
                }
            ]
        )
    }
}
