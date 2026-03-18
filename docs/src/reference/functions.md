# Functions

## Declaration

```
함수 더하기(가: 정수, 나: 정수) -> 정수 {
    반환 가 + 나
}
```

## Calling

```
변수 결과 = 더하기(3, 4)
출력(결과)    // 7
```

## No Return Type (void)

```
함수 인사(이름: 문자열) {
    출력(형식("안녕, {0}!", 이름))
}
```

## Recursion

```
함수 피보나치(n: 정수) -> 정수 {
    만약 n <= 1 이면 {
        반환 n
    }
    반환 피보나치(n - 1) + 피보나치(n - 2)
}
```

## Generics (syntax only)

```
함수 첫번째<T>(arr: [T]) -> T {
    반환 arr[0]
}
```

Type parameters are parsed but erased at runtime.

## Function Type Parameter

```
함수 적용(f: 함수, x: 정수) -> 정수 {
    반환 f(x)
}
```
