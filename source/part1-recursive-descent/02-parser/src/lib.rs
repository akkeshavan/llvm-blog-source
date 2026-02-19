use lumina_part1_lexer::{lex, Token};

/// Parsed type annotation (parser level).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeAnn {
    I64,
    Str,
    Unit,
    Array(Box<TypeAnn>),
    Optional(Box<TypeAnn>),
    /// Record type inline: { name: Type, ... }
    Record(Vec<(String, TypeAnn)>),
    /// Named type (record alias or type param in generic fn)
    Named(String),
    /// Sum type: Variant1(T1, T2) | Variant2 | ...
    Sum(Vec<SumVariant>),
}

/// One variant of a sum type: name and optional payload types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumVariant {
    pub name: String,
    pub payload: Vec<TypeAnn>,
}

/// Type definition: type Name = Body (Body can be record, alias, or sum).
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub body: TypeAnn,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Str(String),
    Var(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
    Unit,
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
        step: Option<Box<Expr>>,
    },
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    ArrayLit(Vec<Expr>),
    Null,
    /// typeof expr
    TypeOf(Box<Expr>),
    /// base.field
    FieldAccess(Box<Expr>, String),
    /// { field: expr, ... }
    RecordLit(Vec<(String, Expr)>),
    /// match expr with | Variant(x, y) -> e1 | Other -> e2 end
    Match(Box<Expr>, Vec<MatchArm>),
}

/// One arm of a match: variant name, binding names (for payload), body expression.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub variant: String,
    pub bindings: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// let name [: Type] = expr;
    Let(String, Option<TypeAnn>, Expr),
    Expr(Expr),
    Return(Option<Expr>),
    For {
        var: String,
        range: Expr,
        body: Vec<Stmt>,
    },
    /// lhs = rhs; where lhs is Var or FieldAccess
    Assign(Expr, Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<(String, TypeAnn)>,
    pub return_type: TypeAnn,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub type_defs: Vec<TypeDef>,
    pub functions: Vec<FunctionDef>,
}

