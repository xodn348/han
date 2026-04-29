---
name: Language proposal
about: Propose a change to Han's Korean syntax, keywords, or built-in functions
title: "[lang] "
labels: ["language-proposal"]
---

### Summary

A one-paragraph description of the proposed change. What new keyword,
syntax form, or built-in are you proposing, and what does it do?

### Motivation

Why would Korean speakers and learners benefit from this? Han's value
comes from feeling like Korean rather than English-with-translated-keywords,
so please ground the motivation in how the change reads and teaches.
Concrete user stories ("a beginner trying to express X currently has to
write Y, which doesn't sound natural") are very welcome.

### Proposed syntax

Show the proposal with `.hgl` code blocks. Before / after pairs are
ideal.

**Before** (current syntax):

```hgl
// how this is written today
```

**After** (with the proposal):

```hgl
// how this would look with the change
```

### Compatibility impact

Does this break any existing programs?

- Does it conflict with an existing keyword or identifier?
- Would programs in `examples/` or `tests/` need to be updated?
- Is there a migration path for existing code?

### Alternatives

Other syntactic forms or keyword choices you considered, and why you
didn't pick them. Naming choices in Korean often have several
candidates — list them so reviewers can weigh in.

### References / prior art

Links to similar features in other languages (Python, Rust, Swift,
Elixir, other localized languages, etc.) or relevant linguistic
discussion. Prior art makes proposals much easier to evaluate.
