use super::CodeGen;
use crate::ast::{Expr, ExprVisitor};

impl CodeGen {
    pub(super) fn gen_print(&mut self, args: &[Expr]) -> String {
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
                let val = self.visit_expr(arg);
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
}
