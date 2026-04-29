---
name: Bug report
about: Report a bug in the Han compiler, interpreter, or tooling
title: "[bug] "
labels: ["bug"]
---

### Description

A clear and concise description of the bug. What did you expect to happen,
and what actually happened?

### Reproduction steps

1. Save the following `.hgl` snippet as `repro.hgl`:

   ```hgl
   // minimal program that triggers the bug
   ```

2. Run:

   ```bash
   hgl interpret repro.hgl
   ```

3. Observe the failure described below.

### Expected behavior

What you expected `hgl` to do.

### Actual behavior

What `hgl` actually did. Paste any error output, stack trace, or
unexpected program output verbatim inside a fenced code block.

### Environment

- `hgl --version`:
- Operating system (and version):
- Install method (e.g. `bash install.sh`, built from source, prebuilt
  binary):
- Rust toolchain (`rustc --version`), if you built from source:

### Additional context

Anything else that might help — links to related issues, screenshots of
playground behavior, or notes about whether this used to work in a
previous version.
