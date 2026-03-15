use crate::ast::{BinaryOpKind, Expr, Program, Stmt, StmtKind, Type, UnaryOpKind};
use std::collections::HashMap;

pub struct CodeGen {
    output: String,
    globals: String,
    temp_count: usize,
    label_count: usize,
    str_count: usize,
    loop_stack: Vec<(String, String)>,
    var_types: HashMap<String, &'static str>,
    struct_defs: HashMap<String, Vec<String>>,
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
            struct_defs: HashMap::new(),
        }
    }

    fn fresh_temp(&mut self) -> String {
        let t = format!("%t{}", self.temp_count);
        self.temp_count += 1;
        t
    }

    fn fresh_label(&mut self) -> usize {
        let l = self.label_count;
        self.label_count += 1;
        l
    }

    fn intern_string(&mut self, s: &str) -> (String, usize) {
        let name = format!("@.str{}", self.str_count);
        self.str_count += 1;

        let mut bytes: Vec<u8> = s.as_bytes().to_vec();
        bytes.push(0u8);
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

    fn llvm_type(ty: &Type) -> &'static str {
        match ty {
            Type::정수 => "i64",
            Type::실수 => "double",
            Type::문자열 => "i8*",
            Type::불 => "i1",
            Type::없음 => "void",
            Type::배열(_) => "i8*",
            Type::구조체(_) => "i8*",
            Type::함수타입 => "i8*",
        }
    }

    fn infer_type(&self, expr: &Expr) -> &'static str {
        match expr {
            Expr::IntLiteral(_) => "i64",
            Expr::FloatLiteral(_) => "double",
            Expr::StringLiteral(_) => "i8*",
            Expr::BoolLiteral(_) => "i1",
            Expr::NullLiteral => "i64",
            Expr::BinaryOp { op, left, .. } => match op {
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
            Expr::Identifier(name) => self.var_types.get(name.as_str()).copied().unwrap_or("i64"),
            Expr::Call { name, .. } => self.var_types.get(name.as_str()).copied().unwrap_or("i64"),
            Expr::ArrayLiteral(_) => "i64*",
            Expr::Index { .. } => "i64",
            _ => "i64",
        }
    }

    fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn gen_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::IntLiteral(n) => {
                let t = self.fresh_temp();
                self.emit(&format!("  {} = add nsw i64 0, {}", t, n));
                t
            }
            Expr::FloatLiteral(f) => {
                let t = self.fresh_temp();
                self.emit(&format!("  {} = fadd double 0.0, {:?}", t, f));
                t
            }
            Expr::BoolLiteral(b) => {
                let t = self.fresh_temp();
                let v = if *b { 1 } else { 0 };
                self.emit(&format!("  {} = add i1 0, {}", t, v));
                t
            }
            Expr::NullLiteral => {
                let t = self.fresh_temp();
                self.emit(&format!("  {} = add nsw i64 0, 0", t));
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
                let var_ty = self.var_types.get(name.as_str()).copied().unwrap_or("i64");
                let t = self.fresh_temp();
                self.emit(&format!(
                    "  {} = load {}, {}* %var_{}",
                    t, var_ty, var_ty, name
                ));
                t
            }
            Expr::Assign { name, value } => {
                let var_ty = self.var_types.get(name.as_str()).copied().unwrap_or("i64");
                let val = self.gen_expr(value);
                self.emit(&format!(
                    "  store {} {}, {}* %var_{}",
                    var_ty, val, var_ty, name
                ));
                val
            }
            Expr::BinaryOp { op, left, right } => {
                let lty = self.infer_type(left);

                // i8* + i8* → string concatenation via C runtime
                if matches!(op, BinaryOpKind::Add) && lty == "i8*" {
                    return self.gen_str_concat(left, right);
                }

                let lv = self.gen_expr(left);
                let rv = self.gen_expr(right);
                let t = self.fresh_temp();

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
                let expr_ty = self.infer_type(expr);
                let v = self.gen_expr(expr);
                let t = self.fresh_temp();
                match op {
                    UnaryOpKind::Neg => {
                        if expr_ty == "double" {
                            self.emit(&format!("  {} = fneg double {}", t, v));
                        } else {
                            self.emit(&format!("  {} = sub nsw i64 0, {}", t, v));
                        }
                    }
                    UnaryOpKind::Not => {
                        self.emit(&format!("  {} = xor i1 {}, true", t, v));
                    }
                }
                t
            }
            Expr::Call { name, args } => {
                if name == "출력" {
                    return self.gen_print(args);
                }

                let arg_types: Vec<&str> = args.iter().map(|a| self.infer_type(a)).collect();
                let arg_vals: Vec<String> = args.iter().map(|a| self.gen_expr(a)).collect();
                let ret_ty = self.var_types.get(name.as_str()).copied().unwrap_or("i64");
                let t = self.fresh_temp();
                let arg_str: String = arg_types
                    .iter()
                    .zip(arg_vals.iter())
                    .map(|(ty, v)| format!("{} {}", ty, v))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.emit(&format!("  {} = call {} @{}({})", t, ret_ty, name, arg_str));
                t
            }
            Expr::ArrayLiteral(elems) => {
                let count = elems.len();
                let byte_size = count * 8;

                let mem = self.fresh_temp();
                self.emit(&format!("  {} = call i8* @malloc(i64 {})", mem, byte_size));

                let data_ptr = self.fresh_temp();
                self.emit(&format!("  {} = bitcast i8* {} to i64*", data_ptr, mem));

                for (i, elem) in elems.iter().enumerate() {
                    let val = self.gen_expr(elem);
                    let elem_ptr = self.fresh_temp();
                    self.emit(&format!(
                        "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
                        elem_ptr, data_ptr, i
                    ));
                    self.emit(&format!("  store i64 {}, i64* {}", val, elem_ptr));
                }

                data_ptr
            }

            Expr::Index { object, index } => {
                let arr_ptr = self.gen_expr(object);
                let idx = self.gen_expr(index);
                let elem_ptr = self.fresh_temp();
                self.emit(&format!(
                    "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
                    elem_ptr, arr_ptr, idx
                ));
                let val = self.fresh_temp();
                self.emit(&format!("  {} = load i64, i64* {}", val, elem_ptr));
                val
            }

            Expr::IndexAssign {
                object,
                index,
                value,
            } => {
                let arr_ptr = self.gen_expr(object);
                let idx = self.gen_expr(index);
                let val = self.gen_expr(value);
                let elem_ptr = self.fresh_temp();
                self.emit(&format!(
                    "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
                    elem_ptr, arr_ptr, idx
                ));
                self.emit(&format!("  store i64 {}, i64* {}", val, elem_ptr));
                val
            }

            Expr::StructLiteral { name, fields } => {
                let field_defs = self.struct_defs.get(name).cloned().unwrap_or_default();
                let num_fields = field_defs.len().max(fields.len());
                let byte_size = num_fields * 8;

                let mem = self.fresh_temp();
                self.emit(&format!("  {} = call i8* @malloc(i64 {})", mem, byte_size));
                let data_ptr = self.fresh_temp();
                self.emit(&format!("  {} = bitcast i8* {} to i64*", data_ptr, mem));

                for (fname, fexpr) in fields {
                    let idx = field_defs.iter().position(|n| n == fname).unwrap_or(0);
                    let val = self.gen_expr(fexpr);
                    let fptr = self.fresh_temp();
                    self.emit(&format!(
                        "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
                        fptr, data_ptr, idx
                    ));
                    self.emit(&format!("  store i64 {}, i64* {}", val, fptr));
                }
                data_ptr
            }

            Expr::FieldAccess { object, field } => {
                let obj_ptr = self.gen_expr(object);
                let idx = self.find_field_index(object, field);
                let fptr = self.fresh_temp();
                self.emit(&format!(
                    "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
                    fptr, obj_ptr, idx
                ));
                let val = self.fresh_temp();
                self.emit(&format!("  {} = load i64, i64* {}", val, fptr));
                val
            }

            Expr::FieldAssign {
                object,
                field,
                value,
            } => {
                let obj_ptr = self.gen_expr(object);
                let idx = self.find_field_index(object, field);
                let val = self.gen_expr(value);
                let fptr = self.fresh_temp();
                self.emit(&format!(
                    "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
                    fptr, obj_ptr, idx
                ));
                self.emit(&format!("  store i64 {}, i64* {}", val, fptr));
                val
            }

            Expr::Range { .. } | Expr::MethodCall { .. } | Expr::Lambda { .. } => {
                let t = self.fresh_temp();
                self.emit(&format!("  {} = add nsw i64 0, 0", t));
                t
            }
        }
    }

    fn find_field_index(&self, object: &Expr, field: &str) -> usize {
        if let Expr::Identifier(name) = object {
            if let Some(struct_name) = self.var_types.get(name.as_str()) {
                if let Some(fields) = self.struct_defs.get(*struct_name) {
                    return fields.iter().position(|f| f == field).unwrap_or(0);
                }
            }
        }
        0
    }

    fn gen_str_concat(&mut self, left: &Expr, right: &Expr) -> String {
        let lv = self.gen_expr(left);
        let rv = self.gen_expr(right);

        let len_l = self.fresh_temp();
        self.emit(&format!("  {} = call i64 @strlen(i8* {})", len_l, lv));

        let len_r = self.fresh_temp();
        self.emit(&format!("  {} = call i64 @strlen(i8* {})", len_r, rv));

        let total = self.fresh_temp();
        self.emit(&format!("  {} = add nsw i64 {}, {}", total, len_l, len_r));

        let total_plus_one = self.fresh_temp();
        self.emit(&format!("  {} = add nsw i64 {}, 1", total_plus_one, total));

        let buf = self.fresh_temp();
        self.emit(&format!(
            "  {} = call i8* @malloc(i64 {})",
            buf, total_plus_one
        ));

        let cpy_tmp = self.fresh_temp();
        self.emit(&format!(
            "  {} = call i8* @strcpy(i8* {}, i8* {})",
            cpy_tmp, buf, lv
        ));

        let result = self.fresh_temp();
        self.emit(&format!(
            "  {} = call i8* @strcat(i8* {}, i8* {})",
            result, buf, rv
        ));

        buf
    }

    fn gen_print(&mut self, args: &[Expr]) -> String {
        if args.is_empty() {
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

        let arg = &args[0];
        match arg {
            Expr::StringLiteral(s) => {
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
                self.emit(&format!(
                    "  {} = call i32 (i8*, ...) @printf(i8* {}, {} {})",
                    t, fmt_ptr, ty, val
                ));
                t
            }
        }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::VarDecl {
                name, ty, value, ..
            } => {
                let llvm_ty = ty
                    .as_ref()
                    .map(|t| Self::llvm_type(t))
                    .unwrap_or_else(|| self.infer_type(value));
                self.var_types.insert(name.clone(), llvm_ty);
                self.emit(&format!("  %var_{} = alloca {}", name, llvm_ty));
                let val = self.gen_expr(value);
                self.emit(&format!(
                    "  store {} {}, {}* %var_{}",
                    llvm_ty, val, llvm_ty, name
                ));
            }
            StmtKind::ExprStmt(expr) => {
                self.gen_expr(expr);
            }
            StmtKind::Return(maybe_expr) => {
                if let Some(expr) = maybe_expr {
                    let ty = self.infer_type(expr);
                    let val = self.gen_expr(expr);
                    self.emit(&format!("  ret {} {}", ty, val));
                } else {
                    self.emit("  ret void");
                }
            }
            StmtKind::If {
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

                self.emit(&format!("{}:", then_lbl));
                for s in then_block {
                    self.gen_stmt(s);
                }
                self.emit(&format!("  br label %{}", end_lbl));

                if let Some(else_stmts) = else_block {
                    self.emit(&format!("{}:", else_lbl));
                    for s in else_stmts {
                        self.gen_stmt(s);
                    }
                    self.emit(&format!("  br label %{}", end_lbl));
                }

                self.emit(&format!("{}:", end_lbl));
            }
            StmtKind::WhileLoop { cond, body } => {
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
            StmtKind::ForLoop {
                init,
                cond,
                step,
                body,
            } => {
                let idx = self.fresh_label();
                let cond_lbl = format!("loop_cond{}", idx);
                let body_lbl = format!("loop_body{}", idx);
                let end_lbl = format!("loop_end{}", idx);

                self.gen_stmt(init);
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

                self.gen_stmt(step);
                self.emit(&format!("  br label %{}", cond_lbl));

                self.emit(&format!("{}:", end_lbl));
            }
            StmtKind::Break => {
                if let Some((_, end_lbl)) = self.loop_stack.last().cloned() {
                    self.emit(&format!("  br label %{}", end_lbl));
                }
            }
            StmtKind::Continue => {
                if let Some((cond_lbl, _)) = self.loop_stack.last().cloned() {
                    self.emit(&format!("  br label %{}", cond_lbl));
                }
            }
            StmtKind::FuncDef { .. } => {}
            StmtKind::StructDef { name, fields } => {
                let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                self.struct_defs.insert(name.clone(), field_names);
            }
            StmtKind::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                for s in try_block {
                    self.gen_stmt(s);
                }
                for s in catch_block {
                    self.gen_stmt(s);
                }
            }
            StmtKind::Import(_) => {}
            StmtKind::Match { expr, arms } => {
                let val = self.gen_expr(expr);
                let end_lbl = format!("match_end{}", self.fresh_label());

                for (i, arm) in arms.iter().enumerate() {
                    let arm_lbl = format!("match_arm{}_{}", self.label_count, i);
                    let next_lbl = if i + 1 < arms.len() {
                        format!("match_test{}_{}", self.label_count, i + 1)
                    } else {
                        end_lbl.clone()
                    };

                    match &arm.pattern {
                        crate::ast::Pattern::Wildcard | crate::ast::Pattern::Identifier(_) => {
                            self.emit(&format!("  br label %{}", arm_lbl));
                        }
                        crate::ast::Pattern::IntLiteral(n) => {
                            let cmp = self.fresh_temp();
                            self.emit(&format!("  {} = icmp eq i64 {}, {}", cmp, val, n));
                            self.emit(&format!(
                                "  br i1 {}, label %{}, label %{}",
                                cmp, arm_lbl, next_lbl
                            ));
                        }
                        crate::ast::Pattern::BoolLiteral(b) => {
                            let bv = if *b { 1 } else { 0 };
                            let cmp = self.fresh_temp();
                            self.emit(&format!("  {} = icmp eq i64 {}, {}", cmp, val, bv));
                            self.emit(&format!(
                                "  br i1 {}, label %{}, label %{}",
                                cmp, arm_lbl, next_lbl
                            ));
                        }
                        _ => {
                            self.emit(&format!("  br label %{}", arm_lbl));
                        }
                    }

                    self.emit(&format!("{}:", arm_lbl));
                    for s in &arm.body {
                        self.gen_stmt(s);
                    }
                    self.emit(&format!("  br label %{}", end_lbl));

                    if i + 1 < arms.len()
                        && !matches!(
                            arm.pattern,
                            crate::ast::Pattern::Wildcard | crate::ast::Pattern::Identifier(_)
                        )
                    {
                        self.emit(&format!("{}:", next_lbl));
                    }
                }

                self.emit(&format!("{}:", end_lbl));
            }
            StmtKind::ImplBlock { .. } => {}
            StmtKind::EnumDef { .. } => {}
            StmtKind::ForIn { iterable, body, .. } => {
                self.gen_expr(iterable);
                for s in body {
                    self.gen_stmt(s);
                }
            }
        }
    }

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

        self.var_types.clear();
        self.emit(&format!("define {} @{}({}) {{", ret_ty, name, param_str));
        self.emit("entry:");

        for (pname, pty) in params {
            let llvm_ty = Self::llvm_type(pty);
            self.var_types.insert(pname.clone(), llvm_ty);
            self.emit(&format!("  %var_{} = alloca {}", pname, llvm_ty));
            self.emit(&format!(
                "  store {} %{}, {}* %var_{}",
                llvm_ty, pname, llvm_ty, pname
            ));
        }

        for stmt in body {
            self.gen_stmt(stmt);
        }

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

    pub fn generate(&mut self, program: &Program) -> String {
        let mut func_defs: Vec<&Stmt> = Vec::new();
        let mut top_level: Vec<&Stmt> = Vec::new();
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
    use crate::ast::{Expr, Program, Stmt, StmtKind, Type};

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
}
