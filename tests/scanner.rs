use loxide::{
    error::HandleLoxResultIter,
    scanner::Scanner,
    source::SourceSpan,
    token::{Token, TokenKind},
};

fn t(kind: TokenKind<'_>, start: usize, end: usize) -> Token<'_> {
    Token {
        kind,
        span: SourceSpan {
            line: 0,
            char_range: start..=end,
            bytes_range: start..=end,
        },
    }
}

fn t_line(kind: TokenKind<'_>, line: usize, start: usize, end: usize) -> Token<'_> {
    Token {
        kind,
        span: SourceSpan {
            line,
            char_range: start..=end,
            bytes_range: start..=end,
        },
    }
}

fn eof(pos: usize) -> Token<'static> {
    Token {
        kind: TokenKind::Eof,
        span: SourceSpan {
            line: 0,
            char_range: pos..=pos,
            bytes_range: pos..=pos,
        },
    }
}

fn eof_line(line: usize, pos: usize) -> Token<'static> {
    Token {
        kind: TokenKind::Eof,
        span: SourceSpan {
            line,
            char_range: pos..=pos,
            bytes_range: pos..=pos,
        },
    }
}

fn scan(input: &str) -> (Vec<Token<'_>>, usize) {
    Scanner::scan(input).process_silent()
}

#[test]
fn scan_number() {
    assert_eq!(
        scan("300.003").0,
        vec![t(TokenKind::Number(300.003), 0, 6), eof(7)]
    );
    assert_eq!(scan("69").0, vec![t(TokenKind::Number(69.0), 0, 1), eof(2)]);
    assert_eq!(scan("0").0, vec![t(TokenKind::Number(0.0), 0, 0), eof(1)]);
    assert_eq!(
        scan("123456789").0,
        vec![t(TokenKind::Number(123456789.0), 0, 8), eof(9)]
    );
    assert_eq!(scan("0.5").0, vec![t(TokenKind::Number(0.5), 0, 2), eof(3)]);
}

#[test]
fn scan_string() {
    assert_eq!(
        scan("\"string\"").0,
        vec![t(TokenKind::String("string"), 0, 7), eof(8)]
    );
    assert_eq!(scan("\"\"").0, vec![t(TokenKind::String(""), 0, 1), eof(2)]);
    assert_eq!(
        scan("\"hello world\"").0,
        vec![t(TokenKind::String("hello world"), 0, 12), eof(13)]
    );
    assert_eq!(
        scan("\"multiple   spaces\"").0,
        vec![t(TokenKind::String("multiple   spaces"), 0, 18), eof(19)]
    );
}

#[test]
fn scan_punctuation() {
    assert_eq!(
        scan("( ) { } , . ;").0,
        vec![
            t(TokenKind::LeftParen, 0, 0),
            t(TokenKind::RightParen, 2, 2),
            t(TokenKind::LeftBrace, 4, 4),
            t(TokenKind::RightBrace, 6, 6),
            t(TokenKind::Comma, 8, 8),
            t(TokenKind::Dot, 10, 10),
            t(TokenKind::Semicolon, 12, 12),
            eof(13),
        ]
    );
}

#[test]
fn scan_operators() {
    assert_eq!(
        scan("- + * /").0,
        vec![
            t(TokenKind::Minus, 0, 0),
            t(TokenKind::Plus, 2, 2),
            t(TokenKind::Star, 4, 4),
            t(TokenKind::Slash, 6, 6),
            eof(7),
        ]
    );
}

#[test]
fn scan_two_char_operators() {
    assert_eq!(
        scan("!= == >= <= > <").0,
        vec![
            t(TokenKind::BangEqual, 0, 1),
            t(TokenKind::EqualEqual, 3, 4),
            t(TokenKind::GreaterEqual, 6, 7),
            t(TokenKind::LessEqual, 9, 10),
            t(TokenKind::Greater, 12, 12),
            t(TokenKind::Less, 14, 14),
            eof(15),
        ]
    );
}

