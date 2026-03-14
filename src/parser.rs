use std::iter::Peekable;

use crate::{
    ast::{Expr, ExprKind, Stmt, StmtKind},
    error::{LoxError, LoxErrorKind, LoxResult},
    source::{IntoSource, Source, SourceSpanTracker, SourceSpanTrackerStack},
    token::{Token, TokenKind},
};

pub struct Parser<'src, I>
where
    I: Iterator<Item = Token<'src>>,
{
    source: Source<'src>,
    stack: SourceSpanTrackerStack,
    tracker: SourceSpanTracker,
    tokens: Peekable<I>,
}

macro_rules! expect {
    ($parser:expr, $token_pat:pat) => {
        if let Some(token) = $parser.peek() {
            match token.kind {
                $token_pat => {
                    let token = $parser.advance()?;
                    $parser.stack.push(token.span.clone());
                    Some(token)
                }
                _ => None,
            }
        } else {
            None
        }
    };
}

macro_rules! binary {
    ($name:ident < $next:ident, $token_pat:pat) => {
        fn $name(&mut self) -> LoxResult<'src, Expr<'src>> {
            let mut expr = self.$next()?;

            while let Some(op) = expect!(self, $token_pat) {
                let right = self.$next()?;
                expr = Expr::new(
                    ExprKind::Binary {
                        left: Box::new(expr),
                        op: op.kind,
                        right: Box::new(right),
                    },
                    self.stack.pop(),
                );
            }

            Ok(expr)
        }
    };
}

impl<'src, I> Parser<'src, I>
where
    I: Iterator<Item = Token<'src>>,
{
    pub fn parse<T>(tokens: T, source: impl IntoSource<'src>) -> Self
    where
        T: IntoIterator<Item = Token<'src>, IntoIter = I>,
    {
        Self {
            tokens: tokens.into_iter().peekable(),
            stack: SourceSpanTrackerStack::default(),
            tracker: SourceSpanTracker::default(),
            source: source.into_source(),
        }
    }

    pub fn stmt(&mut self) -> LoxResult<'src, Stmt<'src>> {
        self.stack.push(self.stack.get());

        if let Some(_) = expect!(self, TokenKind::Print) {
            let expr = self.expr()?;
            self.consume(TokenKind::Semicolon)?;
            return Ok(Stmt::new(StmtKind::Print(Box::new(expr)), self.stack.pop()));
        }

        let expr = self.expr()?;
        if let Some(Token {
            kind: TokenKind::Eof,
            ..
        }) = self.peek()
        {
            return Ok(Stmt::new(
                StmtKind::ExprReturn(Box::new(expr)),
                self.stack.pop(),
            ));
        }

        self.consume(TokenKind::Semicolon)?;
        return Ok(Stmt::new(StmtKind::Expr(Box::new(expr)), self.stack.pop()));
    }

    pub fn expr(&mut self) -> LoxResult<'src, Expr<'src>> {
        self.equality()
    }

    binary!(
        equality < comparison,
        TokenKind::BangEqual | TokenKind::EqualEqual
    );
    binary!(
        comparison < term,
        TokenKind::Greater | TokenKind::GreaterEqual | TokenKind::Less | TokenKind::LessEqual
    );
    binary!(term < factor, TokenKind::Minus | TokenKind::Plus);
    binary!(factor < unary, TokenKind::Slash | TokenKind::Star);

    fn unary(&mut self) -> LoxResult<'src, Expr<'src>> {
        if let Some(op) = expect!(self, TokenKind::Bang | TokenKind::Minus) {
            let right = self.unary()?;
            return Ok(Expr::new(
                ExprKind::Unary {
                    op: op.kind,
                    right: Box::new(right),
                },
                self.stack.pop(),
            ));
        }

        self.primary()
    }

    fn primary(&mut self) -> LoxResult<'src, Expr<'src>> {
        if let Some(_) = expect!(self, TokenKind::False) {
            return Ok(Expr::new(ExprKind::LitBoolean(false), self.stack.pop()));
        }
        if let Some(_) = expect!(self, TokenKind::True) {
            return Ok(Expr::new(ExprKind::LitBoolean(true), self.stack.pop()));
        }
        if let Some(_) = expect!(self, TokenKind::Nil) {
            return Ok(Expr::new(ExprKind::LitNil, self.stack.pop()));
        }

        if let Some(Token {
            kind: TokenKind::String(s),
            ..
        }) = expect!(self, TokenKind::String(..))
        {
            return Ok(Expr::new(ExprKind::LitString(s), self.stack.pop()));
        }
        if let Some(Token {
            kind: TokenKind::Number(n),
            ..
        }) = expect!(self, TokenKind::Number(..))
        {
            return Ok(Expr::new(ExprKind::LitNumber(n), self.stack.pop()));
        }

        if let Some(_) = expect!(self, TokenKind::LeftParen) {
            let inner = self.expr()?;
            self.consume(TokenKind::RightParen)?;
            return Ok(Expr::new(
                ExprKind::Grouping {
                    inner: Box::new(inner),
                },
                self.stack.pop(),
            ));
        }

        if let Some(_) = self.peek() {
            let Token { kind, .. } = self.advance()?;
            Err(self.error(LoxErrorKind::UnexpectedToken(kind)))
        } else {
            Err(self.error(LoxErrorKind::ExpectedExpr))
        }
    }

    fn peek(&mut self) -> Option<&Token<'src>> {
        self.tokens.peek()
    }

    fn advance(&mut self) -> LoxResult<'src, Token<'src>> {
        let Some(next) = self.tokens.next() else {
            return Err(self.error(LoxErrorKind::UnexpectedEof));
        };

        if let Some(Token { span, .. }) = self.peek().cloned() {
            self.stack.advance_to(span.clone());
        } else {
            self.stack.advance_to(next.span.clone());
        }
        self.tracker.set(next.span.clone());

        Ok(next)
    }

    fn consume(&mut self, kind: TokenKind<'src>) -> LoxResult<'src, Token<'src>> {
        let Some(next) = self.peek() else {
            return Err(self.error(LoxErrorKind::ExpectedToken(kind)));
        };

        if next.kind == kind {
            return self.advance();
        }

        Err(self.error(LoxErrorKind::ExpectedToken(kind)))
    }

    fn synchronize(&mut self) {
        while !self.is_end() {
            if let Ok(Token {
                kind: TokenKind::Semicolon,
                ..
            }) = self.advance()
            {
                return;
            }

            match self.peek() {
                Some(Token {
                    kind:
                        TokenKind::Class
                        | TokenKind::Fun
                        | TokenKind::Var
                        | TokenKind::For
                        | TokenKind::If
                        | TokenKind::While
                        | TokenKind::Print
                        | TokenKind::Return,
                    ..
                }) => {
                    return;
                }
                _ => {}
            }
        }
    }

    fn error(&self, kind: LoxErrorKind<'src>) -> LoxError<'src> {
        LoxError::new(kind, self.source.clone(), self.tracker.get())
    }

    fn is_end(&mut self) -> bool {
        matches!(
            self.peek(),
            None | Some(Token {
                kind: TokenKind::Eof,
                ..
            })
        )
    }
}

impl<'src, I> Iterator for Parser<'src, I>
where
    I: Iterator<Item = Token<'src>>,
{
    type Item = LoxResult<'src, Stmt<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_end() {
            return None;
        }

        Some(self.stmt().inspect_err(|err| {
            match err.kind {
                LoxErrorKind::UnexpectedEof => {
                    // unrecoverable
                }
                _ => {
                    self.synchronize();
                }
            }
        }))
    }
}
