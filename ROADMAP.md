# Han (한) Roadmap

Han is an experimental Korean-syntax programming language. This roadmap is
intentionally tentative: it captures direction and priorities, not promises.
Versions and dates will move as we learn what's worth building. Treat this
as a snapshot of the maintainers' current thinking, and please push back
(via a Language Proposal or Feature Request issue) if you see a better
path.

## Now (v0.2.x)

The current focus is hardening what already exists rather than chasing new
surface area:

- **Compiler + interpreter stability.** Tighten the parser, typer, and
  tree-walking interpreter against the existing test suite and `examples/`
  programs. Squash panics into proper Korean error messages.
- **Standard library coverage.** Fill obvious gaps in math, string, and
  array helpers that real programs keep bumping into.
- **Tooling polish.** Smoother LSP behavior, a better-feeling VS Code
  extension, and a faster, friendlier web playground.

If you're looking for a good first issue, this is the band to look at.

## v0.3 — Standard library expansion

Once v0.2.x feels solid, the next theme is making Han more useful for real
programs:

- **Networking primitives** beyond basic HTTP — TCP/UDP sockets, basic
  async I/O, and a few small clients (DNS, WebSocket).
- **Filesystem helpers** — directory walk, glob matching, path
  manipulation, atomic writes.
- **Richer string and array methods** — closing the gap with what users
  expect from Python / JavaScript / Rust standard libraries.
- **Better error types** — structured error values that play well with
  `try`/`catch` and pattern matching.

## v0.4 — Modules and packages

Han currently has a basic module-import story. v0.4 is about graduating
that to something a real ecosystem can grow on:

- **Module system improvements.** Proper namespacing, clearer visibility
  rules, and predictable import resolution.
- **Package distribution.** Either a first-pass package manager (registry +
  manifest), or an import-by-URL story (à la Deno) — to be decided based on
  community feedback.
- **Versioned standard library.** Lock the stdlib surface to the toolchain
  version so programs don't silently break on upgrades.

## v1.0 — Stabilization

The 1.0 milestone is where Han becomes a language you can build on without
worrying about the rug:

- **Language spec freeze.** A written spec for the Korean syntax,
  semantics, and standard library that we commit to keeping
  backward-compatible.
- **crates.io publication** for the toolchain (`hgl`, the LSP, the
  interpreter library) so installation is a single `cargo install`.
- **Long-term-support guarantees** — at minimum, security and
  serious-bug fixes for the previous minor version.

## Long-term experiments

These are the "if it works, that would be amazing" ideas — not on any
particular schedule:

- **Self-hosting compiler.** Han compiling Han.
- **Deeper IDE integrations.** JetBrains plugin, Neovim plugin, richer
  semantic features (rename, code actions, refactors).
- **Educational materials in Korean.** Curriculum aimed at K-12 and
  Korean university CS courses, leveraging the fact that Han reads like
  Korean prose.
- **WASM-first compilation target.** Make the browser the default,
  zero-install runtime — playground, classrooms, and embedded use cases
  all benefit.

---

Have ideas? Open an issue using the **Language Proposal** or **Feature
Request** template — see [CONTRIBUTING.md](./CONTRIBUTING.md).
