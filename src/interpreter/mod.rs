mod builtins;
mod env;
mod evaluator;
mod value;

pub use env::{EnvRef, Environment};
pub use evaluator::{eval_block, eval_expr, eval_stmt};
pub use value::{RuntimeError, Signal, Value};

use crate::ast::Program;
use std::cell::RefCell;

thread_local! {
    static OUTPUT_BUFFER: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[allow(dead_code)]
pub fn capture_start() {
    OUTPUT_BUFFER.with(|b| *b.borrow_mut() = Some(String::new()));
}

#[allow(dead_code)]
pub fn capture_flush() -> String {
    OUTPUT_BUFFER.with(|b| b.borrow_mut().take().unwrap_or_default())
}

pub(super) fn output_line(s: &str) {
    OUTPUT_BUFFER.with(|b| {
        if let Some(buf) = b.borrow_mut().as_mut() {
            buf.push_str(s);
            buf.push('\n');
        } else {
            println!("{}", s);
        }
    });
}

pub fn interpret(program: Program) -> Result<(), RuntimeError> {
    let env = Environment::new_ref();
    eval_block(&program.stmts, &env)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOpKind, Expr, Program, Stmt, StmtKind, Type};

    #[test]
    fn test_env_set_get() {
        let env = Environment::new_ref();
        env.borrow_mut().set("나이".to_string(), Value::Int(20));
        assert!(matches!(env.borrow().get("나이"), Some(Value::Int(20))));
    }

    #[test]
    fn test_env_scope_chain() {
        let outer = Environment::new_ref();
        outer.borrow_mut().set("x".to_string(), Value::Int(10));
        let inner = Environment::new_enclosed(outer);
        assert!(matches!(inner.borrow().get("x"), Some(Value::Int(10))));
        assert!(inner.borrow().get("y").is_none());
    }

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Bool(true).to_string(), "참");
        assert_eq!(Value::Bool(false).to_string(), "거짓");
        assert_eq!(Value::Void.to_string(), "없음");
    }

    #[test]
    fn test_env_update() {
        let env = Environment::new_ref();
        env.borrow_mut().set("x".to_string(), Value::Int(1));
        env.borrow_mut().update("x", Value::Int(2));
        assert!(matches!(env.borrow().get("x"), Some(Value::Int(2))));
    }

    #[test]
    fn test_eval_arithmetic() {
        let env = Environment::new_ref();
        let expr = Expr::BinaryOp {
            op: BinaryOpKind::Add,
            left: Box::new(Expr::IntLiteral(3)),
            right: Box::new(Expr::BinaryOp {
                op: BinaryOpKind::Mul,
                left: Box::new(Expr::IntLiteral(5)),
                right: Box::new(Expr::IntLiteral(2)),
            }),
        };
        let result = eval_expr(&expr, &env, 0).unwrap();
        assert!(matches!(result, Value::Int(13)));
    }

    #[test]
    fn test_eval_var_decl() {
        let env = Environment::new_ref();
        let stmt = Stmt::unspanned(StmtKind::VarDecl {
            name: "나이".to_string(),
            ty: None,
            value: Expr::IntLiteral(20),
            mutable: true,
        });
        eval_stmt(&stmt, &env).unwrap();
        assert!(matches!(env.borrow().get("나이"), Some(Value::Int(20))));
    }

    #[test]
    fn test_eval_if_stmt() {
        let env = Environment::new_ref();
        let stmt = Stmt::unspanned(StmtKind::If {
            cond: Expr::BoolLiteral(true),
            then_block: vec![Stmt::unspanned(StmtKind::VarDecl {
                name: "x".to_string(),
                ty: None,
                value: Expr::IntLiteral(1),
                mutable: false,
            })],
            else_block: None,
        });
        eval_stmt(&stmt, &env).unwrap();
        assert!(matches!(env.borrow().get("x"), Some(Value::Int(1))));
    }

    #[test]
    fn test_const_reassign_error() {
        let env = Environment::new_ref();
        let decl = Stmt::unspanned(StmtKind::VarDecl {
            name: "파이".to_string(),
            ty: None,
            value: Expr::FloatLiteral(3.14),
            mutable: false,
        });
        eval_stmt(&decl, &env).unwrap();
        assert!(env.borrow().is_const("파이"));

        let assign_expr = Expr::Assign {
            name: "파이".to_string(),
            value: Box::new(Expr::FloatLiteral(3.0)),
        };
        let result = eval_expr(&assign_expr, &env, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fibonacci() {
        let fib_body = vec![
            Stmt::unspanned(StmtKind::If {
                cond: Expr::BinaryOp {
                    op: BinaryOpKind::LtEq,
                    left: Box::new(Expr::Identifier("n".to_string())),
                    right: Box::new(Expr::IntLiteral(1)),
                },
                then_block: vec![Stmt::unspanned(StmtKind::Return(Some(Expr::Identifier(
                    "n".to_string(),
                ))))],
                else_block: None,
            }),
            Stmt::unspanned(StmtKind::Return(Some(Expr::BinaryOp {
                op: BinaryOpKind::Add,
                left: Box::new(Expr::Call {
                    name: "피보나치".to_string(),
                    args: vec![Expr::BinaryOp {
                        op: BinaryOpKind::Sub,
                        left: Box::new(Expr::Identifier("n".to_string())),
                        right: Box::new(Expr::IntLiteral(1)),
                    }],
                }),
                right: Box::new(Expr::Call {
                    name: "피보나치".to_string(),
                    args: vec![Expr::BinaryOp {
                        op: BinaryOpKind::Sub,
                        left: Box::new(Expr::Identifier("n".to_string())),
                        right: Box::new(Expr::IntLiteral(2)),
                    }],
                }),
            }))),
        ];

        let program = Program::new(vec![
            Stmt::unspanned(StmtKind::FuncDef {
                name: "피보나치".to_string(),
                params: vec![("n".to_string(), Type::정수)],
                return_type: Some(Type::정수),
                body: fib_body,
            }),
            Stmt::unspanned(StmtKind::VarDecl {
                name: "결과".to_string(),
                ty: None,
                value: Expr::Call {
                    name: "피보나치".to_string(),
                    args: vec![Expr::IntLiteral(10)],
                },
                mutable: false,
            }),
        ]);

        let env = Environment::new_ref();
        eval_block(&program.stmts, &env).unwrap();
        assert!(matches!(env.borrow().get("결과"), Some(Value::Int(55))));
    }

    #[test]
    fn test_출력() {
        let program = Program::new(vec![Stmt::unspanned(StmtKind::ExprStmt(Expr::Call {
            name: "출력".to_string(),
            args: vec![Expr::StringLiteral("안녕".to_string())],
        }))]);
        let result = interpret(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_skips_duplicate_includes() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_path = std::env::temp_dir().join(format!("han_import_once_{}.hgl", suffix));
        let import_source = "카운터 = 카운터 + 1\n";
        fs::write(&temp_path, import_source).unwrap();

        let import_path = temp_path
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");

        let main_source = format!(
            "변수 카운터 = 0\n포함 \"{}\"\n포함 \"{}\"\n",
            import_path, import_path
        );

        let tokens = crate::lexer::tokenize(&main_source);
        let program = crate::parser::parse(tokens).unwrap();
        let env = Environment::new_ref();
        let result = eval_block(&program.stmts, &env);

        let _ = fs::remove_file(&temp_path);

        assert!(result.is_ok());
        assert!(matches!(env.borrow().get("카운터"), Some(Value::Int(1))));
    }
}
