# String Methods

| Method | Description | Example | Result |
|--------|-------------|---------|--------|
| `.길이()` | Character count | `"한글".길이()` | `2` |
| `.분리(sep)` | Split by separator | `"a,b,c".분리(",")` | `["a", "b", "c"]` |
| `.포함(s)` | Contains substring | `"hello".포함("ell")` | `참` |
| `.바꾸기(from, to)` | Replace | `"hello".바꾸기("l", "r")` | `"herro"` |
| `.앞뒤공백제거()` | Trim whitespace | `" hi ".앞뒤공백제거()` | `"hi"` |
| `.대문자()` | Uppercase | `"hello".대문자()` | `"HELLO"` |
| `.소문자()` | Lowercase | `"HELLO".소문자()` | `"hello"` |
| `.시작(s)` | Starts with | `"hello".시작("he")` | `참` |
| `.끝(s)` | Ends with | `"hello".끝("lo")` | `참` |

## String Indexing

```
변수 s = "한글"
출력(s[0])    // 한
출력(s[1])    // 글
```

## String Iteration

```
반복 글자 안에서 "한글" {
    출력(글자)
}
```

## Concatenation

```
변수 full = "안녕" + "하세요"
출력(full)    // 안녕하세요
```
