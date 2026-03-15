# Date & Time

Powered by Rust's `chrono` crate.

## Current Time

```
출력(현재시간())    // 2025-03-15 12:30:45
출력(현재날짜())    // 2025-03-15
출력(타임스탬프())  // 1710500000 (Unix timestamp)
```

## Functions

| Function | Return | Example |
|----------|--------|---------|
| `현재시간()` | `문자열` | `"2025-03-15 12:30:45"` |
| `현재날짜()` | `문자열` | `"2025-03-15"` |
| `타임스탬프()` | `정수` | `1710500000` |

`현재시간()` and `현재날짜()` use local time. `타임스탬프()` returns UTC Unix timestamp.
