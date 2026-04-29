# Contributing to Han (한)

## Welcome

Thanks for considering a contribution to **Han (한)** — a small, experimental
Korean-syntax programming language. The project is young and the surface area
is still moving, so every issue, idea, and pull request genuinely helps shape
where it goes next. Whether you are a Korean speaker exploring Rust, a Rust
developer curious about non-English programming languages, or somewhere in
between, you are welcome here.

## Development setup

You will need a recent Rust toolchain and a C/C++ compiler (for linking the
LLVM-text codegen path).

1. **Install rustup** (Rust toolchain manager):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install clang**:

   - macOS: `xcode-select --install`
   - Debian / Ubuntu: `sudo apt install clang`
   - Fedora / RHEL: `sudo dnf install clang`

3. **Build and install the `hgl` CLI** from the repo root:

   ```bash
   bash install.sh
   ```

4. **Verify** the install:

   ```bash
   hgl --version
   ```

If `hgl --version` prints a version string, you're ready to hack.

## Project layout

A short tour of the repository:

```
src/                # Compiler: lexer, parser, typer, interpreter, codegen
examples/           # .hgl example programs (try `hgl interpret examples/hello.hgl`)
tests/              # Integration tests (cargo test)
editors/vscode/     # VS Code extension (syntax + LSP client)
web/                # Browser playground (WASM build of the interpreter)
docs/               # mdBook documentation source
```

Other top-level files of note: `CHANGELOG.md` (Keep a Changelog format),
`install.sh` (the recommended install path), and `Cargo.toml` (crate
metadata).

## Build, test, run

The standard Rust workflow applies:

```bash
cargo build                       # debug build
cargo test                        # full test suite
cargo clippy -- -D warnings       # lints (warnings treated as errors)
cargo fmt --check                 # formatting check
```

To run a Han program through the tree-walking interpreter:

```bash
hgl interpret examples/hello.hgl
```

Any `.hgl` file under `examples/` is fair game — they are also exercised by
CI, so they double as regression tests.

## Branch naming

Please use a short, hyphenated slug after the type prefix:

- `feat/<short-slug>` — new features
- `fix/<short-slug>` — bug fixes
- `refactor/<short-slug>` — internal refactors with no behavior change
- `docs/<short-slug>` — documentation-only changes
- `chore/<short-slug>` — build, CI, tooling, dependency bumps

Examples: `feat/socket-builtins`, `fix/empty-array-typecheck`,
`docs/playground-screenshot`.

## PR guidelines

A few habits keep reviews fast and friendly:

- **One logical change per PR.** Small PRs land faster and are easier to
  bisect later.
- **Update `CHANGELOG.md`** under an `## [Unreleased]` section using the
  [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) categories
  (`Added`, `Changed`, `Fixed`, `Removed`, `Tests`, etc.).
- **Add a test** for any user-visible behavior change. Parser/typer/interp
  changes belong in `tests/`; new `.hgl` examples can live in `examples/`.
- **Run the full local check before pushing**:

  ```bash
  cargo fmt && cargo clippy -- -D warnings && cargo test
  ```

- **Reference issue numbers** in the PR body when applicable
  (e.g. `Closes #42`).
- **Keep PR titles concise** (under 70 characters); use the body for
  details, screenshots, and design notes.

## Filing issues

We have three issue templates to make this easy:

- **Bug report** — something doesn't work the way it should.
- **Feature request** — something you wish Han had.
- **Language proposal** — a change to Korean keywords, syntax, or
  built-in functions.

Pick the one that fits at <https://github.com/xodn348/han/issues/new/choose>.

For bug reports especially, please include:

- The output of `hgl --version`
- Your operating system and version
- A **minimal** `.hgl` snippet that reproduces the problem

The more we can copy-paste-run, the faster we can help.

## Language proposals

Han's Korean syntax is the soul of the project — it's what makes the language
feel like Korean rather than English-with-translated-keywords. So changes to
keywords, operators, or syntactic forms (for example: `이면`, `그리고`, `또는`,
the pipe operator `|>`, string interpolation rules) deserve a discussion
before code.

Please **open a "language proposal" issue first** to talk through:

- What the change is and why it helps Korean speakers / learners
- How it interacts with existing syntax
- Whether it breaks any existing programs in `examples/` or `tests/`

Once a proposal lands, the implementing PR can move quickly. Trying to land
syntax changes via surprise PR usually results in long review cycles, so
filing the issue first really does save everyone time.

## Code of Conduct

Be respectful. Be constructive. Assume good faith — many contributors are
working in their second language. Harassment, personal attacks, and
discrimination are not welcome. We follow the spirit of the
[Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/);
report concerns by opening an issue or contacting a maintainer.

## Questions

If you're stuck, unsure where to start, or just want to talk through an idea:

- Open a regular issue with the `question` label, or
- Start a [GitHub Discussion](https://github.com/xodn348/han/discussions)
  if Discussions are enabled on the repo

We'd rather answer a "small" question than have you bounce off the project.
Welcome aboard.
