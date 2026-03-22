# Han repository workflow

Use this when the task is about the Han repository itself rather than just writing standalone `.hgl` code.

## Repository map

- `src/` — compiler, interpreter, parser, lexer, typechecker, LSP
- `examples/` — runnable Han programs
- `docs/src/` — mdBook docs and API/reference pages
- `tests/integration.rs` — integration coverage
- `editors/vscode/` — VS Code extension assets
- `web/` — browser playground artifacts

## Source of truth

When syntax docs disagree, trust code in this order:

1. `src/lexer.rs` — keyword inventory
2. `src/parser.rs` — accepted syntax forms
3. `src/interpreter.rs` / `src/typechecker.rs` — runtime and semantic behavior
4. `examples/*.hgl` and `docs/src/` — preferred user-facing style

## Useful commands

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- check examples/안녕.hgl
cargo run -- interpret examples/안녕.hgl
```

## Working rules

- Keep Han examples UTF-8 and preserve Hangul identifiers.
- Prefer the current docs style (`만약 ... 이면`, `처리`, `맞춤`, `포함`).
- If you change language syntax or builtins, update examples and AI-facing references together.
- When adding docs for agents, keep them concise and example-driven.
