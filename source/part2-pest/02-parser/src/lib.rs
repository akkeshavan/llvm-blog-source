//! Part 2: Grammar-driven parser (pest)
//! Grammar in grammar/expr.pest

use pest::Parser;
use pest_derive::Parser;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "grammar/expr.pest"]
struct LuminaExprParser;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Var(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub enum ParseError {
    Unexpected(String),
}

pub fn parse(source: &str) -> Result<Expr, ParseError> {
    let pairs = LuminaExprParser::parse(Rule::program, source).map_err(|e| {
        ParseError::Unexpected(e.to_string())
    })?;

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::expr {
                    return Ok(build_expr(inner));
                }
            }
        }
    }
    Err(ParseError::Unexpected("no expr found".into()))
}

fn build_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::expr => {
            let inner = pair.into_inner().next().unwrap();
            build_expr(inner)
        }
        Rule::add_sub => {
            let inner: Vec<_> = pair.into_inner().collect();
            let mut result = build_atom_or_mul(inner[0].clone());
            for i in 1..inner.len() {
                let parts: Vec<_> = inner[i].clone().into_inner().collect();
                let op = parts[0].as_str(); // add_op
                let rhs = build_atom_or_mul(parts[1].clone()); // mul
                result = if op == "+" {
                    Expr::Add(Box::new(result), Box::new(rhs))
                } else {
                    Expr::Sub(Box::new(result), Box::new(rhs))
                };
            }
            result
        }
        Rule::mul => {
            let inner: Vec<_> = pair.into_inner().collect();
            let mut result = build_atom(inner[0].clone());
            for i in 1..inner.len() {
                let parts: Vec<_> = inner[i].clone().into_inner().collect();
                let rhs = build_atom(parts[1].clone()); // parts[0]=star, parts[1]=atom
                result = Expr::Mul(Box::new(result), Box::new(rhs));
            }
            result
        }
        Rule::atom => build_atom(pair),
        _ => unreachable!(),
    }
}

fn build_atom_or_mul(pair: pest::iterators::Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::mul => build_expr(pair),
        Rule::atom => build_atom(pair),
        _ => unreachable!(),
    }
}

fn build_atom(pair: pest::iterators::Pair<Rule>) -> Expr {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::expr => build_expr(inner),
        Rule::int => Expr::Int(i64::from_str(inner.as_str()).unwrap()),
        Rule::ident => Expr::Var(inner.as_str().to_string()),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_int() {
        assert_eq!(parse("42").unwrap(), Expr::Int(42));
    }

    #[test]
    fn parse_add() {
        let e = parse("1 + 2").unwrap();
        assert_eq!(e, Expr::Add(Box::new(Expr::Int(1)), Box::new(Expr::Int(2))));
    }

    #[test]
    fn parse_precedence() {
        let e = parse("1 + 2 * 3").unwrap();
        assert_eq!(
            e,
            Expr::Add(
                Box::new(Expr::Int(1)),
                Box::new(Expr::Mul(
                    Box::new(Expr::Int(2)),
                    Box::new(Expr::Int(3))
                ))
            )
        );
    }

    #[test]
    fn parse_parens() {
        let e = parse("(1 + 2) * 3").unwrap();
        assert_eq!(
            e,
            Expr::Mul(
                Box::new(Expr::Add(Box::new(Expr::Int(1)), Box::new(Expr::Int(2)))),
                Box::new(Expr::Int(3))
            )
        );
    }
}
