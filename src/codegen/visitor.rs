use super::{CodeGen, LambdaBinding, PendingLambdaBinding};
use crate::ast::{Expr, ExprVisitor, Stmt, StmtKind, Type};
use std::collections::HashSet;

impl CodeGen {
    pub(super) fn capture_binding_name(&mut self, owner_name: &str, capture_name: &str) -> String {
        format!(
            "__capture_{}_{}_{}",
            Self::sanitize_ident(owner_name),
            Self::sanitize_ident(capture_name),
            self.fresh_label()
        )
    }

    pub(super) fn push_captured_identifier(
        &self,
        name: &str,
        bound: &HashSet<String>,
        captures: &mut Vec<String>,
    ) {
        if bound.contains(name) || !self.var_types.contains_key(name) {
            return;
        }
        if captures.iter().any(|existing| existing == name) {
            return;
        }
        captures.push(name.to_string());
    }

    pub(super) fn collect_captures_in_expr(
        &self,
        expr: &Expr,
        bound: &HashSet<String>,
        captures: &mut Vec<String>,
    ) {
        match expr {
            Expr::Identifier(name) => self.push_captured_identifier(name, bound, captures),
            Expr::BinaryOp { left, right, .. } => {
                self.collect_captures_in_expr(left, bound, captures);
                self.collect_captures_in_expr(right, bound, captures);
            }
            Expr::UnaryOp { expr, .. } => self.collect_captures_in_expr(expr, bound, captures),
            Expr::Call { name, args } => {
                self.push_captured_identifier(name, bound, captures);
                for arg in args {
                    self.collect_captures_in_expr(arg, bound, captures);
                }
            }
            Expr::Assign { name, value } => {
                self.push_captured_identifier(name, bound, captures);
                self.collect_captures_in_expr(value, bound, captures);
            }
            Expr::ArrayLiteral(elems) | Expr::TupleLiteral(elems) => {
                for elem in elems {
                    self.collect_captures_in_expr(elem, bound, captures);
                }
            }
            Expr::Index { object, index } => {
                self.collect_captures_in_expr(object, bound, captures);
                self.collect_captures_in_expr(index, bound, captures);
            }
            Expr::IndexAssign {
                object,
                index,
                value,
            } => {
                self.collect_captures_in_expr(object, bound, captures);
                self.collect_captures_in_expr(index, bound, captures);
                self.collect_captures_in_expr(value, bound, captures);
            }
            Expr::MethodCall { object, args, .. } => {
                self.collect_captures_in_expr(object, bound, captures);
                for arg in args {
                    self.collect_captures_in_expr(arg, bound, captures);
                }
            }
            Expr::FieldAccess { object, .. } | Expr::TupleIndex { object, .. } => {
                self.collect_captures_in_expr(object, bound, captures);
            }
            Expr::StructLiteral { fields, .. } => {
                for (_, field_expr) in fields {
                    self.collect_captures_in_expr(field_expr, bound, captures);
                }
            }
            Expr::FieldAssign { object, value, .. } => {
                self.collect_captures_in_expr(object, bound, captures);
                self.collect_captures_in_expr(value, bound, captures);
            }
            Expr::Range { start, end } => {
                self.collect_captures_in_expr(start, bound, captures);
                self.collect_captures_in_expr(end, bound, captures);
            }
            Expr::MapLiteral(entries) => {
                for (key, value) in entries {
                    self.collect_captures_in_expr(key, bound, captures);
                    self.collect_captures_in_expr(value, bound, captures);
                }
            }
            Expr::Lambda { .. }
            | Expr::IntLiteral(_)
            | Expr::FloatLiteral(_)
            | Expr::StringLiteral(_)
            | Expr::BoolLiteral(_)
            | Expr::NullLiteral => {}
        }
    }

