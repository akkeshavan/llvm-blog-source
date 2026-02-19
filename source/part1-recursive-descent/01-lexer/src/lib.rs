mod token;

use std::iter::Peekable;
use std::str::CharIndices;

pub use token::Token;

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub offset: usize,
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut it = source.char_indices().peekable();

    loop {
        while it.peek().map(|(_, c)| c.is_ascii_whitespace()) == Some(true) {
            it.next();
        }
        if let Some(&(i, c)) = it.peek() {
            let tok = match c {
                '0'..='9' => lex_int(&mut it).map_err(|msg| LexError {
                    message: msg,
                    offset: i,
                })?,
                '"' => lex_string(&mut it).map_err(|msg| LexError {
                    message: msg,
                    offset: i,
                })?,
                'a'..='z' | 'A'..='Z' | '_' => lex_ident_or_keyword(&mut it),
                '=' => {
                    it.next();
                    if it.peek().map(|(_, c)| *c) == Some('=') {
                        it.next();
                        Token::EqEq
                    } else {
                        Token::Eq
                    }
                }
                '!' => {
                    it.next();
                    if it.peek().map(|(_, c)| *c) == Some('=') {
                        it.next();
                        Token::Ne
                    } else {
                        return Err(LexError {
                            message: "unexpected '!'".into(),
                            offset: i,
                        });
                    }
                }
                '|' => {
                    it.next();
                    Token::Pipe
                }
                '-' => {
                    it.next();
                    if it.peek().map(|(_, c)| *c) == Some('>') {
                        it.next();
                        Token::Arrow
                    } else {
                        Token::Minus
                    }
                }
                '<' => {
                    it.next();
                    if it.peek().map(|(_, c)| *c) == Some('=') {
                        it.next();
                        Token::Le
                    } else {
                        Token::Lt
                    }
                }
                '>' => {
                    it.next();
                    if it.peek().map(|(_, c)| *c) == Some('=') {
                        it.next();
                        Token::Ge
                    } else {
                        Token::Gt
                    }
                }
                ',' => {
                    it.next();
                    Token::Comma
                }
                ':' => {
                    it.next();
                    Token::Colon
                }
                ';' => {
                    it.next();
                    Token::Semicolon
                }
                '+' => {
                    it.next();
                    Token::Plus
                }
                '*' => {
                    it.next();
                    Token::Star
                }
                '/' => {
                    it.next();
                    Token::Slash
                }
                '%' => {
                    it.next();
                    Token::Percent
                }
                '.' => {
                    it.next();
                    if it.peek().map(|(_, c)| *c) == Some('.') {
                        it.next(); // consume second '.'
                        if it.peek().map(|(_, c)| *c) == Some('=') {
                            it.next();
                            Token::DotDotEq
                        } else {
                            Token::DotDot
                        }
                    } else {
                        Token::Dot
                    }
                }
                '(' => {
                    it.next();
                    Token::LParen
                }
                ')' => {
                    it.next();
                    Token::RParen
                }
                '{' => {
                    it.next();
                    Token::LBrace
                }
                '}' => {
                    it.next();
                    Token::RBrace
                }
                '[' => {
                    it.next();
                    Token::LBracket
                }
                ']' => {
                    it.next();
                    Token::RBracket
                }
                _ => {
                    return Err(LexError {
                        message: format!("unexpected character: {:?}", c),
                        offset: i,
                    });
                }
            };
            tokens.push(tok);
        } else {
            break;
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}

fn lex_string(it: &mut Peekable<CharIndices>) -> Result<Token, String> {
    it.next(); // consume opening "
    let mut s = String::new();
    while let Some(&(_, c)) = it.peek() {
        if c == '"' {
            it.next();
            return Ok(Token::StrLit(s));
        }
        if c == '\\' {
            it.next();
            match it.next() {
                Some((_, 'n')) => s.push('\n'),
                Some((_, 't')) => s.push('\t'),
                Some((_, '"')) => s.push('"'),
                Some((_, '\\')) => s.push('\\'),
                _ => return Err("invalid escape in string".into()),
            }
            continue;
        }
        if c == '\n' {
            return Err("unterminated string".into());
        }
        it.next();
        s.push(c);
    }
    Err("unterminated string".into())
}

fn lex_int(it: &mut Peekable<CharIndices>) -> Result<Token, String> {
    let mut digits = String::new();
    while it.peek().map(|(_, c)| c.is_ascii_digit()) == Some(true) {
        if let Some((_, c)) = it.next() {
            digits.push(c);
        }
    }
    digits
        .parse::<i64>()
        .map(Token::IntLit)
        .map_err(|_| "invalid integer".to_string())
}

fn lex_ident_or_keyword(it: &mut Peekable<CharIndices>) -> Token {
    let mut s = String::new();
    while it
        .peek()
        .map(|(_, c)| c.is_ascii_alphanumeric() || *c == '_')
        == Some(true)
    {
        if let Some((_, c)) = it.next() {
            s.push(c);
        }
    }
    if Token::is_keyword(&s) {
        keyword_to_token(&s)
    } else {
        Token::Ident(s)
    }
}

fn keyword_to_token(s: &str) -> Token {
    match s {
        "let" => Token::Let,
        "for" => Token::For,
        "in" => Token::In,
        "type" => Token::Type,
        "match" => Token::Match,
        "with" => Token::With,
        "end" => Token::End,
        "fn" => Token::Fn,
        "if" => Token::If,
        "then" => Token::Then,
        "else" => Token::Else,
        "return" => Token::Return,
        "null" => Token::Null,
        "typeof" => Token::Typeof,
        _ => Token::Ident(s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_empty() {
        let r = lex("").unwrap();
        assert_eq!(r, vec![Token::Eof]);
    }

    #[test]
    fn lex_keywords() {
        let r = lex("let type fn").unwrap();
        assert_eq!(r, vec![Token::Let, Token::Type, Token::Fn, Token::Eof]);
    }

    #[test]
    fn lex_int_lit() {
        let r = lex("42").unwrap();
        assert_eq!(r, vec![Token::IntLit(42), Token::Eof]);
    }

    #[test]
    fn lex_arrow() {
        let r = lex("->").unwrap();
        assert_eq!(r, vec![Token::Arrow, Token::Eof]);
    }

    #[test]
    fn lex_ident() {
        let r = lex("foo bar_1").unwrap();
        assert_eq!(
            r,
            vec![
                Token::Ident("foo".into()),
                Token::Ident("bar_1".into()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn lex_add_expr() {
        let r = lex("1 + 2").unwrap();
        assert_eq!(
            r,
            vec![
                Token::IntLit(1),
                Token::Plus,
                Token::IntLit(2),
                Token::Eof
            ]
        );
    }

    #[test]
    fn lex_string_lit() {
        let r = lex(r#"println("Hello, World!")"#).unwrap();
        assert!(matches!(&r[0], Token::Ident(_)));
        assert!(matches!(&r[1], Token::LParen));
        assert!(matches!(&r[2], Token::StrLit(s) if s == "Hello, World!"));
    }

    #[test]
    fn lex_range_tokens() {
        let r = lex("1..10 1..=10").unwrap();
        assert!(r.contains(&Token::DotDot));
        assert!(r.contains(&Token::DotDotEq));
    }
}
