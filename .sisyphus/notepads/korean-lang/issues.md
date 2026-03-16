
## 2026-03-16 Gotcha
- Warning mode does not change interpreter behavior: invalid calls like `더하기("문자열", 1)` still continue past type checking but can fail later as runtime errors inside execution.

## 2026-03-16 Verification note
- `cargo test` still fails in unrelated codegen tests (`test_codegen_for_in_reads_array_length_from_header`, `test_codegen_try_catch_uses_error_branching`) while `src/codegen.rs`, `src/parser.rs`, and `tests/integration.rs` already have pre-existing local modifications; the new typechecker tests pass.
