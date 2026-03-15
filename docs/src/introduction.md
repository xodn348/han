# Han (한) Programming Language

Han is a statically-typed, compiled programming language where every keyword is written in Korean (Hangul). It compiles to native binaries through LLVM IR and includes a tree-walking interpreter for instant execution.

## Quick Example

```
함수 인사(이름: 문자열) {
    출력(형식("안녕하세요, {0}!", 이름))
}

인사("세계")
```

Output: `안녕하세요, 세계!`

## Key Features

- **Korean keywords** — `함수`, `만약`, `반복`, `변수`
- **Dual execution** — interpreter (`hgl interpret`) and compiler (`hgl build`)
- **Arrays, structs, enums, tuples, closures, pattern matching**
- **Error handling** — `시도` / `실패` (try/catch)
- **File I/O, format strings, module imports**
- **VS Code extension with LSP support**

## How It Works

```
Source (.hgl) → Lexer → Parser → AST → Interpreter (direct execution)
                                     → CodeGen → LLVM IR → clang → Binary
```
