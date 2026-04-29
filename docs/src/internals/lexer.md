# Lexer & Token Analysis

This page covers Han's lexer (the `lexer.rs` stage that turns Korean source text into a token stream) and how Korean source code interacts with modern AI/LLM tokenizers.

## Token Analysis (AI/LLM)

We tested Han code against Python and JavaScript using GPT-4o's tokenizer (tiktoken):

| Program | Han | Python | JavaScript |
|---------|-----|--------|------------|
| Fibonacci | 88 tokens | 54 tokens | 69 tokens |

**Han uses more tokens, not fewer.** Korean keywords average 2-3 tokens each vs 1 for English. This is because BPE (Byte Pair Encoding) tokenizers are trained on English-dominant data — `function` appears billions of times and merges into a single token, while `함수` is rare and gets split into byte-level pieces.

This is a tokenizer training bias, not a property of Korean. If BPE were trained on Korean-heavy data, `함수` could easily be a single token.

Relevant discussion: [Ukrainian LLM Lapa replaced 80K tokens and achieved 1.5x efficiency](https://news.ycombinator.com/item?id=47381382)

## Why this matters for Han

This finding is part of Han's mission. From the project README:

> LLMs are trained on English-dominant data. BPE tokenizers treat Korean characters as rare, splitting `함수` into multiple byte-level tokens while `function` becomes one. The more Korean code exists on the internet — in repos, in documentation, in examples — the better future tokenizers will represent the Korean language. Han is a small contribution to that corpus.

The current cost is real (more tokens per request when an LLM reads Han), but the long-term benefit accrues to every Korean-language application — not just programming languages.
