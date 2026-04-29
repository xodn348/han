# Han (한): a Korean-syntax programming language written in Rust

## Short pitch

Han is an experimental programming language that makes Korean the primary language of the toolchain: keywords, type names, built-ins, methods, logical operators, booleans, error handling, and examples are written in Korean. The compiler/tooling is written in Rust and supports both fast interpreted execution and a native build path through generated LLVM IR.

## Install

```bash
cargo install han-lang
```

The crates.io package is `han-lang`; the installed CLI command is `hgl`.

## What is included

- **Interpreter** — `hgl interpret <file.hgl>` runs Han programs directly, without needing clang.
- **LLVM IR codegen path** — `hgl build <file.hgl>` emits LLVM IR and uses clang to produce a native binary.
- **LSP** — `hgl lsp` provides editor language-server support such as hover and completion.
- **VS Code extension** — syntax highlighting plus LSP client integration for Han files.
- **Playground** — browser playground backed by the WASM-compiled interpreter.
- **Examples** — Korean-named sample programs covering hello world, math, strings, control flow, structs, pattern matching, file I/O, JSON, HTTP, regex, and tutorial-style lessons.

## Experimental disclaimer

Han is a small experimental language and Rust side project, not a production language. The syntax, standard library, compiler behavior, and editor tooling may change as the language evolves. Feedback, issues, and pull requests are welcome, especially from Rust developers interested in compilers, language tooling, WASM, LSPs, and non-English programming languages.

## Known limitations

- Native builds require clang; interpreter-only usage does not.
- The module story is intentionally simple today: local file inclusion via `포함 "file.hgl"`, not a package manager or dependency registry.
- Han does not yet have a full namespace/package system; included files share the program scope.
- The LLVM path generates text IR and shells out to clang rather than using the LLVM C API or in-process LLVM bindings.
- The playground runs the interpreter path, so it is best for trying language features rather than validating native compilation behavior.
- APIs and diagnostics are still evolving; expect rough edges in a young language implementation.

## Links

- GitHub: https://github.com/xodn348/han
- Docs: https://xodn348.github.io/han/introduction.html
- Playground: https://xodn348.github.io/han/playground/
