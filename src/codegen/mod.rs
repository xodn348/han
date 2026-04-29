use crate::ast::{Program, StmtKind, StmtVisitor};
use std::collections::{HashMap, HashSet};

mod builtins;
mod control_flow;
mod emission;
mod expr;
mod stmt;
mod types;
mod visitor;

#[derive(Clone)]
pub(super) struct PendingLambdaBinding {
    pub(super) full_param_types: Vec<&'static str>,
    pub(super) captured_values: Vec<(String, &'static str)>,
}

#[derive(Clone)]
pub(super) struct LambdaBinding {
    pub(super) full_param_types: Vec<&'static str>,
    pub(super) captured_bindings: Vec<(String, &'static str)>,
}

pub struct CodeGen {
    pub(super) output: String,
    pub(super) globals: String,
    pub(super) temp_count: usize,
    pub(super) label_count: usize,
    pub(super) str_count: usize,
    pub(super) loop_stack: Vec<(String, String)>,
    pub(super) var_types: HashMap<String, &'static str>,
    pub(super) struct_var_types: HashMap<String, String>,
    pub(super) lambda_bindings: HashMap<String, LambdaBinding>,
    pub(super) struct_defs: HashMap<String, Vec<String>>,
    pub(super) enum_defs: HashMap<String, Vec<String>>,
    pub(super) imported_paths: HashSet<String>,
    pub(super) current_error_flag: Option<String>,
    pub(super) current_error_message: Option<String>,
    #[allow(dead_code)]
    pub(super) indent_level: usize,
}

impl Default for CodeGen {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            globals: String::new(),
            temp_count: 0,
            label_count: 0,
            str_count: 0,
            loop_stack: Vec::new(),
            var_types: HashMap::new(),
            struct_var_types: HashMap::new(),
            lambda_bindings: HashMap::new(),
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            imported_paths: HashSet::new(),
            current_error_flag: None,
            current_error_message: None,
            indent_level: 0,
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        let mut func_defs: Vec<&crate::ast::Stmt> = Vec::new();
        let mut impl_defs: Vec<&crate::ast::Stmt> = Vec::new();
        let mut top_level: Vec<&crate::ast::Stmt> = Vec::new();
        let mut has_main = false;

        // first pass: collect struct definitions
        for stmt in &program.stmts {
            if let StmtKind::StructDef { name, fields } = &stmt.kind {
                let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                self.struct_defs.insert(name.clone(), field_names);
            }
        }

        for stmt in &program.stmts {
            match &stmt.kind {
                StmtKind::FuncDef {
                    name, return_type, ..
                } => {
                    if name == "main" {
                        has_main = true;
                    }
                    let ret_ty = return_type
                        .as_ref()
                        .map(|t| Self::llvm_type(t))
                        .unwrap_or("void");
                    self.var_types.insert(name.clone(), ret_ty);
                    func_defs.push(stmt);
                }
                StmtKind::ImplBlock { .. } => impl_defs.push(stmt),
                _ => top_level.push(stmt),
            }
        }

        for stmt in &func_defs {
            if let StmtKind::FuncDef {
                name,
                params,
                return_type,
                body,
            } = &stmt.kind
            {
                self.gen_func(name, params, return_type, body);
            }
        }

        for stmt in &impl_defs {
            self.visit_stmt(stmt);
        }

        if !top_level.is_empty() && !has_main {
            self.var_types.clear();
            self.struct_var_types.clear();
            self.lambda_bindings.clear();
            self.emit("define i32 @main() {");
            self.emit("entry:");
            self.init_error_state();
            for stmt in &top_level {
                self.visit_stmt(stmt);
            }
            self.emit("  ret i32 0");
            self.emit("}");
            self.emit("");
            self.current_error_flag = None;
            self.current_error_message = None;
        }

        let mut module = String::new();
        module.push_str("; Han Language Generated IR\n");
        module.push_str("declare i32 @printf(i8* nocapture readonly, ...)\n");
        module.push_str("declare i8* @fgets(i8*, i32, i8*)\n");
        module.push_str("declare i64 @strlen(i8*)\n");
        module.push_str("declare i8* @malloc(i64)\n");
        module.push_str("declare i8* @strcpy(i8*, i8*)\n");
        module.push_str("declare i8* @strcat(i8*, i8*)\n");
        module.push('\n');

        if !self.globals.is_empty() {
            module.push_str(&self.globals);
            module.push('\n');
        }

