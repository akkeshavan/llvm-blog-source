//! Part 2: Grammar-driven lexer (pest)
//! The grammar is in grammar/lexer.pest

mod token;

use pest::Parser;
use pest_derive::Parser;
use std::str::FromStr;

pub use token::Token;

#[derive(Parser)]
#[grammar = "grammar/lexer.pest"]
struct LuminaLexerParser;

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub offset: usize,
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let pairs = LuminaLexerParser::parse(Rule::tokens, source)
        .map_err(|e| LexError {
            message: e.to_string(),
            offset: 0,
        })?;

    let mut tokens = Vec::new();
    for pair in pairs {
        if pair.as_rule() == Rule::tokens {
            for inner in pair.into_inner() {
                if let Some(tok) = pair_to_token(inner) {
                    tokens.push(tok);
                }
            }
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}

fn pair_to_token(pair: pest::iterators::Pair<Rule>) -> Option<Token> {
    match pair.as_rule() {
        Rule::let_kw => Some(Token::Let),
        Rule::fn_kw => Some(Token::Fn),
        Rule::type_kw => Some(Token::Type),
        Rule::match_kw => Some(Token::Match),
        Rule::with_kw => Some(Token::With),
        Rule::end_kw => Some(Token::End),
        Rule::if_kw => Some(Token::If),
        Rule::then_kw => Some(Token::Then),
        Rule::else_kw => Some(Token::Else),
        Rule::int => {
            let s = pair.as_str();
            i64::from_str(s).ok().map(Token::IntLit)
        }
        Rule::ident => Some(Token::Ident(pair.as_str().to_string())),
        Rule::eq => Some(Token::Eq),
        Rule::arrow => Some(Token::Arrow),
        Rule::pipe => Some(Token::Pipe),
        Rule::lt => Some(Token::Lt),
        Rule::gt => Some(Token::Gt),
        Rule::comma => Some(Token::Comma),
        Rule::semicolon => Some(Token::Semicolon),
        Rule::lparen => Some(Token::LParen),
        Rule::rparen => Some(Token::RParen),
        Rule::plus => Some(Token::Plus),
        Rule::minus => Some(Token::Minus),
        Rule::star => Some(Token::Star),
        _ => None,
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
    fn lex_int() {
        let r = lex("42").unwrap();
        assert_eq!(r, vec![Token::IntLit(42), Token::Eof]);
    }

    #[test]
    fn lex_add() {
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
    fn lex_keywords() {
        let r = lex("let fn type").unwrap();
        assert_eq!(r, vec![Token::Let, Token::Fn, Token::Type, Token::Eof]);
    }
}