    pub(super) fn collect_captures_in_stmt(
        &self,
        stmt: &Stmt,
        bound: &mut HashSet<String>,
        captures: &mut Vec<String>,
    ) {
        match &stmt.kind {
            StmtKind::VarDecl { name, value, .. } => {
                self.collect_captures_in_expr(value, bound, captures);
                bound.insert(name.clone());
            }
            StmtKind::FuncDef { name, .. } | StmtKind::StructDef { name, .. } => {
                bound.insert(name.clone());
            }
            StmtKind::Return(Some(expr)) | StmtKind::ExprStmt(expr) => {
                self.collect_captures_in_expr(expr, bound, captures);
            }
            StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => {}
            StmtKind::If {
                cond,
                then_block,
                else_block,
            } => {
                self.collect_captures_in_expr(cond, bound, captures);
                let mut then_bound = bound.clone();
                for stmt in then_block {
                    self.collect_captures_in_stmt(stmt, &mut then_bound, captures);
                }
                if let Some(else_block) = else_block {
                    let mut else_bound = bound.clone();
                    for stmt in else_block {
                        self.collect_captures_in_stmt(stmt, &mut else_bound, captures);
                    }
                }
            }
            StmtKind::ForLoop {
                init,
                cond,
                step,
                body,
            } => {
                let mut loop_bound = bound.clone();
                self.collect_captures_in_stmt(init, &mut loop_bound, captures);
                self.collect_captures_in_expr(cond, &loop_bound, captures);
                for stmt in body {
                    self.collect_captures_in_stmt(stmt, &mut loop_bound, captures);
                }
                self.collect_captures_in_stmt(step, &mut loop_bound, captures);
            }
            StmtKind::WhileLoop { cond, body } => {
                self.collect_captures_in_expr(cond, bound, captures);
                let mut loop_bound = bound.clone();
                for stmt in body {
                    self.collect_captures_in_stmt(stmt, &mut loop_bound, captures);
                }
            }
            StmtKind::TryCatch {
                try_block,
                error_name,
                catch_block,
            } => {
                let mut try_bound = bound.clone();
                for stmt in try_block {
                    self.collect_captures_in_stmt(stmt, &mut try_bound, captures);
                }
                let mut catch_bound = bound.clone();
                catch_bound.insert(error_name.clone());
                for stmt in catch_block {
                    self.collect_captures_in_stmt(stmt, &mut catch_bound, captures);
                }
            }
            StmtKind::Import(_) | StmtKind::EnumDef { .. } | StmtKind::ImplBlock { .. } => {}
            StmtKind::ForIn {
                var_name,
                iterable,
                body,
            } => {
                self.collect_captures_in_expr(iterable, bound, captures);
                let mut loop_bound = bound.clone();
                loop_bound.insert(var_name.clone());
                for stmt in body {
                    self.collect_captures_in_stmt(stmt, &mut loop_bound, captures);
                }
            }
            StmtKind::Match { expr, arms } => {
                self.collect_captures_in_expr(expr, bound, captures);
                for arm in arms {
                    let mut arm_bound = bound.clone();
                    if let crate::ast::Pattern::Identifier(name) = &arm.pattern {
                        arm_bound.insert(name.clone());
                    }
                    for stmt in &arm.body {
                        self.collect_captures_in_stmt(stmt, &mut arm_bound, captures);
                    }
                }
            }
        }
    }

    pub(super) fn find_captured_vars(
        &self,
        params: &[(String, Option<Type>)],
        body: &[Stmt],
    ) -> Vec<(String, &'static str)> {
        let mut bound: HashSet<String> = params.iter().map(|(name, _)| name.clone()).collect();
        let mut captures = Vec::new();
        for stmt in body {
            self.collect_captures_in_stmt(stmt, &mut bound, &mut captures);
        }
        captures
            .into_iter()
            .filter_map(|name| {
                self.var_types
                    .get(name.as_str())
                    .copied()
                    .map(|ty| (name, ty))
            })
            .collect()
    }

    pub(super) fn describe_lambda_binding(
        &self,
        params: &[(String, Option<Type>)],
        body: &[Stmt],
    ) -> PendingLambdaBinding {
        let captured_values = self.find_captured_vars(params, body);
        let mut full_param_types: Vec<&'static str> = params
            .iter()
            .map(|(_, ty)| Self::llvm_type(ty.as_ref().unwrap_or(&Type::정수)))
            .collect();
        for (_, capture_ty) in &captured_values {
            full_param_types.push(*capture_ty);
        }
        PendingLambdaBinding {
            full_param_types,
            captured_values,
        }
    }

    pub(super) fn materialize_lambda_binding(
        &mut self,
        owner_name: &str,
        binding: PendingLambdaBinding,
    ) -> LambdaBinding {
        let mut captured_bindings = Vec::new();
        for (capture_name, capture_ty) in binding.captured_values {
            let binding_name = self.capture_binding_name(owner_name, &capture_name);
            self.var_types.insert(binding_name.clone(), capture_ty);
            self.emit(&format!(
                "  {} = alloca {}",
                Self::var_ptr(&binding_name),
                capture_ty
            ));
            let capture_value = self.visit_expr(&Expr::Identifier(capture_name));
            self.emit(&format!(
                "  store {} {}, {}* {}",
                capture_ty,
                capture_value,
                capture_ty,
                Self::var_ptr(&binding_name)
            ));
            captured_bindings.push((binding_name, capture_ty));
        }
        LambdaBinding {
            full_param_types: binding.full_param_types,
            captured_bindings,
        }
    }
}
