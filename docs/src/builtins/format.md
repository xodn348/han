# Format Strings

## Positional Arguments

```
형식("이름: {0}, 나이: {1}", "홍길동", 30)
// → "이름: 홍길동, 나이: 30"
```

## Named Arguments (from scope)

```
변수 이름 = "홍길동"
변수 나이 = 30
형식("이름: {이름}, 나이: {나이}")
// → "이름: 홍길동, 나이: 30"
```

## String Interpolation

```hgl
변수 이름 = "홍길동"
출력("${이름}님 안녕하세요")
// desugars to: 출력(형식("{0}님 안녕하세요", 이름))
```

Named mode substitutes `{변수명}` with the variable's value from the current scope. Interpolated strings are automatically desugared to `형식()`.

```hgl
출력("합: ${1 + 2}")
// desugars to: 출력(형식("합: {0}", 1 + 2))
```

Use interpolation for simple expressions. `형식()` remains the most explicit option for complex templates.
