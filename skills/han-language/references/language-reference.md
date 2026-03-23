# Han language quick reference

## Commands

- File extension: `.hgl`
- Interpret: `hgl interpret file.hgl`
- Build: `hgl build file.hgl`
- Run: `hgl run file.hgl`
- Type-check: `hgl check file.hgl`
- REPL: `hgl repl`

Inside the repository, prefer `cargo run -- <subcommand>` while iterating on the compiler itself.

## Current keywords

| Korean | Meaning | Preferred form |
| --- | --- | --- |
| `함수` | function | `함수 이름(매개변수: 타입) -> 타입 { ... }` |
| `반환` | return | `반환 값` |
| `변수` | mutable variable | `변수 이름 = 값` |
| `상수` | constant | `상수 이름 = 값` |
| `만약` | if | `만약 조건 이면 { ... }` |
| `이면` | then / conditional marker | `만약 조건 이면 { ... }` |
| `아니면` | else | `아니면 { ... }`, `아니면 조건 이면 { ... }` |
| `반복` | for | `반복 변수 i = 0; i < n; i += 1 { ... }` |
| `안에서` | in | `반복 값 안에서 배열 { ... }` |
| `동안` | while | `동안 조건 { ... }` or `조건 동안 { ... }` |
| `멈춰` | break | `멈춰` |
| `계속` | continue | `계속` |
| `구조` | struct | `구조 이름 { 필드: 타입 }` |
| `구현` | impl | `구현 구조체 { 함수 메서드(...) { ... } }` |
| `열거` | enum | `열거 이름 { 값1, 값2 }` |
| `시도` | try | `시도 { ... } 처리(오류) { ... }` |
| `처리` | catch | `처리(오류) { ... }` |
| `맞춤` | match | `맞춤 값 { 패턴 => 결과 }` |
| `포함` | import/include | `포함 "파일.hgl"` |
| `참` / `거짓` | booleans | boolean literals |
| `없음` | null / void | null-like literal |
| `그리고` / `또는` | and / or | aliases for `&&` / `||` |

## Types

- `정수` — 64-bit integer
- `실수` — 64-bit float
- `문자열` — UTF-8 string
- `불` — boolean
- `없음` — null / void-like value
- `[정수]` — arrays
- `(정수, 문자열)` — tuples

## Core syntax examples

### Variables and functions

```hgl
변수 이름 = "홍길동"
상수 최대값 = 100

함수 두배(x: 정수) -> 정수 {
    반환 x * 2
}
```

### Conditionals

```hgl
만약 점수 >= 90 이면 {
    출력("A")
} 아니면 점수 >= 80 이면 {
    출력("B")
} 아니면 {
    출력("C")
}
```

### Loops

```hgl
반복 변수 i = 0; i < 3; i += 1 {
    출력(i)
}

반복 값 안에서 [1, 2, 3] {
    출력(값)
}

동안 참 {
    멈춰
}
```

### Structs and methods

```hgl
구조 사람 {
    이름: 문자열,
    나이: 정수
}

구현 사람 {
    함수 소개(자신: 사람) {
        출력(형식("{0} ({1})", 자신.이름, 자신.나이))
    }
}
```

### Match / try / import

```hgl
맞춤 값 {
    1 => 출력("하나")
    _ => 출력("기타")
}

시도 {
    변수 내용 = 파일읽기("data.txt")
    출력(내용)
} 처리(오류) {
    출력(형식("오류: {0}", 오류))
}

포함 "유틸.hgl"
```

## Common builtins

- I/O: `출력`, `출력오류`, `입력`
- Conversion: `정수변환`, `실수변환`, `길이`, `형식`, `타입`
- Files: `파일읽기`, `파일쓰기`, `파일추가`, `파일존재`
- Data: `사전`, `제이슨_파싱`, `제이슨_생성`
- HTTP: `HTTP_포함`, `HTTP_보내기`
- Regex: `정규식_찾기`, `정규식_일치`, `정규식_바꾸기`
- Date/time: `현재시간`, `현재날짜`, `타임스탬프`, `잠자기`
- System: `실행`, `환경변수`, `명령인자`

## Common methods

- Strings: `.분리`, `.포함`, `.바꾸기`, `.대문자`, `.소문자`, `.길이`, `.시작`, `.끝`
- Arrays: `.추가`, `.삭제`, `.길이`, `.포함`, `.정렬`, `.역순`, `.합치기`
- Maps: `.키목록`, `.값목록`, `.길이`, `.포함`, `.삭제`

## Historical aliases to avoid by default

Some older AI-facing materials may still mention older spellings. Prefer the current syntax above unless you are intentionally editing historical content.

- `처리`, not `실패`
- `맞춤`, not `맞춰`
- `포함`, not `가져오기`
- `HTTP_포함`, not `HTTP_가져오기`
- Prefer `만약 조건 이면 { ... }` in user-facing examples
