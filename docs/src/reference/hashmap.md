# HashMap / Dictionary

## Creating a Map

```
변수 점수 = 사전("수학", 95, "영어", 88, "과학", 92)
변수 빈맵 = 사전()
```

Arguments are key-value pairs: `사전(key1, val1, key2, val2, ...)`.

## Access and Mutation

```
출력(점수["수학"])       // 95
점수["국어"] = 100       // add new key
점수["수학"] = 99        // update existing
```

## Methods

| Method | Description | Example |
|--------|-------------|---------|
| `.키목록()` | All keys as array | `점수.키목록()` → `["수학", "영어", "과학"]` |
| `.값목록()` | All values as array | `점수.값목록()` → `[95, 88, 92]` |
| `.길이()` | Number of entries | `점수.길이()` → `3` |
| `.포함(키)` | Key exists | `점수.포함("수학")` → `참` |
| `.삭제(키)` | Remove key | `점수.삭제("영어")` |

## Iteration

```
변수 키들 = 점수.키목록()
반복 키 안에서 키들 {
    출력(형식("{0}: {1}", 키, 점수[키]))
}
```
