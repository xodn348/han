# File I/O

| Function | Description | Example |
|----------|-------------|---------|
| `파일읽기(경로)` | Read file to string | `파일읽기("data.txt")` |
| `파일쓰기(경로, 내용)` | Write string to file | `파일쓰기("out.txt", "hello")` |
| `파일추가(경로, 내용)` | Append to file | `파일추가("log.txt", "line\n")` |
| `파일존재(경로)` | Check if file exists | `파일존재("data.txt")` → `참`/`거짓` |

## Example: Read and Process

```
시도 {
    변수 내용 = 파일읽기("data.txt")
    변수 줄들 = 내용.분리("\n")
    반복 줄 안에서 줄들 {
        출력(줄)
    }
} 처리(오류) {
    출력(형식("파일 오류: {0}", 오류))
}
```
