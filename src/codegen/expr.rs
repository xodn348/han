use super::CodeGen;
use crate::ast::{BinaryOpKind, Expr, ExprVisitor, UnaryOpKind};

impl ExprVisitor<String> for CodeGen {
    fn visit_expr(&mut self, expr: &Expr) -> String {
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
                let pending_binding = match value.as_ref() {
                    Expr::Lambda { params, body } => {
                        Some(self.describe_lambda_binding(params, body))
                    }
                    _ => None,
                };
                let val = self.visit_expr(value);
                match value.as_ref() {
                    Expr::Lambda { .. } => {
                        if let Some(binding) = pending_binding {
                            let binding = self.materialize_lambda_binding(name, binding);
                            self.lambda_bindings.insert(name.clone(), binding);
                        }
                    }
                    Expr::Identifier(source) => {
                        if let Some(binding) = self.lambda_bindings.get(source.as_str()).cloned() {
                            self.lambda_bindings.insert(name.clone(), binding);
                        } else {
                            self.lambda_bindings.remove(name.as_str());
                        }
                    }
                    _ => {
                        self.lambda_bindings.remove(name.as_str());
                    }
                }
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

                let lv = self.visit_expr(left);
                let rv = self.visit_expr(right);
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
                let v = self.visit_expr(expr);
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
                let mut arg_types: Vec<&str> = args.iter().map(|a| self.infer_type(a)).collect();
                let mut arg_vals: Vec<String> = args.iter().map(|a| self.visit_expr(a)).collect();
                let arg_str = |types: &[&str], values: &[String]| {
                    types
                        .iter()
                        .zip(values.iter())
                        .map(|(ty, value)| format!("{} {}", ty, value))
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                let t = self.fresh_temp();
                if self.var_types.contains_key(name.as_str()) {
                    let binding = self.lambda_bindings.get(name.as_str()).cloned();
                    if let Some(binding) = binding.as_ref() {
                        for (binding_name, binding_ty) in &binding.captured_bindings {
                            arg_types.push(*binding_ty);
                            arg_vals.push(self.visit_expr(&Expr::Identifier(binding_name.clone())));
                        }
                    }

                    let fn_ptr_i64 = self.fresh_temp();
                    self.emit(&format!(
                        "  {} = load i64, i64* {}",
                        fn_ptr_i64,
                        Self::var_ptr(name)
                    ));
                    let fn_ptr = self.fresh_temp();
                    let param_types_str = binding
                        .as_ref()
                        .map(|binding| binding.full_param_types.join(", "))
                        .unwrap_or_else(|| arg_types.join(", "));
                    self.emit(&format!(
                        "  {} = inttoptr i64 {} to i64 ({})*",
                        fn_ptr, fn_ptr_i64, param_types_str
                    ));
                    self.emit(&format!(
                        "  {} = call i64 {}({})",
                        t,
                        fn_ptr,
                        arg_str(&arg_types, &arg_vals)
                    ));
                } else {
                    self.emit(&format!(
                        "  {} = call i64 @{}({})",
                        t,
                        Self::sanitize_ident(name),
                        arg_str(&arg_types, &arg_vals)
                    ));
                }
                t
            }
            Expr::ArrayLiteral(elems) => {
                let len_value = elems.len().to_string();
                let data_ptr = self.allocate_array_storage(&len_value);

                for (index, elem) in elems.iter().enumerate() {
                    let val = self.visit_expr(elem);
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
                    let val = self.visit_expr(fexpr);
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
                let obj_ptr = self.visit_expr(object);
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
                let obj_ptr = self.visit_expr(object);
                let idx = self.find_field_index(object, field);
                let val = self.visit_expr(value);
                let fptr = self.fresh_temp();
                self.emit(&format!(
                    "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
                    fptr, obj_ptr, idx
                ));
                self.emit(&format!("  store i64 {}, i64* {}", val, fptr));
                val
            }

            Expr::Range { start, end } => self.gen_range_expr(start, end),
            Expr::Lambda { params, body, .. } => {
                let lambda_name = format!("__lambda_{}", self.fresh_label());
                let saved_output = std::mem::take(&mut self.output);
                let saved_vars = self.var_types.clone();
                let saved_struct_vars = self.struct_var_types.clone();
                let saved_lambda_bindings = self.lambda_bindings.clone();
                let saved_error_flag = self.current_error_flag.clone();
                let saved_error_msg = self.current_error_message.clone();
                let binding = self.describe_lambda_binding(params, body);

                let mut typed_params: Vec<(String, crate::ast::Type)> = params
                    .iter()
                    .map(|(name, ty)| (name.clone(), ty.clone().unwrap_or(crate::ast::Type::정수)))
                    .collect();
                for (capture_name, capture_ty) in &binding.captured_values {
                    typed_params.push((
                        capture_name.clone(),
                        match *capture_ty {
                            "double" => crate::ast::Type::실수,
                            "i8*" => crate::ast::Type::문자열,
                            "i1" => crate::ast::Type::불,
                            _ => crate::ast::Type::정수,
                        },
                    ));
                }
                self.gen_func(
                    &lambda_name,
                    &typed_params,
                    &Some(crate::ast::Type::정수),
                    body,
                );

                let lambda_output = std::mem::replace(&mut self.output, saved_output);
                self.var_types = saved_vars;
                self.struct_var_types = saved_struct_vars;
                self.lambda_bindings = saved_lambda_bindings;
                self.current_error_flag = saved_error_flag;
                self.current_error_message = saved_error_msg;
                self.globals.push_str(&lambda_output);

                let t = self.fresh_temp();
                self.emit(&format!(
                    "  {} = ptrtoint i64 ({})* @{} to i64",
                    t,
                    binding.full_param_types.join(", "),
                    lambda_name
                ));
                t
            }
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let obj_val = self.visit_expr(object);
                let obj_ty = self.infer_type(object);
                let t = self.fresh_temp();

                match method.as_str() {
                    "길이" => {
                        if obj_ty == "i8*" {
                            self.emit(&format!("  {} = call i64 @strlen(i8* {})", t, obj_val));
                        } else {
                            let len_ptr = self.fresh_temp();
                            self.emit(&format!(
                                "  {} = getelementptr inbounds i64, i64* {}, i64 -1",
                                len_ptr, obj_val
                            ));
                            self.emit(&format!("  {} = load i64, i64* {}", t, len_ptr));
                        }
                    }
                    _ => {
                        let arg_vals: Vec<String> =
                            args.iter().map(|a| self.visit_expr(a)).collect();
                        let mut all_args = vec![format!("{} {}", obj_ty, obj_val)];
                        for av in &arg_vals {
                            all_args.push(format!("i64 {}", av));
                        }
                        self.emit(&format!(
                            "  {} = call i64 @{}__{}({})",
                            t,
                            Self::sanitize_ident(&self.guess_struct_type(object)),
                            Self::sanitize_ident(method),
                            all_args.join(", ")
                        ));
                    }
                }
                t
            }
            Expr::TupleLiteral(elems) => {
                let byte_size = elems.len() * 8;
                let mem = self.fresh_temp();
                self.emit(&format!("  {} = call i8* @malloc(i64 {})", mem, byte_size));
                let data_ptr = self.fresh_temp();
                self.emit(&format!("  {} = bitcast i8* {} to i64*", data_ptr, mem));

                for (index, elem) in elems.iter().enumerate() {
                    let val = self.visit_expr(elem);
                    let elem_ptr = self.fresh_temp();
                    self.emit(&format!(
                        "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
                        elem_ptr, data_ptr, index
                    ));
                    self.emit(&format!("  store i64 {}, i64* {}", val, elem_ptr));
                }

                data_ptr
            }
            Expr::TupleIndex { object, index } => {
                let obj_ptr = self.visit_expr(object);
                let elem_ptr = self.fresh_temp();
                self.emit(&format!(
                    "  {} = getelementptr inbounds i64, i64* {}, i64 {}",
                    elem_ptr, obj_ptr, index
                ));
                let val = self.fresh_temp();
                self.emit(&format!("  {} = load i64, i64* {}", val, elem_ptr));
                val
            }
            Expr::MapLiteral(_) => {
                let t = self.fresh_temp();
                self.emit(&format!(
                    "  {} = add nsw i64 0, 0 ; Map not supported in compiled mode",
                    t
                ));
                t
            }
        }
    }
}

impl CodeGen {
    pub(super) fn gen_str_concat(&mut self, left: &Expr, right: &Expr) -> String {
        let lv = self.visit_expr(left);
        let rv = self.visit_expr(right);

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

    pub(super) fn gen_checked_int_div_or_mod(
        &mut self,
        op: &BinaryOpKind,
        left: &Expr,
        right: &Expr,
    ) -> String {
        let lv = self.visit_expr(left);
        let rv = self.visit_expr(right);
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

    pub(super) fn gen_checked_index_load(&mut self, object: &Expr, index: &Expr) -> String {
        let arr_ptr = self.visit_expr(object);
        let idx = self.visit_expr(index);
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

    pub(super) fn gen_checked_index_store(
        &mut self,
        object: &Expr,
        index: &Expr,
        value: &Expr,
    ) -> String {
        let arr_ptr = self.visit_expr(object);
        let idx = self.visit_expr(index);
        let val = self.visit_expr(value);
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
}
