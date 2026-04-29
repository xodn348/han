# Changelog

All notable changes to **Han (한)** are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `description`, `license`, `repository`, `keywords`, `categories`, `authors` metadata in `Cargo.toml` for crates.io publication.
- `CHANGELOG.md` documenting the v0.2.0 release.

### Changed
- Cleaned up clippy warnings: added `Default` impls for `CodeGen` and `Environment`, removed needless `return`s in HTTP builtins, used `HashMap::values()` for enum tag resolution, and collapsed nested `if let` blocks in struct field lookup.

## [0.2.0] - 2026-04

V2 of the Han language: natural Korean syntax (SOV examples, `이면` keyword,
string interpolation), pipe operator, expanded math/linear-algebra/Python interop,
and a stabilized parser/CLI flow.

### Added
- **Pipe operator** `|>` for left-to-right function composition.
- **`이면` keyword** for natural Korean conditional flow (`X 이면 ...`).
- **String interpolation** in V2 grammar.
- **Logical operators** `그리고` (and) / `또는` (or).
- **Math builtins**: `사인`, `코사인`, `탄젠트`, `로그`, `지수`, `올림`, `내림`, `반올림`, `최대`, `최소`, `난수`, `파이`, `자연상수`.
- **Linear-algebra builtins**: `행렬곱`, `전치`, `스칼라곱`, `내적`, `외적`, `행렬합`, `행렬차`, `단위행렬`, `텐서곱`.
- **Python interop** via PyO3: `파이썬()` / `파이썬_값()` builtins for calling NumPy, PyTorch, and arbitrary Python code from Han.
- **Playground examples**: quantum example using `행렬곱`, attention example, V2 SOV examples.
- **V2 education pack** and refreshed release artifacts in `docs/`.
- **Han skill bundle** for AI agent integration with install guidance.
- Han agent setup documentation aligned with current syntax.
- CI: `workflow_dispatch` trigger for manual deploys, auto-rebuild of WASM playground on `src/` changes.

### Changed
- **Parser/CLI flow stabilized** for V2 grammar.
- Refactored example programs to use `이면` for natural conditionals.
- Playground updated to V2: SOV defaults, English comments, trimmed extras.
- Documentation (README, mdBook docs, playground, AI references) refreshed for the V2 surface area.

### Fixed
- `없음` (none) type handling.
- `상수` (const) immutability enforcement.
- `.포함()` (contains) method parsing.
- Version-string handling.
- Empty-array typechecker now returns unknown instead of defaulting to `정수` (int).
- Playground: restored Golden initial code, rounded quantum/attention output.
- Cleaned up dead-code warnings.

### Tests
- Regression tests added for `이면`, `상수`, `.포함`, and linear-algebra builtins.

## [0.1.0] - Initial release

First public Han release: lexer, parser, AST, tree-walking interpreter,
LLVM-IR text codegen, `hgl` CLI, LSP server, VS Code extension, and a
documentation site (mdBook) with `llms.txt` for AI agents.

### Language features (0.1.0)
- Korean Unicode lexer and recursive-descent parser.
- Arrays, structs, tuples, enums, pattern matching.
- For-in loops, range operator, string iteration.
- Closures, format strings, generics, `impl` blocks.
- `try`/`catch`, file I/O, string methods, module imports.
- Korean error messages.

[Unreleased]: https://github.com/xodn348/han/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/xodn348/han/releases/tag/v0.2.0
[0.1.0]: https://github.com/xodn348/han/releases/tag/v0.1.0
