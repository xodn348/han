use super::CodeGen;
use crate::ast::{Expr, ExprVisitor, Stmt, StmtKind, StmtVisitor, Type};

impl StmtVisitor<()> for CodeGen {
    fn visit_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::VarDecl {
                name, ty, value, ..
            } => {
                let llvm_ty = ty
                    .as_ref()
                    .map(|t| Self::llvm_type(t))
                    .unwrap_or_else(|| self.infer_type(value));
                let pending_binding = match value {
                    Expr::Lambda { params, body } => {
                        Some(self.describe_lambda_binding(params, body))
                    }
                    _ => None,
                };
                let val = self.visit_expr(value);
                if let Some(ty) = ty.as_ref() {
                    self.bind_type(name, ty);
                } else {
                    self.bind_llvm_type(name, llvm_ty);
                }
                self.emit(&format!("  {} = alloca {}", Self::var_ptr(name), llvm_ty));
                match value {
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
                    llvm_ty,
                    val,
                    llvm_ty,
                    Self::var_ptr(name)
                ));
            }
            StmtKind::ExprStmt(expr) => {
                self.visit_expr(expr);
            }
            StmtKind::Return(maybe_expr) => {
                if let Some(expr) = maybe_expr {
                    let ty = self.infer_type(expr);
                    let val = self.visit_expr(expr);
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

                let cond_val = self.visit_expr(cond);
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
                    self.visit_stmt(s);
                }
                self.emit(&format!("  br label %{}", end_lbl));

                if let Some(else_stmts) = else_block {
                    self.emit(&format!("{}:", else_lbl));
                    for s in else_stmts {
                        self.visit_stmt(s);
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

                let cond_val = self.visit_expr(cond);
                self.emit(&format!(
                    "  br i1 {}, label %{}, label %{}",
                    cond_val, body_lbl, end_lbl
                ));

                self.emit(&format!("{}:", body_lbl));
                self.loop_stack.push((cond_lbl.clone(), end_lbl.clone()));
                for s in body {
                    self.visit_stmt(s);
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

                self.visit_stmt(init);
                self.emit(&format!("  br label %{}", cond_lbl));

                self.emit(&format!("{}:", cond_lbl));
                let cond_val = self.visit_expr(cond);
                self.emit(&format!(
                    "  br i1 {}, label %{}, label %{}",
                    cond_val, body_lbl, end_lbl
                ));

                self.emit(&format!("{}:", body_lbl));
                self.loop_stack.push((cond_lbl.clone(), end_lbl.clone()));
                for s in body {
                    self.visit_stmt(s);
                }
                self.loop_stack.pop();

                self.visit_stmt(step);
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
            StmtKind::Import(path) => {
                let resolved_path = std::fs::canonicalize(path)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.clone());

                if self.imported_paths.contains(&resolved_path) {
                    return;
                }
                self.imported_paths.insert(resolved_path.clone());

                if let Ok(source) = std::fs::read_to_string(&resolved_path) {
                    let tokens = crate::lexer::tokenize(&source);
                    if let Ok(program) = crate::parser::parse(tokens) {
                        for stmt in &program.stmts {
                            match &stmt.kind {
                                StmtKind::FuncDef { .. }
                                | StmtKind::StructDef { .. }
                                | StmtKind::EnumDef { .. } => {
                                    self.visit_stmt(stmt);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            StmtKind::Match { expr, arms } => {
                let val = self.visit_expr(expr);
                let end_lbl = format!("match_end{}", self.fresh_label());

                for (i, arm) in arms.iter().enumerate() {
                    let arm_lbl = format!("match_arm{}_{}", self.label_count, i);
                    let next_lbl = if i + 1 < arms.len() {
                        format!("match_test{}_{}", self.label_count, i + 1)
                    } else {
                        end_lbl.clone()
                    };
                    let mut needs_next_label = false;

                    match &arm.pattern {
                        crate::ast::Pattern::Wildcard => {
                            self.emit(&format!("  br label %{}", arm_lbl));
                        }
                        crate::ast::Pattern::Identifier(variant_name) => {
                            let tag = self.resolve_enum_tag(variant_name);
                            if let Some(tag_val) = tag {
                                needs_next_label = true;
                                let cmp = self.fresh_temp();
                                self.emit(&format!("  {} = icmp eq i64 {}, {}", cmp, val, tag_val));
                                self.emit(&format!(
                                    "  br i1 {}, label %{}, label %{}",
                                    cmp, arm_lbl, next_lbl
                                ));
                            } else {
                                self.emit(&format!("  br label %{}", arm_lbl));
                            }
                        }
                        crate::ast::Pattern::IntLiteral(n) => {
                            needs_next_label = true;
                            let cmp = self.fresh_temp();
                            self.emit(&format!("  {} = icmp eq i64 {}, {}", cmp, val, n));
                            self.emit(&format!(
                                "  br i1 {}, label %{}, label %{}",
                                cmp, arm_lbl, next_lbl
                            ));
                        }
                        crate::ast::Pattern::BoolLiteral(b) => {
                            needs_next_label = true;
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
                        self.visit_stmt(s);
                    }
                    self.emit(&format!("  br label %{}", end_lbl));

                    if i + 1 < arms.len() && needs_next_label {
                        self.emit(&format!("{}:", next_lbl));
                    }
                }

                self.emit(&format!("{}:", end_lbl));
            }
            StmtKind::ImplBlock {
                struct_name,
                methods,
            } => {
                for method_stmt in methods {
                    if let StmtKind::FuncDef {
                        name,
                        params,
                        return_type,
                        body,
                    } = &method_stmt.kind
                    {
                        let method_name = format!(
                            "{}__{}",
                            Self::sanitize_ident(struct_name),
                            Self::sanitize_ident(name)
                        );
                        self.gen_func(&method_name, params, return_type, body);
                    }
                }
            }
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
}

impl CodeGen {
    pub(super) fn gen_func(
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
        self.struct_var_types.clear();
        self.lambda_bindings.clear();
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
            self.bind_type(pname, pty);
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
            self.visit_stmt(stmt);
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
}
