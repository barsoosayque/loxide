use std::iter::Peekable;

use crate::{
    ast::{Expr, ExprKind, Stmt, StmtKind},
    error::{LoxError, LoxErrorKind, LoxResult},
    source::{IntoSource, Source, SourceSpan, SourceSpanTrackerStack},
    token::{Token, TokenKind},
};

pub struct Parser<'src, I>
where
    I: Iterator<Item = Token<'src>>,
{
    source: Source<'src>,
    stack: SourceSpanTrackerStack,
    tokens: Peekable<I>,
}

macro_rules! expect_many {
    ($parser:expr, $expect:expr; until $token_pat:pat, $delim_pat:pat) => {{
        let mut r = vec![];
        if !matches!(
            $parser.peek(),
            Some(Token {
                kind: $token_pat,
                ..
            })
        ) {
            loop {
                r.push($expect);
                if !expect!($parser, $delim_pat).is_some() {
                    break;
                }
            }
        }
        let _paren = consume!($parser, $token_pat)?;
        r
    }};
    ($parser:expr, $expect:expr; until $token_pat:pat) => {
        expect_many!($parser, $expect; until $token_pat, TokenKind::Comma)
    };
}

macro_rules! expect {
    ($parser:expr, $token:ident in $token_pat:pat => $ret:expr) => {
        matches!(
            $parser.peek(),
            Some(Token {
                #[allow(unused_variables)]
                kind: $token_pat,
                ..
            })
        )
        .then(|| match $parser.advance() {
            Ok(
                $token @ Token {
                    kind: $token_pat, ..
                },
            ) => {
                $parser.stack.push($token.span.clone());
                $ret
            }
            _ => unreachable!(),
        })
    };
    ($parser:expr, $token_pat:pat => $ret:expr) => {
        expect!($parser, _token in $token_pat => $ret)
    };
    ($parser:expr, $token_pat:pat) => {
        expect!($parser, token in $token_pat => token)
    };
}

