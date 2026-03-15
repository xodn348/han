# Enums

## Definition

```
열거 방향 {
    위,
    아래,
    왼쪽,
    오른쪽
}
```

## Access

Variants are accessed with `::`:

```
출력(방향::위)       // 0
출력(방향::아래)     // 1
출력(방향::오른쪽)   // 3
```

Variants are integer values starting from 0.

## Pattern Matching with Enums

```
맞춰 방향::아래 {
    0 => 출력("위")
    1 => 출력("아래")
    _ => 출력("기타")
}
```
