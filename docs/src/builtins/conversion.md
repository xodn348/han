# Type Conversion

| Function | Description | Example | Result |
|----------|-------------|---------|--------|
| `정수변환(x)` | Convert to integer | `정수변환("42")` | `42` |
| `실수변환(x)` | Convert to float | `실수변환(42)` | `42.0` |
| `길이(s)` | String length | `길이("한글")` | `2` |

## Conversion Rules

```
정수변환(3.14)      // 3 (truncates)
정수변환("42")      // 42 (parse string)
정수변환(참)        // 1
정수변환(거짓)      // 0

실수변환(42)        // 42.0
실수변환("3.14")    // 3.14
```
