## 2026-03-12 Task 15: Examples verified
- 5 examples working: 안녕, 피보나치(10)=55, 팩토리얼(10)=3628800, 짝홀, 합계(100)=5050
- Any syntax issues discovered:
  - `-> 없음` return type annotation fails: lexer maps "없음" to Token::없음 (value token), not Token::없음타입 (type token). 없음타입 is never inserted into keyword map. Workaround: omit return type annotation entirely (it's optional per EBNF).
  - `main()` is NOT auto-called by interpreter: `interpret()` calls `eval_block` which only registers FuncDef in env. Must add explicit `main()` call at top level of each file.

## 2026-03-16 Task: Function call warnings
- `typechecker::check` still returns `Vec<TypeError>`, but function call validation now walks statement expressions so top-level calls, return expressions, loop conditions, and other nested call sites can emit warnings.
- Added regression tests in `src/typechecker.rs` for argument type mismatch, argument count mismatch, and valid calls to lock in warning-only behavior at the checker level.

## 2026-03-16 Task: Struct field and array warnings
- `Expr::ArrayLiteral` now infers its element type from the literal contents, which lets `["hello"]` infer as `[문자열]` and keeps `반복 item 안에서 arr` aligned with the array element type.
- Mixed array literals are best checked in two steps: infer a common element type when widening is valid (`정수` -> `실수`), then emit a warning on the first incompatible element so execution can continue.
