use super::CodeGen;

impl CodeGen {
    pub(super) fn sanitize_ident(name: &str) -> String {
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

    pub(super) fn var_ptr(name: &str) -> String {
        format!("%var_{}", Self::sanitize_ident(name))
    }

    pub(super) fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    #[allow(dead_code)]
    pub(super) fn enter_block(&mut self) {
        self.indent_level += 1;
    }

    #[allow(dead_code)]
    pub(super) fn leave_block(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub(super) fn fresh_temp(&mut self) -> String {
        let t = format!("%t{}", self.temp_count);
        self.temp_count += 1;
        t
    }

    pub(super) fn fresh_label(&mut self) -> usize {
        let l = self.label_count;
        self.label_count += 1;
        l
    }

    pub(super) fn intern_string(&mut self, s: &str) -> (String, usize) {
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

    pub(super) fn gen_string_ptr_literal(&mut self, s: &str) -> String {
        let (name, len) = self.intern_string(s);
        let ptr = self.fresh_temp();
        self.emit(&format!(
            "  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i32 0, i32 0",
            ptr, len, len, name
        ));
        ptr
    }

    pub(super) fn init_error_state(&mut self) {
        self.current_error_flag = Some("%error_flag".to_string());
        self.current_error_message = Some("%error_message".to_string());
        self.emit("  %error_flag = alloca i1");
        self.emit("  store i1 0, i1* %error_flag");
        self.emit("  %error_message = alloca i8*");
        self.emit("  store i8* null, i8** %error_message");
    }

    pub(super) fn clear_error_state(&mut self) {
        if let Some(flag_ptr) = self.current_error_flag.clone() {
            self.emit(&format!("  store i1 0, i1* {}", flag_ptr));
        }
        if let Some(message_ptr) = self.current_error_message.clone() {
            self.emit(&format!("  store i8* null, i8** {}", message_ptr));
        }
    }

    pub(super) fn record_runtime_error(&mut self, message: &str) {
        if let Some(flag_ptr) = self.current_error_flag.clone() {
            self.emit(&format!("  store i1 1, i1* {}", flag_ptr));
        }
        if let Some(message_ptr) = self.current_error_message.clone() {
            let ptr = self.gen_string_ptr_literal(message);
            self.emit(&format!("  store i8* {}, i8** {}", ptr, message_ptr));
        }
    }

    pub(super) fn allocate_array_storage(&mut self, len: &str) -> String {
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

    pub(super) fn load_array_length(&mut self, data_ptr: &str) -> String {
        let len_ptr = self.fresh_temp();
        self.emit(&format!(
            "  {} = getelementptr inbounds i64, i64* {}, i64 -1",
            len_ptr, data_ptr
        ));

        let len = self.fresh_temp();
        self.emit(&format!("  {} = load i64, i64* {}", len, len_ptr));
        len
    }
}
