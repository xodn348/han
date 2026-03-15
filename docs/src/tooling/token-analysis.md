# Token Analysis (AI/LLM)

## Benchmark Results

Tested with GPT-4o tokenizer (tiktoken), comparing the same Fibonacci program:

| Language | Tokens |
|----------|--------|
| Python | 54 |
| JavaScript | 69 |
| Han | 88 |

## Why Korean Uses More Tokens

LLM tokenizers use BPE (Byte Pair Encoding):

1. Start with raw bytes
2. Find the most frequent byte pairs in training data
3. Merge them into single tokens
4. Repeat

Since training data is predominantly English:
- `function` → appears billions of times → merged into 1 token
- `함수` → rarely appears → stays as 2-3 byte-level tokens

## Per-Keyword Comparison

| Han | Tokens | English | Tokens |
|-----|--------|---------|--------|
| `함수` | 2 | `function` | 1 |
| `반환` | 2 | `return` | 1 |
| `변수` | 2 | `let` | 1 |
| `아니면` | 3 | `else` | 1 |
| `멈춰` | 3 | `break` | 1 |
| `동안` | 1 | `while` | 1 |
| `참` | 1 | `true` | 1 |

## This Is a Tokenizer Problem, Not a Korean Problem

If BPE were trained on a Korean-heavy corpus, `함수` could be a single token. The inefficiency comes from training data distribution, not from the script itself.

Relevant work:
- Ukrainian LLM "Lapa" replaced 80K tokens and achieved 1.5x efficiency for Ukrainian text
- Custom BPE training on Korean programming text could close the gap
