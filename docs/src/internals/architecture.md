# Compiler Architecture

How the Han toolchain is organized — from `.hgl` source text to either a tree-walked result or a native binary.

## How Han Works

Han follows the classical compiler pipeline, implemented entirely in Rust with zero external compiler dependencies (LLVM IR is generated as plain text):

```
Source (.hgl)
    │
    ▼
┌─────────┐     ┌─────────┐     ┌─────────┐
│  Lexer  │ ──▶ │ Parser  │ ──▶ │   AST   │
│(lexer.rs)│    │(parser.rs)│   │ (ast.rs) │
└─────────┘     └─────────┘     └────┬────┘
                                     │
                        ┌────────────┼────────────┐
                        ▼                         ▼
                ┌──────────────┐         ┌──────────────┐
                │ Interpreter  │         │   CodeGen    │
                │(interpreter.rs)│       │ (codegen.rs) │
                └──────┬───────┘         └──────┬───────┘
                       │                        │
                       ▼                        ▼
                  Direct Output           LLVM IR (.ll)
                                               │
                                               ▼
                                         clang → Binary
```

## Project Structure

```
han/
├── src/
│   ├── main.rs          CLI entry point (hgl command)
│   ├── lexer.rs         Lexer: Korean source → token stream
│   ├── parser.rs        Parser: tokens → AST (recursive descent)
│   ├── ast.rs           AST node type definitions
│   ├── interpreter.rs   Tree-walking interpreter
│   ├── codegen.rs       LLVM IR text code generator
│   └── lsp.rs           LSP server (hover + completion)
├── editors/
│   └── vscode/          VS Code extension (syntax highlighting + LSP)
├── examples/            Example .hgl programs
├── spec/
│   └── SPEC.md          Formal language specification (EBNF)
└── tests/               Integration tests
```

## Design Decisions

**Why text-based LLVM IR instead of the LLVM C API?**
Han generates LLVM IR as plain text strings, avoiding the complexity of linking against LLVM libraries. This keeps the build simple (`cargo build` — no LLVM installation required) while still producing optimized native binaries through clang.

**Why both interpreter and compiler?**
The interpreter enables instant execution without any toolchain dependencies beyond Rust. The compiler path exists for production use where performance matters. Same parser, same AST, two backends.

**Why Rust?**
Rust's enum types map naturally to AST nodes and token variants. Pattern matching makes parser and interpreter logic clear and exhaustive. Memory safety without garbage collection suits a language toolchain.
