# Regex

Powered by Rust's `regex` crate.

## Find All Matches

```
변수 결과 = 정규식_찾기("[0-9]+", "abc 123 def 456")
출력(결과)    // [123, 456]
```

## Test Match

```
출력(정규식_일치("^[0-9]+$", "12345"))    // 참
출력(정규식_일치("^[0-9]+$", "abc"))      // 거짓
```

## Replace

```
변수 결과 = 정규식_바꾸기("[0-9]+", "전화: 010-1234-5678", "***")
출력(결과)    // 전화: ***-***-***
```

## Functions

| Function | Description |
|----------|-------------|
| `정규식_찾기(패턴, 텍스트)` | Find all matches → array of strings |
| `정규식_일치(패턴, 텍스트)` | Test if pattern matches → bool |
| `정규식_바꾸기(패턴, 텍스트, 대체)` | Replace all matches → string |
