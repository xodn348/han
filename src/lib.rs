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
        let warning_output = if type_errors.is_empty() {
            String::new()
        } else {
            let msgs: Vec<String> = type_errors
                .iter()
                .map(|e| {
                    if e.line > 0 {
                        format!("[타입 경고] {}번째 줄: {}", e.line, e.message)
                    } else {
                        format!("[타입 경고] {}", e.message)
                    }
                })
                .collect();
            format!("{}\n", msgs.join("\n"))
        };

        match interpreter::interpret(program) {
            Ok(_) => format!("{}{}", warning_output, interpreter::capture_flush()),
            Err(e) => {
                let output = interpreter::capture_flush();
                format!("{}{}[런타임 에러] {}", warning_output, output, e.message)
            }
        }
    }
}