        module.push_str(&self.output);
        module
    }
}

pub fn codegen(program: &Program) -> String {
    let mut cg = CodeGen::new();
    cg.generate(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOpKind, Expr, Program, Stmt, StmtKind, Type};

    fn make_print_call(s: &str) -> Stmt {
        Stmt::unspanned(StmtKind::ExprStmt(Expr::Call {
            name: "출력".to_string(),
            args: vec![Expr::StringLiteral(s.to_string())],
        }))
    }

    #[test]
    fn test_codegen_hello() {
        let program = Program::new(vec![make_print_call("안녕하세요!")]);
        let ir = codegen(&program);
        assert!(ir.contains("@printf") || ir.contains("printf"));
    }

    #[test]
    fn test_codegen_function() {
        let program = Program::new(vec![Stmt::unspanned(StmtKind::FuncDef {
            name: "더하기".to_string(),
            params: vec![
                ("가".to_string(), Type::정수),
                ("나".to_string(), Type::정수),
            ],
            return_type: Some(Type::정수),
            body: vec![Stmt::unspanned(StmtKind::Return(Some(Expr::BinaryOp {
                op: BinaryOpKind::Add,
                left: Box::new(Expr::Identifier("가".to_string())),
                right: Box::new(Expr::Identifier("나".to_string())),
            })))],
        })]);
        let ir = codegen(&program);
        assert!(ir.contains("define"));
    }

    #[test]
    fn test_codegen_main_wrapper() {
        let program = Program::new(vec![make_print_call("hi")]);
        let ir = codegen(&program);
        assert!(ir.contains("define i32 @main()"));
    }

    #[test]
    fn test_codegen_if_else() {
        let program = Program::new(vec![Stmt::unspanned(StmtKind::If {
            cond: Expr::BoolLiteral(true),
            then_block: vec![make_print_call("then")],
            else_block: Some(vec![make_print_call("else")]),
        })]);
        let ir = codegen(&program);
        assert!(ir.contains("br i1"));
        assert!(ir.contains("then0"));
        assert!(ir.contains("else0"));
    }

    #[test]
    fn test_codegen_while_loop() {
        let program = Program::new(vec![Stmt::unspanned(StmtKind::WhileLoop {
            cond: Expr::BoolLiteral(false),
            body: vec![],
        })]);
        let ir = codegen(&program);
        assert!(ir.contains("loop_cond0"));
        assert!(ir.contains("loop_end0"));
    }

    #[test]
    fn test_codegen_var_decl() {
        let program = Program::new(vec![Stmt::unspanned(StmtKind::VarDecl {
            name: "x".to_string(),
            ty: Some(Type::정수),
            value: Expr::IntLiteral(42),
            mutable: true,
        })]);
        let ir = codegen(&program);
        assert!(ir.contains("alloca"));
        assert!(ir.contains("store"));
    }

    #[test]
    fn test_codegen_module_header() {
        let program = Program::new(vec![]);
        let ir = codegen(&program);
        assert!(ir.contains("; Han Language Generated IR"));
        assert!(ir.contains("declare i32 @printf"));
    }

    #[test]
    fn test_codegen_for_in_reads_array_length_from_header() {
        let program = Program::new(vec![Stmt::unspanned(StmtKind::ForIn {
            var_name: "i".to_string(),
            iterable: Expr::Range {
                start: Box::new(Expr::IntLiteral(0)),
                end: Box::new(Expr::IntLiteral(5)),
            },
            body: vec![Stmt::unspanned(StmtKind::ExprStmt(Expr::Call {
                name: "출력".to_string(),
                args: vec![Expr::Identifier("i".to_string())],
            }))],
        })]);
        let ir = codegen(&program);
        assert!(ir.contains("i64 -1"));
        assert!(ir.contains("loop_cond"));
        assert!(ir.contains("load i64, i64* %"));
    }

    #[test]
    fn test_codegen_try_catch_uses_error_branching() {
        let program = Program::new(vec![Stmt::unspanned(StmtKind::TryCatch {
            try_block: vec![Stmt::unspanned(StmtKind::VarDecl {
                name: "x".to_string(),
                ty: Some(Type::정수),
                value: Expr::BinaryOp {
                    op: BinaryOpKind::Div,
                    left: Box::new(Expr::IntLiteral(1)),
                    right: Box::new(Expr::IntLiteral(0)),
                },
                mutable: true,
            })],
            error_name: "오류".to_string(),
            catch_block: vec![make_print_call("caught")],
        })]);
        let ir = codegen(&program);
        assert!(ir.contains("catch"));
        assert!(ir.contains("store i1 1, i1* %error_flag"));
        assert!(ir.contains("load i1, i1* %error_flag"));
    }

    #[test]
    fn test_codegen_enum_match_switches_on_enum_tag() {
        let program = Program::new(vec![
            Stmt::unspanned(StmtKind::EnumDef {
                name: "Direction".to_string(),
                variants: vec!["Up".to_string(), "Down".to_string()],
            }),
            Stmt::unspanned(StmtKind::VarDecl {
                name: "dir".to_string(),
                ty: None,
                value: Expr::Identifier("Direction::Down".to_string()),
                mutable: true,
            }),
            Stmt::unspanned(StmtKind::Match {
                expr: Expr::Identifier("dir".to_string()),
                arms: vec![
                    crate::ast::MatchArm {
                        pattern: crate::ast::Pattern::Identifier("Up".to_string()),
                        body: vec![make_print_call("up")],
                    },
                    crate::ast::MatchArm {
                        pattern: crate::ast::Pattern::Identifier("Down".to_string()),
                        body: vec![make_print_call("down")],
                    },
                    crate::ast::MatchArm {
                        pattern: crate::ast::Pattern::Wildcard,
                        body: vec![make_print_call("default")],
                    },
                ],
            }),
        ]);
        let ir = codegen(&program);
        assert!(ir.contains("icmp eq i64"));
        assert!(ir.contains("match_arm"));
    }
}
