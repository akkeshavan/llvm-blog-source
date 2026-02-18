use lumina_part1_parser::{Expr, FunctionDef, Program, Stmt, TypeAnn, SumVariant};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Str,
    Unit,
    Range,
    Array(Box<Type>),
    Optional(Box<Type>),
    /// Record type: list of (field name, field type)
    Record(Vec<(String, Type)>),
    /// Type variable for generics (e.g. T in fn id<T>(x: T) -> T)
    TypeVar(String),
    /// Sum type: list of (variant name, payload types)
    Sum(Vec<(String, Vec<Type>)>),
}

#[derive(Debug)]
pub enum TypeError {
    Unbound(String),
    UnboundFunction(String),
    Mismatch(Type, Type),
    ArgCount { name: String, expected: usize, got: usize },
    NotCallable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpr {
    Int(i64),
    Str(String),
    Var(String, Type),
    Add(Box<TypedExpr>, Box<TypedExpr>),
    Sub(Box<TypedExpr>, Box<TypedExpr>),
    Mul(Box<TypedExpr>, Box<TypedExpr>),
    Div(Box<TypedExpr>, Box<TypedExpr>),
    Mod(Box<TypedExpr>, Box<TypedExpr>),
    Eq(Box<TypedExpr>, Box<TypedExpr>),
    Ne(Box<TypedExpr>, Box<TypedExpr>),
    Lt(Box<TypedExpr>, Box<TypedExpr>),
    Le(Box<TypedExpr>, Box<TypedExpr>),
    Gt(Box<TypedExpr>, Box<TypedExpr>),
    Ge(Box<TypedExpr>, Box<TypedExpr>),
    Call(String, Vec<TypedExpr>, Type),
    Unit,
    Range {
        start: Box<TypedExpr>,
        end: Box<TypedExpr>,
        inclusive: bool,
        step: Option<Box<TypedExpr>>,
    },
    If(Box<TypedExpr>, Box<TypedExpr>, Box<TypedExpr>),
    ArrayLit(Vec<TypedExpr>, Type),
    Null(Type),
    /// typeof expr; the String is the type name for codegen (e.g. "i64")
    TypeOf(Box<TypedExpr>, String),
    /// base.field with inferred type
    FieldAccess(Box<TypedExpr>, String, Type),
    /// { field: expr, ... } with record type
    RecordLit(Vec<(String, TypedExpr)>, Type),
    /// Constructor: variant name, payload exprs, sum type
    Constructor(String, Vec<TypedExpr>, Type),
    /// match scrut with | Variant(bindings) -> body ...
    Match(Box<TypedExpr>, Vec<(String, Vec<String>, TypedExpr)>, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedStmt {
    Let(String, TypedExpr),
    Expr(TypedExpr),
    Return(Option<TypedExpr>),
    For {
        var: String,
        range: TypedExpr,
        body: Vec<TypedStmt>,
    },
    /// LHS must be Var or FieldAccess
    Assign(TypedExpr, TypedExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedFunctionDef {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Vec<TypedStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedProgram {
    pub functions: Vec<TypedFunctionDef>,
}

fn type_ann_to_type(
    ann: &TypeAnn,
    type_defs: &HashMap<String, Type>,
    type_params: &[String],
) -> Result<Type, TypeError> {
    match ann {
        TypeAnn::I64 => Ok(Type::Int),
        TypeAnn::Str => Ok(Type::Str),
        TypeAnn::Unit => Ok(Type::Unit),
        TypeAnn::Array(e) => Ok(Type::Array(Box::new(type_ann_to_type(e, type_defs, type_params)?))),
        TypeAnn::Optional(e) => Ok(Type::Optional(Box::new(type_ann_to_type(e, type_defs, type_params)?))),
        TypeAnn::Record(fields) => {
            let fs: Result<Vec<_>, _> = fields
                .iter()
                .map(|(n, t)| Ok((n.clone(), type_ann_to_type(t, type_defs, type_params)?)))
                .collect();
            Ok(Type::Record(fs?))
        }
        TypeAnn::Named(s) => {
            if type_params.contains(&s) {
                Ok(Type::TypeVar(s.clone()))
            } else if let Some(t) = type_defs.get(s) {
                Ok(t.clone())
            } else {
                Err(TypeError::Unbound(format!("unknown type: {}", s)))
            }
        }
        TypeAnn::Sum(variants) => {
            let v: Result<Vec<_>, _> = variants
                .iter()
                .map(|sv: &SumVariant| {
                    let payload: Result<Vec<_>, _> = sv
                        .payload
                        .iter()
                        .map(|t| type_ann_to_type(t, type_defs, type_params))
                        .collect();
                    Ok((sv.name.clone(), payload?))
                })
                .collect();
            Ok(Type::Sum(v?))
        }
    }
}

/// Built-in function signatures: name -> (param types, return type).
fn stdlib_signatures() -> HashMap<String, (Vec<Type>, Type)> {
    let mut m = HashMap::new();
    m.insert("println".to_string(), (vec![Type::Str], Type::Unit));
    m.insert("println_i64".to_string(), (vec![Type::Int], Type::Unit));
    m.insert("print".to_string(), (vec![Type::Str], Type::Unit));
    m.insert("print_i64".to_string(), (vec![Type::Int], Type::Unit));
    m.insert("sqrt".to_string(), (vec![Type::Int], Type::Int));
    m.insert("max".to_string(), (vec![Type::Int, Type::Int], Type::Int));
    m.insert("min".to_string(), (vec![Type::Int, Type::Int], Type::Int));
    // Array ops (overloaded by first-arg type; codegen resolves to _i64/_str)
    m.insert("append_i64".to_string(), (vec![Type::Array(Box::new(Type::Int)), Type::Int], Type::Unit));
    m.insert("append_str".to_string(), (vec![Type::Array(Box::new(Type::Str)), Type::Str], Type::Unit));
    m.insert("get_i64".to_string(), (vec![Type::Array(Box::new(Type::Int)), Type::Int], Type::Int));
    m.insert("get_str".to_string(), (vec![Type::Array(Box::new(Type::Str)), Type::Int], Type::Str));
    m.insert("set_i64".to_string(), (vec![Type::Array(Box::new(Type::Int)), Type::Int, Type::Int], Type::Unit));
    m.insert("set_str".to_string(), (vec![Type::Array(Box::new(Type::Str)), Type::Int, Type::Str], Type::Unit));
    m.insert("array_i64_new".to_string(), (vec![], Type::Array(Box::new(Type::Int))));
    m.insert("array_str_new".to_string(), (vec![], Type::Array(Box::new(Type::Str))));
    m.insert("array_i64_len".to_string(), (vec![Type::Array(Box::new(Type::Int))], Type::Int));
    m.insert("array_str_len".to_string(), (vec![Type::Array(Box::new(Type::Str))], Type::Int));
    // Optional
    m.insert("unwrap_i64".to_string(), (vec![Type::Optional(Box::new(Type::Int))], Type::Int));
    m.insert("unwrap_str".to_string(), (vec![Type::Optional(Box::new(Type::Str))], Type::Str));
    m
}

pub struct TypeChecker {
    env: HashMap<String, Type>,
    functions: HashMap<String, (Vec<Type>, Type)>,
    /// Resolved type definitions (record/alias names -> Type)
    type_defs: HashMap<String, Type>,
    /// Constructor name -> sum type name (for sum type that defines this variant)
    constructors: HashMap<String, String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            functions: stdlib_signatures(),
            type_defs: HashMap::new(),
            constructors: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<TypedProgram, TypeError> {
        self.type_defs.clear();
        self.constructors.clear();
        for def in &program.type_defs {
            let ty = type_ann_to_type(&def.body, &self.type_defs, &[])?;
            if let Type::Sum(ref variants) = ty {
                for (vname, _) in variants {
                    self.constructors.insert(vname.clone(), def.name.clone());
                }
            }
            self.type_defs.insert(def.name.clone(), ty);
        }
        for f in &program.functions {
            if f.type_params.is_empty() {
                let params: Vec<Type> = f
                    .params
                    .iter()
                    .map(|(_, t)| type_ann_to_type(t, &self.type_defs, &f.type_params).unwrap())
                    .collect();
                let ret = type_ann_to_type(&f.return_type, &self.type_defs, &f.type_params).unwrap();
                self.functions
                    .insert(f.name.clone(), (params.clone(), ret));
            }
        }
        let mut typed_functions = Vec::new();
        for f in &program.functions {
            if !f.type_params.is_empty() {
                continue;
            }
            let tf = self.check_function(f)?;
            typed_functions.push(tf);
        }
        Ok(TypedProgram {
            functions: typed_functions,
        })
    }

    fn check_function(&mut self, f: &FunctionDef) -> Result<TypedFunctionDef, TypeError> {
        if !f.type_params.is_empty() {
            return Err(TypeError::NotCallable("generic functions not yet supported".into()));
        }
        self.env.clear();
        for (name, t) in &f.params {
            let ty = type_ann_to_type(t, &self.type_defs, &f.type_params)?;
            self.env.insert(name.clone(), ty);
        }
        let return_type = type_ann_to_type(&f.return_type, &self.type_defs, &f.type_params)?;
        let mut body = Vec::new();
        for s in &f.body {
            body.push(self.check_stmt(s, &return_type, &f.type_params)?);
        }
        Ok(TypedFunctionDef {
            name: f.name.clone(),
            params: f
                .params
                .iter()
                .map(|(n, t)| (n.clone(), type_ann_to_type(t, &self.type_defs, &f.type_params).unwrap()))
                .collect(),
            return_type,
            body,
        })
    }

    fn check_stmt(&mut self, s: &Stmt, ret_ty: &Type, type_params: &[String]) -> Result<TypedStmt, TypeError> {
        match s {
            Stmt::Let(name, type_ann, e) => {
                // Empty array literal with type annotation: let x: Array<T> = []
                let te = if let (Some(ann), Expr::ArrayLit(elems)) = (type_ann.as_ref(), e) {
                    if elems.is_empty() {
                        let expected = type_ann_to_type(ann, &self.type_defs, type_params)?;
                        if let Type::Array(_) = &expected {
                            self.env.insert(name.clone(), expected.clone());
                            return Ok(TypedStmt::Let(
                                name.clone(),
                                TypedExpr::ArrayLit(vec![], expected),
                            ));
                        }
                    }
                    self.check_expr(e)?
                } else {
                    self.check_expr(e)?
                };
                let inferred = expr_type(&te);
                if let Some(ann) = type_ann {
                    let expected = type_ann_to_type(ann, &self.type_defs, type_params)?;
                    if !types_compat(&inferred, &expected) {
                        return Err(TypeError::Mismatch(expected, inferred));
                    }
                    self.env.insert(name.clone(), expected);
                } else {
                    self.env.insert(name.clone(), inferred.clone());
                }
                Ok(TypedStmt::Let(name.clone(), te))
            }
            Stmt::Expr(e) => {
                let te = self.check_expr(e)?;
                Ok(TypedStmt::Expr(te))
            }
            Stmt::For { var, range, body } => {
                let tr = self.check_expr(range)?;
                if !matches!(expr_type(&tr), Type::Range) {
                    return Err(TypeError::Mismatch(Type::Range, expr_type(&tr)));
                }

                let saved = self.env.clone();
                self.env.insert(var.clone(), Type::Int);
                let mut typed_body = Vec::new();
                for s in body {
                    typed_body.push(self.check_stmt(s, ret_ty, type_params)?);
                }
                self.env = saved;

                Ok(TypedStmt::For {
                    var: var.clone(),
                    range: tr,
                    body: typed_body,
                })
            }
            Stmt::Assign(lhs, rhs) => {
                let tl = self.check_expr(lhs)?;
                let tr = self.check_expr(rhs)?;
                let lhs_ty = expr_type(&tl);
                if !matches!(&tl, TypedExpr::Var(_, _) | TypedExpr::FieldAccess(_, _, _)) {
                    return Err(TypeError::Mismatch(lhs_ty, expr_type(&tr)));
                }
                if !types_compat(&lhs_ty, &expr_type(&tr)) {
                    return Err(TypeError::Mismatch(lhs_ty, expr_type(&tr)));
                }
                Ok(TypedStmt::Assign(tl, tr))
            }
            Stmt::Return(e_opt) => match e_opt {
                None => {
                    if !matches!(ret_ty, Type::Unit) {
                        return Err(TypeError::Mismatch(ret_ty.clone(), Type::Unit));
                    }
                    Ok(TypedStmt::Return(None))
                }
                Some(e) => {
                    let te = self.check_expr(e)?;
                    let got = expr_type(&te);
                    if !types_compat(&got, ret_ty) {
                        return Err(TypeError::Mismatch(ret_ty.clone(), got));
                    }
                    Ok(TypedStmt::Return(Some(te)))
                }
            }
        }
    }

    fn check_expr(&mut self, e: &Expr) -> Result<TypedExpr, TypeError> {
        match e {
            Expr::Int(n) => Ok(TypedExpr::Int(*n)),
            Expr::Str(s) => Ok(TypedExpr::Str(s.clone())),
            Expr::Unit => Ok(TypedExpr::Unit),
            Expr::Var(name) => {
                let ty = self
                    .env
                    .get(name)
                    .cloned()
                    .ok_or(TypeError::Unbound(name.clone()))?;
                Ok(TypedExpr::Var(name.clone(), ty))
            }
            Expr::Add(l, r) => {
                let tl = self.check_expr(l)?;
                let tr = self.check_expr(r)?;
                require_int(&expr_type(&tl))?;
                require_int(&expr_type(&tr))?;
                Ok(TypedExpr::Add(Box::new(tl), Box::new(tr)))
            }
            Expr::Sub(l, r) => {
                let tl = self.check_expr(l)?;
                let tr = self.check_expr(r)?;
                require_int(&expr_type(&tl))?;
                require_int(&expr_type(&tr))?;
                Ok(TypedExpr::Sub(Box::new(tl), Box::new(tr)))
            }
            Expr::Mul(l, r) => {
                let tl = self.check_expr(l)?;
                let tr = self.check_expr(r)?;
                require_int(&expr_type(&tl))?;
                require_int(&expr_type(&tr))?;
                Ok(TypedExpr::Mul(Box::new(tl), Box::new(tr)))
            }
            Expr::Div(l, r) => {
                let tl = self.check_expr(l)?;
                let tr = self.check_expr(r)?;
                require_int(&expr_type(&tl))?;
                require_int(&expr_type(&tr))?;
                Ok(TypedExpr::Div(Box::new(tl), Box::new(tr)))
            }
            Expr::Mod(l, r) => {
                let tl = self.check_expr(l)?;
                let tr = self.check_expr(r)?;
                require_int(&expr_type(&tl))?;
                require_int(&expr_type(&tr))?;
                Ok(TypedExpr::Mod(Box::new(tl), Box::new(tr)))
            }
            Expr::Eq(l, r) | Expr::Ne(l, r) => {
                let tl = self.check_expr(l)?;
                let tr = self.check_expr(r)?;
                let a = expr_type(&tl);
                let b = expr_type(&tr);
                if !types_compat(&a, &b) {
                    return Err(TypeError::Mismatch(a, b));
                }
                Ok(match e {
                    Expr::Eq(_, _) => TypedExpr::Eq(Box::new(tl), Box::new(tr)),
                    _ => TypedExpr::Ne(Box::new(tl), Box::new(tr)),
                })
            }
            Expr::Lt(l, r) | Expr::Le(l, r) | Expr::Gt(l, r) | Expr::Ge(l, r) => {
                let tl = self.check_expr(l)?;
                let tr = self.check_expr(r)?;
                require_int(&expr_type(&tl))?;
                require_int(&expr_type(&tr))?;
                Ok(match e {
                    Expr::Lt(_, _) => TypedExpr::Lt(Box::new(tl), Box::new(tr)),
                    Expr::Le(_, _) => TypedExpr::Le(Box::new(tl), Box::new(tr)),
                    Expr::Gt(_, _) => TypedExpr::Gt(Box::new(tl), Box::new(tr)),
                    _ => TypedExpr::Ge(Box::new(tl), Box::new(tr)),
                })
            }
            Expr::Call(name, args) => {
                if let Some(type_name) = self.constructors.get(name) {
                    let sum_ty = self.type_defs.get(type_name).ok_or(TypeError::Unbound(type_name.clone()))?.clone();
                    let payload_tys: Vec<Type> = match &sum_ty {
                        Type::Sum(v) => v.iter().find(|(vname, _)| vname == name).map(|(_, pt)| pt.clone()).ok_or(TypeError::UnboundFunction(name.clone()))?,
                        _ => return Err(TypeError::Mismatch(Type::Sum(vec![]), sum_ty)),
                    };
                    if args.len() != payload_tys.len() {
                        return Err(TypeError::ArgCount { name: name.clone(), expected: payload_tys.len(), got: args.len() });
                    }
                    let typed_args: Vec<TypedExpr> = args
                        .iter()
                        .zip(payload_tys.iter())
                        .map(|(a, pt)| {
                            let ta = self.check_expr(a)?;
                            if !types_compat(&expr_type(&ta), pt) {
                                return Err(TypeError::Mismatch(pt.clone(), expr_type(&ta)));
                            }
                            Ok(ta)
                        })
                        .collect::<Result<_, _>>()?;
                    return Ok(TypedExpr::Constructor(name.clone(), typed_args, sum_ty));
                }
                if (name == "println" || name == "print") && args.len() == 1 {
                    let first = self.check_expr(&args[0])?;
                    let t = expr_type(&first);
                    let overload = if matches!(t, Type::Str) { name.clone() } else if matches!(t, Type::Int) {
                        if name == "println" { "println_i64".into() } else { "print_i64".into() }
                    } else {
                        return Err(TypeError::Mismatch(Type::Str, t));
                    };
                    let (param_tys, ret_ty) = self.functions.get(&overload).cloned().ok_or(TypeError::UnboundFunction(name.clone()))?;
                    let typed_args = vec![first];
                    if !types_compat(&expr_type(&typed_args[0]), &param_tys[0]) {
                        return Err(TypeError::Mismatch(param_tys[0].clone(), expr_type(&typed_args[0])));
                    }
                    return Ok(TypedExpr::Call(overload, typed_args, ret_ty));
                }
                if (name == "append" || name == "get" || name == "set" || name == "unwrap") && !args.is_empty() {
                    let first = self.check_expr(&args[0])?;
                    let t0 = expr_type(&first);
                    let (resolved, rest_args, ret) = match (name.as_str(), args.len(), &t0) {
                        ("append", 2, Type::Array(inner)) => {
                            let second = self.check_expr(&args[1])?;
                            if !types_compat(&expr_type(&second), inner) {
                                return Err(TypeError::Mismatch(inner.as_ref().clone(), expr_type(&second)));
                            }
                            let name2 = match inner.as_ref() {
                                Type::Int => "append_i64",
                                Type::Str => "append_str",
                                Type::Record(_) => "append_i64", // store record pointer as i64
                                _ => return Err(TypeError::Mismatch(Type::Int, inner.as_ref().clone())),
                            };
                            (name2.to_string(), vec![first, second], Type::Unit)
                        }
                        ("get", 2, Type::Array(inner)) => {
                            let second = self.check_expr(&args[1])?;
                            require_int(&expr_type(&second))?;
                            let name2 = match inner.as_ref() {
                                Type::Int => "get_i64",
                                Type::Str => "get_str",
                                Type::Record(_) => "get_i64", // returns ptr as i64, codegen will int2ptr
                                _ => return Err(TypeError::Mismatch(Type::Int, inner.as_ref().clone())),
                            };
                            (name2.to_string(), vec![first, second], inner.as_ref().clone())
                        }
                        ("set", 3, Type::Array(inner)) => {
                            let second = self.check_expr(&args[1])?;
                            let third = self.check_expr(&args[2])?;
                            require_int(&expr_type(&second))?;
                            if !types_compat(&expr_type(&third), inner) {
                                return Err(TypeError::Mismatch(inner.as_ref().clone(), expr_type(&third)));
                            }
                            let name2 = match inner.as_ref() {
                                Type::Int => "set_i64",
                                Type::Str => "set_str",
                                Type::Record(_) => "set_i64", // store record pointer as i64
                                _ => return Err(TypeError::Mismatch(Type::Int, inner.as_ref().clone())),
                            };
                            (name2.to_string(), vec![first, second, third], Type::Unit)
                        }
                        ("unwrap", 1, Type::Optional(inner)) => {
                            let name2 = match inner.as_ref() {
                                Type::Int => "unwrap_i64",
                                Type::Str => "unwrap_str",
                                _ => return Err(TypeError::Mismatch(Type::Int, inner.as_ref().clone())),
                            };
                            (name2.to_string(), vec![first], inner.as_ref().clone())
                        }
                        ("append", _, _) => return Err(TypeError::ArgCount { name: name.clone(), expected: 2, got: args.len() }),
                        ("get", _, _) => return Err(TypeError::ArgCount { name: name.clone(), expected: 2, got: args.len() }),
                        ("set", _, _) => return Err(TypeError::ArgCount { name: name.clone(), expected: 3, got: args.len() }),
                        ("unwrap", _, _) => return Err(TypeError::ArgCount { name: name.clone(), expected: 1, got: args.len() }),
                        _ => return Err(TypeError::UnboundFunction(name.clone())),
                    };
                    return Ok(TypedExpr::Call(resolved, rest_args, ret));
                }
                if name == "ArrayLen" && args.len() == 1 {
                    let first = self.check_expr(&args[0])?;
                    let t0 = expr_type(&first).clone();
                    let resolved = match &t0 {
                        Type::Array(inner) => match inner.as_ref() {
                            Type::Int => "array_i64_len",
                            Type::Str => "array_str_len",
                            Type::Record(_) => "array_i64_len", // same backing as array of ptrs
                            _ => return Err(TypeError::Mismatch(Type::Array(Box::new(Type::Int)), t0)),
                        },
                        _ => return Err(TypeError::Mismatch(Type::Array(Box::new(Type::Int)), t0)),
                    };
                    return Ok(TypedExpr::Call(resolved.to_string(), vec![first], Type::Int));
                }
                let (param_tys, ret_ty) = self.functions
                        .get(name)
                        .cloned()
                        .ok_or(TypeError::UnboundFunction(name.clone()))?;
                    if args.len() != param_tys.len() {
                        return Err(TypeError::ArgCount {
                            name: name.clone(),
                            expected: param_tys.len(),
                            got: args.len(),
                        });
                    }
                    let typed_args: Vec<TypedExpr> = args
                        .iter()
                        .zip(param_tys.iter())
                        .map(|(a, pt)| {
                            let ta = self.check_expr(a)?;
                            if !types_compat(&expr_type(&ta), pt) {
                                return Err(TypeError::Mismatch(pt.clone(), expr_type(&ta)));
                            }
                            Ok(ta)
                        })
                        .collect::<Result<_, _>>()?;
                Ok(TypedExpr::Call(name.clone(), typed_args, ret_ty.clone()))
            }
            Expr::ArrayLit(elems) => {
                let mut typed = Vec::new();
                let mut elem_ty = None;
                for e in elems {
                    let te = self.check_expr(e)?;
                    let t = expr_type(&te);
                    match &elem_ty {
                        Some(prev) if !types_compat(prev, &t) => return Err(TypeError::Mismatch(prev.clone(), t)),
                        Some(_) => {}
                        None => elem_ty = Some(t),
                    }
                    typed.push(te);
                }
                let elem_ty = elem_ty.unwrap_or(Type::Int);
                if !matches!(elem_ty, Type::Int | Type::Str | Type::Record(_)) {
                    return Err(TypeError::Mismatch(Type::Int, elem_ty));
                }
                Ok(TypedExpr::ArrayLit(typed, Type::Array(Box::new(elem_ty))))
            }
            Expr::TypeOf(inner) => {
                let te = self.check_expr(inner)?;
                let ty = expr_type(&te);
                let name = type_to_string(&ty);
                Ok(TypedExpr::TypeOf(Box::new(te), name))
            }
            Expr::FieldAccess(base, field) => {
                let tb = self.check_expr(base)?;
                let base_ty = expr_type(&tb);
                let field_ty = match &base_ty {
                    Type::Record(fields) => fields
                        .iter()
                        .find(|(n, _)| n == field)
                        .map(|(_, t)| t.clone())
                        .ok_or_else(|| TypeError::Unbound(format!("record has no field '{}'", field)))?,
                    _ => return Err(TypeError::Mismatch(Type::Record(vec![]), base_ty)),
                };
                Ok(TypedExpr::FieldAccess(Box::new(tb), field.clone(), field_ty))
            }
            Expr::RecordLit(fields) => {
                let mut typed = Vec::new();
                let mut record_fields = Vec::new();
                for (name, e) in fields {
                    let te = self.check_expr(e)?;
                    let ty = expr_type(&te);
                    typed.push((name.clone(), te));
                    record_fields.push((name.clone(), ty));
                }
                Ok(TypedExpr::RecordLit(typed, Type::Record(record_fields)))
            }
            Expr::Match(scrut, arms) => {
                let tscrut = self.check_expr(scrut)?;
                let sum_ty = match expr_type(&tscrut) {
                    Type::Sum(v) => v,
                    other => return Err(TypeError::Mismatch(Type::Sum(vec![]), other)),
                };
                let mut covered = std::collections::HashSet::new();
                let mut typed_arms = vec![];
                let mut result_ty = None;
                for arm in arms {
                    if covered.contains(&arm.variant) {
                        return Err(TypeError::NotCallable(format!("duplicate match arm '{}'", arm.variant)));
                    }
                    let (_, payload_tys) = sum_ty.iter().find(|(vname, _)| vname == &arm.variant)
                        .ok_or_else(|| TypeError::NotCallable(format!("unknown variant '{}'", arm.variant)))?;
                    if arm.bindings.len() != payload_tys.len() {
                        return Err(TypeError::ArgCount {
                            name: arm.variant.clone(),
                            expected: payload_tys.len(),
                            got: arm.bindings.len(),
                        });
                    }
                    covered.insert(arm.variant.clone());
                    let saved = self.env.clone();
                    for (b, pt) in arm.bindings.iter().zip(payload_tys.iter()) {
                        self.env.insert(b.clone(), pt.clone());
                    }
                    let tbody = self.check_expr(&arm.body)?;
                    let body_ty = expr_type(&tbody);
                    if let Some(ref rt) = result_ty {
                        if !types_compat(rt, &body_ty) {
                            self.env = saved;
                            return Err(TypeError::Mismatch(rt.clone(), body_ty));
                        }
                    } else {
                        result_ty = Some(body_ty);
                    }
                    self.env = saved;
                    typed_arms.push((arm.variant.clone(), arm.bindings.clone(), tbody));
                }
                for (vname, _) in &sum_ty {
                    if !covered.contains(vname) {
                        return Err(TypeError::NotCallable(format!("match non-exhaustive: missing '{}'", vname)));
                    }
                }
                let ret_ty = result_ty.ok_or(TypeError::NotCallable("match has no arms".into()))?;
                Ok(TypedExpr::Match(Box::new(tscrut), typed_arms, ret_ty))
            }
            Expr::Null => {
                Ok(TypedExpr::Null(Type::Optional(Box::new(Type::Int))))
            }
            Expr::Range {
                start,
                end,
                inclusive,
                step,
            } => {
                let ts = self.check_expr(start)?;
                let te = self.check_expr(end)?;
                require_int(&expr_type(&ts))?;
                require_int(&expr_type(&te))?;
                let tstep = if let Some(s) = step {
                    let t = self.check_expr(s)?;
                    require_int(&expr_type(&t))?;
                    Some(Box::new(t))
                } else {
                    None
                };
                Ok(TypedExpr::Range {
                    start: Box::new(ts),
                    end: Box::new(te),
                    inclusive: *inclusive,
                    step: tstep,
                })
            }
            Expr::If(cond, then_b, else_b) => {
                let tc = self.check_expr(cond)?;
                require_int(&expr_type(&tc))?;
                let tt = self.check_expr(then_b)?;
                let te = self.check_expr(else_b)?;
                let ty_t = expr_type(&tt);
                let ty_e = expr_type(&te);
                if !types_compat(&ty_t, &ty_e) {
                    return Err(TypeError::Mismatch(ty_t, ty_e));
                }
                Ok(TypedExpr::If(
                    Box::new(tc),
                    Box::new(tt),
                    Box::new(te),
                ))
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

fn type_to_string(t: &Type) -> String {
    match t {
        Type::Int => "i64".into(),
        Type::Str => "str".into(),
        Type::Unit => "unit".into(),
        Type::Range => "range".into(),
        Type::Array(inner) => format!("array<{}>", type_to_string(inner)),
        Type::Optional(inner) => format!("optional<{}>", type_to_string(inner)),
        Type::Record(fields) => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|(n, t)| format!("{}: {}", n, type_to_string(t)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::TypeVar(s) => s.clone(),
        Type::Sum(variants) => variants
            .iter()
            .map(|(n, p)| if p.is_empty() { n.clone() } else { format!("{}({})", n, p.iter().map(type_to_string).collect::<Vec<_>>().join(", ")) })
            .collect::<Vec<_>>()
            .join(" | "),
    }
}

fn expr_type(e: &TypedExpr) -> Type {
    match e {
        TypedExpr::Int(_) => Type::Int,
        TypedExpr::Str(_) => Type::Str,
        TypedExpr::Var(_, t) => t.clone(),
        TypedExpr::Add(_, _) | TypedExpr::Sub(_, _) | TypedExpr::Mul(_, _) | TypedExpr::Div(_, _) | TypedExpr::Mod(_, _) => Type::Int,
        TypedExpr::Eq(_, _) | TypedExpr::Ne(_, _) | TypedExpr::Lt(_, _) | TypedExpr::Le(_, _) | TypedExpr::Gt(_, _) | TypedExpr::Ge(_, _) => Type::Int,
        TypedExpr::Call(_, _, ret) => ret.clone(),
        TypedExpr::Unit => Type::Unit,
        TypedExpr::Range { .. } => Type::Range,
        TypedExpr::ArrayLit(_, ty) => ty.clone(),
        TypedExpr::Null(ty) => ty.clone(),
        TypedExpr::TypeOf(_, _) => Type::Str,
        TypedExpr::FieldAccess(_, _, ty) => ty.clone(),
        TypedExpr::RecordLit(_, ty) => ty.clone(),
        TypedExpr::Constructor(_, _, ty) => ty.clone(),
        TypedExpr::Match(_, _, ty) => ty.clone(),
        TypedExpr::If(_, t, _) => expr_type(t),
    }
}

fn types_compat(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::Record(fa), Type::Record(fb)) => {
            fa.len() == fb.len()
                && fa.iter().zip(fb.iter()).all(|((na, ta), (nb, tb))| na == nb && types_compat(ta, tb))
        }
        (Type::Sum(va), Type::Sum(vb)) => {
            va.len() == vb.len()
                && va.iter().zip(vb.iter()).all(|((na, pa), (nb, pb))| {
                    na == nb && pa.len() == pb.len() && pa.iter().zip(pb.iter()).all(|(ta, tb)| types_compat(ta, tb))
                })
        }
        (Type::TypeVar(s), Type::TypeVar(t)) => s == t,
        _ => a == b,
    }
}

fn require_int(t: &Type) -> Result<(), TypeError> {
    if matches!(t, Type::Int) {
        Ok(())
    } else {
        Err(TypeError::Mismatch(Type::Int, t.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumina_part1_parser::parse;

    #[test]
    fn check_int() {
        let mut tc = TypeChecker::new();
        let prog = parse("42").unwrap();
        let typed = tc.check_program(&prog).unwrap();
        assert_eq!(typed.functions.len(), 1);
        assert!(matches!(
            typed.functions[0].body.as_slice(),
            [TypedStmt::Expr(_), TypedStmt::Return(None)]
        ));
    }

    #[test]
    fn check_add() {
        let mut tc = TypeChecker::new();
        let prog = parse("1 + 2").unwrap();
        let typed = tc.check_program(&prog).unwrap();
        assert_eq!(typed.functions.len(), 1);
        assert!(matches!(
            typed.functions[0].body.as_slice(),
            [TypedStmt::Expr(_), TypedStmt::Return(None)]
        ));
    }

    #[test]
    fn check_fn_main() {
        let mut tc = TypeChecker::new();
        let prog = parse("fn main() -> i64 { return 1 + 2; }").unwrap();
        let typed = tc.check_program(&prog).unwrap();
        assert_eq!(typed.functions.len(), 1);
        assert_eq!(typed.functions[0].name, "main");
        assert!(matches!(
            typed.functions[0].body.as_slice(),
            [TypedStmt::Return(Some(TypedExpr::Add(_, _)))]
        ));
    }

    #[test]
    fn check_hello_world() {
        let mut tc = TypeChecker::new();
        let prog = parse(r#"println("Hello, World!")"#).unwrap();
        let typed = tc.check_program(&prog).unwrap();
        assert_eq!(typed.functions.len(), 1);
        assert!(matches!(
            typed.functions[0].body.as_slice(),
            [TypedStmt::Expr(_), TypedStmt::Return(None)]
        ));
    }
}