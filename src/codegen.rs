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
    enum_defs: HashMap<String, Vec<String>>,
    current_error_flag: Option<String>,
    current_error_message: Option<String>,
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
            enum_defs: HashMap::new(),
            current_error_flag: None,
            current_error_message: None,
        }
    }

    fn sanitize_ident(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c.to_string()
                } else {
                    format!("u{:04X}", c as u32)
                }
            })
            .collect()
    }

    fn var_ptr(name: &str) -> String {
        format!("%var_{}", Self::sanitize_ident(name))
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
            Type::튜플(_) => "i8*",
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
            Expr::Range { .. } => "i64*",
            Expr::Index { .. } => "i64",
            _ => "i64",
        }
    }

    fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn gen_string_ptr_literal(&mut self, s: &str) -> String {
        let (name, len) = self.intern_string(s);
        let ptr = self.fresh_temp();
        self.emit(&format!(
            "  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i32 0, i32 0",
            ptr, len, len, name
        ));
        ptr
    }

    fn init_error_state(&mut self) {
        self.current_error_flag = Some("%error_flag".to_string());
        self.current_error_message = Some("%error_message".to_string());
        self.emit("  %error_flag = alloca i1");
        self.emit("  store i1 0, i1* %error_flag");
        self.emit("  %error_message = alloca i8*");
        self.emit("  store i8* null, i8** %error_message");
    }

    fn clear_error_state(&mut self) {
        if let Some(flag_ptr) = self.current_error_flag.clone() {
            self.emit(&format!("  store i1 0, i1* {}", flag_ptr));
        }
        if let Some(message_ptr) = self.current_error_message.clone() {
            self.emit(&format!("  store i8* null, i8** {}", message_ptr));
        }
    }

    fn record_runtime_error(&mut self, message: &str) {
        if let Some(flag_ptr) = self.current_error_flag.clone() {
            self.emit(&format!("  store i1 1, i1* {}", flag_ptr));
        }
        if let Some(message_ptr) = self.current_error_message.clone() {
            let ptr = self.gen_string_ptr_literal(message);
            self.emit(&format!("  store i8* {}, i8** {}", ptr, message_ptr));
        }
    }

    fn allocate_array_storage(&mut self, len: &str) -> String {
        let total_slots = self.fresh_temp();
        self.emit(&format!("  {} = add nsw i64 {}, 1", total_slots, len));

        let byte_size = self.fresh_temp();
        self.emit(&format!("  {} = mul nsw i64 {}, 8", byte_size, total_slots));

        let mem = self.fresh_temp();
        self.emit(&format!("  {} = call i8* @malloc(i64 {})", mem, byte_size));

        let header_ptr = self.fresh_temp();
        self.emit(&format!("  {} = bitcast i8* {} to i64*", header_ptr, mem));
        self.emit(&format!("  store i64 {}, i64* {}", len, header_ptr));

        let data_ptr = self.fresh_temp();
        self.emit(&format!(
            "  {} = getelementptr inbounds i64, i64* {}, i64 1",
            data_ptr, header_ptr
        ));
        data_ptr
    }

    fn load_array_length(&mut self, data_ptr: &str) -> String {
        let len_ptr = self.fresh_temp();
        self.emit(&format!(
            "  {} = getelementptr inbounds i64, i64* {}, i64 -1",
            len_ptr, data_ptr
        ));

        let len = self.fresh_temp();
        self.emit(&format!("  {} = load i64, i64* {}", len, len_ptr));
        len
    }

    fn gen_checked_int_div_or_mod(
        &mut self,
        op: &BinaryOpKind,
        left: &Expr,
        right: &Expr,
    ) -> String {
        let lv = self.gen_expr(left);
        let rv = self.gen_expr(right);
        let result_ptr = self.fresh_temp();
        self.emit(&format!("  {} = alloca i64", result_ptr));
        self.emit(&format!("  store i64 0, i64* {}", result_ptr));

        let is_zero = self.fresh_temp();
        self.emit(&format!("  {} = icmp eq i64 {}, 0", is_zero, rv));

        let idx = self.fresh_label();
        let error_lbl = format!("arith_error{}", idx);
        let ok_lbl = format!("arith_ok{}", idx);
        let end_lbl = format!("arith_end{}", idx);

        self.emit(&format!(
            "  br i1 {}, label %{}, label %{}",
            is_zero, error_lbl, ok_lbl
        ));

        self.emit(&format!("{}:", error_lbl));
        let message = if matches!(op, BinaryOpKind::Div) {
            "0으로 나눌 수 없습니다"
        } else {
            "0으로 나머지 연산 불가"
        };
        self.record_runtime_error(message);
        self.emit(&format!("  br label %{}", end_lbl));

        self.emit(&format!("{}:", ok_lbl));
        let result = self.fresh_temp();
        let instr = if matches!(op, BinaryOpKind::Div) {
            format!("sdiv i64 {}, {}", lv, rv)
        } else {
            format!("srem i64 {}, {}", lv, rv)
        };
        self.emit(&format!("  {} = {}", result, instr));
        self.emit(&format!("  store i64 {}, i64* {}", result, result_ptr));
        self.emit(&format!("  br label %{}", end_lbl));

        self.emit(&format!("{}:", end_lbl));
        let final_result = self.fresh_temp();
        self.emit(&format!(
            "  {} = load i64, i64* {}",
            final_result, result_ptr
        ));
        final_result
    }

    fn gen_checked_index_load(&mut self, object: &Expr, index: &Expr) -> String {
        let arr_ptr = self.gen_expr(object);
        let idx = self.gen_expr(index);
        let len = self.load_array_length(&arr_ptr);
        let result_ptr = self.fresh_temp();
        self.emit(&format!("  {} = alloca i64", result_ptr));
        self.emit(&format!("  store i64 0, i64* {}", result_ptr));

        let is_non_negative = self.fresh_temp();
        self.emit(&format!("  {} = icmp sge i64 {}, 0", is_non_negative, idx));
        let is_before_end = self.fresh_temp();
        self.emit(&format!(
            "  {} = icmp slt i64 {}, {}",
            is_before_end, idx, len
        ));
        let in_bounds = self.fresh_temp();
        self.emit(&format!(
            "  {} = and i1 {}, {}",
            in_bounds, is_non_negative, is_before_end
        ));

        let block_id = self.fresh_label();
        let ok_lbl = format!("index_ok{}", block_id);
        let error_lbl = format!("index_error{}", block_id);
        let end_lbl = format!("index_end{}", block_id);

        self.emit(&format!(
            "  br i1 {}, label %{}, label %{}",
            in_bounds, ok_lbl, error_lbl
        ));

        self.emit(&format!("{}:", ok_lbl));
        let elem_ptr = self.fresh_temp();
        self.emit(&format!(
            "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
            elem_ptr, arr_ptr, idx
        ));
        let value = self.fresh_temp();
        self.emit(&format!("  {} = load i64, i64* {}", value, elem_ptr));
        self.emit(&format!("  store i64 {}, i64* {}", value, result_ptr));
        self.emit(&format!("  br label %{}", end_lbl));

        self.emit(&format!("{}:", error_lbl));
        self.record_runtime_error("인덱스 범위 초과");
        self.emit(&format!("  br label %{}", end_lbl));

        self.emit(&format!("{}:", end_lbl));
        let result = self.fresh_temp();
        self.emit(&format!("  {} = load i64, i64* {}", result, result_ptr));
        result
    }

    fn gen_checked_index_store(&mut self, object: &Expr, index: &Expr, value: &Expr) -> String {
        let arr_ptr = self.gen_expr(object);
        let idx = self.gen_expr(index);
        let val = self.gen_expr(value);
        let len = self.load_array_length(&arr_ptr);
        let result_ptr = self.fresh_temp();
        self.emit(&format!("  {} = alloca i64", result_ptr));
        self.emit(&format!("  store i64 {}, i64* {}", val, result_ptr));

        let is_non_negative = self.fresh_temp();
        self.emit(&format!("  {} = icmp sge i64 {}, 0", is_non_negative, idx));
        let is_before_end = self.fresh_temp();
        self.emit(&format!(
            "  {} = icmp slt i64 {}, {}",
            is_before_end, idx, len
        ));
        let in_bounds = self.fresh_temp();
        self.emit(&format!(
            "  {} = and i1 {}, {}",
            in_bounds, is_non_negative, is_before_end
        ));

        let block_id = self.fresh_label();
        let ok_lbl = format!("index_store_ok{}", block_id);
        let error_lbl = format!("index_store_error{}", block_id);
        let end_lbl = format!("index_store_end{}", block_id);

        self.emit(&format!(
            "  br i1 {}, label %{}, label %{}",
            in_bounds, ok_lbl, error_lbl
        ));

        self.emit(&format!("{}:", ok_lbl));
        let elem_ptr = self.fresh_temp();
        self.emit(&format!(
            "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
            elem_ptr, arr_ptr, idx
        ));
        self.emit(&format!("  store i64 {}, i64* {}", val, elem_ptr));
        self.emit(&format!("  br label %{}", end_lbl));

        self.emit(&format!("{}:", error_lbl));
        self.record_runtime_error("인덱스 범위 초과");
        self.emit(&format!("  br label %{}", end_lbl));

        self.emit(&format!("{}:", end_lbl));
        let result = self.fresh_temp();
        self.emit(&format!("  {} = load i64, i64* {}", result, result_ptr));
        result
    }

    fn gen_range_expr(&mut self, start: &Expr, end: &Expr) -> String {
        let start_val = self.gen_expr(start);
        let end_val = self.gen_expr(end);
        let raw_len = self.fresh_temp();
        self.emit(&format!(
            "  {} = sub nsw i64 {}, {}",
            raw_len, end_val, start_val
        ));

        let is_negative = self.fresh_temp();
        self.emit(&format!("  {} = icmp slt i64 {}, 0", is_negative, raw_len));
        let len = self.fresh_temp();
        self.emit(&format!(
            "  {} = select i1 {}, i64 0, i64 {}",
            len, is_negative, raw_len
        ));

        let data_ptr = self.allocate_array_storage(&len);
        let idx_ptr = self.fresh_temp();
        self.emit(&format!("  {} = alloca i64", idx_ptr));
        self.emit(&format!("  store i64 0, i64* {}", idx_ptr));

        let block_id = self.fresh_label();
        let cond_lbl = format!("range_cond{}", block_id);
        let body_lbl = format!("range_body{}", block_id);
        let end_lbl = format!("range_end{}", block_id);

        self.emit(&format!("  br label %{}", cond_lbl));
        self.emit(&format!("{}:", cond_lbl));
        let idx = self.fresh_temp();
        self.emit(&format!("  {} = load i64, i64* {}", idx, idx_ptr));
        let has_more = self.fresh_temp();
        self.emit(&format!("  {} = icmp slt i64 {}, {}", has_more, idx, len));
        self.emit(&format!(
            "  br i1 {}, label %{}, label %{}",
            has_more, body_lbl, end_lbl
        ));

        self.emit(&format!("{}:", body_lbl));
        let value = self.fresh_temp();
        self.emit(&format!("  {} = add nsw i64 {}, {}", value, start_val, idx));
        let elem_ptr = self.fresh_temp();
        self.emit(&format!(
            "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
            elem_ptr, data_ptr, idx
        ));
        self.emit(&format!("  store i64 {}, i64* {}", value, elem_ptr));
        let next_idx = self.fresh_temp();
        self.emit(&format!("  {} = add nsw i64 {}, 1", next_idx, idx));
        self.emit(&format!("  store i64 {}, i64* {}", next_idx, idx_ptr));
        self.emit(&format!("  br label %{}", cond_lbl));

        self.emit(&format!("{}:", end_lbl));
        data_ptr
    }

    fn gen_for_in_stmt(&mut self, var_name: &str, iterable: &Expr, body: &[Stmt]) {
        let data_ptr = self.gen_expr(iterable);
        let len = self.load_array_length(&data_ptr);
        let idx_ptr = self.fresh_temp();
        self.emit(&format!("  {} = alloca i64", idx_ptr));
        self.emit(&format!("  store i64 0, i64* {}", idx_ptr));

        let var_ptr_name = Self::var_ptr(var_name);
        if !self.var_types.contains_key(var_name) {
            self.emit(&format!("  {} = alloca i64", var_ptr_name));
        }
        self.var_types.insert(var_name.to_string(), "i64");

        let loop_id = self.fresh_label();
        let cond_lbl = format!("loop_cond{}", loop_id);
        let body_lbl = format!("loop_body{}", loop_id);
        let end_lbl = format!("loop_end{}", loop_id);

        self.emit(&format!("  br label %{}", cond_lbl));
        self.emit(&format!("{}:", cond_lbl));
        let idx = self.fresh_temp();
        self.emit(&format!("  {} = load i64, i64* {}", idx, idx_ptr));
        let has_more = self.fresh_temp();
        self.emit(&format!("  {} = icmp slt i64 {}, {}", has_more, idx, len));
        self.emit(&format!(
            "  br i1 {}, label %{}, label %{}",
            has_more, body_lbl, end_lbl
        ));

        self.emit(&format!("{}:", body_lbl));
        let elem_ptr = self.fresh_temp();
        self.emit(&format!(
            "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
            elem_ptr, data_ptr, idx
        ));
        let elem_val = self.fresh_temp();
        self.emit(&format!("  {} = load i64, i64* {}", elem_val, elem_ptr));
        self.emit(&format!("  store i64 {}, i64* {}", elem_val, var_ptr_name));

        self.loop_stack.push((cond_lbl.clone(), end_lbl.clone()));
        for stmt in body {
            self.gen_stmt(stmt);
        }
        self.loop_stack.pop();

        let next_idx = self.fresh_temp();
        self.emit(&format!("  {} = add nsw i64 {}, 1", next_idx, idx));
        self.emit(&format!("  store i64 {}, i64* {}", next_idx, idx_ptr));
        self.emit(&format!("  br label %{}", cond_lbl));
        self.emit(&format!("{}:", end_lbl));
    }

    fn gen_try_catch_stmt(&mut self, try_block: &[Stmt], error_name: &str, catch_block: &[Stmt]) {
        let try_id = self.fresh_label();
        let catch_lbl = format!("catch{}", try_id);
        let end_lbl = format!("try_end{}", try_id);

        self.clear_error_state();

        if try_block.is_empty() {
            self.emit(&format!("  br label %{}", end_lbl));
        } else {
            let first_lbl = format!("try_stmt{}_0", try_id);
            self.emit(&format!("  br label %{}", first_lbl));

            for (index, stmt) in try_block.iter().enumerate() {
                let stmt_lbl = format!("try_stmt{}_{}", try_id, index);
                let next_lbl = if index + 1 < try_block.len() {
                    format!("try_stmt{}_{}", try_id, index + 1)
                } else {
                    end_lbl.clone()
                };

                self.emit(&format!("{}:", stmt_lbl));
                self.gen_stmt(stmt);
                let error_flag = self.fresh_temp();
                self.emit(&format!("  {} = load i1, i1* %error_flag", error_flag));
                self.emit(&format!(
                    "  br i1 {}, label %{}, label %{}",
                    error_flag, catch_lbl, next_lbl
                ));
            }
        }

        self.emit(&format!("{}:", catch_lbl));
        let error_var = Self::var_ptr(error_name);
        if !self.var_types.contains_key(error_name) {
            self.emit(&format!("  {} = alloca i8*", error_var));
        }
        self.var_types.insert(error_name.to_string(), "i8*");
        let error_value = self.fresh_temp();
        self.emit(&format!(
            "  {} = load i8*, i8** %error_message",
            error_value
        ));
        self.emit(&format!("  store i8* {}, i8** {}", error_value, error_var));
        self.clear_error_state();
        for stmt in catch_block {
            self.gen_stmt(stmt);
        }
        self.emit(&format!("  br label %{}", end_lbl));

        self.emit(&format!("{}:", end_lbl));
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
                    "  {} = load {}, {}* {}",
                    t,
                    var_ty,
                    var_ty,
                    Self::var_ptr(name)
                ));
                t
            }
            Expr::Assign { name, value } => {
                let var_ty = self.var_types.get(name.as_str()).copied().unwrap_or("i64");
                let val = self.gen_expr(value);
                self.emit(&format!(
                    "  store {} {}, {}* {}",
                    var_ty,
                    val,
                    var_ty,
                    Self::var_ptr(name)
                ));
                val
            }
            Expr::BinaryOp { op, left, right } => {
                let lty = self.infer_type(left);

                // i8* + i8* → string concatenation via C runtime
                if matches!(op, BinaryOpKind::Add) && lty == "i8*" {
                    return self.gen_str_concat(left, right);
                }

                if lty != "double" && matches!(op, BinaryOpKind::Div | BinaryOpKind::Mod) {
                    return self.gen_checked_int_div_or_mod(op, left, right);
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
                self.emit(&format!(
                    "  {} = call {} @{}({})",
                    t,
                    ret_ty,
                    Self::sanitize_ident(name),
                    arg_str
                ));
                t
            }
            Expr::ArrayLiteral(elems) => {
                let len_value = elems.len().to_string();
                let data_ptr = self.allocate_array_storage(&len_value);

                for (index, elem) in elems.iter().enumerate() {
                    let val = self.gen_expr(elem);
                    let elem_ptr = self.fresh_temp();
                    self.emit(&format!(
                        "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
                        elem_ptr, data_ptr, index
                    ));
                    self.emit(&format!("  store i64 {}, i64* {}", val, elem_ptr));
                }

                data_ptr
            }

            Expr::Index { object, index } => self.gen_checked_index_load(object, index),

            Expr::IndexAssign {
                object,
                index,
                value,
            } => self.gen_checked_index_store(object, index, value),

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

            Expr::Range { start, end } => self.gen_range_expr(start, end),
            Expr::MethodCall { .. }
            | Expr::Lambda { .. }
            | Expr::TupleLiteral(_)
            | Expr::TupleIndex { .. }
            | Expr::MapLiteral(_) => {
                let t = self.fresh_temp();
                self.emit(&format!("  {} = add nsw i64 0, 0", t));
                t
            }
        }
    }

    fn resolve_enum_tag(&self, variant: &str) -> Option<usize> {
        for (_enum_name, variants) in &self.enum_defs {
            if let Some(pos) = variants.iter().position(|v| v == variant) {
                return Some(pos);
            }
        }
        None
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
                self.emit(&format!("  {} = alloca {}", Self::var_ptr(name), llvm_ty));
                let val = self.gen_expr(value);
                self.emit(&format!(
                    "  store {} {}, {}* {}",
                    llvm_ty,
                    val,
                    llvm_ty,
                    Self::var_ptr(name)
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
                error_name,
                catch_block,
            } => self.gen_try_catch_stmt(try_block, error_name, catch_block),
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
                        crate::ast::Pattern::Wildcard => {
                            self.emit(&format!("  br label %{}", arm_lbl));
                        }
                        crate::ast::Pattern::Identifier(variant_name) => {
                            let tag = self.resolve_enum_tag(variant_name);
                            if let Some(tag_val) = tag {
                                let tag_tmp = self.fresh_temp();
                                self.emit(&format!(
                                    "  {} = extractvalue {{ i64, i64 }} {}, 0",
                                    tag_tmp, val
                                ));
                                let cmp = self.fresh_temp();
                                self.emit(&format!(
                                    "  {} = icmp eq i64 {}, {}",
                                    cmp, tag_tmp, tag_val
                                ));
                                self.emit(&format!(
                                    "  br i1 {}, label %{}, label %{}",
                                    cmp, arm_lbl, next_lbl
                                ));
                            } else {
                                self.emit(&format!("  br label %{}", arm_lbl));
                            }
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
            StmtKind::EnumDef { name, variants } => {
                self.enum_defs.insert(name.clone(), variants.clone());
            }
            StmtKind::ForIn {
                var_name,
                iterable,
                body,
            } => self.gen_for_in_stmt(var_name, iterable, body),
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
            .map(|(pname, pty)| {
                format!("{} %{}", Self::llvm_type(pty), Self::sanitize_ident(pname))
            })
            .collect::<Vec<_>>()
            .join(", ");

        self.var_types.clear();
        self.emit(&format!(
            "define {} @{}({}) {{",
            ret_ty,
            Self::sanitize_ident(name),
            param_str
        ));
        self.emit("entry:");
        self.init_error_state();

        for (pname, pty) in params {
            let llvm_ty = Self::llvm_type(pty);
            self.var_types.insert(pname.clone(), llvm_ty);
            self.emit(&format!("  {} = alloca {}", Self::var_ptr(pname), llvm_ty));
            self.emit(&format!(
                "  store {} %{}, {}* {}",
                llvm_ty,
                Self::sanitize_ident(pname),
                llvm_ty,
                Self::var_ptr(pname)
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
        self.current_error_flag = None;
        self.current_error_message = None;
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
            self.init_error_state();
            for stmt in &top_level {
                self.gen_stmt(stmt);
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
        assert!(ir.contains("{ i64, i64 }"));
        assert!(ir.contains("extractvalue { i64, i64 }"));
        assert!(ir.contains("icmp eq i64"));
        assert!(ir.contains("match_arm"));
    }
}
