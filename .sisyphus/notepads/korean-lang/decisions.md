# Decisions

## 2026-03-12 Project Setup
- Language name: Han
- CLI command: `hgl`
- File extension: `.hgl`
- Compiler: Rust
- Backend: LLVM IR text output → clang (no inkwell for MVP)
- Parser: Recursive Descent — nom/pest forbidden
- Korean identifiers: allowed (변수 나이 = 20)
- Memory: stack-based, no GC for MVP
- Interpreter: tree-walking included for language verification
- Project dir: /Users/jnnj92/han

## Korean Keywords
함수, 반환, 변수, 상수, 만약, 아니면, 반복, 동안, 멈춰, 계속, 참, 거짓, 출력, 입력

## MVP Types
정수(i64), 실수(f64), 문자열(String), 불(bool), 없음(void)

## 2026-03-16 Warning-only function call validation
- Kept the type checker as a collector (`Vec<TypeError>`) and switched CLI/WASM callers to render `[타입 경고]` messages instead of exiting early, so static analysis stays advisory.
- Implemented argument validation by looking up function signatures from `env.funcs` and comparing both arity and inferred argument types at each `Expr::Call`.