macro_rules! consume {
    ($parser:expr, $token_pat:pat => $ret:expr, $expected:expr) => {
        expect!($parser, $token_pat => $ret)
            .ok_or_else(|| $parser.error_next_char(LoxErrorKind::Expected($expected)))
    };
    ($parser:expr, $token_pat:pat => $ret:expr) => {
        consume!($parser, $token_pat => $ret, stringify!($token_pat))
    };
    ($parser:expr, $token_pat:pat, $expected:expr) => {
        consume!($parser, a @ $token_pat => a, $expected)
    };
    ($parser:expr, $token_pat:pat) => {
        consume!($parser, a @ $token_pat => a)
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
            source: source.into_source(),
        }
    }

    pub fn decl(&mut self) -> LoxResult<'src, Stmt<'src>> {
        self.stack.push(self.stack.get());

        if let Some(_) = expect!(self, TokenKind::Var) {
            return self.var_decl();
        }

        self.stmt()
    }

    pub fn var_decl(&mut self) -> LoxResult<'src, Stmt<'src>> {
        let id = consume!(self, TokenKind::Identifier(id) => id, "identifier name")?;

        let init = expect!(self, TokenKind::Equal)
            .map(|_op| self.expr())
            .transpose()?
            .map(Box::new);

        consume!(self, TokenKind::Semicolon, "';' after variable declaration")?;

        return Ok(Stmt::new(
            StmtKind::VariableDecl { id, init },
            self.stack.pop(),
        ));
    }

    pub fn stmt(&mut self) -> LoxResult<'src, Stmt<'src>> {
        if let Some(_) = expect!(self, TokenKind::For) {
            consume!(self, TokenKind::LeftParen, "'(' after 'for'")?;
            let initializer = match expect!(self, t in TokenKind::Semicolon | TokenKind::Var => t.kind)
            {
                Some(TokenKind::Semicolon) => None,
                Some(TokenKind::Var) => Some(self.var_decl()?),
                _ => {
                    let expr = self.expr()?;
                    consume!(self, TokenKind::Semicolon, "';' after loop initializer")?;
                    Some(Stmt::new(StmtKind::Expr(Box::new(expr)), self.stack.pop()))
                }
            };
            let condition = match expect!(self, TokenKind::Semicolon) {
                Some(_) => Expr::new(ExprKind::LitBoolean(true), self.stack.pop()),
                _ => {
                    let expr = self.expr()?;
                    consume!(self, TokenKind::Semicolon, "';' after loop condition")?;
                    expr
                }
            };
            let increment = match expect!(self, TokenKind::RightParen) {
                Some(_) => None,
                _ => {
                    let expr = self.expr()?;
                    consume!(self, TokenKind::RightParen, "')' after loop increment")?;
                    Some(Stmt::new(StmtKind::Expr(Box::new(expr)), self.stack.pop()))
                }
            };

            let mut body = self.stmt()?;
            let span = body.span.clone();
            if let Some(increment) = increment {
                body = Stmt::new(
                    StmtKind::Block(vec![Box::new(body), Box::new(increment)]),
                    span.clone(),
                );
            }
            body = Stmt::new(
                StmtKind::While {
                    condition: Box::new(condition),
                    body: Box::new(body),
                },
                span.clone(),
            );
            if let Some(initializer) = initializer {
                body = Stmt::new(
                    StmtKind::Block(vec![Box::new(initializer), Box::new(body)]),
                    span.clone(),
                );
            }

            return Ok(body);
        }

        if let Some(_) = expect!(self, TokenKind::If) {
            consume!(self, TokenKind::LeftParen, "'(' after if")?;
            let condition = self.expr()?;
            consume!(self, TokenKind::RightParen, "')' after if")?;

            let then = self.stmt()?;
            let or_else = expect!(self, TokenKind::Else)
                .map(|_| self.stmt())
                .transpose()?;

            return Ok(Stmt::new(
                StmtKind::Conditional {
                    condition: Box::new(condition),
                    then: Box::new(then),
                    or_else: or_else.map(Box::new),
                },
                self.stack.pop(),
            ));
        }

        if let Some(_) = expect!(self, TokenKind::Print) {
            let expr = self.expr()?;
            consume!(self, TokenKind::Semicolon, "';' after print statement")?;
            return Ok(Stmt::new(StmtKind::Print(Box::new(expr)), self.stack.pop()));
        }

        if let Some(_) = expect!(self, TokenKind::While) {
            consume!(self, TokenKind::LeftParen, "'(' after 'while'")?;
            let condition = self.expr()?;
            consume!(self, TokenKind::RightParen, "')' after condition")?;
            let body = self.stmt()?;
            return Ok(Stmt::new(
                StmtKind::While {
                    condition: Box::new(condition),
                    body: Box::new(body),
                },
                self.stack.pop(),
            ));
        }

        if let Some(_) = expect!(self, TokenKind::Fun) {
            let name = consume!(self, TokenKind::Identifier(name) => name, "function name")?;
            consume!(self, TokenKind::LeftParen, "'(' after function name")?;
            let params = expect_many!(
                self,
                consume!(self, TokenKind::Identifier(name) => name, "parameter name")?;
                until TokenKind::RightParen
            );
            consume!(self, TokenKind::LeftBrace, "'{' before function body")?;
            let body = self.block()?;
            return Ok(Stmt::new(
                StmtKind::Function {
                    name,
                    params,
                    body: Box::new(body),
                },
                self.stack.pop(),
            ));
        }

        if let Some(_) = expect!(self, TokenKind::LeftBrace) {
            return self.block();
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

        consume!(self, TokenKind::Semicolon, "';' after statement")?;
        return Ok(Stmt::new(StmtKind::Expr(Box::new(expr)), self.stack.pop()));
    }

    pub fn block(&mut self) -> LoxResult<'src, Stmt<'src>> {
        let mut stmts = vec![];
        while match self.peek() {
            Some(Token {
                kind: TokenKind::RightBrace,
                ..
            }) => false,
            Some(_) => true,
            _ => false,
        } {
            stmts.push(Box::new(self.decl()?));
        }
        consume!(self, TokenKind::RightBrace, "'}' after block")?;
        Ok(Stmt::new(StmtKind::Block(stmts), self.stack.pop()))
    }

    pub fn expr(&mut self) -> LoxResult<'src, Expr<'src>> {
        self.assignment()
    }

    pub fn assignment(&mut self) -> LoxResult<'src, Expr<'src>> {
        let mut expr = self.or()?;

        if let Some(_op) = expect!(self, TokenKind::Equal) {
            match expr.kind {
                ExprKind::Var(id) => {
                    let value = self.assignment()?;
                    expr = Expr::new(
                        ExprKind::Assign {
                            id,
                            value: Box::new(value),
                        },
                        self.stack.pop(),
                    );
                }
                _ => {
                    return Err(self.error_for(LoxErrorKind::InvalidAssignmentTarget, expr.span));
                }
            }
        }

        Ok(expr)
    }

    pub fn or(&mut self) -> LoxResult<'src, Expr<'src>> {
        let mut expr = self.and()?;

        while let Some(op) = expect!(self, TokenKind::Or) {
            let right = self.and()?;
            expr = Expr::new(
                ExprKind::Logic {
                    left: Box::new(expr),
                    op: op.kind,
                    right: Box::new(right),
                },
                self.stack.pop(),
            );
        }

        Ok(expr)
    }

    pub fn and(&mut self) -> LoxResult<'src, Expr<'src>> {
        let mut expr = self.equality()?;

        while let Some(op) = expect!(self, TokenKind::And) {
            let right = self.equality()?;
            expr = Expr::new(
                ExprKind::Logic {
                    left: Box::new(expr),
                    op: op.kind,
                    right: Box::new(right),
                },
                self.stack.pop(),
            );
        }

        Ok(expr)
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

        self.call()
    }

    fn call(&mut self) -> LoxResult<'src, Expr<'src>> {
        let mut expr = self.primary()?;

        loop {
            if expect!(self, TokenKind::LeftParen).is_some() {
                let args = expect_many!(self, Box::new(self.expr()?); until TokenKind::RightParen);

                expr = Expr::new(
                    ExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    },
                    self.stack.pop(),
                );
            } else {
                break;
            }
        }

        Ok(expr)
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

        if let Some(s) = expect!(self, TokenKind::String(s) => s) {
            return Ok(Expr::new(ExprKind::LitString(s), self.stack.pop()));
        }
        if let Some(n) = expect!(self, TokenKind::Number(n) => n) {
            return Ok(Expr::new(ExprKind::LitNumber(n), self.stack.pop()));
        }
        if let Some(id) = expect!(self, TokenKind::Identifier(id) => id) {
            return Ok(Expr::new(ExprKind::Var(id), self.stack.pop()));
        }

        if let Some(_) = expect!(self, TokenKind::LeftParen) {
            let inner = self.expr()?;
            consume!(self, TokenKind::RightParen, "closing ')'")?;
            return Ok(Expr::new(
                ExprKind::Grouping {
                    inner: Box::new(inner),
                },
                self.stack.pop(),
            ));
        }

        if let Some(_) = self.peek() {
            let Token { kind, span } = self.advance()?;
            Err(self.error_for(LoxErrorKind::UnexpectedToken(kind), span))
        } else {
            Err(self.error_next_char(LoxErrorKind::ExpectedExpr))
        }
    }

    fn peek(&mut self) -> Option<&Token<'src>> {
        self.tokens.peek()
    }

    fn advance(&mut self) -> LoxResult<'src, Token<'src>> {
        let Some(next) = self.tokens.next() else {
            return Err(self.error_next_char(LoxErrorKind::UnexpectedEof));
        };

        if let Some(Token { span, .. }) = self.peek().cloned() {
            self.stack.advance_to(span.clone());
        } else {
            self.stack.advance_to(next.span.clone());
        }

        Ok(next)
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

    fn error_for(&self, kind: LoxErrorKind<'src>, span: SourceSpan) -> LoxError<'src> {
        LoxError::new(kind, self.source.clone(), span)
    }

    fn error_next_char(&self, kind: LoxErrorKind<'src>) -> LoxError<'src> {
        let span = self.stack.get();
        let span = SourceSpan {
            line: span.line,
            char_range: (span.char_end().saturating_add(1)..=span.char_end().saturating_add(1)),
            bytes_range: (span.bytes_end().saturating_add(1)..=span.bytes_end().saturating_add(1)),
        };
        LoxError::new(kind, self.source.clone(), span)
    }

    fn is_end(&mut self) -> bool {
        return matches!(
            self.peek(),
            None | Some(Token {
                kind: TokenKind::Eof,
                ..
            })
        );
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

        Some(self.decl().inspect_err(|err| {
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
