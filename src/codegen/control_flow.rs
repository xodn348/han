use super::CodeGen;
use crate::ast::{Expr, ExprVisitor, Stmt, StmtVisitor};

impl CodeGen {
    pub(super) fn gen_range_expr(&mut self, start: &Expr, end: &Expr) -> String {
        let start_val = self.visit_expr(start);
        let end_val = self.visit_expr(end);
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

    pub(super) fn gen_for_in_stmt(&mut self, var_name: &str, iterable: &Expr, body: &[Stmt]) {
        let data_ptr = self.visit_expr(iterable);
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
            self.visit_stmt(stmt);
        }
        self.loop_stack.pop();

        let next_idx = self.fresh_temp();
        self.emit(&format!("  {} = add nsw i64 {}, 1", next_idx, idx));
        self.emit(&format!("  store i64 {}, i64* {}", next_idx, idx_ptr));
        self.emit(&format!("  br label %{}", cond_lbl));
        self.emit(&format!("{}:", end_lbl));
    }

    pub(super) fn gen_try_catch_stmt(
        &mut self,
        try_block: &[Stmt],
        error_name: &str,
        catch_block: &[Stmt],
    ) {
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
                self.visit_stmt(stmt);
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
            self.visit_stmt(stmt);
        }
        self.emit(&format!("  br label %{}", end_lbl));

        self.emit(&format!("{}:", end_lbl));
    }
}
