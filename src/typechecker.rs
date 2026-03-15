use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TypeError {
    pub message: String,
    pub line: usize,
}

impl TypeError {
    fn new(msg: impl Into<String>, line: usize) -> Self {
        Self {
            message: msg.into(),
            line,
        }
    }
}

struct TypeEnv {
    vars: HashMap<String, Type>,
    funcs: HashMap<String, (Vec<Type>, Option<Type>)>,
    structs: HashMap<String, Vec<(String, Type)>>,
}

impl TypeEnv {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
            funcs: HashMap::new(),
            structs: HashMap::new(),
        }
    }
}

fn infer_expr_type(expr: &Expr, env: &TypeEnv) -> Option<Type> {
    match expr {
        Expr::IntLiteral(_) => Some(Type::정수),
        Expr::FloatLiteral(_) => Some(Type::실수),
        Expr::StringLiteral(_) => Some(Type::문자열),
        Expr::BoolLiteral(_) => Some(Type::불),
        Expr::NullLiteral => Some(Type::없음),
        Expr::Identifier(name) => env.vars.get(name).cloned(),
        Expr::BinaryOp { op, left, right } => {
            let lt = infer_expr_type(left, env)?;
            let rt = infer_expr_type(right, env)?;
            match op {
                BinaryOpKind::Eq
                | BinaryOpKind::NotEq
                | BinaryOpKind::Lt
                | BinaryOpKind::Gt
                | BinaryOpKind::LtEq
                | BinaryOpKind::GtEq
                | BinaryOpKind::And
                | BinaryOpKind::Or => Some(Type::불),
                BinaryOpKind::Add => match (&lt, &rt) {
                    (Type::문자열, Type::문자열) => Some(Type::문자열),
                    (Type::실수, _) | (_, Type::실수) => Some(Type::실수),
                    _ => Some(lt),
                },
                _ => {
                    if matches!(lt, Type::실수) || matches!(rt, Type::실수) {
                        Some(Type::실수)
                    } else {
                        Some(lt)
                    }
                }
            }
        }
        Expr::Call { name, .. } => env.funcs.get(name).and_then(|(_, ret)| ret.clone()),
        Expr::ArrayLiteral(_) => Some(Type::배열(Box::new(Type::정수))),
        Expr::TupleLiteral(_) => Some(Type::튜플(Vec::new())),
        Expr::Range { .. } => Some(Type::배열(Box::new(Type::정수))),
        _ => None,
    }
}

pub fn check(program: &Program) -> Vec<TypeError> {
    let mut errors = Vec::new();
    let mut env = TypeEnv::new();

    for stmt in &program.stmts {
        check_stmt(stmt, &mut env, &mut errors);
    }

    errors
}

fn check_stmt(stmt: &Stmt, env: &mut TypeEnv, errors: &mut Vec<TypeError>) {
    let line = stmt.span.line;

    match &stmt.kind {
        StmtKind::VarDecl {
            name, ty, value, ..
        } => {
            if let Some(declared_ty) = ty {
                if let Some(actual_ty) = infer_expr_type(value, env) {
                    if !types_compatible(declared_ty, &actual_ty) {
                        errors.push(TypeError::new(
                            format!(
                                "타입 불일치: '{}' 는 {:?} 타입으로 선언되었지만 {:?} 값이 할당됨",
                                name, declared_ty, actual_ty
                            ),
                            line,
                        ));
                    }
                }
                env.vars.insert(name.clone(), declared_ty.clone());
            } else if let Some(inferred) = infer_expr_type(value, env) {
                env.vars.insert(name.clone(), inferred);
            }
        }

        StmtKind::FuncDef {
            name,
            params,
            return_type,
            body,
        } => {
            let param_types: Vec<Type> = params.iter().map(|(_, t)| t.clone()).collect();
            env.funcs
                .insert(name.clone(), (param_types, return_type.clone()));

            for (pname, pty) in params {
                env.vars.insert(pname.clone(), pty.clone());
            }

            for s in body {
                check_stmt(s, env, errors);
            }

            if let Some(ret_ty) = return_type {
                check_return_types(body, ret_ty, env, errors);
            }
        }

        StmtKind::StructDef { name, fields } => {
            env.structs.insert(name.clone(), fields.clone());
        }

        StmtKind::If {
            then_block,
            else_block,
            ..
        } => {
            for s in then_block {
                check_stmt(s, env, errors);
            }
            if let Some(else_stmts) = else_block {
                for s in else_stmts {
                    check_stmt(s, env, errors);
                }
            }
        }

        StmtKind::ForLoop { init, body, .. } => {
            check_stmt(init, env, errors);
            for s in body {
                check_stmt(s, env, errors);
            }
        }

        StmtKind::ForIn { var_name, body, .. } => {
            env.vars.insert(var_name.clone(), Type::정수);
            for s in body {
                check_stmt(s, env, errors);
            }
        }

        StmtKind::WhileLoop { body, .. } => {
            for s in body {
                check_stmt(s, env, errors);
            }
        }

        StmtKind::TryCatch {
            try_block,
            error_name,
            catch_block,
        } => {
            for s in try_block {
                check_stmt(s, env, errors);
            }
            env.vars.insert(error_name.clone(), Type::문자열);
            for s in catch_block {
                check_stmt(s, env, errors);
            }
        }

        StmtKind::Match { arms, .. } => {
            for arm in arms {
                for s in &arm.body {
                    check_stmt(s, env, errors);
                }
            }
        }

        StmtKind::ImplBlock { methods, .. } => {
            for m in methods {
                check_stmt(m, env, errors);
            }
        }

        _ => {}
    }
}

fn check_return_types(body: &[Stmt], expected: &Type, env: &TypeEnv, errors: &mut Vec<TypeError>) {
    for stmt in body {
        if let StmtKind::Return(Some(expr)) = &stmt.kind {
            if let Some(actual) = infer_expr_type(expr, env) {
                if !types_compatible(expected, &actual) {
                    errors.push(TypeError::new(
                        format!("반환 타입 불일치: {:?} 예상, {:?} 반환", expected, actual),
                        stmt.span.line,
                    ));
                }
            }
        }
    }
}

fn types_compatible(declared: &Type, actual: &Type) -> bool {
    match (declared, actual) {
        (Type::정수, Type::정수) => true,
        (Type::실수, Type::실수) => true,
        (Type::실수, Type::정수) => true,
        (Type::문자열, Type::문자열) => true,
        (Type::불, Type::불) => true,
        (Type::없음, Type::없음) => true,
        (Type::배열(_), Type::배열(_)) => true,
        (Type::튜플(_), Type::튜플(_)) => true,
        (Type::구조체(a), Type::구조체(b)) => a == b,
        (Type::함수타입, _) => true,
        _ => false,
    }
}
