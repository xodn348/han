# CLI Usage

```
hgl interpret <file.hgl>    Run with interpreter
hgl build <file.hgl>        Compile to native binary
hgl run <file.hgl>          Compile and run immediately
hgl check <file.hgl>        Type-check only (no execution)
hgl init [name]             Create new Han project
hgl repl                    Interactive REPL
hgl lsp                     Start LSP server
```

## Examples

```bash
hgl interpret examples/피보나치.hgl
hgl check examples/합계.hgl
hgl build examples/합계.hgl && ./합계
hgl init hello-han
hgl repl
```
