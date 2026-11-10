# @monochromatic-dev/jsonc-edit-core-ts.fuzz

Fuzz and coverage-gate sidecar for `@monochromatic-dev/jsonc-edit-core-ts`,
 kept out of the
runtime package so its `src/` stays pure production code (and a whole-package mutation run
stays scoped to real runtime files).

It holds:

- Fast-check property tests (`*.property.unit.test.ts`):
   round-trip idempotency,
   and the STB
  comment-safety guard proving an arbitrary comment body always emits parseable,
   fixpoint JSONC.
- `fuzz-budget.ts`:
   names `JSONC_EDIT_FUZZ_RUNS`,
   the variable that raises the per-property run count
   `@monochromatic-dev/fuzz-core` supplies.
- The deterministic V8 coverage-reachability gate (`coverage-driver.ts`,
   `coverage-report.ts`,
  `coverage-baseline.json`):
   counts covered functions per runtime source file and fails on any
  per-file regression.

## Run

[`mise.toml`](mise.toml) defines this package's tasks.
Run one with `mise run //package/jsonc-edit/core-ts.fuzz:<task>`.
The unit suite runs each property 200 times by default.
Refreeze the baseline after intentionally changing reachable runtime functions.

The gate measures the runtime package's `src/` reachability,
 not this sidecar:
 its
`SOURCE_MARKER` targets `package/jsonc-edit/core-ts/src`.
