# Complete API Reference

This page is optimized for LLM consumption. It contains every keyword, type, builtin, method, and operator in Han in a structured, machine-readable format.

## Language: Han (한)
## File Extension: .hgl
## Execution: `hgl interpret <file>`, `hgl build <file>`, `hgl check <file>`, or `hgl init [name]`

---

## KEYWORDS

```
함수    → function definition
반환    → return
변수    → mutable variable (let)
상수    → immutable constant (const)
만약    → if
이면    → then (conditional marker)
아니면  → else
그리고  → logical and
또는    → logical or
반복    → for loop
동안    → while loop
멈춰    → break
계속    → continue
안에서  → in (for-in iteration)
구조    → struct definition
구현    → impl block
열거    → enum definition
시도    → try
처리    → catch
맞춤    → match (pattern matching)
포함    → import
참      → true
거짓    → false
없음    → null/void
```

## TYPES

```
정수    → i64 (64-bit integer)
실수    → f64 (64-bit float)
문자열  → String (UTF-8)
불      → bool
없음    → void
[정수]  → Array of integers
(정수, 문자열) → Tuple
```

## OPERATORS

```
+  -  *  /  %           → arithmetic
== !=  <  >  <=  >=     → comparison
&& || !                 → logical
그리고 또는              → Korean logical aliases
=  +=  -=  *=  /=       → assignment
->                      → return type arrow
=>                      → match arm arrow
..                      → range (0..10)
::                      → enum access (방향::위)
.                       → field/method access
```

## BUILTIN FUNCTIONS

```
출력(값...)              → print to stdout
입력()                   → read line from stdin → 문자열
출력오류(값...)           → print to stderr
제곱근(x)                → sqrt(x) → 실수
절댓값(x)                → abs(x) → 숫자
거듭제곱(밑, 지수)        → pow(base, exp) → 실수
정수변환(x)              → int(x) → 정수
실수변환(x)              → float(x) → 실수
길이(s)                  → len(s) → 정수
행렬곱(A, B)             → matrix multiply → [[실수]]
전치(A)                  → matrix transpose → [[실수]]
형식(template, args...)  → format string / interpolation target → 문자열
파일읽기(경로)            → read file → 문자열
파일쓰기(경로, 내용)      → write file
파일추가(경로, 내용)      → append to file
파일존재(경로)            → file exists → 불
사전(키, 값, ...)        → create HashMap → 사전
제이슨_파싱(문자열)       → parse JSON → value
제이슨_생성(값)           → value → JSON string
제이슨_예쁘게(값)         → value → pretty JSON string
HTTP_포함(url)        → GET request → 문자열
HTTP_보내기(url, body)    → POST request → 문자열
정규식_찾기(패턴, 텍스트)  → find matches → [문자열]
정규식_일치(패턴, 텍스트)  → test match → 불
정규식_바꾸기(패턴, 텍스트, 대체) → replace → 문자열
현재시간()               → current datetime → 문자열
현재날짜()               → current date → 문자열
타임스탬프()             → unix timestamp → 정수
실행(명령어)             → shell command → 문자열
환경변수(이름)           → env var → 문자열 or 없음
명령인자()               → CLI args → [문자열]
잠자기(밀리초)           → sleep
타입(값)                 → type name → 문자열
```

## MAP METHODS

```
.키목록()        → all keys → [T]
.값목록()        → all values → [T]
.길이()          → entry count → 정수
.포함(키)        → key exists → 불
.삭제(키)        → remove entry → removed value
```

## ARRAY METHODS

```
.추가(값)        → push element
.삭제(인덱스)    → remove at index → removed value
.길이()          → length → 정수
.포함(값)        → contains → 불
.역순()          → reversed copy → [T]
.정렬()          → sorted copy → [T]
.합치기(구분자)  → join to string → 문자열
```

## STRING METHODS

```
.길이()          → character count → 정수
.분리(구분자)    → split → [문자열]
.포함(부분)      → contains → 불
.바꾸기(전, 후)  → replace → 문자열
.앞뒤공백제거()  → trim → 문자열
.대문자()        → uppercase → 문자열
.소문자()        → lowercase → 문자열
.시작(접두사)    → starts with → 불
.끝(접미사)      → ends with → 불
```

## SYNTAX PATTERNS

```
// Variable declaration
변수 이름 = 값
변수 이름: 타입 = 값
상수 이름 = 값

// Function
함수 이름(매개변수: 타입) -> 반환타입 {
    반환 값
}

// If/else (Korean-default)
만약 조건 이면 {
    ...
} 아니면 조건2 이면 {
    ...
} 아니면 {
    ...
}

// Older minimal form is still supported, but docs prefer the `이면` pattern.

// For loop
반복 변수 i = 0; i < n; i += 1 {
    ...
}

// For-in
반복 항목 안에서 배열 {
    ...
}

// While (SOV default)
조건 동안 {
    ...
}

// While (SVO alternative)
동안 조건 {
    ...
}

// Struct
구조 이름 {
    필드: 타입,
    필드2: 타입
}

// Impl
구현 구조체이름 {
    함수 메서드(자신: 구조체이름) {
        ...
    }
}

// Enum
열거 이름 {
    변형1,
    변형2
}

// Match
맞춤 값 {
    패턴1 => 결과1
    패턴2 => { ... }
    _ => 기본값
}

// Try/catch
시도 {
    ...
} 처리(오류변수) {
    ...
}

// Import
포함 "파일.hgl"

// Closure
변수 f = 함수(x: 정수) { 반환 x * 2 }

// Tuple
변수 t = (1, "hello", 참)
t.0  // 1

// Range
0..10  // [0, 1, 2, ..., 9]

// Array
변수 arr = [1, 2, 3]
arr[0]      // indexing
arr[-1]     // negative indexing
arr[0] = 99 // mutation
```
