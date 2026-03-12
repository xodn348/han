#![allow(dead_code, unused)]
mod ast;
mod lexer;
mod parser;
mod interpreter;
mod codegen;

use clap::{Parser, Subcommand};

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

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build { file } => eprintln!("빌드 중: {}", file),
        Commands::Run { file } => eprintln!("실행 중: {}", file),
        Commands::Interpret { file } => eprintln!("인터프리팅: {}", file),
    }
}
