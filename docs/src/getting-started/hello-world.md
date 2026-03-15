# Hello World

Create a file called `hello.hgl`:

```
출력("안녕하세요, 세계!")
```

Run it:

```bash
hgl interpret hello.hgl
```

Output:
```
안녕하세요, 세계!
```

## With a Function

```
함수 main() {
    출력("Hello from Han!")
}

main()
```

## Compile to Binary

```bash
hgl build hello.hgl    # creates ./hello binary
./hello                 # runs natively
```
