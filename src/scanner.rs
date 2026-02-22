use std::{ops::RangeInclusive, path::Path, str::CharIndices};

use crate::error::{LoxError, LoxErrorAcc};

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
    Identifier,
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
    pub lexeme: Option<&'src str>,
    pub span: RangeInclusive<usize>,
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Token/{:?}", self.kind,))?;

        if self.span.start().abs_diff(*self.span.end()) == 0 {
            f.write_fmt(format_args!("@{}", self.span.start()))?;
        } else {
            f.write_fmt(format_args!("@{}..{}", self.span.start(), self.span.end()))?;
        }

        if let Some(lexeme) = self.lexeme {
            f.write_fmt(format_args!(": {lexeme}"))?;
        }

        Ok(())
    }
}

impl<'src> Token<'src> {
    pub fn empty(kind: TokenKind<'src>, span: RangeInclusive<usize>) -> Self {
        Self {
            kind,
            span,
            lexeme: None,
        }
    }
}

#[derive(Debug)]
pub struct Scanner<'src> {
    source: &'src str,
    location: Option<&'src Path>,

    tokens: Vec<Token<'src>>,
    errors: LoxErrorAcc,

    iter: CharIndices<'src>,
    current_line: usize,
    current_char: usize,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str, location: Option<&'src Path>) -> Self {
        Self {
            source,
            location,
            tokens: vec![],
            errors: LoxErrorAcc::default(),
            iter: source.char_indices(),
            current_line: 0,
            current_char: 0,
        }
    }

    // TODO: return iterator
    pub fn scan(mut self) -> Result<Vec<Token<'src>>, LoxErrorAcc> {
        while !self.is_end() {
            let Some(token) = self.scan_token() else {
                continue;
            };
            self.tokens.push(token);
        }

        self.tokens.push(Token::empty(
            TokenKind::Eof,
            self.source.len()..=self.source.len(),
        ));

        if self.errors.is_empty() {
            Ok(self.tokens)
        } else {
            Err(self.errors)
        }
    }

    fn scan_token(&mut self) -> Option<Token<'src>> {
        let lexeme_start = self.current_char;

        macro_rules! token {
            ($kind:expr) => {
                Some(Token::empty($kind, lexeme_start..=(self.current_char - 1)))
            };
            ($char:expr => $kind:expr, else => $other:expr) => {
                if self.find($char) {
                    token!($kind)
                } else {
                    token!($other)
                }
            };
        }

        match self.advance() {
            Some('(') => token!(TokenKind::LeftParen),
            Some(')') => token!(TokenKind::RightParen),
            Some('{') => token!(TokenKind::LeftBrace),
            Some('}') => token!(TokenKind::RightBrace),
            Some(',') => token!(TokenKind::Comma),
            Some('.') => token!(TokenKind::Dot),
            Some('-') => token!(TokenKind::Minus),
            Some('+') => token!(TokenKind::Plus),
            Some(';') => token!(TokenKind::Semicolon),
            Some('*') => token!(TokenKind::Star),
            Some('!') => token!('=' => TokenKind::BangEqual, else => TokenKind::Bang),
            Some('=') => token!('=' => TokenKind::EqualEqual, else => TokenKind::Equal),
            Some('<') => token!('=' => TokenKind::LessEqual, else => TokenKind::Less),
            Some('>') => token!('=' => TokenKind::GreaterEqual, else => TokenKind::Greater),
            Some('/') => {
                if self.find('/') {
                    self.consume_until('\n');
                    None
                } else {
                    token!(TokenKind::Slash)
                }
            }
            Some(' ') | Some('\r') | Some('\t') => None,
            Some('\n') => {
                self.current_line += 1;
                None
            }
            Some('"') => self.string(lexeme_start),
            Some('0'..='9') => self.number(lexeme_start),
            Some('a'..='z') | Some('A'..='Z') => self.ident(lexeme_start),
            Some(c) => {
                self.errors.push(
                    LoxError::UnexpectedCharacter {
                        c,
                        n: self.current_char - 1,
                    },
                    self.location,
                    self.current_line,
                );
                None
            }
            None => None,
        }
    }

    fn string(&mut self, start: usize) -> Option<Token<'src>> {
        // Current byte - 1 to include starting "
        let start_byte = self.current_byte()? - 1;

        if !self.consume_until('"') {
            self.errors.push(
                LoxError::UnterminatedString { start },
                self.location,
                self.current_line,
            );
            return None;
        }

        // Closing "
        let end_byte = self.current_byte()?;
        self.advance();

        let s = self.source.get((start_byte + 1)..=(end_byte - 1))?;
        Some(Token {
            kind: TokenKind::String(s),
            lexeme: self.source.get(start_byte..=end_byte),
            span: start..=(self.current_char - 1),
        })
    }

    fn number(&mut self, start: usize) -> Option<Token<'src>> {
        // Current byte - 1 to include starting digit
        let start_byte = self.current_byte()? - 1;

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

        // No idea why - 1
        let end_byte = self.current_byte()? - 1;
        let lexeme = self.source.get(start_byte..=end_byte)?;
        let n = lexeme
            .parse::<f64>()
            .or_else(|_| lexeme.parse::<u64>().map(|n| n as f64))
            .inspect_err(|_err| {
                self.errors.push(
                    LoxError::InvalidNumber {
                        s: lexeme.to_string(),
                    },
                    self.location,
                    self.current_line,
                );
            })
            .ok()?;

        Some(Token {
            kind: TokenKind::Number(n),
            lexeme: Some(lexeme),
            span: start..=(self.current_char - 1),
        })
    }

    fn ident(&mut self, start: usize) -> Option<Token<'src>> {
        // Current byte - 1 to include starting character
        let start_byte = self.current_byte()? - 1;

        while matches!(
            self.peek(),
            Some('0'..='9') | Some('a'..='z') | Some('A'..='Z')
        ) {
            self.advance()?;
        }
        let end_byte = self.current_byte()? - 1;

        let lexeme = self.source.get(start_byte..=end_byte)?;
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
            _ => TokenKind::Identifier,
        };

        Some(Token {
            kind,
            lexeme: Some(lexeme),
            span: start..=(self.current_char - 1),
        })
    }

    fn current_byte(&self) -> Option<usize> {
        self.iter.clone().peekable().next().map(|(idx, _)| idx)
    }

    fn advance(&mut self) -> Option<char> {
        let (_idx, char) = self.iter.next()?;
        self.current_char += 1;
        Some(char)
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

    fn consume_until(&mut self, expected: char) -> bool {
        while let Some(peek) = self.peek() {
            if peek == expected {
                return true;
            } else {
                let Some(c) = self.advance() else {
                    continue;
                };
                if c == '\n' {
                    self.current_line += 1;
                }
            }
        }
        false
    }

    fn is_end(&self) -> bool {
        self.iter.clone().peekable().next().is_none()
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    #[test]
    fn scan_string() {
        let tokens = Scanner::new(r#""string""#, None).scan().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::String("string"),
                    lexeme: Some("\"string\""),
                    span: 0..=7,
                },
                Token {
                    kind: TokenKind::Eof,
                    lexeme: None,
                    span: 8..=8
                }
            ]
        )
    }
}
