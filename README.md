# Han (한) Programming Language

> A general-purpose compiled language with Korean keywords — written in Rust

<p align="center">
  <strong>함수 · 만약 · 반복 · 변수 · 출력</strong><br>
  <em>Write code in one of the world's most beautifully designed writing systems.</em>
</p>

---

## About

Han is a statically-typed, compiled programming language where every keyword is written in Korean. It compiles to native binaries through LLVM IR and also ships with a tree-walking interpreter for instant execution. The compiler toolchain is written entirely in Rust.

Han was born from the idea that programming doesn't have to look the same in every country. Hangul — the Korean writing system — is one of the most scientifically designed scripts in human history, and Han puts it to work as a first-class programming language rather than just a display string.

---

## Features

- **Korean keywords** — `함수`, `만약`, `반복`, `변수` — write logic in Hangul
- **Hangul identifiers** — name your variables and functions in Korean
- **Compiled language** — generates LLVM IR → clang → native binary
- **Interpreter mode** — run instantly without clang
- **REPL** — interactive mode with `hgl repl`
- **LSP server** — `hgl lsp` for editor hover docs and completion
- **Static typing** — 5 primitive types: `정수` (int), `실수` (float), `문자열` (string), `불` (bool), `없음` (void)
- **Arrays** — `[1, 2, 3]`, indexing, negative indexing, `.추가/.삭제/.정렬/.역순` etc.
- **Structs** — `구조 사람 { 이름: 문자열 }` with field access and impl blocks
- **Closures** — `변수 f = 함수(x: 정수) { 반환 x * 2 }` with env capture
- **Pattern matching** — `맞춰 값 { 1 => ..., _ => ... }`
- **Error handling** — `시도 { } 실패(오류) { }` try/catch
- **File I/O** — `파일읽기`, `파일쓰기`, `파일추가`, `파일존재`
- **Format strings** — `형식("이름: {0}", 이름)` positional or `형식("이름: {이름}")` named
- **String methods** — `.분리`, `.포함`, `.바꾸기`, `.대문자`, `.소문자`, etc.
- **Module imports** — `가져오기 "파일.hgl"`
- **Generics syntax** — `함수 최대값<T>(a: T, b: T) -> T`
- **Built-in math** — `제곱근`, `절댓값`, `거듭제곱`, `정수변환`, `실수변환`, `길이`

---

## Quick Start

Create `hello.hgl`:

```
출력("안녕하세요, 세계!")
```

Run it:

```bash
hgl interpret hello.hgl
# Output: 안녕하세요, 세계!
```

Or jump into the REPL:

```bash
hgl repl
한> 출력("안녕!")
안녕!
```

---

## Practical Examples

### Word Counter

```
변수 텍스트 = "hello world hello han world hello"
변수 단어들 = 텍스트.분리(" ")
변수 단어목록 = []
변수 개수목록 = []

반복 변수 i = 0; i < 단어들.길이(); i += 1 {
    변수 찾음 = 거짓
    반복 변수 j = 0; j < 단어목록.길이(); j += 1 {
        만약 단어목록[j] == 단어들[i] {
            개수목록[j] = 개수목록[j] + 1
            찾음 = 참
        }
    }
    만약 찾음 == 거짓 {
        단어목록.추가(단어들[i])
        개수목록.추가(1)
    }
}

반복 변수 i = 0; i < 단어목록.길이(); i += 1 {
    출력(형식("{0}: {1}", 단어목록[i], 개수목록[i]))
}
```

```
hello: 3
world: 2
han: 1
```

### String Calculator

```
함수 계산(식: 문자열) -> 정수 {
    변수 부분 = 식.분리(" ")
    변수 왼쪽 = 정수변환(부분[0])
    변수 연산자 = 부분[1]
    변수 오른쪽 = 정수변환(부분[2])

    맞춰 연산자 {
        "+" => { 반환 왼쪽 + 오른쪽 }
        "-" => { 반환 왼쪽 - 오른쪽 }
        "*" => { 반환 왼쪽 * 오른쪽 }
        "/" => {
            만약 오른쪽 == 0 {
                출력("오류: 0으로 나눌 수 없습니다")
                반환 0
            }
            반환 왼쪽 / 오른쪽
        }
        _ => {
            출력(형식("알 수 없는 연산자: {0}", 연산자))
            반환 0
        }
    }
}

출력(계산("10 + 20"))     // 30
출력(계산("6 * 7"))       // 42
```

### Todo List with Structs

