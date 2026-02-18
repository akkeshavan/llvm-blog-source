use lumina_part1_parser::Expr;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
}

#[derive(Debug)]
pub enum TypeError {
    Unbound(String),
    Mismatch(Type, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpr {
    Int(i64),
    Var(String),
    Add(Box<TypedExpr>, Box<TypedExpr>),
    Sub(Box<TypedExpr>, Box<TypedExpr>),
    Mul(Box<TypedExpr>, Box<TypedExpr>),
}

pub struct TypeChecker {
    env: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
        }
    }

    pub fn check(&mut self, expr: &Expr) -> Result<TypedExpr, TypeError> {
        match expr {
            Expr::Int(n) => Ok(TypedExpr::Int(*n)),
            Expr::Var(name) => {
                let _ty = self
                    .env
                    .get(name)
                    .cloned()
                    .ok_or(TypeError::Unbound(name.clone()))?;
                Ok(TypedExpr::Var(name.clone()))
            }
            Expr::Add(l, r) => {
                let tl = self.check(l)?;
                let tr = self.check(r)?;
                Ok(TypedExpr::Add(Box::new(tl), Box::new(tr)))
            }
            Expr::Sub(l, r) => {
                let tl = self.check(l)?;
                let tr = self.check(r)?;
                Ok(TypedExpr::Sub(Box::new(tl), Box::new(tr)))
            }
            Expr::Mul(l, r) => {
                let tl = self.check(l)?;
                let tr = self.check(r)?;
                Ok(TypedExpr::Mul(Box::new(tl), Box::new(tr)))
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumina_part1_parser::parse;

    #[test]
    fn check_int() {
        let mut tc = TypeChecker::new();
        let expr = parse("42").unwrap();
        let typed = tc.check(&expr).unwrap();
        assert!(matches!(typed, TypedExpr::Int(42)));
    }

    #[test]
    fn check_add() {
        let mut tc = TypeChecker::new();
        let expr = parse("1 + 2").unwrap();
        let typed = tc.check(&expr).unwrap();
        assert!(matches!(typed, TypedExpr::Add(_, _)));
    }
}
