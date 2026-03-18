mod ast;
mod builtins;
mod codegen;
mod interpreter;
mod lexer;
mod lsp;
mod parser;
mod typechecker;

use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, BufRead, Write};
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
    Check { file: String },
    Init { name: Option<String> },
    Repl,
    Lsp,
}

fn format_error(source: &str, line: usize, category: &str, message: &str) {
    let lines: Vec<&str> = source.lines().collect();
    eprintln!();
    eprintln!("── {} ──────────────────────────────", category);
    eprintln!();
    if line > 0 && line <= lines.len() {
        let src_line = lines[line - 1];
        eprintln!("{:>4} │   {}", line, src_line);
        eprintln!("     │");
    }
    eprintln!("{}", message);
    eprintln!();
}

fn run_pipeline(source: &str) -> ast::Program {
    let tokens = lexer::tokenize(source);
    match parser::parse(tokens) {
        Ok(program) => {
            let type_errors = typechecker::check(&program);
            for err in &type_errors {
                format_error(source, err.line, "타입 경고", &err.message);
            }
            program
        }
        Err(e) => {
            format_error(source, e.line, "문법 오류", &e.message);
            process::exit(1);
        }
    }
}

fn try_parse(source: &str) -> Result<ast::Program, parser::ParseError> {
    let tokens = lexer::tokenize(source);
    parser::parse(tokens)
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

fn print_banner() {
    let b = "\x1b[1m";
    let d = "\x1b[2m";
    let rs = "\x1b[0m";
    let red = "\x1b[91m";
    let blu = "\x1b[94m";

    if std::env::var("HAN_SIMPLE_BANNER").is_ok() {
        let mg = "\x1b[38;2;206;80;120m";
        let c = "\x1b[36m";
        println!();
        println!(
            "  {}◓{}◒{}{} ─────────────────────────────────── {}◓{}◒{}",
            red, blu, rs, d, red, blu, rs
        );
        println!();
        println!(
            "    {}✿{}  {}{}Han (한) Programming Language{}",
            mg, rs, c, b, rs
        );
        println!("    {}v0.1.0 · github.com/xodn348/han{}", d, rs);
        println!();
        println!(
            "  {}◓{}◒{}{} ─────────────────────────────────── {}◓{}◒{}",
            red, blu, rs, d, red, blu, rs
        );
        println!();
        return;
    }

    let mg = "\x1b[38;2;206;80;120m";
    let w = "\x1b[97m";

    let flower = [
        "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
        "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
        "⠀⠀⠀⠀⠀⠀⠀⠀⠀⣦⣀⣴⠀⠀⠀⠀⠀⠀⠀⠀",
        "⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀",
        "⠀⠀⠀⠀⢿⣶⣤⣀⠀⢻⣿⡟⠀⣠⣴⣾⡟⠀⠀⠀",
        "⠀⠀⠀⠲⢿⣿⣿⣿⣷⡄⠋⢰⣿⣿⣿⣿⠿⠆⠀⠀",
        "⠀⠀⠀⠀⠀⠀⠀⣀⣤⣶⠀⣶⣤⡀⠀⠀⠀⠀⠀⠀",
        "⠀⠀⠀⠀⠀⢠⣾⣿⣿⡏⠀⢻⣿⣿⣦⡀⠀⠀⠀⠀",
        "⠀⠀⠀⠀⠀⠛⠻⣿⠟⠀⠀⠈⢻⣿⠛⠛⠀⠀⠀⠀",
        "⠀⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀⠉⠀⠀⠀⠀⠀⠀",
        "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
        "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    ];

    let han = [
        "⠀⠀⠀⣀⣀⣀⣀⣀⣀⡀⠀⠀⠀⠀⠀⢰⣶⡆",
        "⠀⠀⠀⠛⠛⠛⠛⠛⠛⠃⠀⠀⠀⠀⠀⢸⣿⡇",
        "⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⠀⠀⢸⣿⡇",
        "⠉⠉⠉⠉⠉⠉⣉⠉⠉⠉⠉⠉⠉⠀⠀⢸⣿⡇",
        "⠀⠀⣤⣶⡿⠿⠿⠿⢿⣷⣦⡀⠀⠀⠀⢸⣿⣷⣶⣶⣶",
        "⠀⣸⣿⠋⠀⠀⠀⠀⠀⠘⣿⣧⠀⠀⠀⢸⣿⡇",
        "⠀⠸⣿⣦⣀⠀⠀⠀⢀⣴⣿⠏⠀⠀⠀⢸⣿⡇",
        "⠀⠀⠈⠛⠿⠿⠿⠿⠿⠛⠁⠀⠀⠀⠀⢸⣿⡇",
        "⠀⠀⠀⣤⣤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠸⠿⠇",
        "⠀⠀⠀⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
        "⠀⠀⠀⣿⣿⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⡄",
        "⠀⠀⠀⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠁",
    ];

    println!();
    println!(
        " {}◓{}◒{}{} ─────────────────────────────────── {}◓{}◒{}",
        red, blu, rs, d, red, blu, rs
    );
    println!();

    for (i, h) in han.iter().enumerate() {
        let f = if i < flower.len() {
            flower[i]
        } else {
            "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
        };
        println!(" {}{}{}{}{}{}{}", mg, f, rs, w, b, h, rs);
    }

    println!();
    println!(" {}    Korean Programming Language v0.1.0{}", d, rs);
    println!(" {}    github.com/xodn348/han{}", d, rs);
    println!();
    println!(
        " {}◓{}◒{}{} ─────────────────────────────────── {}◓{}◒{}",
        red, blu, rs, d, red, blu, rs
    );
    println!();
}

fn run_repl() {
    print_banner();
    println!("종료: Ctrl+D 또는 '나가기' 입력\n");

    let stdin = io::stdin();
    let mut env = interpreter::Environment::new();

    loop {
        print!("한> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,
            Err(e) => {
                eprintln!("입력 오류: {}", e);
                break;
            }
            Ok(_) => {}
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "나가기" {
            break;
        }

        match try_parse(trimmed) {
            Ok(program) => match interpreter::eval_block(&program.stmts, &mut env) {
                Ok(_) => {}
                Err(e) => {
                    if e.line > 0 {
                        eprintln!("[에러] {}번째 줄: {}", e.line, e.message);
                    } else {
                        eprintln!("[에러] {}", e.message);
                    }
                }
            },
            Err(e) => {
                eprintln!("[파서 에러] {}", e.message);
            }
        }
    }

    println!("\n안녕히 가세요!");
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
                    if e.line > 0 {
                        eprintln!("[런타임 에러] {}번째 줄: {}", e.line, e.message);
                    } else {
                        eprintln!("[런타임 에러] {}", e.message);
                    }
                    if !e.stack_trace.is_empty() {
                        eprintln!("스택 트레이스:");
                        for frame in &e.stack_trace {
                            eprintln!("{}", frame);
                        }
                    }
                    process::exit(1);
                }
            }
        }

        Commands::Check { file } => {
            let source = fs::read_to_string(&file).unwrap_or_else(|e| {
                eprintln!("파일 읽기 실패 '{}': {}", file, e);
                process::exit(1);
            });

            let tokens = lexer::tokenize(&source);
            match parser::parse(tokens) {
                Ok(program) => {
                    let type_errors = typechecker::check(&program);
                    if type_errors.is_empty() {
                        println!("✓ 타입 검사 통과");
                    } else {
                        for err in &type_errors {
                            if err.line > 0 {
                                eprintln!("[타입 경고] {}번째 줄: {}", err.line, err.message);
                            } else {
                                eprintln!("[타입 경고] {}", err.message);
                            }
                        }
                        process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("[파서 에러] {}번째 줄: {}", e.line, e.message);
                    process::exit(1);
                }
            }
        }

        Commands::Init { name } => {
            let dir = name.as_deref().unwrap_or(".");
            if dir != "." {
                fs::create_dir_all(dir).unwrap_or_else(|e| {
                    eprintln!("디렉토리 생성 실패: {}", e);
                    process::exit(1);
                });
            }
            let main_path = if dir == "." {
                "main.hgl".to_string()
            } else {
                format!("{}/main.hgl", dir)
            };
            let content = "// Han 프로그래밍 언어\n// 자세한 내용: https://github.com/xodn348/han\n\n함수 main() {\n    출력(\"안녕하세요!\")\n}\n\nmain()\n";
            fs::write(&main_path, content).unwrap_or_else(|e| {
                eprintln!("파일 생성 실패: {}", e);
                process::exit(1);
            });
            println!("✓ 프로젝트 초기화 완료: {}", main_path);
        }

        Commands::Repl => {
            run_repl();
        }

        Commands::Lsp => {
            lsp::run_lsp();
        }
    }
}
