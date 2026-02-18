use lumina_part1_lexer::{lex, Token};

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
    let tokens = lex(source).map_err(|e| ParseError::Unexpected(e.message))?;
    let mut parser = Parser::new(&tokens);
    parser.parse_expr()
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_primary()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => {
                    self.advance();
                    BinOp::Add
                }
                Some(Token::Minus) => {
                    self.advance();
                    BinOp::Sub
                }
                Some(Token::Star) => {
                    self.advance();
                    BinOp::Mul
                }
                _ => break,
            };
            let (l_bp, r_bp) = op.precedence();
            if l_bp < min_bp {
                break;
            }
            let rhs = self.parse_expr_bp(r_bp)?;
            lhs = match op {
                BinOp::Add => Expr::Add(Box::new(lhs), Box::new(rhs)),
                BinOp::Sub => Expr::Sub(Box::new(lhs), Box::new(rhs)),
                BinOp::Mul => Expr::Mul(Box::new(lhs), Box::new(rhs)),
            };
        }
        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Some(Token::IntLit(n)) => Ok(Expr::Int(*n)),
            Some(Token::Ident(s)) => Ok(Expr::Var(s.clone())),
            Some(Token::LParen) => {
                let e = self.parse_expr()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(e),
                    _ => Err(ParseError::Unexpected("expected )".into())),
                }
            }
            _ => Err(ParseError::Unexpected("expected primary expression".into())),
        }
    }
}

enum BinOp {
    Add,
    Sub,
    Mul,
}

impl BinOp {
    fn precedence(&self) -> (u8, u8) {
        match self {
            BinOp::Add | BinOp::Sub => (1, 2),
            BinOp::Mul => (3, 4),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_int() {
        let expr = parse("42").unwrap();
        assert_eq!(expr, Expr::Int(42));
    }

    #[test]
    fn parse_add() {
        let expr = parse("1 + 2").unwrap();
        assert_eq!(
            expr,
            Expr::Add(Box::new(Expr::Int(1)), Box::new(Expr::Int(2)))
        );
    }

    #[test]
    fn parse_precedence() {
        let expr = parse("1 + 2 * 3").unwrap();
        assert_eq!(
            expr,
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
        let expr = parse("(1 + 2) * 3").unwrap();
        assert_eq!(
            expr,
            Expr::Mul(
                Box::new(Expr::Add(Box::new(Expr::Int(1)), Box::new(Expr::Int(2)))),
                Box::new(Expr::Int(3))
            )
        );
    }
}
