pub mod ast;
pub mod codegen;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod typechecker;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn run_han(code: &str) -> String {
        interpreter::capture_start();

        let tokens = lexer::tokenize(code);
        let program = match parser::parse(tokens) {
            Ok(p) => p,
            Err(e) => {
                return format!("[파서 에러] {}번째 줄: {}", e.line, e.message);
            }
        };

        let type_errors = typechecker::check(&program);
        if !type_errors.is_empty() {
            let msgs: Vec<String> = type_errors
                .iter()
                .map(|e| format!("[타입 에러] {}번째 줄: {}", e.line, e.message))
                .collect();
            return msgs.join("\n");
        }

        match interpreter::interpret(program) {
            Ok(_) => interpreter::capture_flush(),
            Err(e) => {
                let output = interpreter::capture_flush();
                format!("{}[런타임 에러] {}", output, e.message)
            }
        }
    }
}
