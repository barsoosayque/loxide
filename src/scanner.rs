use peek_again::Peekable;
use std::str::CharIndices;

use crate::{
    error::{LoxError, LoxErrorKind, LoxResult},
    source::{IntoSource, Source, SourceSpan, SourceSpanTracker},
    token::{Token, TokenKind},
};

#[derive(Debug)]
pub struct Scanner<'src> {
    source: Source<'src>,
    iter: Peekable<CharIndices<'src>>,
    tracker: SourceSpanTracker,
    is_terminated: bool,
}

impl<'src> Scanner<'src> {
    pub fn scan(source: impl IntoSource<'src>) -> Self {
        let source = source.into_source();
        let iter = Peekable::new(source.script.char_indices());

        Self {
            source,
            iter,
            tracker: SourceSpanTracker::default(),
            is_terminated: false,
        }
    }

    fn next_token(&mut self) -> LoxResult<'src, Token<'src>> {
        macro_rules! token {
            ($kind:expr) => {
                Ok(Token {
                    kind: $kind,
                    span: self.tracker.consume(),
                })
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
            '"' => self.string(),
            '0'..='9' => self.number(),
            'a'..='z' | 'A'..='Z' => self.ident(),
            ' ' | '\r' | '\t' | '\n' => {
                self.tracker.consume();
                self.next_token()
            }
            c => {
                self.tracker.consume();
                Err(self.error(LoxErrorKind::UnexpectedCharacter(c)))
            }
        }
    }

    fn try_next_token(&mut self) -> LoxResult<'src, Option<Token<'src>>> {
        match self.next_token() {
            Ok(token) => Ok(Some(token)),
            Err(LoxError {
                kind: LoxErrorKind::UnexpectedEof,
                ..
            }) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn string(&mut self) -> LoxResult<'src, Token<'src>> {
        if !self.consume_until('"')? {
            return Err(self.error(LoxErrorKind::UnterminatedString));
        }

        self.advance()?;

        let lexeme = self.current_span_lexeme();
        let s = lexeme.trim_matches('"');

        Ok(Token {
            kind: TokenKind::String(s),
            span: self.tracker.consume(),
        })
    }

    fn number(&mut self) -> LoxResult<'src, Token<'src>> {
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance()?;
        }

        match (self.peek(), self.peek_2()) {
            (Some('.'), Some('0'..='9')) => {
                self.advance()?;
                while matches!(self.peek(), Some('0'..='9')) {
                    self.advance()?;
                }
            }
            _ => {}
        }

        let lexeme = self.current_span_lexeme();
        let n = lexeme
            .parse::<f64>()
            .or_else(|_| lexeme.parse::<u64>().map(|n| n as f64))
            .map_err(|_err| self.error(LoxErrorKind::InvalidNumber(lexeme.to_string())))?;

        Ok(Token {
            kind: TokenKind::Number(n),
            span: self.tracker.consume(),
        })
    }

    fn ident(&mut self) -> LoxResult<'src, Token<'src>> {
        while matches!(
            self.peek(),
            Some('0'..='9') | Some('a'..='z') | Some('A'..='Z')
        ) {
            self.advance()?;
        }

        let lexeme = self.current_span_lexeme();
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
            span: self.tracker.consume(),
        })
    }

    fn error(&self, kind: LoxErrorKind<'src>) -> LoxError<'src> {
        LoxError::new(kind, self.source.clone(), self.tracker.get())
    }

    fn current_span_lexeme(&self) -> &'src str {
        self.source.span(&self.tracker.get())
    }

    fn advance(&mut self) -> LoxResult<'src, char> {
        let (_idx, char) = self
            .iter
            .next()
            .ok_or_else(|| self.error(LoxErrorKind::UnexpectedEof))?;

        match char {
            '\n' => {
                self.tracker.advance_line(1);
            }
            _ => {}
        }
        self.tracker.advance_char(char);

        Ok(char)
    }

    fn peek(&mut self) -> Option<char> {
        self.iter.peek().get().map(|(_, c)| *c)
    }

    fn peek_2(&mut self) -> Option<char> {
        self.iter.peek_2().map(|(_, c)| *c)
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

    fn consume_until(&mut self, expected: char) -> LoxResult<'src, bool> {
        while let Some(peek) = self.peek() {
            if peek == expected {
                return Ok(true);
            } else {
                match self.advance() {
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

impl<'src> Iterator for Scanner<'src> {
    type Item = LoxResult<'src, Token<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_end() {
            if self.is_terminated {
                return None;
            } else {
                self.is_terminated = true;
                return Some(Ok(Token {
                    kind: TokenKind::Eof,
                    span: self.tracker.eof(),
                }));
            }
        }
        self.try_next_token().transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::HandleLoxResultIter;

    fn simple_token(kind: TokenKind<'_>, len: usize) -> Vec<Token<'_>> {
        let end = len - 1;
        vec![
            Token {
                kind,
                span: SourceSpan {
                    line: 0,
                    char_range: 0..=end,
                    bytes_range: 0..=end,
                },
            },
            Token {
                kind: TokenKind::Eof,
                span: SourceSpan {
                    line: 0,
                    char_range: len..=len,
                    bytes_range: len..=len,
                },
            },
        ]
    }

    #[test]
    fn scan_number() {
        assert_eq!(
            Scanner::scan(r#"300.003"#).process_silent(),
            simple_token(TokenKind::Number(300.003), 7)
        );
        assert_eq!(
            Scanner::scan(r#"69"#).process_silent(),
            simple_token(TokenKind::Number(69.0), 2)
        );
        assert_ne!(
            Scanner::scan(r#"200."#).process_silent(),
            simple_token(TokenKind::Number(200.0), 4)
        );
    }

    #[test]
    fn scan_string() {
        assert_eq!(
            Scanner::scan(r#""string""#).process_silent(),
            simple_token(TokenKind::String("string"), 8)
        );
    }
}
