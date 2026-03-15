# I/O

## Output

```
출력("안녕하세요")        // print to stdout
출력(42)                 // prints any value
출력("이름:", 이름)       // multiple args, space-separated
출력오류("에러 메시지")    // print to stderr
```

## Input

```
변수 이름 = 입력()
출력(형식("안녕, {0}!", 이름))
```

`입력()` reads one line from stdin and returns it as a string.
