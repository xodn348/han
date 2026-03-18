# Han (한) Programming Language

Han is a statically-typed, compiled programming language where every keyword is written in Korean (Hangul). It compiles to native binaries through LLVM IR and includes a tree-walking interpreter for instant execution.

## Quick Example

```hgl
함수 인사(이름: 문자열) {
    출력("${이름}님 안녕하세요")
}

인사("세계")
```

Output: `세계님 안녕하세요`

## Key Features

- **Korean keywords** — `함수`, `만약`, `반복`, `변수`
- **SOV word order** — `조건 만약 { }`, `조건 동안 { }`
- **String interpolation** — `"${expr}"` auto-desugars to `형식()`
- **Korean logical operators** — `그리고`, `또는`
- **Dual execution** — interpreter (`hgl interpret`) and compiler (`hgl build`)
- **Tooling** — `hgl check`, `hgl init`, VS Code extension, and LSP support
- **Arrays, structs, enums, tuples, closures, pattern matching**
- **Error handling** — `시도` / `처리` with Elm-style source-context errors
- **File I/O, format strings, module imports**

## How It Works

```
Source (.hgl) → Lexer → Parser → AST → Interpreter (direct execution)
                                     → CodeGen → LLVM IR → clang → Binary
```

