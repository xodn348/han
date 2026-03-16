
## 2026-03-16 Gotcha
- Warning mode does not change interpreter behavior: invalid calls like `더하기("문자열", 1)` still continue past type checking but can fail later as runtime errors inside execution.
