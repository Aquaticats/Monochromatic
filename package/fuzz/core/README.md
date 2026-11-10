# `@monochromatic-dev/fuzz-core`

The tooling every fuzz sidecar shares, so a sidecar carries the configuration that is its own and not a copy of the
work.

## Run budget

A sidecar runs its `*.property.unit.test.ts` files in two layers: the normal unit suite, which needs a cheap bounded
count, and its `fuzz` task, which sets an environment variable for a longer campaign. Issue 1419 found that count, the
environment read and the fallback copied character-identically into ten sidecars, so the default and the policy for a
value that does not parse were decided ten times with no owner. They live here now:

- `DEFAULT_FUZZ_RUNS` (`200`):
  runs per property when no campaign variable supplies one.
- `fuzzRunsFromEnvironment({ envName, defaultRuns?, environment? })`:
  the count one sidecar's properties take.
  Only a positive integer raises it;
  a variable that is unset, blank, not a number, zero, negative or fractional leaves the fallback.
  The fallback is `DEFAULT_FUZZ_RUNS` unless the sidecar passes its own,
  and `environment` lets a test state one instead of changing the process's own.
- `FuzzRunBudget`:
  the shape of that argument.

`0` is refused rather than taken as written because fast-check reads `numRuns` verbatim and runs a property zero times
for it, so an empty variable would turn a sidecar's gate green without exercising anything.

A sidecar passes only the name of its variable and keeps its own properties reading a value:

```ts
// package/upstream-fork/p-map.fuzz/src/fuzz-budget.ts
export const fuzzRuns: number = fuzzRunsFromEnvironment({ envName: 'P_MAP_FORK_FUZZ_RUNS' });
```

## One owner, held by a test

`src/run-budget-owner.unit.test.ts` lists the ten sidecars that read a run budget,
fails when one of them parses its own count or stops naming its own variable,
fails when a file other than `src/run-budget.ts` declares a default count,
and fails when a sidecar reads a run budget without being listed.
A copy therefore cannot come back unnoticed, and a new sidecar cannot skip the ledger.

## Folder name

Issue 613 proposes a `fuzz-coverage` package for the whole harness.
The folder cannot be named `coverage`:
`.gitignore` ignores every directory of that name as scratch output,
and `doc/decision/gitignore-negations.md` forbids re-including a tracked path past such a pattern instead of removing
the collision.
This package is `package/fuzz/core` and prints as `@monochromatic-dev/fuzz-core`,
which is the kind segment the other shared libraries use.

## Verification

`mise run //package/fuzz/core:test:unit` runs the tests.
`mise run //package/fuzz/core:lint` type-checks and lints it.
