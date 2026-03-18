# Keyword Reference

## Control Keywords

| Keyword | English | Syntax |
|---------|---------|--------|
| `함수` | function | `함수 이름(매개변수: 타입) -> 반환타입 { }` |
| `반환` | return | `반환 값` |
| `변수` | let (mutable) | `변수 이름 = 값` |
| `상수` | const | `상수 이름 = 값` |
| `만약` | if | `조건 만약 { }` or `만약 조건 { }` |
| `아니면` | else | `아니면 { }` or `아니면 만약 조건 { }` |
| `그리고` | and (logical) | `a 그리고 b` |
| `또는` | or (logical) | `a 또는 b` |
| `반복` | for | `반복 변수 i = 0; i < n; i += 1 { }` |
| `동안` | while | `조건 동안 { }` or `동안 조건 { }` |
| `멈춰` | break | `멈춰` |
| `계속` | continue | `계속` |
| `안에서` | in (for-in) | `반복 x 안에서 배열 { }` |

## Type Keywords

| Keyword | English | LLVM |
|---------|---------|------|
| `정수` | int | i64 |
| `실수` | float | f64/double |
| `문자열` | string | i8* |
| `불` | bool | i1 |
| `없음` | void/null | void |

## Structure Keywords

| Keyword | English | Syntax |
|---------|---------|--------|
| `구조` | struct | `구조 이름 { 필드: 타입 }` |
| `구현` | impl | `구현 구조체 { 함수 메서드(자신: T) { } }` |
| `열거` | enum | `열거 이름 { 변형1, 변형2 }` |
| `시도` | try | `시도 { } 처리(오류) { }` |
| `처리` | catch | `처리(오류) { }` |
| `맞춤` | match | `맞춤 값 { 패턴 => 결과 }` |
| `포함` | import | `포함 "파일.hgl"` |

## Literal Keywords

| Keyword | English |
|---------|---------|
| `참` | true |
| `거짓` | false |
| `없음` | null/void |

## Built-in Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `출력(값)` | `(any...) -> 없음` | Print to stdout |
| `입력()` | `() -> 문자열` | Read line from stdin |
| `제곱근(x)` | `(숫자) -> 실수` | Square root |
| `절댓값(x)` | `(숫자) -> 숫자` | Absolute value |
| `거듭제곱(밑, 지수)` | `(숫자, 숫자) -> 실수` | Power |
| `정수변환(x)` | `(any) -> 정수` | Convert to integer |
| `실수변환(x)` | `(any) -> 실수` | Convert to float |
| `길이(s)` | `(문자열) -> 정수` | String length |
| `형식(template, ...)` | `(문자열, any...) -> 문자열` | Format string |
| `파일읽기(경로)` | `(문자열) -> 문자열` | Read file |
| `파일쓰기(경로, 내용)` | `(문자열, 문자열) -> 없음` | Write file |
| `파일추가(경로, 내용)` | `(문자열, 문자열) -> 없음` | Append to file |
| `파일존재(경로)` | `(문자열) -> 불` | File exists |
| `출력오류(값)` | `(any...) -> 없음` | Print to stderr |