```
구조 할일 {
    제목: 문자열,
    완료: 불
}

변수 목록 = []

함수 추가하기(목록: [할일], 제목: 문자열) {
    목록.추가(할일 { 제목: 제목, 완료: 거짓 })
}

함수 완료처리(목록: [할일], index: 정수) {
    목록[index].완료 = 참
}

함수 출력목록(목록: [할일]) {
    반복 변수 i = 0; i < 목록.길이(); i += 1 {
        변수 상태 = "[ ]"
        만약 목록[i].완료 {
            상태 = "[✓]"
        }
        출력(형식("{0} {1}. {2}", 상태, i + 1, 목록[i].제목))
    }
}

추가하기(목록, "한글 프로그래밍 언어 만들기")
추가하기(목록, "README 작성하기")
추가하기(목록, "HN에 올리기")

완료처리(목록, 0)
완료처리(목록, 1)

출력("=== 할일 목록 ===")
출력목록(목록)
```

```
=== 할일 목록 ===
[✓] 1. 한글 프로그래밍 언어 만들기
[✓] 2. README 작성하기
[ ] 3. HN에 올리기
```

### File Line Counter

```
함수 줄수세기(경로: 문자열) -> 정수 {
    시도 {
        변수 내용 = 파일읽기(경로)
        변수 줄들 = 내용.분리("\n")
        반환 줄들.길이()
    } 실패(오류) {
        출력(형식("오류: {0}", 오류))
        반환 0
    }
}

파일쓰기("/tmp/test.txt", "첫번째 줄\n두번째 줄\n세번째 줄\n")
출력(형식("줄 수: {0}", 줄수세기("/tmp/test.txt")))
```

```
줄 수: 4
```

---

## Installation

### Prerequisites

