# Han Language — VS Code Extension

Syntax highlighting and LSP support for the [Han programming language](https://github.com/xodn348/han).

## Features

- Syntax highlighting for `.hgl` files
- Keyword, type, builtin function, string, number, comment highlighting
- Auto-closing brackets and quotes
- LSP integration (hover docs + completion) via `hgl lsp`

## Install from source

```bash
cd editors/vscode
npm install
npm run compile
```

Then press `F5` in VS Code to launch a development instance with the extension loaded.

## LSP Setup

Make sure `hgl` is in your PATH:

```bash
cd /path/to/han
cargo install --path .
```

The extension will automatically start `hgl lsp` when you open a `.hgl` file.
