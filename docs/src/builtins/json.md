# JSON

Powered by Rust's `serde_json` crate.

## Parse JSON

```
변수 텍스트 = "{\"이름\": \"홍길동\", \"나이\": 30}"
변수 데이터 = 제이슨_파싱(텍스트)
출력(데이터["이름"])    // 홍길동
출력(데이터["나이"])    // 30
```

JSON objects become `사전`, arrays become `배열`.

## Generate JSON

```
변수 사용자 = 사전("이름", "홍길동", "나이", 30)
변수 json = 제이슨_생성(사용자)
출력(json)    // {"나이":30,"이름":"홍길동"}
```

## Pretty Print

```
변수 예쁜 = 제이슨_예쁘게(사용자)
출력(예쁜)
```

## Functions

| Function | Description |
|----------|-------------|
| `제이슨_파싱(문자열)` | Parse JSON string → Han value |
| `제이슨_생성(값)` | Han value → JSON string |
| `제이슨_예쁘게(값)` | Han value → pretty-printed JSON |