- [Rust](https://rustup.rs) (1.70+)
- clang (for `hgl build` / `hgl run`) — `xcode-select --install` or `brew install llvm`

### Install

```bash
git clone https://github.com/xodn348/han.git
cd han
cargo install --path .
```

That's it. `hgl` is now available globally.

---

## CLI Usage

```
hgl interpret <file.hgl>    Run with interpreter (no clang needed)
hgl build <file.hgl>        Compile to native binary (requires clang)
hgl run <file.hgl>          Compile and run immediately
hgl repl                    Interactive REPL
hgl lsp                     Start LSP server (hover + completion)
```

---

## What Han Can Do Right Now

### ✅ Fully Working

**Data types**
- Integers (`정수`), floats (`실수`), strings (`문자열`), booleans (`불`)
- Arrays with negative indexing — `arr[-1]` returns the last element
- Structs with field access and mutation — `사람.이름 = "홍길동"`

**Control flow**
- `만약` / `아니면 만약` / `아니면` (if / else-if / else)
- `반복` for-loop with init, condition, step
- `동안` while-loop
- `멈춰` (break), `계속` (continue)
- `맞춰` pattern matching — integer, string, bool, wildcard `_`, binding

**Functions**
- Named functions with typed parameters and return types
- Recursion (fibonacci, factorial, etc.)
- Closures / anonymous functions with environment capture — `변수 f = 함수(x: 정수) { 반환 x * 2 }`
- Closures passed as arguments (without type annotation)

**Strings**
- Concatenation with `+`
- Methods: `.분리(sep)`, `.포함(s)`, `.바꾸기(from, to)`, `.앞뒤공백제거()`, `.대문자()`, `.소문자()`, `.시작(s)`, `.끝(s)`, `.길이()`
- Character indexing — `문자열[0]`

**Arrays**
- Methods: `.추가(v)`, `.삭제(i)`, `.길이()`, `.포함(v)`, `.역순()`, `.정렬()`, `.합치기(sep)`
- Index read/write — `arr[i]`, `arr[i] = v`
- Negative indexing — `arr[-1]`

**Structs & methods**
- Define: `구조 사람 { 이름: 문자열, 나이: 정수 }`
- Instantiate: `변수 p = 사람 { 이름: "홍길동", 나이: 30 }`
- Impl block methods with `자신` (self): `구현 사람 { 함수 인사(자신: 사람) { ... } }`

**Error handling**
- `시도 { } 실패(오류) { }` — catches any runtime error including division by zero, file not found, out-of-bounds

**File I/O**
- `파일읽기("path")` — reads whole file to string
- `파일쓰기("path", content)` — writes string to file
- `파일추가("path", content)` — appends to file
- `파일존재("path")` — returns bool

**Math builtins**
- `제곱근(x)`, `절댓값(x)`, `거듭제곱(밑, 지수)`
- `정수변환(x)`, `실수변환(x)`, `길이(s)`

**Format strings**
- Named: `형식("이름: {이름}, 나이: {나이}")` — substitutes from current scope
- Positional: `형식("이름: {0}, 나이: {1}", 이름, 나이)`

**Modules**
- `가져오기 "파일.hgl"` — runs another `.hgl` file in the current scope

**Generics syntax**
- `함수 최대값<T>(a: T, b: T) -> T` — type params are parsed and erased at runtime

---

### ⚠️ Partial / Edge Cases

| Feature | Status |
|---------|--------|
| Functions as typed parameters | Syntax not yet supported — `f: 함수` fails. Pass closures without type annotation. |
| `없음` as a literal value | Cannot write `변수 x = 없음` yet — parser doesn't handle `없음` as expression |
| Float + Int mixed arithmetic | No implicit coercion — `1 + 1.5` fails. Use `실수변환(1) + 1.5` |
| Nested struct field mutation | `a.b.c = v` not supported — only one level deep |
| Closures in `맞춰` arm | Works, but arm body must use `{ }` block syntax |
| `hgl build` with arrays/structs | Codegen stubs for arrays/structs — interpreter only for those features |

---

### ❌ Not Yet Implemented

| Feature | Notes |
|---------|-------|
| Multi-return / tuples | No tuple type yet |
| Enums | No `열거` keyword yet |
| Null safety / Option type | No `없음?` or Option<T> |
| Async / concurrency | Single-threaded only |
| Standard library: network, process | No HTTP, no subprocess |
| Garbage collection | Reference counting only via `Rc<RefCell<>>` — cycles leak |
| Tail call optimization | Deep recursion will stack overflow |

---

## Why Han?

<details>
<summary>The Beauty of Hangul</summary>

Hangul (한글) is not just a writing system — it is a feat of deliberate linguistic design. Created in 1443 by King Sejong the Great, each character encodes phonetic information in its geometric shape. Consonants mirror the tongue and mouth positions used to pronounce them. Vowels are composed from three cosmic symbols: heaven (·), earth (ㅡ), and human (ㅣ).

Han brings this elegance into programming. When you write `함수 피보나치(n: 정수) -> 정수`, you are not just defining a function — you are writing in a script that was purpose-built for clarity and beauty.

</details>

<details>
<summary>Riding the Korean Wave</summary>

The global interest in Korean culture has never been higher. From K-pop and Korean cinema to Korean cuisine and language learning, millions worldwide are engaging with Korean culture. Over 16 million people are actively studying Korean as a foreign language.

Han offers these learners something unexpected: a way to practice reading and writing Hangul through programming. It bridges the gap between cultural interest and technical skill, making Korean literacy functional in a domain where it has never existed before.

</details>

---

<details>
<summary><strong>Language Guide</strong></summary>

### Variables and Constants

```
변수 이름 = 42              // mutable variable
변수 메시지 = "안녕하세요"     // string variable
상수 파이 = 3.14            // immutable constant
```

With explicit type annotations:

```
변수 나이: 정수 = 25
변수 키: 실수 = 175.5
변수 이름: 문자열 = "홍길동"
변수 활성: 불 = 참
```

### Functions

```
함수 더하기(가: 정수, 나: 정수) -> 정수 {
    반환 가 + 나
}

함수 인사(이름: 문자열) {
    출력("안녕하세요, " + 이름)
}
```

### Conditionals

```
만약 점수 >= 90 {
    출력("A")
} 아니면 만약 점수 >= 80 {
    출력("B")
} 아니면 {
    출력("C")
}
```

### Loops

**For loop** (`반복`):

```
반복 변수 i = 0; i < 10; i += 1 {
    출력(i)
}
```

**While loop** (`동안`):

```
변수 n = 0
동안 n < 5 {
    출력(n)
    n += 1
}
```

**Loop control** — `멈춰` (break) and `계속` (continue):

```
반복 변수 i = 0; i < 100; i += 1 {
    만약 i == 50 {
        멈춰
    }
    만약 i % 2 == 0 {
        계속
    }
    출력(i)
}
```

</details>

<details>
<summary><strong>Example Programs</strong></summary>

### Factorial

```
함수 팩토리얼(n: 정수) -> 정수 {
    만약 n <= 1 {
        반환 1
    }
    반환 n * 팩토리얼(n - 1)
}

함수 main() {
    출력(팩토리얼(10))
}

main()
```

Output: `3628800`

### Sum 1 to 100

```
함수 합계(n: 정수) -> 정수 {
    변수 합 = 0
    반복 변수 i = 1; i <= n; i += 1 {
        합 += i
    }
    반환 합
}

함수 main() {
    출력(합계(100))
}

main()
```

Output: `5050`

### Even/Odd Checker

```
함수 main() {
    반복 변수 i = 1; i <= 10; i += 1 {
        만약 i % 2 == 0 {
            출력("짝수")
        } 아니면 {
            출력("홀수")
        }
    }
}

main()
```

</details>

<details>
<summary><strong>Keyword Reference</strong></summary>

| Keyword | Meaning | English Equivalent |
|---------|---------|-------------------|
| `함수` | function definition | `fn` / `function` |
| `반환` | return value | `return` |
| `변수` | mutable variable | `let mut` / `var` |
| `상수` | immutable constant | `const` |
| `만약` | conditional | `if` |
| `아니면` | else branch | `else` |
| `반복` | for loop | `for` |
| `동안` | while loop | `while` |
| `멈춰` | break loop | `break` |
| `계속` | continue loop | `continue` |
| `참` | boolean true | `true` |
| `거짓` | boolean false | `false` |
| `출력` | print to console | `print` |
| `입력` | read from console | `input` |

</details>

<details>
<summary><strong>Type System & Operators</strong></summary>

| Type | Description | LLVM Type | Examples |
|------|-------------|-----------|----------|
| `정수` | 64-bit integer | `i64` | `42`, `-10` |
| `실수` | 64-bit float | `f64` | `3.14`, `-0.5` |
| `문자열` | UTF-8 string | `i8*` | `"안녕하세요"` |
| `불` | boolean | `i1` | `참`, `거짓` |
| `없음` | void / no value | `void` | (function return type) |

| Operator | Description |
|----------|-------------|
| `+`, `-`, `*`, `/`, `%` | Arithmetic |
| `==`, `!=` | Equality |
| `<`, `>`, `<=`, `>=` | Comparison |
| `&&`, `\|\|`, `!` | Logical |
| `=`, `+=`, `-=`, `*=`, `/=` | Assignment |

</details>

<details>
<summary><strong>Design and Architecture</strong></summary>

### How Han Works

Han follows the classical compiler pipeline, implemented entirely in Rust with zero external compiler dependencies (LLVM IR is generated as plain text):

```
Source (.hgl)
    │
    ▼
┌─────────┐     ┌─────────┐     ┌─────────┐
│  Lexer  │ ──▶ │ Parser  │ ──▶ │   AST   │
│(lexer.rs)│    │(parser.rs)│   │ (ast.rs) │
└─────────┘     └─────────┘     └────┬────┘
                                     │
                        ┌────────────┼────────────┐
                        ▼                         ▼
                ┌──────────────┐         ┌──────────────┐
                │ Interpreter  │         │   CodeGen    │
                │(interpreter.rs)│       │ (codegen.rs) │
                └──────┬───────┘         └──────┬───────┘
                       │                        │
                       ▼                        ▼
                  Direct Output           LLVM IR (.ll)
                                               │
                                               ▼
                                         clang → Binary
```

### Project Structure

```
han/
├── src/
│   ├── main.rs          CLI entry point (hgl command)
│   ├── lexer.rs         Lexer: Korean source → token stream
│   ├── parser.rs        Parser: tokens → AST (recursive descent)
│   ├── ast.rs           AST node type definitions
│   ├── interpreter.rs   Tree-walking interpreter
│   ├── codegen.rs       LLVM IR text code generator
│   └── lsp.rs           LSP server (hover + completion)
├── examples/            Example .hgl programs
├── spec/
│   └── SPEC.md          Formal language specification (EBNF)
└── tests/               Integration tests
```

### Design Decisions

**Why text-based LLVM IR instead of the LLVM C API?**
Han generates LLVM IR as plain text strings, avoiding the complexity of linking against LLVM libraries. This keeps the build simple (`cargo build` — no LLVM installation required) while still producing optimized native binaries through clang.

**Why both interpreter and compiler?**
The interpreter enables instant execution without any toolchain dependencies beyond Rust. The compiler path exists for production use where performance matters. Same parser, same AST, two backends.

**Why Rust?**
Rust's enum types map naturally to AST nodes and token variants. Pattern matching makes parser and interpreter logic clear and exhaustive. Memory safety without garbage collection suits a language toolchain.

</details>

---

## Running Tests

```bash
cargo test
```

46 tests (41 unit + 5 integration) covering the lexer, parser, AST, interpreter, and code generator.

---

## License

MIT

---

<p align="center">
  <em>Han — where the beauty of Hangul meets the precision of code.</em>
</p>
