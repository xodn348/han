# System & Process

## Shell Command

```
변수 결과 = 실행("ls -la")
출력(결과)
```

Runs the command via `sh -c` and returns stdout as a string.

## Environment Variables

```
변수 홈 = 환경변수("HOME")
출력(홈)    // /Users/username

변수 없는변수 = 환경변수("NONEXISTENT")
출력(없는변수)    // 없음
```

Returns `없음` if the variable doesn't exist.

## CLI Arguments

```
변수 인자들 = 명령인자()
반복 인자 안에서 인자들 {
    출력(인자)
}
```

Returns arguments passed after the filename: `hgl interpret file.hgl arg1 arg2`

## Sleep

```
잠자기(1000)    // sleep 1 second (1000 milliseconds)
```

## Type Introspection

```
출력(타입(42))          // 정수
출력(타입("hello"))     // 문자열
출력(타입([1,2,3]))     // 배열
출력(타입(사전()))      // 사전
출력(타입(참))          // 불
```

## Functions

| Function | Description |
|----------|-------------|
| `실행(명령어)` | Run shell command → stdout string |
| `환경변수(이름)` | Get env var → string or 없음 |
| `명령인자()` | CLI args → array of strings |
| `잠자기(밀리초)` | Sleep for N milliseconds |
| `타입(값)` | Type name → string |
| `출력오류(값)` | Print to stderr |
