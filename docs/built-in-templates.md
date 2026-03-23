# EA Code v2 — Built-in Pipeline Templates

Five templates ship with EA Code v2. They cannot be deleted but can be cloned and customised.

---

## 1. Full Review Loop

The default template. Runs a complete analyse-review-implement-test cycle with up to 5 iterations.

| Field              | Value |
|--------------------|-------|
| ID                 | `built-in-full-review` |
| Description        | Full analysis, review, implementation, and test cycle with iterative refinement |
| Max iterations     | 5 |
| Stop on first pass | Yes |

### Stages

| Position | Label     | Type | Provider | Model   | Session Group | Execution Intent |
|:--------:|-----------|------|----------|---------|:-------------:|------------------|
| 0        | Analyse   | text | claude   | opus    | A             | Understand the task, identify affected files, assess complexity |
| 1        | Review    | text | claude   | opus    | A             | Review the analysis, identify risks, refine the approach |
| 2        | Implement | code | claude   | sonnet  | B             | Write the code changes based on the analysis and review |
| 3        | Test      | code | claude   | sonnet  | B             | Run tests, verify changes compile, check for regressions |

**Session group rationale:** Analyse and Review share group A (Opus) for compounding context. Implement and Test share group B (Sonnet) for cost-efficient code execution with shared workspace context.

---

## 2. Quick Fix

Minimal template for small, well-understood changes. Single iteration, no analysis phase.

| Field              | Value |
|--------------------|-------|
| ID                 | `built-in-quick-fix` |
| Description        | Fast implement-and-test for straightforward changes |
| Max iterations     | 1 |
| Stop on first pass | Yes |

### Stages

| Position | Label     | Type | Provider | Model   | Session Group | Execution Intent |
|:--------:|-----------|------|----------|---------|:-------------:|------------------|
| 0        | Implement | code | claude   | sonnet  | A             | Implement the requested change directly |
| 1        | Test      | code | claude   | sonnet  | A             | Verify the change compiles and tests pass |

**Session group rationale:** Both stages share group A. The test stage sees the full implementation context.

---

## 3. Research Only

Investigation template that produces analysis without modifying code.

| Field              | Value |
|--------------------|-------|
| ID                 | `built-in-research` |
| Description        | Deep analysis and review without code changes |
| Max iterations     | 1 |
| Stop on first pass | N/A (no judge stage) |

### Stages

| Position | Label   | Type | Provider | Model | Session Group | Execution Intent |
|:--------:|---------|------|----------|-------|:-------------:|------------------|
| 0        | Analyse | text | claude   | opus  | A             | Investigate the codebase, understand architecture, identify patterns |
| 1        | Review  | text | claude   | opus  | A             | Synthesise findings, assess trade-offs, produce recommendations |

**Session group rationale:** Both stages share group A. The review stage builds on the full analysis context for a coherent final report.

---

## 4. Multi-Brain Review

Cross-provider analysis for high-stakes changes. Three different AI backends review independently before implementation.

| Field              | Value |
|--------------------|-------|
| ID                 | `built-in-multi-brain` |
| Description        | Three independent AI reviewers followed by implementation and testing |
| Max iterations     | 3 |
| Stop on first pass | No |

### Stages

| Position | Label     | Type | Provider | Model                  | Session Group | Execution Intent |
|:--------:|-----------|------|----------|------------------------|:-------------:|------------------|
| 0        | Analyse   | text | claude   | opus                   | A             | Primary analysis of the task and codebase |
| 1        | Review    | text | gemini   | gemini-3.1-pro-preview | B             | Independent review from a second perspective |
| 2        | Review 2  | text | codex    | codex-5.3              | C             | Independent review from a third perspective |
| 3        | Implement | code | claude   | sonnet                 | D             | Implement changes incorporating all three reviews |
| 4        | Test      | code | claude   | sonnet                 | D             | Verify changes compile and tests pass |

**Session group rationale:** Each reviewer gets its own group (A, B, C) because they use different providers — cross-provider session resume is not possible. Implement and Test share group D for shared coding context. Reviews reach the implementer via `{{previous_output}}`.

---

## 5. Security Audit

Security-focused pipeline with dedicated security review stage and iterative fixing.

| Field              | Value |
|--------------------|-------|
| ID                 | `built-in-security-audit` |
| Description        | Security-focused analysis, review, and remediation |
| Max iterations     | 2 |
| Stop on first pass | No |

### Stages

| Position | Label           | Type | Provider | Model  | Session Group | Execution Intent |
|:--------:|-----------------|------|----------|--------|:-------------:|------------------|
| 0        | Analyse         | text | claude   | opus   | A             | Map attack surface, identify entry points, catalogue data flows |
| 1        | Security Review | text | claude   | opus   | A             | Check OWASP Top 10, assess auth/authz, review crypto usage, flag secrets |
| 2        | Review          | text | claude   | opus   | A             | Synthesise security findings, prioritise by severity, recommend fixes |
| 3        | Implement       | code | claude   | sonnet | B             | Apply security fixes and hardening measures |
| 4        | Test            | code | claude   | sonnet | B             | Run tests, verify fixes do not break functionality |

**Session group rationale:** All three review stages share group A (Opus) so the security review builds on the analysis and the final review has full context of both. Implement and Test share group B (Sonnet) for code execution.
