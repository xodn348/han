---
name: han-language
description: Write, read, explain, debug, review, translate, or generate Han (.hgl) code — a programming language with Korean/Hangul keywords. Use when the task mentions Han, 한, Hangul syntax, Korean programming keywords, or .hgl files, or when working on the Han compiler/docs/examples repository.
metadata:
  author: xodn348
  version: "1.0.0"
---

# Han Language

Han is a compiled/interpreted programming language whose syntax is written in Korean.

Prefer the current keyword set used by the compiler and docs:
- conditionals: `만약 조건 이면 { }`
- catch blocks: `처리(오류)`
- pattern matching: `맞춤 값 { ... }`
- imports/includes: `포함 "파일.hgl"`
- HTTP GET: `HTTP_포함(url)`

## Use when

- The user wants Han or `.hgl` code written, explained, translated, reviewed, or debugged
- The task involves Korean/Hangul programming syntax rather than ordinary Korean prose
- The task modifies the Han compiler, interpreter, docs, examples, REPL, or tooling

## Do not use when

- The request is only natural-language Korean translation with no code
- The task is about Hangul text but not the Han language or `.hgl` files

## Workflow

1. For language usage, read `references/language-reference.md`.
2. For repository work, also read `references/repo-workflow.md`.
3. Mirror existing style from nearby `.hgl` examples when possible.
4. Preserve Hangul keywords exactly and keep source files UTF-8.
5. Prefer the docs-default syntax with `이면`, even if older minimal forms still parse.

## Validation

If you are working inside the Han repository, prefer these commands:

```bash
cargo run -- check path/to/file.hgl
cargo run -- interpret path/to/file.hgl
cargo run -- build path/to/file.hgl
cargo test
```

Use the globally installed `hgl` binary only when the repository is not available.

## Notes

- If repository docs and implementation disagree, trust `src/lexer.rs`, `src/parser.rs`, and the current examples over stale references.
- Explain both the Korean keyword and the English meaning when teaching the language.
- When translating from another language, preserve semantics first and then make the Han code idiomatic.