#[derive(Debug)]
pub enum ParseError {
    Unexpected(String),
}

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = lex(source).map_err(|e| ParseError::Unexpected(e.message))?;
    let mut parser = Parser::new(&tokens);
    parser.parse_program()
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

    fn expect_lparen(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::LParen) => Ok(()),
            _ => Err(ParseError::Unexpected("expected (".into())),
        }
    }
    fn expect_rparen(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::RParen) => Ok(()),
            _ => Err(ParseError::Unexpected("expected )".into())),
        }
    }
    fn expect_lbrace(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::LBrace) => Ok(()),
            _ => Err(ParseError::Unexpected("expected {".into())),
        }
    }
    fn expect_rbrace(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::RBrace) => Ok(()),
            _ => Err(ParseError::Unexpected("expected }".into())),
        }
    }
    fn expect_semicolon(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::Semicolon) => Ok(()),
            _ => Err(ParseError::Unexpected("expected ;".into())),
        }
    }
    fn expect_eq(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::Eq) => Ok(()),
            _ => Err(ParseError::Unexpected("expected =".into())),
        }
    }
    fn expect_comma(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::Comma) => Ok(()),
            _ => Err(ParseError::Unexpected("expected ,".into())),
        }
    }
    fn expect_lt(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::Lt) => Ok(()),
            _ => Err(ParseError::Unexpected("expected <".into())),
        }
    }
    fn expect_gt(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::Gt) => Ok(()),
            _ => Err(ParseError::Unexpected("expected >".into())),
        }
    }
    fn expect_colon(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::Colon) => Ok(()),
            _ => Err(ParseError::Unexpected("expected :".into())),
        }
    }
    fn expect_then(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::Then) => Ok(()),
            _ => Err(ParseError::Unexpected("expected then".into())),
        }
    }
    fn expect_else(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::Else) => Ok(()),
            _ => Err(ParseError::Unexpected("expected else".into())),
        }
    }
    fn expect_arrow(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::Arrow) => Ok(()),
            _ => Err(ParseError::Unexpected("expected ->".into())),
        }
    }
    fn expect_with(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::With) => Ok(()),
            _ => Err(ParseError::Unexpected("expected with".into())),
        }
    }
    fn expect_end(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            Some(Token::End) => Ok(()),
            _ => Err(ParseError::Unexpected("expected end".into())),
        }
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut type_defs = Vec::new();
        while matches!(self.peek(), Some(Token::Type)) {
            type_defs.push(self.parse_type_def()?);
        }
        let mut functions = Vec::new();
        if !matches!(self.peek(), Some(Token::Fn)) {
            let body = if matches!(self.peek(), Some(Token::Let) | Some(Token::For) | Some(Token::Return)) {
                let mut stmts = Vec::new();
                while !matches!(self.peek(), Some(Token::Eof)) {
                    stmts.push(self.parse_stmt()?);
                }
                self.skip_eof();
                if !matches!(stmts.last(), Some(Stmt::Return(_))) {
                    stmts.push(Stmt::Return(None));
                }
                stmts
            } else {
                let expr = self.parse_expr()?;
                self.skip_eof();
                match &expr {
                    Expr::Call(..) => vec![Stmt::Expr(expr), Stmt::Return(None)],
                    _ => vec![
                        Stmt::Expr(Expr::Call("println".to_string(), vec![expr])),
                        Stmt::Return(None),
                    ],
                }
            };
            functions.push(FunctionDef {
                name: "main".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: TypeAnn::Unit,
                body,
            });
            return Ok(Program { type_defs, functions });
        }
        while matches!(self.peek(), Some(Token::Fn)) {
            functions.push(self.parse_function()?);
        }
        if !matches!(self.peek(), Some(Token::Eof)) {
            let have_main = functions.iter().any(|f| f.name == "main");
            if have_main {
                return Err(ParseError::Unexpected(
                    "top-level statements are not allowed when 'main' is defined".into(),
                ));
            }
            let mut body = Vec::new();
            while !matches!(self.peek(), Some(Token::Eof)) {
                body.push(self.parse_stmt()?);
            }
            if !matches!(body.last(), Some(Stmt::Return(_))) {
                body.push(Stmt::Return(None));
            }
            functions.push(FunctionDef {
                name: "main".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: TypeAnn::Unit,
                body,
            });
        }
        self.skip_eof();
        Ok(Program { type_defs, functions })
    }

    fn parse_type_def(&mut self) -> Result<TypeDef, ParseError> {
        self.advance(); // type
        let name = match self.advance() {
            Some(Token::Ident(s)) => s.clone(),
            _ => return Err(ParseError::Unexpected("expected type name".into())),
        };
        self.expect_eq()?;
        let body = if matches!(self.peek(), Some(Token::Ident(_))) {
            let first = match self.advance() {
                Some(Token::Ident(s)) => s.clone(),
                _ => return Err(ParseError::Unexpected("expected variant or type name".into())),
            };
            if matches!(self.peek(), Some(Token::LParen) | Some(Token::Pipe)) {
                let mut variants = vec![];
                let payload = if matches!(self.peek(), Some(Token::LParen)) {
                    self.parse_type_list()?
                } else {
                    vec![]
                };
                variants.push(SumVariant { name: first, payload });
                while matches!(self.peek(), Some(Token::Pipe)) {
                    self.advance(); // |
                    let vname = match self.advance() {
                        Some(Token::Ident(s)) => s.clone(),
                        _ => return Err(ParseError::Unexpected("expected variant name".into())),
                    };
                    let payload = if matches!(self.peek(), Some(Token::LParen)) {
                        self.parse_type_list()?
                    } else {
                        vec![]
                    };
                    variants.push(SumVariant { name: vname, payload });
                }
                TypeAnn::Sum(variants)
            } else {
                TypeAnn::Named(first)
            }
        } else {
            self.parse_type()?
        };
        self.expect_semicolon()?;
        Ok(TypeDef { name, body })
    }

    fn parse_type_list(&mut self) -> Result<Vec<TypeAnn>, ParseError> {
        self.expect_lparen()?;
        let mut types = vec![];
        while !matches!(self.peek(), Some(Token::RParen)) {
            types.push(self.parse_type()?);
            if !matches!(self.peek(), Some(Token::RParen)) {
                self.expect_comma()?;
            }
        }
        self.advance(); // RParen
        Ok(types)
    }

    fn skip_eof(&mut self) {
        if matches!(self.peek(), Some(Token::Eof)) {
            self.advance();
        }
    }

    fn parse_function(&mut self) -> Result<FunctionDef, ParseError> {
        self.advance(); // fn
        let name = match self.advance() {
            Some(Token::Ident(s)) => s.clone(),
            _ => return Err(ParseError::Unexpected("expected function name".into())),
        };
        let type_params = if matches!(self.peek(), Some(Token::Lt)) {
            self.advance(); // <
            let mut params = Vec::new();
            loop {
                let p = match self.advance() {
                    Some(Token::Ident(s)) => s.clone(),
                    _ => return Err(ParseError::Unexpected("expected type parameter name".into())),
                };
                params.push(p);
                if matches!(self.peek(), Some(Token::Comma)) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect_gt()?;
            params
        } else {
            vec![]
        };
        self.expect_lparen()?;
        let params = self.parse_params()?;
        self.expect_rparen()?;
        let return_type = if matches!(self.peek(), Some(Token::Arrow)) {
            self.advance();
            self.parse_type()?
        } else {
            TypeAnn::Unit
        };
        self.expect_lbrace()?;
        let body = self.parse_block()?;
        Ok(FunctionDef {
            name,
            type_params,
            params,
            return_type,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<(String, TypeAnn)>, ParseError> {
        let mut params = Vec::new();
        while !matches!(self.peek(), Some(Token::RParen)) {
            let name = match self.advance() {
                Some(Token::Ident(s)) => s.clone(),
                _ => return Err(ParseError::Unexpected("expected param name".into())),
            };
            let ty = if matches!(self.peek(), Some(Token::Colon)) {
                self.expect_colon()?;
                self.parse_type()?
            } else {
                TypeAnn::I64
            };
            params.push((name, ty));
            if !matches!(self.peek(), Some(Token::RParen)) {
                self.expect_comma()?;
            }
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<TypeAnn, ParseError> {
        let tok = self.advance();
        match tok {
            Some(Token::Ident(s)) => match s.as_str() {
                "i64" => Ok(TypeAnn::I64),
                "str" => Ok(TypeAnn::Str),
                "unit" => Ok(TypeAnn::Unit),
                "array" | "Array" => {
                    self.expect_lt()?;
                    let elem = self.parse_type()?;
                    self.expect_gt()?;
                    Ok(TypeAnn::Array(Box::new(elem)))
                }
                "optional" => {
                    self.expect_lt()?;
                    let inner = self.parse_type()?;
                    self.expect_gt()?;
                    Ok(TypeAnn::Optional(Box::new(inner)))
                }
                _ => Ok(TypeAnn::Named(s.clone())),
            },
            Some(Token::LBrace) => {
                let mut fields = Vec::new();
                while !matches!(self.peek(), Some(Token::RBrace)) {
                    let field_name = match self.advance() {
                        Some(Token::Ident(s)) => s.clone(),
                        _ => return Err(ParseError::Unexpected("expected field name".into())),
                    };
                    self.expect_colon()?;
                    let field_ty = self.parse_type()?;
                    fields.push((field_name, field_ty));
                    if !matches!(self.peek(), Some(Token::RBrace)) {
                        self.expect_comma()?;
                    }
                }
                self.advance(); // RBrace
                Ok(TypeAnn::Record(fields))
            }
            Some(Token::LParen) => {
                self.expect_rparen()?;
                Ok(TypeAnn::Unit)
            }
            _ => Err(ParseError::Unexpected("expected type".into())),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Some(Token::Eof) | Some(Token::RBrace)) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect_rbrace()?;
        Ok(stmts)
    }

    fn is_assignable(e: &Expr) -> bool {
        matches!(e, Expr::Var(_) | Expr::FieldAccess(_, _))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        if matches!(self.peek(), Some(Token::Let)) {
            self.advance();
            let name = match self.advance() {
                Some(Token::Ident(s)) => s.clone(),
                _ => return Err(ParseError::Unexpected("expected variable name".into())),
            };
            let type_ann = if matches!(self.peek(), Some(Token::Colon)) {
                self.expect_colon()?;
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect_eq()?;
            let e = self.parse_expr()?;
            self.expect_semicolon()?;
            return Ok(Stmt::Let(name, type_ann, e));
        }
        if matches!(self.peek(), Some(Token::For)) {
            self.advance(); // for
            let var = match self.advance() {
                Some(Token::Ident(s)) => s.clone(),
                _ => return Err(ParseError::Unexpected("expected loop variable".into())),
            };
            match self.advance() {
                Some(Token::In) => {}
                _ => return Err(ParseError::Unexpected("expected 'in'".into())),
            }
            let range = self.parse_expr()?;
            self.expect_lbrace()?;
            let body = self.parse_block()?;
            return Ok(Stmt::For { var, range, body });
        }
        if matches!(self.peek(), Some(Token::Return)) {
            self.advance();
            let e = if matches!(self.peek(), Some(Token::Semicolon)) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            self.expect_semicolon()?;
            return Ok(Stmt::Return(e));
        }
        let e = self.parse_expr()?;
        if matches!(self.peek(), Some(Token::Eq)) && Self::is_assignable(&e) {
            self.advance(); // =
            let rhs = self.parse_expr()?;
            self.expect_semicolon()?;
            return Ok(Stmt::Assign(e, rhs));
        }
        self.expect_semicolon()?;
        Ok(Stmt::Expr(e))
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr_if()
    }

    fn parse_expr_if(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Some(Token::Match)) {
            self.advance(); // match
            let scrut = self.parse_expr_compare()?;
            self.expect_with()?;
            let mut arms = vec![];
            loop {
                if !matches!(self.peek(), Some(Token::Pipe)) {
                    break;
                }
                self.advance(); // |
                let variant = match self.advance() {
                    Some(Token::Ident(s)) => s.clone(),
                    _ => return Err(ParseError::Unexpected("expected variant name".into())),
                };
                let bindings = if matches!(self.peek(), Some(Token::LParen)) {
                    self.advance(); // (
                    let mut bind = vec![];
                    while !matches!(self.peek(), Some(Token::RParen)) {
                        let b = match self.advance() {
                            Some(Token::Ident(s)) => s.clone(),
                            _ => return Err(ParseError::Unexpected("expected binding name".into())),
                        };
                        bind.push(b);
                        if !matches!(self.peek(), Some(Token::RParen)) {
                            self.expect_comma()?;
                        }
                    }
                    self.advance(); // )
                    bind
                } else {
                    vec![]
                };
                self.expect_arrow()?;
                let body = self.parse_expr_if()?;
                arms.push(MatchArm { variant, bindings, body });
            }
            self.expect_end()?;
            return Ok(Expr::Match(Box::new(scrut), arms));
        }
        if matches!(self.peek(), Some(Token::If)) {
            self.advance();
            let cond = self.parse_expr_compare()?;
            self.expect_then()?;
            let then_b = self.parse_expr_if()?;
            self.expect_else()?;
            let else_b = self.parse_expr_if()?;
            return Ok(Expr::If(
                Box::new(cond),
                Box::new(then_b),
                Box::new(else_b),
            ));
        }
        self.parse_expr_compare()
    }

    fn parse_expr_compare(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_expr_range()?;
        loop {
            let op = match self.peek() {
                Some(Token::EqEq) => {
                    self.advance();
                    Some(1)
                }
                Some(Token::Ne) => {
                    self.advance();
                    Some(2)
                }
                Some(Token::Lt) => {
                    self.advance();
                    Some(3)
                }
                Some(Token::Le) => {
                    self.advance();
                    Some(4)
                }
                Some(Token::Gt) => {
                    self.advance();
                    Some(5)
                }
                Some(Token::Ge) => {
                    self.advance();
                    Some(6)
                }
                _ => None,
            };
            if let Some(kind) = op {
                let rhs = self.parse_expr_range()?;
                lhs = match kind {
                    1 => Expr::Eq(Box::new(lhs), Box::new(rhs)),
                    2 => Expr::Ne(Box::new(lhs), Box::new(rhs)),
                    3 => Expr::Lt(Box::new(lhs), Box::new(rhs)),
                    4 => Expr::Le(Box::new(lhs), Box::new(rhs)),
                    5 => Expr::Gt(Box::new(lhs), Box::new(rhs)),
                    6 => Expr::Ge(Box::new(lhs), Box::new(rhs)),
                    _ => unreachable!(),
                };
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_expr_range(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_expr_add()?;

        // start, next .. end  (derived step = next - start); only if .. or ..= follows "next"
        if matches!(self.peek(), Some(Token::Comma)) {
            let saved = self.pos;
            self.advance();
            let next = self.parse_expr_add()?;
            if matches!(self.peek(), Some(Token::DotDot) | Some(Token::DotDotEq)) {
                let inclusive = matches!(self.advance(), Some(Token::DotDotEq));
                let end = self.parse_expr_add()?;
                let step = Expr::Sub(Box::new(next), Box::new(lhs.clone()));
                return Ok(Expr::Range {
                    start: Box::new(lhs),
                    end: Box::new(end),
                    inclusive,
                    step: Some(Box::new(step)),
                });
            }
            self.pos = saved;
        }

        if matches!(self.peek(), Some(Token::DotDot) | Some(Token::DotDotEq)) {
            let inclusive = matches!(self.advance(), Some(Token::DotDotEq));
            let end = self.parse_expr_add()?;
            return Ok(Expr::Range {
                start: Box::new(lhs),
                end: Box::new(end),
                inclusive,
                step: None,
            });
        }

        Ok(lhs)
    }

    fn parse_expr_add(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr_bp(1)
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
                Some(Token::Slash) => {
                    self.advance();
                    BinOp::Div
                }
                Some(Token::Percent) => {
                    self.advance();
                    BinOp::Mod
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
                BinOp::Div => Expr::Div(Box::new(lhs), Box::new(rhs)),
                BinOp::Mod => Expr::Mod(Box::new(lhs), Box::new(rhs)),
            };
        }
        Ok(lhs)
    }

    /// Parse primary and then any postfix . field accesses.
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_primary_inner()?;
        while matches!(self.peek(), Some(Token::Dot)) {
            self.advance(); // .
            let field = match self.advance() {
                Some(Token::Ident(s)) => s.clone(),
                _ => return Err(ParseError::Unexpected("expected field name".into())),
            };
            e = Expr::FieldAccess(Box::new(e), field);
        }
        Ok(e)
    }

    fn parse_primary_inner(&mut self) -> Result<Expr, ParseError> {
        let tok = self.advance().cloned();
        match tok.as_ref() {
            Some(Token::Typeof) => {
                let e = self.parse_primary()?;
                Ok(Expr::TypeOf(Box::new(e)))
            }
            Some(Token::IntLit(n)) => Ok(Expr::Int(*n)),
            Some(Token::StrLit(s)) => Ok(Expr::Str(s.clone())),
            Some(Token::Null) => Ok(Expr::Null),
            Some(Token::Ident(s)) => {
                if s == "unit" {
                    return Ok(Expr::Unit);
                }
                let name = s.clone();
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.advance();
                    let args = self.parse_expr_list()?;
                    self.expect_rparen()?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Some(Token::LParen) => {
                let e = self.parse_expr()?;
                self.expect_rparen()?;
                Ok(e)
            }
            Some(Token::LBrace) => {
                let mut fields = Vec::new();
                while !matches!(self.peek(), Some(Token::RBrace)) {
                    let field_name = match self.advance() {
                        Some(Token::Ident(s)) => s.clone(),
                        _ => return Err(ParseError::Unexpected("expected field name".into())),
                    };
                    self.expect_colon()?;
                    let field_expr = self.parse_expr_add()?;
                    fields.push((field_name, field_expr));
                    if !matches!(self.peek(), Some(Token::RBrace)) {
                        self.expect_comma()?;
                    }
                }
                self.advance(); // RBrace
                Ok(Expr::RecordLit(fields))
            }
            Some(Token::LBracket) => {
                let mut elems = Vec::new();
                while !matches!(self.peek(), Some(Token::RBracket)) {
                    elems.push(self.parse_expr_add()?);
                    if !matches!(self.peek(), Some(Token::RBracket)) {
                        self.expect_comma()?;
                    }
                }
                self.advance(); // RBracket
                Ok(Expr::ArrayLit(elems))
            }
            _ => Err(ParseError::Unexpected("expected primary expression".into())),
        }
    }

    /// Parse call arguments (use add-level so "a, 0" is two args, not range start,next).
    fn parse_expr_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        while !matches!(self.peek(), Some(Token::RParen)) {
            args.push(self.parse_expr_add()?);
            if !matches!(self.peek(), Some(Token::RParen)) {
                self.expect_comma()?;
            }
        }
        Ok(args)
    }
}

enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl BinOp {
    fn precedence(&self) -> (u8, u8) {
        match self {
            BinOp::Add | BinOp::Sub => (1, 2),
            BinOp::Mul | BinOp::Div | BinOp::Mod => (3, 4),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_int() {
        let prog = parse("42").unwrap();
        assert_eq!(prog.functions.len(), 1);
        assert_eq!(prog.functions[0].name, "main");
        assert_eq!(prog.functions[0].return_type, TypeAnn::Unit);
        assert!(matches!(
            prog.functions[0].body.as_slice(),
            [Stmt::Expr(Expr::Call(_, _)), Stmt::Return(None)]
        ));
    }

    #[test]
    fn parse_add() {
        let prog = parse("1 + 2").unwrap();
        assert_eq!(prog.functions.len(), 1);
        assert!(matches!(
            prog.functions[0].body.as_slice(),
            [Stmt::Expr(Expr::Call(_, _)), Stmt::Return(None)]
        ));
    }

    #[test]
    fn parse_precedence() {
        let prog = parse("1 + 2 * 3").unwrap();
        assert_eq!(prog.functions.len(), 1);
        assert!(matches!(
            prog.functions[0].body.as_slice(),
            [Stmt::Expr(Expr::Call(_, _)), Stmt::Return(None)]
        ));
    }

    #[test]
    fn parse_fn_main() {
        let prog = parse("fn main() -> i64 { return 42; }").unwrap();
        assert_eq!(prog.functions.len(), 1);
        assert_eq!(prog.functions[0].name, "main");
        assert_eq!(prog.functions[0].params.len(), 0);
        assert_eq!(prog.functions[0].return_type, TypeAnn::I64);
        assert!(matches!(
            prog.functions[0].body.as_slice(),
            [Stmt::Return(Some(Expr::Int(42)))]
        ));
    }

    #[test]
    fn parse_range_exclusive() {
        let prog = parse("for i in 1..10 { println(i); }").unwrap();
        assert!(matches!(
            &prog.functions[0].body[0],
            Stmt::For { var, range: Expr::Range { inclusive: false, .. }, .. } if var == "i"
        ));
    }

    #[test]
    fn parse_range_inclusive() {
        let prog = parse("for i in 1..=10 { println(i); }").unwrap();
        assert!(matches!(
            &prog.functions[0].body[0],
            Stmt::For { range: Expr::Range { inclusive: true, .. }, .. }
        ));
    }

    #[test]
    fn parse_range_step_derived() {
        let prog = parse("for i in 100,98..0 { println(i); }").unwrap();
        assert!(matches!(
            &prog.functions[0].body[0],
            Stmt::For { range: Expr::Range { step: Some(_), .. }, .. }
        ));
    }

    #[test]
    fn parse_array_lit_and_call() {
        let prog = parse("let a = [1]; println(get(a, 0));").unwrap();
        assert_eq!(prog.functions.len(), 1);
        assert!(matches!(
            &prog.functions[0].body[0],
            Stmt::Let(_, _, Expr::ArrayLit(_))
        ));
    }

    #[test]
    fn parse_array_lit_with_newlines_and_return() {
        // Exact content that fails when run via CLI from file
        let src = "let a = [1];\nprintln(get(a, 0));\nreturn;\n";
        let prog = parse(src).unwrap();
        assert_eq!(prog.functions.len(), 1);
        assert_eq!(prog.functions[0].body.len(), 3);
        assert!(matches!(
            &prog.functions[0].body[0],
            Stmt::Let(_, _, Expr::ArrayLit(_))
        ));
        assert!(matches!(
            &prog.functions[0].body[1],
            Stmt::Expr(Expr::Call(n, _)) if n == "println"
        ));
        assert!(matches!(&prog.functions[0].body[2], Stmt::Return(None)));
    }
}
