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
