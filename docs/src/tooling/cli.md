# CLI Usage

```
hgl interpret <file.hgl>    Run with interpreter
hgl build <file.hgl>        Compile to native binary
hgl run <file.hgl>          Compile and run immediately
hgl repl                    Interactive REPL
hgl lsp                     Start LSP server
```

## Examples

```bash
hgl interpret examples/피보나치.hgl
hgl build examples/합계.hgl && ./합계
hgl repl
```
