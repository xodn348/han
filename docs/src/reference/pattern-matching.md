# Pattern Matching

## Basic Match

```
맞춰 값 {
    1 => 출력("하나")
    2 => 출력("둘")
    _ => 출력("기타")
}
```

## With Blocks

```
맞춰 상태 {
    "활성" => {
        출력("활성 상태")
        처리하기()
    }
    "비활성" => 출력("비활성")
    _ => 출력("알 수 없음")
}
```

## Pattern Types

| Pattern | Example | Description |
|---------|---------|-------------|
| Integer | `42` | Matches exact integer |
| String | `"hello"` | Matches exact string |
| Boolean | `참`, `거짓` | Matches boolean |
| Wildcard | `_` | Matches anything |
| Binding | `x` | Matches anything, binds to variable `x` |
| Array | `[1, 2, 3]` | Matches array structure |

## Variable Binding

```
맞춰 값 {
    0 => 출력("영")
    n => 출력(형식("값: {0}", n))
}
```
