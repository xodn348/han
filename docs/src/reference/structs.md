# Structs

## Definition

```
구조 사람 {
    이름: 문자열,
    나이: 정수
}
```

## Instantiation

```
변수 홍길동 = 사람 { 이름: "홍길동", 나이: 30 }
```

## Field Access

```
출력(홍길동.이름)    // 홍길동
출력(홍길동.나이)    // 30
```

## Field Mutation

```
홍길동.나이 = 31
```

## Nested Structs

```
구조 주소 { 도시: 문자열 }
구조 직원 { 이름: 문자열, 주소: 주소 }

변수 p = 직원 { 이름: "김철수", 주소: 주소 { 도시: "서울" } }
출력(p.주소.도시)        // 서울
p.주소.도시 = "부산"     // nested mutation
```

## Impl Blocks (Methods)

```
구현 사람 {
    함수 소개(자신: 사람) {
        출력(형식("{0}, {1}세", 자신.이름, 자신.나이))
    }
}

홍길동.소개()    // 홍길동, 30세
```

`자신` is the self parameter — refers to the struct instance.
