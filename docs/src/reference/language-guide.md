# Language Guide

A tour of Han's syntax — variables, functions, conditionals, and loops. For deeper coverage of any single feature, see the dedicated pages under "Language Reference" in the sidebar.

## Variables and Constants

```
변수 이름 = 42              // mutable variable
변수 메시지 = "안녕하세요"     // string variable
상수 파이 = 3.14            // immutable constant
```

With explicit type annotations:

```
변수 나이: 정수 = 25
변수 키: 실수 = 175.5
변수 이름: 문자열 = "홍길동"
변수 활성: 불 = 참
```

## Functions

```
함수 더하기(가: 정수, 나: 정수) -> 정수 {
    반환 가 + 나
}

함수 인사(이름: 문자열) {
    출력("안녕하세요, " + 이름)
}
```

## Conditionals

Han still supports the older minimal conditional form, but the Korean-default docs style uses `이면`:

```hgl
만약 점수 >= 90 이면 {
    출력("A")
} 아니면 점수 >= 80 이면 {
    출력("B")
} 아니면 {
    출력("C")
}
```

## Loops

**For loop** (`반복`):

```
반복 변수 i = 0; i < 10; i += 1 {
    출력(i)
}
```

**While loop** (`동안`):

SOV:

```hgl
변수 n = 0
n < 5 동안 {
    출력(n)
    n += 1
}
```

SVO alternative:

```hgl
변수 n = 0
동안 n < 5 {
    출력(n)
    n += 1
}
```

**Loop control** — `멈춰` (break) and `계속` (continue):

```
반복 변수 i = 0; i < 100; i += 1 {
    만약 i == 50 이면 {
        멈춰
    }
    만약 i % 2 == 0 이면 {
        계속
    }
    출력(i)
}
```

## Keyword Reference

| Keyword | Meaning | English Equivalent |
|---------|---------|-------------------|
| `함수` | function definition | `fn` / `function` |
| `반환` | return value | `return` |
| `변수` | mutable variable | `let mut` / `var` |
| `상수` | immutable constant | `const` |
| `만약` | conditional | `if` |
| `이면` | conditional marker | `then` / `if-then` |
| `아니면` | else branch | `else` |
| `그리고` | and (logical) | `&&` |
| `또는` | or (logical) | `||` |
| `반복` | for loop | `for` |
| `동안` | while loop | `while` |
| `멈춰` | break loop | `break` |
| `계속` | continue loop | `continue` |
| `참` | boolean true | `true` |
| `거짓` | boolean false | `false` |
| `출력` | print to console | `print` |
| `입력` | read from console | `input` |

## Type System & Operators

| Type | Description | LLVM Type | Examples |
|------|-------------|-----------|----------|
| `정수` | 64-bit integer | `i64` | `42`, `-10` |
| `실수` | 64-bit float | `f64` | `3.14`, `-0.5` |
| `문자열` | UTF-8 string | `i8*` | `"안녕하세요"` |
| `불` | boolean | `i1` | `참`, `거짓` |
| `없음` | void / no value | `void` | (function return type) |

| Operator | Description |
|----------|-------------|
| `+`, `-`, `*`, `/`, `%` | Arithmetic |
| `==`, `!=` | Equality |
| `<`, `>`, `<=`, `>=` | Comparison |
| `&&`, `\|\|`, `!`, `그리고`, `또는` | Logical |
| `=`, `+=`, `-=`, `*=`, `/=` | Assignment |
