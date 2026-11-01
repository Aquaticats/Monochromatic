## Problem statement: inline error diagnostics for `@monochromatic-dev/test-harness-core`

### Context

`@monochromatic-dev/test-harness-core` is a self-hosted test harness.
 Tests are defined via `it({ name, fn })` and grouped via `describe({ name, children })`.
 The harness logs pass/fail status through a tagged logger
and propagates failures via `throw new Error(name, { cause })`.

### Current behavior (verified)

When a test fails,
 the harness emits only a summary line at `error` level.
 Full error content (message,
 stack trace,
 `.cause` chain,
 `AggregateError.errors`) is **not** logged by the harness.

In `src/it.ts:216-219`:

```ts
if (threw) {
  l.error(`FAIL${runLabel} (${durationMs.toFixed(0,)}ms)`,);
  throw new Error(name, { cause: caughtError, },);
}
```

The `l.error(...)` call outputs only the test name and duration.
 The `Error` object thrown here carries the full cause chain,
 but that chain is only visible when the rejection propagates uncaught to the runtime
and the runtime's unhandled-rejection printer dumps it.

Similarly,
 in `src/describe.ts:297-304`,
 suite-level failures emit only:

```ts
l.error(`FAIL${runLabel} (${durationMs.toFixed(0,)}ms)`,);
throw new Error(name, { cause, },);
```

### Verified consequences

1. **Real failures show diagnostics only via the runtime printer.
   **

   Running a real failure produces harness log lines with no message content,
    followed by Bun's cause-chain dump at the bottom:

   ```text
   [error] [real-failure-suite] [will-throw] FAIL (0ms)
   [error] [real-failure-suite] FAIL (2ms)
   error: real-failure-suite      ← Bun, not the harness
   error: will-throw              ← Bun, not the harness
   error: something went wrong    ← Bun, not the harness
   ```

2. **Caught rejections show no diagnostics at all.
   **

   Self-tests catch rejections to assert on error shape
   (`await expect(rejection).rejects.toHaveProperty('cause', original)`).
    Since the rejection never reaches the runtime,
    the full error content never appears anywhere in output.
    Self-test output shows `FAIL (0ms)` lines with adjacent silence.

3. **`fails: true` tests have a similar gap.
   **

   When a test marked `fails: true` unexpectedly passes,
    the harness emits:

   ```text
   FAIL: expected to throw but passed (0ms)
   ```

   But the generated synthetic error (`new Error('Expected test to throw but it passed')`) is thrown,
    not logged inline.

### Problem

The most informative diagnostic output is the least guaranteed to appear.
 It depends on:

- The rejection propagating uncaught through every parent layer.
- The runtime (Bun) having a competent unhandled-rejection printer.

If a consumer wraps `await describe(...)` in a custom reporter that catches the rejection,
 or runs in an environment without Bun's rich error printer,
 the log stream contains `FAIL` entries with zero diagnostic context.

### Desired behavior

The harness should emit full error content inline,
 at `error` level,
 adjacent to the `FAIL` summary line,
 before re-throwing.
 The log stream alone should be sufficient to diagnose failures without depending on the runtime printer.

Example of the desired output for a real failure:

```text
[error] [suite] [test] FAIL (Nms)
[error] [suite] [test]   Error: something went wrong
[error] [suite] [test]       at fn (/path/to/test.ts:9:19)
[error] [suite] [test]       at runFnOnce (...)
[error] [suite] FAIL (Nms)
```

The harness should still re-throw after logging,
 preserving the existing `Error(name, { cause })` contract for programmatic consumers.

### Acceptance criteria

1. **`runIt` in `src/it.ts`** logs the full caught error before re-throwing.
    This includes:
   - The primary error message and stack.
   - The `.cause` chain (recursive walk).
   - Expansion of `AggregateError.errors` when present.

2. **Assertion-count failures** are also logged inline before throwing.

3. The logger tag chain for inline error lines remains consistent
   with the `FAIL` summary line (same `[suite] [test]` prefix).

4. Existing behavior is preserved:
    the harness still throws the same errors with the same `.cause` chains,
    and `describe` still wraps results the same way.
    This change is additive (logging) only.

5. Self-tests should continue to pass.
    The added inline logs will cause self-test output to contain error diagnostics where there was previously silence;
    this is expected and acceptable.

### Open design decisions

1. **Double-print tradeoff.
   ** Currently Bun prints the full cause chain when the top-level `describe(...)` rejects.
    Adding inline logging means the same error content may appear twice (once inline via the logger,
    once at the bottom via Bun).
    Options:
   - Accept the duplication as a short-term state.
   - Add a top-level harness helper (e.g. `run(descriptor)`) that catches,
      logs inline,
      sets `process.exitCode = 1`,
      and swallows the rejection to suppress the runtime dump.
   - Decide this is out of scope for the initial fix and document as a known side effect.

2. **`fails: true` suppression.
   ** For tests with `fails: true`,
    a deliberate `throw` is expected behavior and already logs `PASS` at `debug`.
    Should inline error dumping be suppressed for `fails: true` tests that throw?
    If not,
    every expected failure will spam error-level logs.
    The current behavior of logging nothing for expected throws may be preferable.

   Recommendation:
    suppress inline error logging when `fails !== false` and the test threw (expected behavior).
    Log inline errors only for:
   - Unexpected throws (`fails: false` and test threw).
   - Assertion-count mismatches.
   - Unexpected passes (`fails: true` and test passed).

3. **Formatting helper.
   ** A `formatErrorDeep` helper is needed.
    It should:
   - Print `error.message`.
   - Print `error.stack` minus the message line (to avoid duplication).
   - Walk `.cause` recursively with indentation or prefix markers.
   - Expand `AggregateError.errors` similarly.
   - Handle cross-realm or malformed error objects gracefully (no `.stack`,
      non-string `.message`,
      etc.).
   - Respect the logger's line-oriented output:
      multi-line strings should be split and each line prefixed with the current tag.

### Non-goals

- Do not change the public API of `it`,
   `describe`,
   or `TestDescriptor`.
- Do not modify `src/describe.ts` suite-level wrapping logic
  except to decide whether suite-level inline dumping is needed
  (see acceptance criteria:
   leaf-level only required).
- Do not remove the existing `throw` behavior or change the `Error(name, { cause })` shape.

### Files to modify

- `package/test-harness/core/src/it.ts`:
   add inline error logging in failure paths.
- `package/test-harness/core/src/self.unit.test.ts`:
   if self-test output assertions exist,
   update expectations to account for new log lines (or verify if any non-build assertions are affected).
