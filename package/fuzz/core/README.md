# `@monochromatic-dev/fuzz-core`

The tooling every fuzz sidecar shares.
A sidecar then carries the configuration that is its own,
and not a copy of the work.

## Run budget

A sidecar runs its `*.property.unit.test.ts` files in two layers.
The normal unit suite needs a cheap bounded count.
Its `fuzz` task sets an environment variable for a longer campaign.

Issue 1419 found that count,
its environment read and its fallback copied character-identically into ten sidecars.
The default and the policy for a value that does not parse were decided ten times,
with no owner.
They live here now:

- `DEFAULT_FUZZ_RUNS` (`200`):
  runs per property when no campaign variable supplies one.
- `fuzzRunsFromEnvironment({ envName, defaultRuns?, environment? })`:
  the count one sidecar's properties take.
  Only a positive integer raises it.
  A variable that is unset,
  blank,
  not a number,
  zero,
  negative or fractional leaves the fallback.
  The fallback is `DEFAULT_FUZZ_RUNS` unless the sidecar passes its own.
  `environment` lets a test state one instead of changing the process's own.
- `FuzzRunBudget`:
  the shape of that argument.

A zero count is refused rather than taken as written.
fast-check reads `numRuns` verbatim and runs a property zero times for it.
An empty variable would therefore turn a sidecar's gate green,
without exercising anything.

A sidecar passes only the name of its variable:

```ts
// package/upstream-fork/p-map.fuzz/src/fuzz-budget.ts
export const fuzzRuns: number = fuzzRunsFromEnvironment({ envName: 'P_MAP_FORK_FUZZ_RUNS' });
```

## One owner, held by a test

`src/run-budget-owner.unit.test.ts` lists the ten sidecars that read a run budget.
It fails when one of them parses its own count,
or stops naming its own variable.
It fails when a file other than `src/run-budget.ts` declares a default count,
and when a sidecar reads a run budget without being listed.
A copy cannot come back unnoticed,
and a new sidecar cannot skip the ledger.

## Folder name

Issue 613 proposes a `fuzz-coverage` package for the whole harness.
The folder cannot be named `coverage`.
`.gitignore` ignores every directory of that name as scratch output,
and `doc/decision/gitignore-negations.md` forbids re-including a tracked path past such a pattern.
This package is `package/fuzz/core`,
and prints as `@monochromatic-dev/fuzz-core`,
the kind segment the other shared libraries use.

## Verification

`mise run //package/fuzz/core:test:unit` runs the tests.
`mise run //package/fuzz/core:lint` type-checks and lints it.
