#![allow(dead_code, unused)]

use crate::ast::{BinaryOpKind, Expr, Program, Stmt, Type, UnaryOpKind};

/// LLVM IR text code generator for Han language.
/// Emits LLVM IR as a String — no inkwell, no LLVM APIs.
pub struct CodeGen {
    /// Accumulated function/body IR text
    output: String,
    /// Global string constants (emitted before functions)
    globals: String,
    /// Counter for SSA temporaries: %t0, %t1, ...
    temp_count: usize,
    /// Counter for branch labels: then0, else0, endif0, ...
    label_count: usize,
    /// Counter for global string constants: @.str0, @.str1, ...
    str_count: usize,
    /// Stack of loop-end labels for break/continue support
    loop_stack: Vec<(String, String)>, // (cond_label, end_label)
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
        }
    }

    /// Allocate a new SSA temporary name like %t0
    fn fresh_temp(&mut self) -> String {
        let t = format!("%t{}", self.temp_count);
        self.temp_count += 1;
        t
    }

    /// Allocate a new label index (returns the number, caller formats label names)
    fn fresh_label(&mut self) -> usize {
        let l = self.label_count;
        self.label_count += 1;
        l
    }

    /// Intern a string literal as a global constant.
    /// Returns (global_name, byte_len_including_null).
    fn intern_string(&mut self, s: &str) -> (String, usize) {
        let name = format!("@.str{}", self.str_count);
        self.str_count += 1;

        // Encode the string as LLVM IR hex escape bytes
        let mut bytes: Vec<u8> = s.as_bytes().to_vec();
        bytes.push(0u8); // null terminator
        let len = bytes.len();

        let encoded: String = bytes
            .iter()
            .map(|b| {
                if b.is_ascii_graphic() && *b != b'"' && *b != b'\\' {
                    format!("{}", *b as char)
                } else {
                    format!("\\{:02X}", b)
                }
            })
            .collect();

        self.globals.push_str(&format!(
            "{} = private unnamed_addr constant [{} x i8] c\"{}\"\n",
            name, len, encoded
        ));

        (name, len)
    }

    /// Map Han Type to LLVM IR type string
    fn llvm_type(ty: &Type) -> &'static str {
        match ty {
            Type::정수 => "i64",
            Type::실수 => "double",
            Type::문자열 => "i8*",
            Type::불 => "i1",
            Type::없음 => "void",
        }
    }

    /// Infer the LLVM type of an expression (best-effort, defaults to i64)
    fn infer_type(&self, expr: &Expr) -> &'static str {
        match expr {
            Expr::IntLiteral(_) => "i64",
            Expr::FloatLiteral(_) => "double",
            Expr::StringLiteral(_) => "i8*",
            Expr::BoolLiteral(_) => "i1",
            Expr::BinaryOp { op, left, right } => match op {
                BinaryOpKind::Eq
                | BinaryOpKind::NotEq
                | BinaryOpKind::Lt
                | BinaryOpKind::Gt
                | BinaryOpKind::LtEq
                | BinaryOpKind::GtEq
                | BinaryOpKind::And
                | BinaryOpKind::Or => "i1",
                _ => self.infer_type(left),
            },
            Expr::UnaryOp { op, expr } => match op {
                UnaryOpKind::Not => "i1",
                UnaryOpKind::Neg => self.infer_type(expr),
            },
            _ => "i64",
        }
    }

    /// Emit a line into the function body output
    fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    /// Generate IR for an expression. Returns the SSA value name holding the result.
    fn gen_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::IntLiteral(n) => {
                let t = self.fresh_temp();
                self.emit(&format!("  {} = add nsw i64 0, {}", t, n));
                t
            }
            Expr::FloatLiteral(f) => {
                let t = self.fresh_temp();
                // LLVM requires hex float or decimal; use decimal with enough precision
                self.emit(&format!("  {} = fadd double 0.0, {:?}", t, f));
                t
            }
            Expr::BoolLiteral(b) => {
                let t = self.fresh_temp();
                let v = if *b { 1 } else { 0 };
                self.emit(&format!("  {} = add i1 0, {}", t, v));
                t
            }
            Expr::StringLiteral(s) => {
                let (name, len) = self.intern_string(s);
                let t = self.fresh_temp();
                self.emit(&format!(
                    "  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i32 0, i32 0",
                    t, len, len, name
                ));
                t
            }
            Expr::Identifier(name) => {
                // Load from alloca'd variable. We don't track types per-variable here,
                // so we default to i64 for identifiers (simplification).
                let t = self.fresh_temp();
                self.emit(&format!("  {} = load i64, i64* %var_{}", t, name));
                t
            }
            Expr::Assign { name, value } => {
                let val = self.gen_expr(value);
                self.emit(&format!("  store i64 {}, i64* %var_{}", val, name));
                val
            }
            Expr::BinaryOp { op, left, right } => {
                let lv = self.gen_expr(left);
                let rv = self.gen_expr(right);
                let t = self.fresh_temp();
                let lty = self.infer_type(left);

                let instr = match (op, lty) {
                    (BinaryOpKind::Add, "double") => format!("fadd double {}, {}", lv, rv),
                    (BinaryOpKind::Sub, "double") => format!("fsub double {}, {}", lv, rv),
                    (BinaryOpKind::Mul, "double") => format!("fmul double {}, {}", lv, rv),
                    (BinaryOpKind::Div, "double") => format!("fdiv double {}, {}", lv, rv),
                    (BinaryOpKind::Add, _) => format!("add nsw i64 {}, {}", lv, rv),
                    (BinaryOpKind::Sub, _) => format!("sub nsw i64 {}, {}", lv, rv),
                    (BinaryOpKind::Mul, _) => format!("mul nsw i64 {}, {}", lv, rv),
                    (BinaryOpKind::Div, _) => format!("sdiv i64 {}, {}", lv, rv),
                    (BinaryOpKind::Mod, _) => format!("srem i64 {}, {}", lv, rv),
                    (BinaryOpKind::Eq, "double") => format!("fcmp oeq double {}, {}", lv, rv),
                    (BinaryOpKind::NotEq, "double") => format!("fcmp one double {}, {}", lv, rv),
                    (BinaryOpKind::Lt, "double") => format!("fcmp olt double {}, {}", lv, rv),
                    (BinaryOpKind::Gt, "double") => format!("fcmp ogt double {}, {}", lv, rv),
                    (BinaryOpKind::LtEq, "double") => format!("fcmp ole double {}, {}", lv, rv),
                    (BinaryOpKind::GtEq, "double") => format!("fcmp oge double {}, {}", lv, rv),
                    (BinaryOpKind::Eq, _) => format!("icmp eq i64 {}, {}", lv, rv),
                    (BinaryOpKind::NotEq, _) => format!("icmp ne i64 {}, {}", lv, rv),
                    (BinaryOpKind::Lt, _) => format!("icmp slt i64 {}, {}", lv, rv),
                    (BinaryOpKind::Gt, _) => format!("icmp sgt i64 {}, {}", lv, rv),
                    (BinaryOpKind::LtEq, _) => format!("icmp sle i64 {}, {}", lv, rv),
                    (BinaryOpKind::GtEq, _) => format!("icmp sge i64 {}, {}", lv, rv),
                    (BinaryOpKind::And, _) => format!("and i1 {}, {}", lv, rv),
                    (BinaryOpKind::Or, _) => format!("or i1 {}, {}", lv, rv),
                };
                self.emit(&format!("  {} = {}", t, instr));
                t
            }
            Expr::UnaryOp { op, expr } => {
                let v = self.gen_expr(expr);
                let t = self.fresh_temp();
                match op {
                    UnaryOpKind::Neg => {
                        self.emit(&format!("  {} = sub nsw i64 0, {}", t, v));
                    }
                    UnaryOpKind::Not => {
                        self.emit(&format!("  {} = xor i1 {}, true", t, v));
                    }
                }
                t
            }
            Expr::Call { name, args } => {
                // Special builtin: 출력 → printf
                if name == "출력" {
                    return self.gen_print(args);
                }

                // Regular function call — default return type i64
                let arg_vals: Vec<String> = args.iter().map(|a| self.gen_expr(a)).collect();
                let t = self.fresh_temp();
                let arg_str = arg_vals
                    .iter()
                    .map(|v| format!("i64 {}", v))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.emit(&format!("  {} = call i64 @{}({})", t, name, arg_str));
                t
            }
        }
    }

    /// Emit printf call for 출력 builtin.
    fn gen_print(&mut self, args: &[Expr]) -> String {
        if args.is_empty() {
            // Print newline
            let (nl_name, nl_len) = self.intern_string("\n");
            let ptr = self.fresh_temp();
            self.emit(&format!(
                "  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i32 0, i32 0",
                ptr, nl_len, nl_len, nl_name
            ));
            let t = self.fresh_temp();
            self.emit(&format!(
                "  {} = call i32 (i8*, ...) @printf(i8* {})",
                t, ptr
            ));
            return t;
        }

        // For each argument, determine how to print it
        let arg = &args[0];
        match arg {
            Expr::StringLiteral(s) => {
                // Append newline to the string for convenience
                let with_nl = format!("{}\n", s);
                let (name, len) = self.intern_string(&with_nl);
                let ptr = self.fresh_temp();
                self.emit(&format!(
                    "  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i32 0, i32 0",
                    ptr, len, len, name
                ));
                let t = self.fresh_temp();
                self.emit(&format!(
                    "  {} = call i32 (i8*, ...) @printf(i8* {})",
                    t, ptr
                ));
                t
            }
            _ => {
                // For non-string args, use a format string based on inferred type
                let ty = self.infer_type(arg);
                let fmt = match ty {
                    "double" => "%f\n",
                    "i1" => "%d\n",
                    _ => "%lld\n",
                };
                let (fmt_name, fmt_len) = self.intern_string(fmt);
                let val = self.gen_expr(arg);
                let fmt_ptr = self.fresh_temp();
                self.emit(&format!(
                    "  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i32 0, i32 0",
                    fmt_ptr, fmt_len, fmt_len, fmt_name
                ));
                let t = self.fresh_temp();
                let llvm_ty = ty;
                self.emit(&format!(
                    "  {} = call i32 (i8*, ...) @printf(i8* {}, {} {})",
                    t, fmt_ptr, llvm_ty, val
                ));
                t
            }
        }
    }

    /// Generate IR for a statement.
    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name, ty, value, ..
            } => {
                // Alloca on stack, then store initial value
                let llvm_ty = ty.as_ref().map(|t| Self::llvm_type(t)).unwrap_or("i64");
                self.emit(&format!("  %var_{} = alloca {}", name, llvm_ty));
                let val = self.gen_expr(value);
                self.emit(&format!(
                    "  store {} {}, {}* %var_{}",
                    llvm_ty, val, llvm_ty, name
                ));
            }
            Stmt::ExprStmt(expr) => {
                self.gen_expr(expr);
            }
            Stmt::Return(maybe_expr) => {
                if let Some(expr) = maybe_expr {
                    let ty = self.infer_type(expr);
                    let val = self.gen_expr(expr);
                    self.emit(&format!("  ret {} {}", ty, val));
                } else {
                    self.emit("  ret void");
                }
            }
            Stmt::If {
                cond,
                then_block,
                else_block,
            } => {
                let idx = self.fresh_label();
                let then_lbl = format!("then{}", idx);
                let else_lbl = format!("else{}", idx);
                let end_lbl = format!("endif{}", idx);

                let cond_val = self.gen_expr(cond);
                if else_block.is_some() {
                    self.emit(&format!(
                        "  br i1 {}, label %{}, label %{}",
                        cond_val, then_lbl, else_lbl
                    ));
                } else {
                    self.emit(&format!(
                        "  br i1 {}, label %{}, label %{}",
                        cond_val, then_lbl, end_lbl
                    ));
                }

                // then block
                self.emit(&format!("{}:", then_lbl));
                for s in then_block {
                    self.gen_stmt(s);
                }
                self.emit(&format!("  br label %{}", end_lbl));

                // else block
                if let Some(else_stmts) = else_block {
                    self.emit(&format!("{}:", else_lbl));
                    for s in else_stmts {
                        self.gen_stmt(s);
                    }
                    self.emit(&format!("  br label %{}", end_lbl));
                }

                self.emit(&format!("{}:", end_lbl));
            }
            Stmt::WhileLoop { cond, body } => {
                let idx = self.fresh_label();
                let cond_lbl = format!("loop_cond{}", idx);
                let body_lbl = format!("loop_body{}", idx);
                let end_lbl = format!("loop_end{}", idx);

                self.emit(&format!("  br label %{}", cond_lbl));
                self.emit(&format!("{}:", cond_lbl));

                let cond_val = self.gen_expr(cond);
                self.emit(&format!(
                    "  br i1 {}, label %{}, label %{}",
                    cond_val, body_lbl, end_lbl
                ));

                self.emit(&format!("{}:", body_lbl));
                self.loop_stack.push((cond_lbl.clone(), end_lbl.clone()));
                for s in body {
                    self.gen_stmt(s);
                }
                self.loop_stack.pop();
                self.emit(&format!("  br label %{}", cond_lbl));

                self.emit(&format!("{}:", end_lbl));
            }
            Stmt::ForLoop {
                init,
                cond,
                step,
                body,
            } => {
                let idx = self.fresh_label();
                let cond_lbl = format!("loop_cond{}", idx);
                let body_lbl = format!("loop_body{}", idx);
                let end_lbl = format!("loop_end{}", idx);

                // init
                self.gen_stmt(init);
                self.emit(&format!("  br label %{}", cond_lbl));

                // condition
                self.emit(&format!("{}:", cond_lbl));
                let cond_val = self.gen_expr(cond);
                self.emit(&format!(
                    "  br i1 {}, label %{}, label %{}",
                    cond_val, body_lbl, end_lbl
                ));

                // body
                self.emit(&format!("{}:", body_lbl));
                self.loop_stack.push((cond_lbl.clone(), end_lbl.clone()));
                for s in body {
                    self.gen_stmt(s);
                }
                self.loop_stack.pop();

                // step
                self.gen_stmt(step);
                self.emit(&format!("  br label %{}", cond_lbl));

                self.emit(&format!("{}:", end_lbl));
            }
            Stmt::Break => {
                if let Some((_, end_lbl)) = self.loop_stack.last().cloned() {
                    self.emit(&format!("  br label %{}", end_lbl));
                }
            }
            Stmt::Continue => {
                if let Some((cond_lbl, _)) = self.loop_stack.last().cloned() {
                    self.emit(&format!("  br label %{}", cond_lbl));
                }
            }
            Stmt::FuncDef { .. } => {
                // Function definitions are handled at top level in generate()
                // Nested function defs are not supported in LLVM IR directly
            }
        }
    }

    /// Generate IR for a function definition.
    fn gen_func(
        &mut self,
        name: &str,
        params: &[(String, Type)],
        return_type: &Option<Type>,
        body: &[Stmt],
    ) {
        let ret_ty = return_type
            .as_ref()
            .map(|t| Self::llvm_type(t))
            .unwrap_or("void");

        let param_str = params
            .iter()
            .map(|(pname, pty)| format!("{} %{}", Self::llvm_type(pty), pname))
            .collect::<Vec<_>>()
            .join(", ");

        self.emit(&format!("define {} @{}({}) {{", ret_ty, name, param_str));
        self.emit("entry:");

        // Alloca + store for each parameter so they can be reassigned
        for (pname, pty) in params {
            let llvm_ty = Self::llvm_type(pty);
            self.emit(&format!("  %var_{} = alloca {}", pname, llvm_ty));
            self.emit(&format!(
                "  store {} %{}, {}* %var_{}",
                llvm_ty, pname, llvm_ty, pname
            ));
        }

        for stmt in body {
            self.gen_stmt(stmt);
        }

        // Ensure function ends with a terminator
        if ret_ty == "void" {
            self.emit("  ret void");
        } else if ret_ty == "i32" {
            self.emit("  ret i32 0");
        } else {
            self.emit("  ret i64 0");
        }

        self.emit("}");
        self.emit("");
    }

    /// Generate the complete LLVM IR module for a program.
    pub fn generate(&mut self, program: &Program) -> String {
        // Separate top-level function defs from other statements
        let mut func_defs: Vec<&Stmt> = Vec::new();
        let mut top_level: Vec<&Stmt> = Vec::new();
        let mut has_main = false;

        for stmt in &program.stmts {
            match stmt {
                Stmt::FuncDef { name, .. } => {
                    if name == "main" {
                        has_main = true;
                    }
                    func_defs.push(stmt);
                }
                _ => top_level.push(stmt),
            }
        }

        // Generate all function definitions
        for stmt in &func_defs {
            if let Stmt::FuncDef {
                name,
                params,
                return_type,
                body,
            } = stmt
            {
                self.gen_func(name, params, return_type, body);
            }
        }

        // If there are top-level statements and no explicit main, wrap them
        if !top_level.is_empty() && !has_main {
            self.emit("define i32 @main() {");
            self.emit("entry:");
            for stmt in &top_level {
                self.gen_stmt(stmt);
            }
            self.emit("  ret i32 0");
            self.emit("}");
            self.emit("");
        }

        // Build the final module
        let mut module = String::new();
        module.push_str("; Han Language Generated IR\n");
        module.push_str("declare i32 @printf(i8* nocapture readonly, ...)\n");
        module.push_str("declare i8* @fgets(i8*, i32, i8*)\n");
        module.push_str("\n");

        // Emit global string constants
        if !self.globals.is_empty() {
            module.push_str(&self.globals);
            module.push('\n');
        }

        // Emit function bodies
        module.push_str(&self.output);

        module
    }
}

