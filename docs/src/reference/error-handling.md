# Error Handling

## Try / Catch

```
시도 {
    변수 내용 = 파일읽기("없는파일.txt")
    출력(내용)
} 처리(오류) {
    출력(형식("에러: {0}", 오류))
}
```

The error variable (`오류`) contains the error message as a string. V2 errors use Elm-style messaging with source context, so the printed text points back to the original code location.

## What Gets Caught

- Division by zero
- File not found
- Index out of bounds
- Undefined variable access
- Type mismatches at runtime

## Example: Safe Division

```hgl
함수 안전나누기(a: 정수, b: 정수) -> 정수 {
    시도 {
        반환 a / b
    } 처리(오류) {
        출력(형식("나누기 실패: {0}", 오류))
        반환 0
    }
}
```

Example error shape:

```text
타입 오류: 정수를 기대했지만 문자열을 받았습니다
 --> main.hgl:3:10
  |
3 | 변수 x: 정수 = "hello"
  |          ^^^^^^^^^^^^^ 여기서 타입이 맞지 않습니다
```
