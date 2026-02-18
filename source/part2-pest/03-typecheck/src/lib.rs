//! Part 2: Type checking - reuses Part 1's TypeChecker via Expr conversion

use lumina_part1_parser::Expr as P1Expr;
use lumina_part1_typecheck::{TypeChecker, TypedExpr};
use lumina_part2_parser::{parse, Expr};

pub use lumina_part1_typecheck::TypeError;

fn to_part1_expr(e: &Expr) -> P1Expr {
    match e {
        Expr::Int(n) => P1Expr::Int(*n),
        Expr::Var(s) => P1Expr::Var(s.clone()),
        Expr::Add(l, r) => P1Expr::Add(
            Box::new(to_part1_expr(l)),
            Box::new(to_part1_expr(r)),
        ),
        Expr::Sub(l, r) => P1Expr::Sub(
            Box::new(to_part1_expr(l)),
            Box::new(to_part1_expr(r)),
        ),
        Expr::Mul(l, r) => P1Expr::Mul(
            Box::new(to_part1_expr(l)),
            Box::new(to_part1_expr(r)),
        ),
    }
}

pub fn typecheck(source: &str) -> Result<TypedExpr, TypeError> {
    let expr = parse(source).map_err(|_| {
        lumina_part1_typecheck::TypeError::Unbound("parse error".into())
    })?;
    let p1_expr = to_part1_expr(&expr);
    TypeChecker::new().check(&p1_expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typecheck_int() {
        let typed = typecheck("42").unwrap();
        assert!(matches!(typed, TypedExpr::Int(42)));
    }

    #[test]
    fn typecheck_add() {
        let typed = typecheck("1 + 2").unwrap();
        assert!(matches!(typed, TypedExpr::Add(_, _)));
    }
}
