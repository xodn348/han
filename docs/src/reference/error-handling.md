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

The error variable (`오류`) contains the error message as a string.

## What Gets Caught

- Division by zero
- File not found
- Index out of bounds
- Undefined variable access
- Type mismatches at runtime

## Example: Safe Division

```
함수 안전나누기(a: 정수, b: 정수) -> 정수 {
    시도 {
        반환 a / b
    } 처리(오류) {
        출력(형식("나누기 실패: {0}", 오류))
        반환 0
    }
}
```
