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
| `interpreter.rs` | ~1760 | Tree-walking interpreter, builtins, methods |
| `codegen.rs` | ~1430 | LLVM IR text generation (Korean→ASCII sanitization) |
| `typechecker.rs` | ~280 | Compile-time type checker (warning mode) |
| `lsp.rs` | ~330 | LSP server (hover, completion) |
| `main.rs` | ~310 | CLI entry point (clap) |
| `builtins/` | — | Builtin function catalog (math, io, string, system) |

## Builtin Module Structure

```
src/builtins/
├── mod.rs      — module declarations
├── math.rs     — 제곱근, 절댓값, 거듭제곱, 정수변환, 실수변환
├── io.rs       — 출력, 입력, 형식, 파일읽기/쓰기
├── string.rs   — 길이, 분리, 포함, 바꾸기, 대문자, 소문자
└── system.rs   — 실행, HTTP, 정규식, 제이슨, 날짜/시간
```

## Codegen: Korean Identifier Handling

LLVM IR only allows ASCII identifiers. Han's codegen sanitizes Korean variable and function names using Unicode hex encoding:

```
변수 두배 = ...  →  %var_uB450uBC30 = alloca i64
함수 인사() { }  →  define void @uc778uc0ac() { }
```

This allows Korean-named functions and variables to compile to native binaries.
