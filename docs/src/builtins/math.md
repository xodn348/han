# Math Functions

| Function | Description | Example | Result |
|----------|-------------|---------|--------|
| `제곱근(x)` | Square root | `제곱근(16.0)` | `4.0` |
| `절댓값(x)` | Absolute value | `절댓값(-5)` | `5` |
| `거듭제곱(밑, 지수)` | Power | `거듭제곱(2, 10)` | `1024.0` |
| `사인(x)` | Sine | `사인(파이() / 2.0)` | `1.0` |
| `코사인(x)` | Cosine | `코사인(0.0)` | `1.0` |
| `탄젠트(x)` | Tangent |  |  |
| `로그(x)` | Natural log (ln) | `로그(자연상수())` | `1.0` |
| `로그10(x)` | Log base 10 |  |  |
| `지수(x)` | e^x | `지수(1.0)` | `2.718...` |
| `올림(x)` | Ceiling | `올림(2.3)` | `3.0` |
| `내림(x)` | Floor | `내림(2.9)` | `2.0` |
| `반올림(x)` | Round | `반올림(2.5)` | `3.0` |
| `최대(a, b)` | Maximum | `최대(3, 7)` | `7.0` |
| `최소(a, b)` | Minimum |  |  |
| `난수()` | Random 0~1 |  |  |
| `난수(a, b)` | Random int a~b | `난수(1, 10)` |  |
| `파이()` | Pi constant |  | `3.14159...` |
| `자연상수()` | Euler's number |  | `2.71828...` |
| `행렬곱(A, B)` | Matrix multiply | `행렬곱(A, B)` | `[[실수]]` |
| `전치(A)` | Matrix transpose | `전치(A)` | `[[실수]]` |
| `스칼라곱(A, s)` | Scalar multiply | `스칼라곱(A, 2.0)` | `[[실수]]` |
| `행렬합(A, B)` | Matrix addition | `행렬합(A, B)` | `[[실수]]` |
| `행렬차(A, B)` | Matrix subtraction | `행렬차(A, B)` | `[[실수]]` |
| `내적(a, b)` | Dot product | `내적([1,2], [3,4])` | `11.0` |
| `외적(a, b)` | Cross product (3D) | `외적([1,0,0], [0,1,0])` | `[0,0,1]` |
| `단위행렬(n)` | Identity matrix | `단위행렬(3)` | `[[1,0,0],...]` |
| `텐서곱(A, B)` | Tensor/Kronecker product | `텐서곱(A, B)` | `[[실수]]` |

All math functions accept both `정수` and `실수` inputs.

## Matrix Operations

`행렬곱` and `전치` work with 2D arrays (array of arrays):

```
변수 A = [[1.0, 2.0], [3.0, 4.0]]
변수 B = [[5.0, 6.0], [7.0, 8.0]]

변수 결과 = 행렬곱(A, B)
// [[19.0, 22.0], [43.0, 50.0]]

변수 T = 전치(A)
// [[1.0, 3.0], [2.0, 4.0]]
```

These can be used to implement algorithms like **self-attention** (`Attention(Q,K,V) = softmax(QK^T / √d_k) × V`). See `examples/어텐션.hgl` for a full implementation.
