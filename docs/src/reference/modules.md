# Modules

## Importing Files

```
가져오기 "수학도구.hgl"
```

This executes the file and imports all its definitions (functions, variables, structs) into the current scope.

## Example

`수학도구.hgl`:
```
함수 최대값(a: 정수, b: 정수) -> 정수 {
    만약 a > b { 반환 a }
    반환 b
}
```

`main.hgl`:
```
가져오기 "수학도구.hgl"
출력(최대값(10, 20))    // 20
```
