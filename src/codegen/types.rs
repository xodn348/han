use super::CodeGen;
use crate::ast::{BinaryOpKind, Expr, Type, UnaryOpKind};

impl CodeGen {
    pub(super) fn llvm_type(ty: &Type) -> &'static str {
        match ty {
            Type::정수 => "i64",
            Type::실수 => "double",
            Type::문자열 => "i8*",
            Type::불 => "i1",
            Type::없음 => "void",
            Type::배열(_) => "i64*",
            Type::구조체(_) => "i64*",
            Type::함수타입 => "i8*",
            Type::튜플(_) => "i64*",
        }
    }

    pub(super) fn bind_type(&mut self, name: &str, ty: &Type) {
        self.var_types.insert(name.to_string(), Self::llvm_type(ty));
        match ty {
            Type::구조체(struct_name) => {
                self.struct_var_types
                    .insert(name.to_string(), struct_name.clone());
            }
            _ => {
                self.struct_var_types.remove(name);
            }
        }
    }

    pub(super) fn bind_llvm_type(&mut self, name: &str, llvm_ty: &'static str) {
        self.var_types.insert(name.to_string(), llvm_ty);
        self.struct_var_types.remove(name);
    }

    pub(super) fn struct_name_for_expr(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Identifier(name) => self.struct_var_types.get(name.as_str()).cloned(),
            Expr::StructLiteral { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    pub(super) fn infer_type(&self, expr: &Expr) -> &'static str {
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
            Expr::StructLiteral { .. } => "i64*",
            Expr::TupleLiteral(_) => "i64*",
            Expr::Range { .. } => "i64*",
            Expr::Index { .. } | Expr::TupleIndex { .. } => "i64",
            _ => "i64",
        }
    }

    pub(super) fn guess_struct_type(&self, expr: &Expr) -> String {
        self.struct_name_for_expr(expr)
            .unwrap_or_else(|| "unknown".to_string())
    }

    pub(super) fn resolve_enum_tag(&self, variant: &str) -> Option<usize> {
        for variants in self.enum_defs.values() {
            if let Some(pos) = variants.iter().position(|v| v == variant) {
                return Some(pos);
            }
        }
        None
    }

    pub(super) fn find_field_index(&self, object: &Expr, field: &str) -> usize {
        if let Some(struct_name) = self.struct_name_for_expr(object)
            && let Some(fields) = self.struct_defs.get(&struct_name)
        {
            return fields.iter().position(|f| f == field).unwrap_or(0);
        }
        0
    }
}