#[test]
fn scan_keywords() {
    assert_eq!(scan("and").0, vec![t(TokenKind::And, 0, 2), eof(3)]);
    assert_eq!(scan("class").0, vec![t(TokenKind::Class, 0, 4), eof(5)]);
    assert_eq!(scan("else").0, vec![t(TokenKind::Else, 0, 3), eof(4)]);
    assert_eq!(scan("false").0, vec![t(TokenKind::False, 0, 4), eof(5)]);
    assert_eq!(scan("fun").0, vec![t(TokenKind::Fun, 0, 2), eof(3)]);
    assert_eq!(scan("for").0, vec![t(TokenKind::For, 0, 2), eof(3)]);
    assert_eq!(scan("if").0, vec![t(TokenKind::If, 0, 1), eof(2)]);
    assert_eq!(scan("nil").0, vec![t(TokenKind::Nil, 0, 2), eof(3)]);
    assert_eq!(scan("or").0, vec![t(TokenKind::Or, 0, 1), eof(2)]);
    assert_eq!(scan("print").0, vec![t(TokenKind::Print, 0, 4), eof(5)]);
    assert_eq!(scan("return").0, vec![t(TokenKind::Return, 0, 5), eof(6)]);
    assert_eq!(scan("super").0, vec![t(TokenKind::Super, 0, 4), eof(5)]);
    assert_eq!(scan("this").0, vec![t(TokenKind::This, 0, 3), eof(4)]);
    assert_eq!(scan("true").0, vec![t(TokenKind::True, 0, 3), eof(4)]);
    assert_eq!(scan("var").0, vec![t(TokenKind::Var, 0, 2), eof(3)]);
    assert_eq!(scan("while").0, vec![t(TokenKind::While, 0, 4), eof(5)]);
}

#[test]
fn scan_identifiers() {
    assert_eq!(
        scan("foo").0,
        vec![t(TokenKind::Identifier("foo"), 0, 2), eof(3)]
    );
    assert_eq!(
        scan("bar123").0,
        vec![t(TokenKind::Identifier("bar123"), 0, 5), eof(6)]
    );
    assert_eq!(
        scan("myVar").0,
        vec![t(TokenKind::Identifier("myVar"), 0, 4), eof(5)]
    );
    assert_eq!(
        scan("CamelCase").0,
        vec![t(TokenKind::Identifier("CamelCase"), 0, 8), eof(9)]
    );
}

#[test]
fn scan_comments() {
    assert_eq!(scan("// this is a comment").0, vec![eof(20)]);
    assert_eq!(scan("//").0, vec![eof(2)]);
    assert_eq!(scan("// a").0, vec![eof(4)]);
}

#[test]
fn scan_whitespace() {
    assert_eq!(scan("   ").0, vec![eof(3)]);
    assert_eq!(scan("\t\n\r").0, vec![eof_line(1, 3)]);
}

#[test]
fn scan_whitespace_with_tokens() {
    assert_eq!(
        scan(" x ").0,
        vec![t(TokenKind::Identifier("x"), 1, 1), eof(3)]
    );
    assert_eq!(scan("\t+\t").0, vec![t(TokenKind::Plus, 1, 1), eof(3)]);
    assert_eq!(
        scan("  42  ").0,
        vec![t(TokenKind::Number(42.0), 2, 3), eof(6)]
    );
    assert_eq!(
        scan("a\nb").0,
        vec![
            t_line(TokenKind::Identifier("a"), 0, 0, 0),
            t_line(TokenKind::Identifier("b"), 1, 2, 2),
            eof_line(1, 3),
        ]
    );
}

#[test]
fn scan_mixed_tokens() {
    let input = r#"var x = 10; if (x != 5) { print "hello" + "world"; } // test!"#;
    let tokens = scan(input).0;

    assert_eq!(tokens[0], t(TokenKind::Var, 0, 2));
    assert_eq!(tokens[1], t(TokenKind::Identifier("x"), 4, 4));
    assert_eq!(tokens[2], t(TokenKind::Equal, 6, 6));
    assert_eq!(tokens[3], t(TokenKind::Number(10.0), 8, 9));
    assert_eq!(tokens[4], t(TokenKind::Semicolon, 10, 10));
    assert_eq!(tokens[5], t(TokenKind::If, 12, 13));
    assert_eq!(tokens[6], t(TokenKind::LeftParen, 15, 15));
    assert_eq!(tokens[7], t(TokenKind::Identifier("x"), 16, 16));
    assert_eq!(tokens[8], t(TokenKind::BangEqual, 18, 19));
    assert_eq!(tokens[9], t(TokenKind::Number(5.0), 21, 21));
    assert_eq!(tokens[10], t(TokenKind::RightParen, 22, 22));
    assert_eq!(tokens[11], t(TokenKind::LeftBrace, 24, 24));
    assert_eq!(tokens[12], t(TokenKind::Print, 26, 30));
    assert_eq!(tokens[13], t(TokenKind::String("hello"), 32, 38));
    assert_eq!(tokens[14], t(TokenKind::Plus, 40, 40));
    assert_eq!(tokens[15], t(TokenKind::String("world"), 42, 48));
    assert_eq!(tokens[16], t(TokenKind::Semicolon, 49, 49));
    assert_eq!(tokens[17], t(TokenKind::RightBrace, 51, 51));
    assert_eq!(tokens[18], eof(61));
    assert_eq!(tokens.len(), 19);
}
