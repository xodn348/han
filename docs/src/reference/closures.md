# Closures

## Anonymous Functions

```
변수 두배 = 함수(x: 정수) { 반환 x * 2 }
출력(두배(5))    // 10
```

## Environment Capture

Closures capture variables from their enclosing scope:

```
변수 배수 = 3
변수 곱하기 = 함수(x: 정수) { 반환 x * 배수 }
출력(곱하기(5))    // 15
```

## Passing as Arguments

```
함수 적용(f: 함수, x: 정수) -> 정수 {
    반환 f(x)
}

변수 제곱 = 함수(x: 정수) { 반환 x * x }
출력(적용(제곱, 4))    // 16
```
