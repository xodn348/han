# Complete API Reference

This page is optimized for LLM consumption. It contains every keyword, type, builtin, method, and operator in Han in a structured, machine-readable format.

## Language: Han (한)
## File Extension: .hgl
## Execution: `hgl interpret <file>` or `hgl build <file>`

---

## KEYWORDS

```
함수    → function definition
반환    → return
변수    → mutable variable (let)
상수    → immutable constant (const)
만약    → if
아니면  → else
반복    → for loop
동안    → while loop
멈춰    → break
계속    → continue
안에서  → in (for-in iteration)
구조    → struct definition
구현    → impl block
열거    → enum definition
시도    → try
실패    → catch
맞춰    → match (pattern matching)
가져오기 → import
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
&& ||  !                → logical
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
형식(template, args...)  → format string → 문자열
파일읽기(경로)            → read file → 문자열
파일쓰기(경로, 내용)      → write file
파일추가(경로, 내용)      → append to file
파일존재(경로)            → file exists → 불
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

// If/else
만약 조건 {
    ...
} 아니면 만약 조건2 {
    ...
} 아니면 {
    ...
}

// For loop
반복 변수 i = 0; i < n; i += 1 {
    ...
}

// For-in
반복 항목 안에서 배열 {
    ...
}

// While
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
맞춰 값 {
    패턴1 => 결과1
    패턴2 => { ... }
    _ => 기본값
}

// Try/catch
시도 {
    ...
} 실패(오류변수) {
    ...
}

// Import
가져오기 "파일.hgl"

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
