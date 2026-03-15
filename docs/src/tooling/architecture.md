# Architecture

## Compiler Pipeline

```
Source (.hgl)
    │
    ▼
┌─────────────┐
│   Lexer     │  Source text → Token stream
│ (lexer.rs)  │  "함수" → Token::함수
└──────┬──────┘
       ▼
┌─────────────┐
│   Parser    │  Token stream → AST
│ (parser.rs) │  Recursive descent, precedence climbing
└──────┬──────┘
       ▼
┌─────────────┐
│    AST      │  Tree representation of the program
│  (ast.rs)   │  Expr, StmtKind, Pattern, Type
└──────┬──────┘
       │
       ├─────────────────┐
       ▼                 ▼
┌─────────────┐   ┌─────────────┐
│ Interpreter │   │  CodeGen    │
│(interpreter)│   │(codegen.rs) │
│             │   │             │
│ Tree-walking│   │ LLVM IR text│
│ execution   │   │ generation  │
└─────────────┘   └──────┬──────┘
                         ▼
                    clang → Binary
```

## Source Files

| File | Lines | Purpose |
|------|-------|---------|
| `lexer.rs` | ~550 | Tokenization, Korean keyword recognition |
| `parser.rs` | ~1280 | Recursive descent parser, precedence climbing |
| `ast.rs` | ~270 | AST node definitions (Expr, Stmt, Type, Pattern) |
| `interpreter.rs` | ~1340 | Tree-walking interpreter, builtins, methods |
| `codegen.rs` | ~900 | LLVM IR text generation |
| `lsp.rs` | ~330 | LSP server (hover, completion) |
| `main.rs` | ~200 | CLI entry point (clap) |
