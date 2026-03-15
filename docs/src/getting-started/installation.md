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

```bash
git clone https://github.com/xodn348/han.git
cd han
cargo install --path .
```

`hgl` is now available globally.

## Verify Installation

```bash
hgl --help
```
