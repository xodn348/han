# Examples Index

Han ships with a growing set of example programs in the [`examples/`](https://github.com/xodn348/han/tree/main/examples) directory of the repository. Each file is a self-contained `.hgl` program you can run with:

```bash
hgl interpret examples/<name>.hgl
```

## Tutorial Series (교육)

A 10-step progressive tutorial covering the language from basics to integration:

| File | Topic |
|------|-------|
| [교육_01_변수조건.hgl](https://github.com/xodn348/han/blob/main/examples/교육_01_변수조건.hgl) | Variables and conditionals |
| [교육_02_반복문.hgl](https://github.com/xodn348/han/blob/main/examples/교육_02_반복문.hgl) | Loops |
| [교육_03_함수.hgl](https://github.com/xodn348/han/blob/main/examples/교육_03_함수.hgl) | Functions |
| [교육_04_배열.hgl](https://github.com/xodn348/han/blob/main/examples/교육_04_배열.hgl) | Arrays |
| [교육_05_구조체.hgl](https://github.com/xodn348/han/blob/main/examples/교육_05_구조체.hgl) | Structs |
| [교육_06_클로저.hgl](https://github.com/xodn348/han/blob/main/examples/교육_06_클로저.hgl) | Closures |
| [교육_07_패턴매칭.hgl](https://github.com/xodn348/han/blob/main/examples/교육_07_패턴매칭.hgl) | Pattern matching |
| [교육_08_모듈.hgl](https://github.com/xodn348/han/blob/main/examples/교육_08_모듈.hgl) | Modules |
| [교육_09_에러처리.hgl](https://github.com/xodn348/han/blob/main/examples/교육_09_에러처리.hgl) | Error handling |
| [교육_10_종합.hgl](https://github.com/xodn348/han/blob/main/examples/교육_10_종합.hgl) | Integrated example |

## Classic Algorithms

| File | Topic |
|------|-------|
| [피보나치.hgl](https://github.com/xodn348/han/blob/main/examples/피보나치.hgl) | Fibonacci (recursive) |
| [피보나치_SOV.hgl](https://github.com/xodn348/han/blob/main/examples/피보나치_SOV.hgl) | Fibonacci with SOV `동안` form |
| [팩토리얼.hgl](https://github.com/xodn348/han/blob/main/examples/팩토리얼.hgl) | Factorial |
| [소수판별.hgl](https://github.com/xodn348/han/blob/main/examples/소수판별.hgl) | Prime number check |
| [정렬알고리즘.hgl](https://github.com/xodn348/han/blob/main/examples/정렬알고리즘.hgl) | Sorting algorithms |
| [합계.hgl](https://github.com/xodn348/han/blob/main/examples/합계.hgl) | Sum 1..n |
| [구구단.hgl](https://github.com/xodn348/han/blob/main/examples/구구단.hgl) | Multiplication table |
| [별찍기.hgl](https://github.com/xodn348/han/blob/main/examples/별찍기.hgl) | Star pattern printing |
| [짝홀.hgl](https://github.com/xodn348/han/blob/main/examples/짝홀.hgl) | Even/odd checker |

## Strings, Arrays, Structures

| File | Topic |
|------|-------|
| [문자열보간.hgl](https://github.com/xodn348/han/blob/main/examples/문자열보간.hgl) | String interpolation |
| [단어세기.hgl](https://github.com/xodn348/han/blob/main/examples/단어세기.hgl) | Word counter |
| [배열연산.hgl](https://github.com/xodn348/han/blob/main/examples/배열연산.hgl) | Array operations |
| [구조체메서드.hgl](https://github.com/xodn348/han/blob/main/examples/구조체메서드.hgl) | Struct methods |
| [패턴매칭.hgl](https://github.com/xodn348/han/blob/main/examples/패턴매칭.hgl) | Pattern matching |
| [범위반복.hgl](https://github.com/xodn348/han/blob/main/examples/범위반복.hgl) | Range iteration |
| [클로저고차.hgl](https://github.com/xodn348/han/blob/main/examples/클로저고차.hgl) | Higher-order closures |
| [할일목록.hgl](https://github.com/xodn348/han/blob/main/examples/할일목록.hgl) | Todo list with structs |
| [한글연산자.hgl](https://github.com/xodn348/han/blob/main/examples/한글연산자.hgl) | Korean logical operators |

## I/O and Systems

| File | Topic |
|------|-------|
| [파일처리.hgl](https://github.com/xodn348/han/blob/main/examples/파일처리.hgl) | File read/write |
| [에러처리.hgl](https://github.com/xodn348/han/blob/main/examples/에러처리.hgl) | `시도`/`처리` error handling |
| [정규식.hgl](https://github.com/xodn348/han/blob/main/examples/정규식.hgl) | Regex |
| [JSON처리.hgl](https://github.com/xodn348/han/blob/main/examples/JSON처리.hgl) | JSON parsing and generation |
| [HTTP_API.hgl](https://github.com/xodn348/han/blob/main/examples/HTTP_API.hgl) | HTTP GET/POST |
| [시스템정보.hgl](https://github.com/xodn348/han/blob/main/examples/시스템정보.hgl) | System / environment info |
| [온도변환.hgl](https://github.com/xodn348/han/blob/main/examples/온도변환.hgl) | Temperature conversion |

## Mini Apps

| File | Topic |
|------|-------|
| [계산기.hgl](https://github.com/xodn348/han/blob/main/examples/계산기.hgl) | String calculator |
| [가위바위보.hgl](https://github.com/xodn348/han/blob/main/examples/가위바위보.hgl) | Rock-paper-scissors |
| [어텐션.hgl](https://github.com/xodn348/han/blob/main/examples/어텐션.hgl) | Attention mechanism demo |
| [안녕.hgl](https://github.com/xodn348/han/blob/main/examples/안녕.hgl) | Hello world |

## Sample: Factorial

```
함수 팩토리얼(n: 정수) -> 정수 {
    만약 n <= 1 이면 {
        반환 1
    }
    반환 n * 팩토리얼(n - 1)
}

함수 main() {
    출력(팩토리얼(10))
}

main()
```

Output: `3628800`

## Sample: Sum 1 to 100

```
함수 합계(n: 정수) -> 정수 {
    변수 합 = 0
    반복 변수 i = 1; i <= n; i += 1 {
        합 += i
    }
    반환 합
}

함수 main() {
    출력(합계(100))
}

main()
```

Output: `5050`
