# Han 언어 전체 고도화

## TL;DR

> **Quick Summary**: Han 프로그래밍 언어의 타입 체커 강화, codegen 완전 구현, stdlib 모듈화, 에러 통일, docs/README 업데이트
> 
> **Deliverables**:
> - 타입 체커: 함수 인자, 구조체 필드, 배열 요소 타입 검증 (경고 모드)
> - codegen: 클로저, 메서드, 열거, 임포트, Range, ForIn 전부 실제 LLVM IR
> - stdlib: interpreter.rs에서 빌트인 함수 모듈 분리
> - 에러 메시지 포맷 통일
> - docs 사이트 + README 반영
> 
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: CI → 타입 체커 → codegen 스텁→에러 → codegen 구현 → stdlib → docs → README

---

## Context

### Original Request
Han 프로그래밍 언어 전체 고도화 — 타입 체커, codegen, stdlib, 에러, docs, README 전부.

### Interview Summary
**Key Discussions**:
- codegen: 전부 구현 (클로저, 메서드, 열거, 임포트 포함)
- 타입 체커: 경고 모드 (기존 코드 안 깨뜨림)
- AI에게 DOCS 주고 구현 시킬 예정

**Current State**:
- 46 tests pass (41 unit + 5 integration)
- interpreter: 1757줄, 완전 작동
- codegen: 905줄, 스텁 다수 (MethodCall, Lambda, Tuple, Map, Enum, Impl, Import)
- typechecker: 236줄, 기초만 (변수 선언 + 반환 타입)
- 에러 메시지: 대부분 한국어, 포맷 불일치

### Metis Review
**Identified Gaps** (addressed):
- codegen TryCatch가 가짜 — 두 블록 순차 실행만 함
- codegen ForIn이 가짜 — 반복 로직 없음
- types_compatible가 배열 요소 타입 무시
- ForIn 변수를 항상 정수로 추론
- codegen 문자열이 UTF-8 바이트 기반 (한글 3바이트 문제)
- CI 테스트 워크플로 없음

---

## Work Objectives

### Core Objective
Han 컴파일러 파이프라인 전체를 production-ready 수준으로 끌어올리기

### Concrete Deliverables
- `src/typechecker.rs` 강화 (경고 모드)
- `src/codegen.rs` 스텁 전부 실제 구현
- `src/builtins/` 디렉토리 생성 및 모듈화
- 에러 메시지 포맷 통일
- `docs/src/` 페이지 업데이트
- `README.md` 반영

### Definition of Done
- [ ] `cargo test` — 기존 46개 + 새 테스트 전부 통과
- [ ] `hgl build examples/*.hgl` — 클로저, 메서드, 열거 포함 예제가 실제 바이너리로 컴파일
- [ ] `hgl interpret` 와 `hgl build` 결과가 동일
- [ ] docs 사이트 빌드 성공 (`cd docs && mdbook build`)

### Must Have
- 기존 46개 테스트 깨지지 않음
- codegen 스텁 전부 제거 (더미 `add nsw i64 0, 0` 없음)
- 타입 체커 경고 모드 (exit 안 함)

### Must NOT Have (Guardrails)
- parser.rs, ast.rs, lexer.rs 변경 금지
- interpreter 동작 변경 금지 (reference implementation)
- 새 언어 기능/문법 추가 금지
- WASM/playground 코드 변경 금지
- codegen 아키텍처 변경 금지 (string-based LLVM IR 유지)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: YES (TDD)
- **Framework**: `cargo test`
- **If TDD**: Each task follows RED (failing test) → GREEN (minimal impl) → REFACTOR

### QA Policy
Every task MUST include `cargo test` verification.
codegen 작업은 추가로 end-to-end 테스트: `hgl build` → 실행 → 출력 비교

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
├── Task 1: CI 테스트 워크플로 추가
└── Task 2: 컴파일러 경고 수정

Wave 2 (After Wave 1):
├── Task 3: 타입 체커 — 함수 인자 검증 (TDD)
├── Task 4: 타입 체커 — 구조체/배열 검증 (depends: 3)
└── Task 5: 에러 메시지 포맷 통일 (parallel with 3)

