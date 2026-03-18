# Control Flow

## If / Else-If / Else

SOV (Korean natural order):

```hgl
점수 >= 90 만약 {
    출력("A")
} 아니면 점수 >= 80 만약 {
    출력("B")
} 아니면 {
    출력("C")
}
```

SVO (traditional alternative):

```hgl
만약 점수 >= 90 {
    출력("A")
} 아니면 만약 점수 >= 80 {
    출력("B")
} 아니면 {
    출력("C")
}
```

## Logical Operators

```hgl
만약 로그인됨 그리고 관리자 {
    출력("관리자 메뉴")
}

만약 오프라인 또는 점검중 {
    출력("잠시 후 다시 시도하세요")
}
```

## For Loop

```
반복 변수 i = 0; i < 10; i += 1 {
    출력(i)
}
```

## For-In Loop

Iterate over arrays:
```
반복 과일 안에서 ["사과", "배", "포도"] {
    출력(과일)
}
```

Iterate over strings:
```
반복 글자 안에서 "한글" {
    출력(글자)    // 한, 글
}
```

Iterate over ranges:
```
반복 i 안에서 0..5 {
    출력(i)    // 0, 1, 2, 3, 4
}
```

## While Loop

SOV:

```hgl
변수 n = 0
n < 5 동안 {
    출력(n)
    n += 1
}
```

SVO (traditional alternative):

```hgl
변수 n = 0
동안 n < 5 {
    출력(n)
    n += 1
}
```

## Break and Continue

```
반복 i 안에서 0..100 {
    만약 i == 50 { 멈춰 }
    만약 i % 2 == 0 { 계속 }
    출력(i)
}
```

## Range Operator

```
변수 범위 = 0..10     // creates [0, 1, 2, ..., 9]
변수 길이 = 범위.길이()  // 10
```
