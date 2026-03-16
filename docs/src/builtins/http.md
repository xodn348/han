# HTTP

Powered by Rust's `reqwest` crate (blocking mode).

## GET Request

```
변수 응답 = HTTP_포함("https://httpbin.org/get")
출력(응답)
```

## POST Request

```
변수 본문 = 제이슨_생성(사전("이름", "홍길동"))
변수 응답 = HTTP_보내기("https://httpbin.org/post", 본문)
출력(응답)
```

POST sends with `Content-Type: application/json`. If the body is not a string, it's auto-converted to JSON.

## Error Handling

```
시도 {
    변수 응답 = HTTP_포함("https://invalid-url.example")
} 처리(오류) {
    출력(형식("HTTP 오류: {0}", 오류))
}
```

## Functions

| Function | Description |
|----------|-------------|
| `HTTP_포함(url)` | GET request → response body as string |
| `HTTP_보내기(url, body)` | POST request with JSON body → response as string |
