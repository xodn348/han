# Modules

## Importing Files

```
포함 "수학도구.hgl"
```

This executes the file and imports all its definitions (functions, variables, structs) into the current scope.

## Duplicate Includes

Han tracks imported files by canonical path and skips duplicate includes.

```hgl
포함 "utils.hgl"
포함 "./utils.hgl"   // same file -> skipped
```

This keeps include behavior idempotent for repeated module wiring.

## Example

`수학도구.hgl`:
```
함수 최대값(a: 정수, b: 정수) -> 정수 {
    만약 a > b 이면 { 반환 a }
    반환 b
}
```

`main.hgl`:
```
포함 "수학도구.hgl"
출력(최대값(10, 20))    // 20
```
