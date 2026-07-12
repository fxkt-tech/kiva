# Bug Analysis: Script Author Headers Timeout

## 1. Root Cause Category

- **Category**: E — Implicit Assumption, with D — Test Coverage Gap.
- **Specific cause**: the first implementation assumed one provider request could return 73 structured speech beats. Local heuristic tests return instantly and did not model provider first-byte latency. Volcengine Ark timed out before response headers while producing the oversized response.

## 2. Why the first implementation failed

1. One-shot narrative generation minimized code but coupled the entire episode to one very large response.
2. Persisted failed/retry state recovered operationally but retried the same unstable request shape.
3. Raising fetch timeout would only hide the request-shape problem and make the UI wait longer.

## 3. Prevention mechanisms

| Priority | Mechanism | Action | Status |
|---|---|---|---|
| P0 | Architecture | Generate outline separately and cap beat batches at 12. | DONE |
| P0 | Regression | Fake provider throws the exact Headers Timeout error above 12 beats. | DONE |
| P1 | Runtime | Retry one Headers Timeout per small request; then persist failed state. | DONE |
| P1 | Spec | Record request-size and local-context contracts. | DONE |

## 4. Systematic expansion

- Other long-form LLM features must bound both prompt scope and requested output cardinality.
- Local heuristic clients validate schema flow, not real provider latency; provider-shaped failure tests are required for large structured generation.
- Transport retry is useful only after requests are independently bounded and idempotent.

## 5. Knowledge capture

- Episode Script workflow spec updated with outline/batch contracts and timeout behavior.
- Regression test runs at the authoring seam and reproduces the user's exact error text.
