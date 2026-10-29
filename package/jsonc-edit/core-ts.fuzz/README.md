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
   the per-property run count,
   overridable with `JSONC_EDIT_FUZZ_RUNS`.
- The deterministic V8 coverage-reachability gate (`coverage-driver.ts`,
   `coverage-report.ts`,
  `coverage-baseline.json`):
   counts covered functions per runtime source file and fails on any
  per-file regression.

## Run

```sh
# Property tests at the default run budget (also runs under test:unit):
mise run //package/jsonc-edit/core-ts.fuzz:test:unit

# Longer fast-check campaign (default 5000 runs per property):
mise run //package/jsonc-edit/core-ts.fuzz:fuzz
mise run //package/jsonc-edit/core-ts.fuzz:fuzz --runs 20000

# Coverage-reachability gate (check against the frozen baseline):
mise run //package/jsonc-edit/core-ts.fuzz:fuzz:coverage
# Refreeze the baseline after intentionally adding reachable runtime functions:
mise run //package/jsonc-edit/core-ts.fuzz:fuzz:coverage --write
```

The gate measures the runtime package's `src/` reachability,
 not this sidecar:
 its
`SOURCE_MARKER` targets `package/jsonc-edit/core-ts/src`.
