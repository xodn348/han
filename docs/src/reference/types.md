# Data Types

## Primitive Types

| Type | Han | Description | Examples |
|------|-----|-------------|----------|
| Integer | `정수` | 64-bit signed integer | `42`, `-10`, `0` |
| Float | `실수` | 64-bit floating point | `3.14`, `-0.5` |
| String | `문자열` | UTF-8 string | `"안녕하세요"` |
| Boolean | `불` | true/false | `참`, `거짓` |
| Void | `없음` | null/void | `없음` |

## Type Coercion

Int and Float mix automatically:

```
출력(1 + 1.5)     // 2.5 (Float)
출력(3 * 2.0)     // 6.0 (Float)
출력(10 - 0.5)    // 9.5 (Float)
```

## Compound Types

- **Arrays**: `[1, 2, 3]` — type `[정수]`
- **Tuples**: `(1, "hello", 참)` — type `(정수, 문자열, 불)`
- **Structs**: `구조 사람 { 이름: 문자열 }`
- **Enums**: `열거 방향 { 위, 아래 }`
