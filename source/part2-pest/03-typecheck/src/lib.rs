//! Part 2: Type checking - reuses Part 1's TypeChecker via Expr conversion

use lumina_part1_parser::{Expr as P1Expr, FunctionDef, Program, Stmt, TypeAnn};
use lumina_part1_typecheck::{TypeChecker, TypedProgram};
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

pub fn typecheck(source: &str) -> Result<TypedProgram, TypeError> {
    let expr = parse(source).map_err(|_| {
        lumina_part1_typecheck::TypeError::Unbound("parse error".into())
    })?;
    let p1_expr = to_part1_expr(&expr);
    let program = Program {
        type_defs: vec![],
        functions: vec![FunctionDef {
            name: "main".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: TypeAnn::I64,
            body: vec![Stmt::Return(Some(p1_expr))],
        }],
    };
    TypeChecker::new().check_program(&program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumina_part1_typecheck::{TypedExpr, TypedStmt};

    fn main_expr(tp: &TypedProgram) -> &TypedExpr {
        let main_fn = tp.functions.iter().find(|f| f.name == "main").unwrap();
        let stmt = main_fn.body.first().unwrap();
        match stmt {
            TypedStmt::Return(Some(e)) => e,
            _ => panic!("expected return stmt"),
        }
    }

    #[test]
    fn typecheck_int() {
        let tp = typecheck("42").unwrap();
        assert!(matches!(main_expr(&tp), TypedExpr::Int(42)));
    }

    #[test]
    fn typecheck_add() {
        let tp = typecheck("1 + 2").unwrap();
        assert!(matches!(main_expr(&tp), TypedExpr::Add(_, _)));
    }
}
