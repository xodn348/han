#![allow(dead_code, unused)]
mod ast;
mod codegen;
mod interpreter;
mod lexer;
mod parser;

use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::{self, Command};

#[derive(Parser)]
#[command(name = "hgl", about = "Han 프로그래밍 언어 컴파일러")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build { file: String },
    Run { file: String },
    Interpret { file: String },
}

fn run_pipeline(source: &str) -> ast::Program {
    let tokens = lexer::tokenize(source);
    match parser::parse(tokens) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("[파서 에러] {}번째 줄: {}", e.line, e.message);
            process::exit(1);
        }
    }
}

fn output_binary_name(file_path: &str) -> String {
    let stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    format!("./{}", stem)
}

fn compile_to_binary(source: &str, output_path: &str) -> Result<(), String> {
    let program = run_pipeline(source);
    let ir = codegen::codegen(&program);

    fs::write("/tmp/han_build.ll", &ir).map_err(|e| format!("임시 파일 쓰기 실패: {}", e))?;

    let clang_result = Command::new("clang")
        .args(["/tmp/han_build.ll", "-o", output_path, "-lm"])
        .status();

    let _ = fs::remove_file("/tmp/han_build.ll");

    match clang_result {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err("clang이 필요합니다. brew install llvm 또는 xcode-select --install".to_string())
        }
        Err(e) => Err(format!("clang 실행 실패: {}", e)),
        Ok(status) if !status.success() => Err(format!(
            "컴파일 실패 (clang exit code: {})",
            status.code().unwrap_or(-1)
        )),
        Ok(_) => Ok(()),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { file } => {
            let source = fs::read_to_string(&file).unwrap_or_else(|e| {
                eprintln!("파일 읽기 실패 '{}': {}", file, e);
                process::exit(1);
            });

            let output = output_binary_name(&file);

            match compile_to_binary(&source, &output) {
                Ok(()) => println!("빌드 완료: {}", output),
                Err(e) => {
                    eprintln!("[빌드 에러] {}", e);
                    process::exit(1);
                }
            }
        }

        Commands::Run { file } => {
            let source = fs::read_to_string(&file).unwrap_or_else(|e| {
                eprintln!("파일 읽기 실패 '{}': {}", file, e);
                process::exit(1);
            });

            let output = output_binary_name(&file);

            match compile_to_binary(&source, &output) {
                Ok(()) => {
                    let run_status = Command::new(&output).status().unwrap_or_else(|e| {
                        eprintln!("실행 실패 '{}': {}", output, e);
                        process::exit(1);
                    });

                    let _ = fs::remove_file(&output);
                    process::exit(run_status.code().unwrap_or(0));
                }
                Err(e) => {
                    eprintln!("[빌드 에러] {}", e);
                    process::exit(1);
                }
            }
        }

        Commands::Interpret { file } => {
            let source = fs::read_to_string(&file).unwrap_or_else(|e| {
                eprintln!("파일 읽기 실패 '{}': {}", file, e);
                process::exit(1);
            });

            let program = run_pipeline(&source);

            match interpreter::interpret(program) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("[런타임 에러] {}", e.message);
                    process::exit(1);
                }
            }
        }
    }
}
