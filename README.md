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

## Why Han?

### The Beauty of Hangul

Hangul (한글) is not just a writing system — it is a feat of deliberate linguistic design. Created in 1443 by King Sejong the Great, each character encodes phonetic information in its geometric shape. Consonants mirror the tongue and mouth positions used to pronounce them. Vowels are composed from three cosmic symbols: heaven (·), earth (ㅡ), and human (ㅣ).

Han brings this elegance into programming. When you write `함수 피보나치(n: 정수) -> 정수`, you are not just defining a function — you are writing in a script that was purpose-built for clarity and beauty.

### Token Efficiency

Korean keywords are remarkably dense. A single Hangul syllable block packs an initial consonant, a vowel, and an optional final consonant into one character. This means Han keywords carry more semantic weight per token than their English equivalents:

| Han | English | Tokens (approx.) |
|-----|---------|-------------------|
| `함수` | `function` | 1 vs 1-2 |
| `만약` | `if` | 1 vs 1 |
| `변수` | `let mut` / `var` | 1 vs 1-2 |
| `반환` | `return` | 1 vs 1 |
| `동안` | `while` | 1 vs 1 |
| `아니면` | `else` | 1 vs 1 |

In AI-assisted development workflows where token budgets matter — code generation, LLM context windows, prompt engineering — fewer tokens per keyword means more room for actual logic.

### Riding the Korean Wave

The global interest in Korean culture has never been higher. From K-pop and Korean cinema to Korean cuisine and language learning, millions worldwide are engaging with Korean culture. Over 16 million people are actively studying Korean as a foreign language.

Han offers these learners something unexpected: a way to practice reading and writing Hangul through programming. It bridges the gap between cultural interest and technical skill, making Korean literacy functional in a domain where it has never existed before.

---

## Features

- **Korean keywords** — `함수`, `만약`, `반복`, `변수` — write logic in Hangul
- **Hangul identifiers** — name your variables and functions in Korean
- **Compiled language** — generates LLVM IR → clang → native binary
- **Interpreter mode** — run instantly without clang
- **Static typing** — 5 primitive types: `정수` (int), `실수` (float), `문자열` (string), `불` (bool), `없음` (void)
- **Dual execution** — choose between compilation for performance or interpretation for convenience

---

## Installation

### Prerequisites

- [Rust](https://rustup.rs) (1.70+)
- clang (for compilation mode) — `xcode-select --install` or `brew install llvm`

### Build from Source

```bash
git clone https://github.com/xodn348/han.git
cd han
cargo build --release
```

The binary is at `./target/release/hgl`. To install system-wide:

```bash
cp ./target/release/hgl /usr/local/bin/hgl
```

---

## Quick Start

### Hello World

Create `hello.hgl`:

```
함수 main() {
    출력("안녕하세요, 세계!")
}

main()
```

Run it:

```bash
hgl interpret hello.hgl
# Output: 안녕하세요, 세계!
```

---

## CLI Usage

```
hgl interpret <file.hgl>    Run with interpreter (no clang needed)
hgl build <file.hgl>        Compile to native binary (requires clang)
hgl run <file.hgl>          Compile and run immediately
```

### Examples

```bash
# Interpret (fastest way to start)
hgl interpret examples/피보나치.hgl

# Compile to binary
hgl build examples/피보나치.hgl
./피보나치

# Compile and run
hgl run examples/팩토리얼.hgl
```

---

## Language Guide

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
    출력("A 학점")
} 아니면 {
    출력("B 학점 이하")
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

---

## Example Programs

### Fibonacci Sequence

```
함수 피보나치(n: 정수) -> 정수 {
    만약 n <= 1 {
        반환 n
    }
    반환 피보나치(n - 1) + 피보나치(n - 2)
}

함수 main() {
    출력(피보나치(10))
}

main()
```

Output: `55`

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

---

## Keyword Reference

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

## Type System

| Type | Description | LLVM Type | Examples |
|------|-------------|-----------|----------|
| `정수` | 64-bit integer | `i64` | `42`, `-10` |
| `실수` | 64-bit float | `f64` | `3.14`, `-0.5` |
| `문자열` | UTF-8 string | `i8*` | `"안녕하세요"` |
| `불` | boolean | `i1` | `참`, `거짓` |
| `없음` | void / no value | `void` | (function return type) |

## Operators

| Operator | Description |
|----------|-------------|
| `+`, `-`, `*`, `/`, `%` | Arithmetic |
| `==`, `!=` | Equality |
| `<`, `>`, `<=`, `>=` | Comparison |
| `&&`, `\|\|`, `!` | Logical |
| `=` | Assignment |
| `+=`, `-=`, `*=`, `/=` | Compound assignment |

---

## Design and Architecture

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
│   └── codegen.rs       LLVM IR text code generator
├── examples/            Example .hgl programs
├── spec/
│   └── SPEC.md          Formal language specification (EBNF)
├── docs/                Documentation
└── tests/               Integration tests
```

### Design Decisions

**Why text-based LLVM IR instead of the LLVM C API?**
Han generates LLVM IR as plain text strings, avoiding the complexity of linking against LLVM libraries. This keeps the build simple (`cargo build` — no LLVM installation required) while still producing optimized native binaries through clang.

**Why both interpreter and compiler?**
The interpreter enables instant execution without any toolchain dependencies beyond Rust. The compiler path exists for production use where performance matters. Same parser, same AST, two backends.

**Why Rust?**
Rust's enum types map naturally to AST nodes and token variants. Pattern matching makes parser and interpreter logic clear and exhaustive. Memory safety without garbage collection suits a language toolchain.

---

## Running Tests

```bash
cargo test
```

46 tests (41 unit + 5 integration) covering the lexer, parser, AST, interpreter, and code generator.

---

## Roadmap

- [x] Symbol table and type tracking in codegen
- [x] `아니면 만약` (else-if) chaining
- [x] REPL mode (`hgl repl`)
- [x] Standard library expansion (math functions, type conversion)
- [x] Error messages with line/column info
- [x] String concatenation in codegen
- [x] Integration test harness
- [ ] Array/list types (`[정수]`, `[문자열]`)
- [ ] Structs (`구조`)
- [ ] Module/import system
- [ ] Pattern matching (`맞춰`)
- [ ] Error handling (`시도` / `실패`)
- [ ] LSP server for editor support

---

## License

MIT

---

<p align="center">
  <em>Han — where the beauty of Hangul meets the precision of code.</em>
</p>