/// Public entry point: generate LLVM IR text from a parsed Program.
pub fn codegen(program: &Program) -> String {
    let mut cg = CodeGen::new();
    cg.generate(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Program, Stmt, Type};

    fn make_print_call(s: &str) -> Stmt {
        Stmt::ExprStmt(Expr::Call {
            name: "출력".to_string(),
            args: vec![Expr::StringLiteral(s.to_string())],
        })
    }

    #[test]
    fn test_codegen_hello() {
        let program = Program::new(vec![make_print_call("안녕하세요!")]);
        let ir = codegen(&program);
        assert!(
            ir.contains("@printf") || ir.contains("printf"),
            "IR should contain printf call, got:\n{}",
            ir
        );
    }

    #[test]
    fn test_codegen_function() {
        let program = Program::new(vec![Stmt::FuncDef {
            name: "더하기".to_string(),
            params: vec![
                ("가".to_string(), Type::정수),
                ("나".to_string(), Type::정수),
            ],
            return_type: Some(Type::정수),
            body: vec![Stmt::Return(Some(Expr::BinaryOp {
                op: BinaryOpKind::Add,
                left: Box::new(Expr::Identifier("가".to_string())),
                right: Box::new(Expr::Identifier("나".to_string())),
            }))],
        }]);
        let ir = codegen(&program);
        assert!(
            ir.contains("define"),
            "IR should contain 'define', got:\n{}",
            ir
        );
    }

    #[test]
    fn test_codegen_main_wrapper() {
        // Top-level 출력("hi") should be wrapped in define i32 @main()
        let program = Program::new(vec![make_print_call("hi")]);
        let ir = codegen(&program);
        assert!(
            ir.contains("define i32 @main()"),
            "IR should wrap top-level stmts in main, got:\n{}",
            ir
        );
    }

    #[test]
    fn test_codegen_if_else() {
        let program = Program::new(vec![Stmt::If {
            cond: Expr::BoolLiteral(true),
            then_block: vec![make_print_call("then")],
            else_block: Some(vec![make_print_call("else")]),
        }]);
        let ir = codegen(&program);
        assert!(ir.contains("br i1"), "IR should contain conditional branch");
        assert!(ir.contains("then0"), "IR should have then label");
        assert!(ir.contains("else0"), "IR should have else label");
    }

    #[test]
    fn test_codegen_while_loop() {
        let program = Program::new(vec![Stmt::WhileLoop {
            cond: Expr::BoolLiteral(false),
            body: vec![],
        }]);
        let ir = codegen(&program);
        assert!(ir.contains("loop_cond0"), "IR should have loop_cond label");
        assert!(ir.contains("loop_end0"), "IR should have loop_end label");
    }

    #[test]
    fn test_codegen_var_decl() {
        let program = Program::new(vec![Stmt::VarDecl {
            name: "x".to_string(),
            ty: Some(Type::정수),
            value: Expr::IntLiteral(42),
            mutable: true,
        }]);
        let ir = codegen(&program);
        assert!(
            ir.contains("alloca"),
            "IR should contain alloca for var decl"
        );
        assert!(ir.contains("store"), "IR should contain store for var decl");
    }

    #[test]
    fn test_codegen_module_header() {
        let program = Program::new(vec![]);
        let ir = codegen(&program);
        assert!(
            ir.contains("; Han Language Generated IR"),
            "IR should have module header"
        );
        assert!(
            ir.contains("declare i32 @printf"),
            "IR should declare printf"
        );
    }
}
