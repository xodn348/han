mod ast;
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
    Repl,
    Lsp,
}

fn run_pipeline(source: &str) -> ast::Program {
    let tokens = lexer::tokenize(source);
    match parser::parse(tokens) {
        Ok(program) => {
            let type_errors = typechecker::check(&program);
            for err in &type_errors {
                if err.line > 0 {
                    eprintln!("[타입 에러] {}번째 줄: {}", err.line, err.message);
                } else {
                    eprintln!("[타입 에러] {}", err.message);
                }
            }
            if !type_errors.is_empty() {
                process::exit(1);
            }
            program
        }
        Err(e) => {
            eprintln!("[파서 에러] {}번째 줄: {}", e.line, e.message);
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
    let c = "\x1b[36m";
    let b = "\x1b[1m";
    let d = "\x1b[2m";
    let r = "\x1b[0m";
    let red = "\x1b[91m";
    let blu = "\x1b[94m";
    let y = "\x1b[93m";

    println!();
    println!("  {y}✿{r} {red}◓{blu}◒{r}{d} ────────────────────────────────────────────────────────── {red}◓{blu}◒{r} {y}✿{r}");
    println!();
    println!("  {y}      ⢠⣦⣠⡆{r}       {c}{b}⢀⣀⣀⣀⣀⣀⡀⠀⠀⠀⠀⢠⣶⡆{r}       {y}⢠⣦⣠⡆{r}");
    println!("  {y}      ⠸⣿⣿⡇{r}       {c}{b}⠘⠛⠛⠛⠛⠛⠃⠀⠀⠀⠀⢸⣿⡇{r}       {y}⠸⣿⣿⡇{r}");
    println!("  {y}  ⣸⣿⣶⣤⡀⢻⡿⢁⣤⣶⣿⣇{r} {c}{b}⢰⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⣶⡆⠀⢸⣿⡇{r} {y}⣸⣿⣶⣤⡀⢻⡿⢁⣤⣶⣿⣇{r}");
    println!("  {y}⠈⠛⠛⠿⠿⠟⡂⢀⠻⠿⠿⠛⠛⠁{r}{c}{b}   ⣀⣀⣀⣀⣀⠀⠀⠀⠀⠀⢸⣿⡇{r} {y}⠈⠛⠛⠿⠿⠟⡂⢀⠻⠿⠿⠛⠛⠁{r}");
    println!("  {y}    ⣠⣶⣿⡇⢸⣿⣶⣄{r}     {c}{b}⣴⣿⠟⠛⠛⠛⠻⣿⣦⠀⠀⠀⢸⣿⡿⠿⠿⠇{r} {y}    ⣠⣶⣿⡇⢸⣿⣶⣄{r}");
    println!("  {y}  ⠼⢿⣿⡟⠀⠀⢻⣿⡿⠧{r}   {c}{b}⢸⣿⡇⠀⠀⠀⠀⠀⢸⣿⡇⠀⠀⢸⣿⡇{r}     {y}⠼⢿⣿⡟⠀⠀⢻⣿⡿⠧{r}");
    println!("  {y}      ⠋⠀⠀⠀⠀⠙{r}       {c}{b}⠻⣿⣦⣤⣤⣤⣴⡿⠟⠀⠀⠀⢸⣿⡇{r}       {y}⠋⠀⠀⠀⠀⠙{r}");
    println!("                       {c}{b}⢀⣈⠉⠉⠉⠁⠀⠀⠀⠀⠀⢸⣿⡇{r}");
    println!("                       {c}{b}⢸⣿⠀{r}");
    println!("                       {c}{b}⢸⣿⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⡀{r}");
    println!("                       {c}{b}⠘⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠃{r}");
    println!();
    println!("  {d}               한글 프로그래밍 언어 v0.1.0{r}");
    println!("  {d}               github.com/xodn348/han{r}");
    println!();
    println!("  {y}✿{r} {red}◓{blu}◒{r}{d} ────────────────────────────────────────────────────────── {red}◓{blu}◒{r} {y}✿{r}");
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

        Commands::Repl => {
            run_repl();
        }

        Commands::Lsp => {
            lsp::run_lsp();
        }
    }
}