Wave 3 (After Wave 2):
├── Task 6: codegen — 클로저/Lambda 구현 (TDD)
├── Task 7: codegen — 메서드/Impl 구현 (TDD)
├── Task 8: codegen — 열거/패턴매칭 구현 (TDD)
├── Task 9: codegen — Range/ForIn/TryCatch 구현 (TDD)
├── Task 10: codegen — Import 구현 (TDD)
└── Task 11: stdlib 모듈 분리 (parallel with 6-10)

Wave 4 (After Wave 3):
├── Task 12: docs 업데이트
└── Task 13: README 반영 (depends: 12)
```

---

## TODOs

- [x] 1. CI 테스트 워크플로 추가

  **What to do**:
  - `.github/workflows/test.yml` 생성
  - push/PR to main 시 `cargo test` 실행
  - Rust stable toolchain

  **Acceptance Criteria**:
  - [ ] `cat .github/workflows/test.yml` 존재, `cargo test` 포함
  - [ ] GitHub Actions에서 green 확인

  **Commit**: `chore: add CI test workflow`

- [x] 2. 컴파일러 경고 수정

  **What to do**:
  - `interpreter.rs` capture_start/capture_flush 경고 해결
  - thread_local allow(dead_code) 올바른 위치로
  - `cargo test 2>&1 | grep warning` = 0

  **Acceptance Criteria**:
  - [ ] `cargo build 2>&1 | grep "warning" | wc -l` = 0

  **Commit**: `fix: resolve compiler warnings`

- [x] 3. 타입 체커 — 함수 인자 검증

  **What to do**:
  - typechecker.rs의 `check_stmt`에서 `Expr::Call` 처리
  - `env.funcs`에서 함수 시그니처 조회 → 인자 개수 + 타입 비교
  - 불일치 시 경고 출력 (exit 안 함 — `process::exit(1)` 제거, `eprintln` 경고로)
  - main.rs에서 type_errors를 경고로 출력하고 계속 실행

  **Must NOT do**:
  - 기존 프로그램 실행 차단 금지

  **References**:
  - `src/typechecker.rs:69` — 현재 Call 처리 (반환타입만)
  - `src/typechecker.rs:119-121` — 함수 시그니처 등록
  - `src/main.rs:35-45` — type error 처리 (현재 exit)

  **Acceptance Criteria**:
  - [ ] `더하기("문자열", 1)` → 타입 경고 출력
  - [ ] `더하기(1)` → 인자 개수 경고
  - [ ] `더하기(1, 2)` → 경고 없음, 정상 실행
  - [ ] 기존 46개 테스트 통과

  **Commit**: `feat: type checker function argument validation (warning mode)`

- [x] 4. 타입 체커 — 구조체/배열 검증

  **What to do**:
  - 구조체 리터럴 필드 타입 검증
  - 배열 요소 타입 일관성 검증
  - `types_compatible` 배열 요소 타입 비교 추가 (line 230)
  - ForIn 변수 타입 추론 수정 (line 163 — 항상 정수 → iterable 기반)

  **References**:
  - `src/typechecker.rs:222-235` — types_compatible
  - `src/typechecker.rs:163` — ForIn 변수 타입

  **Acceptance Criteria**:
  - [ ] `구조 사람 { 나이: 정수 }; 변수 p = 사람 { 나이: "문자" }` → 타입 경고
  - [ ] `변수 arr = [1, "hello", 참]` → 혼합 타입 경고
  - [ ] 기존 46개 테스트 통과

  **Commit**: `feat: type checker struct field and array element validation`

- [x] 5. 에러 메시지 포맷 통일

  **What to do**:
  - 정규형 정의: `"{함수/연산자}: {메시지}"`
  - 121개 RuntimeError::new 호출 감사
  - 불일치하는 포맷 수정
  - 의미 변경 없이 포맷만 통일

  **Must NOT do**:
  - 에러 의미 변경 금지
  - 테스트에서 에러 문자열 비교하는 곳 깨뜨리지 않기

  **References**:
  - `src/interpreter.rs` — 121개 RuntimeError::new 호출

  **Acceptance Criteria**:
  - [ ] 모든 에러 메시지가 `"{prefix}: {message}"` 패턴
  - [ ] 기존 46개 테스트 통과

  **Commit**: `style: standardize error message format`

- [ ] 6. codegen — 클로저/Lambda 구현

  **What to do**:
  - Lambda/Closure를 LLVM IR로 구현
  - 클로저 변환: 캡처 변수를 구조체로 패킹 → 함수 포인터 + 환경 포인터 쌍
  - `Expr::Lambda` → 별도 함수 정의 + 환경 구조체 생성
  - 클로저 호출: 환경 구조체 언팩 → 함수 호출

  **References**:
  - `src/codegen.rs:349-358` — 현재 스텁 위치
  - `src/interpreter.rs:18-26` — Closure Value 구조 (params, body, env snapshot)
  - Crafting Interpreters ch25 — closure upvalue 패턴

  **Acceptance Criteria**:
  - [ ] `변수 두배 = 함수(x: 정수) { 반환 x * 2 }; 출력(두배(5))` → `hgl build` → 실행 → `10`
  - [ ] 캡처 테스트: 외부 변수 참조하는 클로저가 올바른 값 출력
  - [ ] 기존 테스트 통과

  **Commit**: `feat: codegen closure/lambda implementation`

- [ ] 7. codegen — 메서드/Impl 구현

  **What to do**:
  - `Expr::MethodCall` → 구조체 포인터 + 메서드 함수 호출
  - `StmtKind::ImplBlock` → 구조체별 메서드 함수 정의
  - 배열/문자열 빌트인 메서드 (.길이(), .분리() 등) codegen
  - `자신` 파라미터 → 첫 번째 인자로 구조체 포인터 전달

  **References**:
  - `src/codegen.rs:350` — MethodCall 스텁
  - `src/codegen.rs:677` — ImplBlock 스텁
  - `src/interpreter.rs:545-700` — 메서드 호출 구현 (interpreter)

  **Acceptance Criteria**:
  - [ ] `"hello".길이()` → codegen → 실행 → `5`
  - [ ] 구조체 impl 메서드 호출 작동
  - [ ] 기존 테스트 통과

  **Commit**: `feat: codegen method calls and impl blocks`

- [ ] 8. codegen — 열거/패턴매칭 구현

  **What to do**:
  - `StmtKind::EnumDef` → 열거형을 태그(정수) + 값 유니온으로 표현
  - `StmtKind::Match` → 태그 비교 분기 IR 생성
  - 각 variant를 고유 정수 태그로

  **References**:
  - `src/codegen.rs:678` — EnumDef 스텁
  - `src/interpreter.rs:1475-1590` — Match 구현 (interpreter)

  **Acceptance Criteria**:
  - [ ] 열거형 정의 + 패턴매칭 → codegen → 실행 → 올바른 분기
  - [ ] 기존 테스트 통과

  **Commit**: `feat: codegen enums and pattern matching`

- [x] 9. codegen — Range/ForIn/TryCatch 구현

  **What to do**:
  - `Expr::Range` → 배열 할당 + 루프로 채우기
  - `StmtKind::ForIn` → 배열 순회 루프 IR
  - `StmtKind::TryCatch` → setjmp/longjmp 패턴 또는 에러 코드 반환 방식
  - 현재 TryCatch가 두 블록 순차 실행 → 실제 에러 잡기로

  **References**:
  - `src/codegen.rs:349` — Range 스텁
  - `src/codegen.rs:679-684` — ForIn (가짜)
  - `src/codegen.rs:608-619` — TryCatch (가짜)

  **Acceptance Criteria**:
  - [ ] `반복 i 안에서 0..10 { 출력(i) }` → codegen → 실행 → 0~9 출력
  - [ ] `시도 { 1/0 } 처리(e) { 출력(e) }` → codegen → 에러 잡힘
  - [ ] 기존 테스트 통과

  **Commit**: `feat: codegen range, for-in, try-catch`

- [ ] 10. codegen — Import 구현

  **What to do**:
  - `StmtKind::Import(path)` → 파일 읽기 → 파싱 → 해당 모듈의 함수/구조체를 현재 모듈에 링크
  - 컴파일 시 임포트된 파일도 같이 IR 생성 → 하나의 바이너리로

  **References**:
  - `src/codegen.rs:620` — Import 스텁
  - `src/interpreter.rs` — `포함` 구현 참조

  **Acceptance Criteria**:
  - [ ] `포함 "수학도구.hgl"` + 함수 호출 → codegen → 실행 → 올바른 결과
  - [ ] 기존 테스트 통과

  **Commit**: `feat: codegen module imports`

- [ ] 11. stdlib 모듈 분리

  **What to do**:
  - `src/builtins/` 디렉토리 생성
  - `mod.rs`, `math.rs`, `io.rs`, `string.rs`, `system.rs` 분리
  - interpreter.rs에서 빌트인 함수 코드를 각 모듈로 이동
  - interpreter.rs에서 모듈 import로 호출
  - 동작 변경 없음

  **References**:
  - `src/interpreter.rs:200-1100` — 빌트인 함수들 (eval_expr 내)

  **Acceptance Criteria**:
  - [ ] `ls src/builtins/` → mod.rs, math.rs, io.rs, string.rs, system.rs
  - [ ] `wc -l src/interpreter.rs` 유의미하게 감소
  - [ ] 기존 46개 테스트 전부 통과 (동작 동일)

  **Commit**: `refactor: extract stdlib builtins into src/builtins/`

- [ ] 12. docs 업데이트

  **What to do**:
  - `docs/src/tooling/architecture.md` — builtins 모듈 구조 반영
  - `docs/src/reference/types.md` — 타입 체커 경고 모드 설명
  - codegen 지원 기능 목록 업데이트
  - `cd docs && mdbook build` 성공 확인

  **Acceptance Criteria**:
  - [ ] `mdbook build` 성공
  - [ ] architecture 페이지에 builtins 모듈 언급

  **Commit**: `docs: update for language improvements`

- [ ] 13. README 반영

  **What to do**:
  - codegen 지원 기능 테이블 업데이트
  - 테스트 수 업데이트
  - architecture 다이어그램에 builtins 추가

  **Acceptance Criteria**:
  - [ ] README의 기능 테이블이 실제 구현과 일치
  - [ ] 테스트 수 정확

  **Commit**: `docs: update README for language improvements`

---

## Final Verification Wave

- [ ] F1. **전체 테스트 통과** — `cargo test` 전체 통과
- [ ] F2. **codegen 스텁 제거 확인** — `grep "add nsw i64 0, 0" src/codegen.rs` 결과 0건
- [ ] F3. **docs 빌드** — `cd docs && mdbook build` 성공
- [ ] F4. **예제 end-to-end** — playground 예제 6개 모두 `hgl interpret` + `hgl build` 동일 출력

---

## Commit Strategy

- Wave 1: `chore: add CI test workflow` / `fix: resolve compiler warnings`
- Wave 2: `feat: type checker function arg validation` / `feat: type checker struct/array` / `style: standardize error message format`
- Wave 3: `feat: codegen closures` / `feat: codegen methods+impl` / `feat: codegen enums` / `feat: codegen range+forin+trycatch` / `feat: codegen imports` / `refactor: extract stdlib modules`
- Wave 4: `docs: update for language improvements` / `docs: update README`

---

## Success Criteria

### Verification Commands
```bash
cargo test                    # 모든 테스트 통과
cargo build 2>&1 | grep warning  # 경고 0개
cd docs && mdbook build       # docs 빌드 성공
```

### Final Checklist
- [ ] 기존 46개 테스트 통과
- [ ] 새 타입 체커 테스트 통과
- [ ] codegen 스텁 전부 제거
- [ ] end-to-end 예제 통과
- [ ] docs 업데이트 완료
- [ ] README 반영 완료
