# Arrays

## Creating Arrays

```
변수 숫자 = [1, 2, 3, 4, 5]
변수 빈배열 = []
```

## Indexing

```
출력(숫자[0])     // 1
출력(숫자[-1])    // 5 (negative indexing)
숫자[0] = 99      // mutation
```

## Methods

| Method | Description | Example |
|--------|-------------|---------|
| `.추가(값)` | Append element | `arr.추가(6)` |
| `.삭제(인덱스)` | Remove at index | `arr.삭제(0)` |
| `.길이()` | Length | `arr.길이()` → `5` |
| `.포함(값)` | Contains | `arr.포함(3)` → `참` |
| `.역순()` | Reverse (new array) | `arr.역순()` |
| `.정렬()` | Sort (new array) | `arr.정렬()` |
| `.합치기(구분자)` | Join to string | `arr.합치기(", ")` |

## Iteration

```
반복 항목 안에서 [1, 2, 3] {
    출력(항목)
}
```
