# Installation

## Prerequisites

- [Rust](https://rustup.rs) 1.70+
- clang (for `hgl build` / `hgl run`) — optional for interpreter-only use

### macOS
```bash
xcode-select --install
```

### Linux
```bash
sudo apt install clang
```

## Install Han

For most Rust users, install Han from crates.io:

```bash
cargo install han-lang
```

The crates.io package is named `han-lang`. The language name remains **Han**, and the installed command-line tool remains `hgl`.

The default crates.io install does not enable Python interop, so the CLI does not require a local Python dynamic library. Enable it explicitly with `cargo install han-lang --features python` if you need the `파이썬()` / `파이썬_값()` built-ins.

To install from a local checkout instead:

```bash
git clone https://github.com/xodn348/han.git
cd han
cargo install --path .
```

After either install path, `hgl` is available globally.

## Verify Installation

```bash
hgl --help
```
