use crate::source::SourceSpan;

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub span: SourceSpan,
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Token::{:?}", self.kind,))?;

        if self.span.is_char() {
            f.write_fmt(format_args!("@{}", self.span.char_start()))?;
        } else {
            f.write_fmt(format_args!(
                "@{}..{}",
                self.span.char_start(),
                self.span.char_end()
            ))?;
        }

        Ok(())
    }
}

impl<'src> Token<'src> {
    pub fn empty(kind: TokenKind<'src>, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}
