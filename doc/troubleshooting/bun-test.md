# `bun:test` reporter merges duplicate `describe` titles, dropping logs and failures; `expect.assertions(n)` is unreliable under default concurrent execution

This file groups two independent `bun:test` quirks that bite the
workspace.
 Each gets its own canonical section.

(Distinct from `doc/troubleshooting/testing.md`,
 which covers the
in-tree `@monochromatic-dev/test-harness-core` harness rather than
`bun:test`.
)

---

## Bug 1: Duplicate `describe` titles at the same scope cause the reporter to merge suites and drop logs / failures

### Symptom

- Reporter output omits some `expect(...)` failures.
- Reporter output omits or misattributes `console.log` messages.
- Test counts in the reporter are lower than the number of tests
  defined in the file.
- Running a single file shows fewer suites than expected.

Local to a file where duplicates exist;
 does not propagate to
sibling files.

### Root cause

The `bun:test` reporter merges suites sharing the same title at
the same lexical scope.
 Merged suites collapse logs and results
from earlier suites into the last suite with the same title.
Duplicate test titles under the same suite compound the
ambiguity and hide earlier failures.

### Verification

Version under test:
 `bun:test` shipped with Bun 1.3.
x.

Reproduce:

```ts
// example.duplicate-describe.unit.test.ts
import {
  describe,
  expect,
  test,
} from 'bun:test';

describe('suite', () => {
  test('first', () => {
    // oxlint-disable-next-line no-console -- demonstration
    console.log('from first suite',);
    expect(1,).toBe(2,);
  });
});

describe('suite', () => {
  test('second', () => {
    // oxlint-disable-next-line no-console -- demonstration
    console.log('from second suite',);
    expect(1,).toBe(1,);
  });
});
```

The reporter may omit the failure from the first `suite` and
show only the second `suite`.
 Console output from the first
`suite` can be missing or attributed to the second.

### Verified workaround

Use a single top-level `describe` per subject (API,
 function,
or feature) within a file.
 Use nested `describe` groups under
the top-level suite to partition contexts.
 Keep `describe`
titles unique at the same scope.

Template:

```ts
import {
  describe,
  expect,
  test,
} from 'bun:test';

describe('subject-under-test', () => {
  //region Success and retry behavior

  describe('success and retry behavior', () => {
    test('returns result on success', () => {/* ... */});
    test('retries on true decision', () => {/* ... */});
  });

  //endregion Success and retry behavior

  //region Null suppression behavior

  describe('null suppression behavior', () => {
    test('returns undefined on null decision', () => {/* ... */});
  });

  //endregion Null suppression behavior

  //region Validation and type checking

  describe('validation and type checking', () => {
    test('invalid decision type throws TypeError', () => {/* ... */});
  });

  //endregion Validation and type checking
});
```

Tradeoff:
 every file must enforce single-top-level-describe by
convention;
 no linter rule yet warns about duplicate
`describe` titles.
 Mirror structure between sync and async
suites for quick visual diffing.

### Detection script

```bash
rg -n "^[[:space:]]*describe\(" -t ts .
```

Focus on files with multiple top-level `describe` entries
sharing the same string title.
 Verify suspicious files by
running them directly:

```bash
mise run test -- package/module/es/src/function.tryCatch.unit.test.ts
```

Use `test.only('case under investigation', () => { /* ... */ })`
to isolate.

### What does not work

- Renaming only the duplicates and assuming the reporter
  recovers:
   the merging happens at suite-registration time,
   so
  renaming fixes it but does not retroactively un-merge an
  already-collapsed history.
- Setting `BUN_TEST_REPORTER=verbose` (or similar):
   no such env
  knob exists;
   the merge behaviour is baked into the reporter.

### Why we do not file this upstream

1. **Is it really upstream's fault?
   ** Yes;
    reporter dedup that
   swallows test output is a real defect.
2. **Can upstream fix it?
   ** Yes;
    either reject duplicate titles
   with a warning or disambiguate them in output (e.g. append
   `#2`).
3. **Are they supporting this use case?
   ** `bun:test` aims to be
   a fast jest-compatible runner;
    jest disambiguates merged
   suites,
    so the gap is a compat issue.
4. **Will they likely fix it?
   ** Plausible;
    Bun has accepted
   reporter improvements in the past.
5. **Have we prototyped a minimal fix?
   ** No.

Decision:
 worth filing if the failure mode keeps biting,
 but
the workaround (single-top-level-describe convention) is
cheap.
 Currently no upstream report from us.

---

## Bug 2: `expect.assertions(n)` does not survive default concurrent execution

### Symptom

Tests using `expect.assertions(n)` fail with assertion-count
mismatches in concurrent mode,
 even when each test's assertions
fire correctly.
 Tests pass in isolation (sequential) but fail
when run together.

### Root cause

Recent Bun releases run tests concurrently by default.
`expect.assertions(n)` maintains state shared across the test
suite to track assertion counts.
 When tests run concurrently,
assertions from different tests can interfere with the counter,
producing false negatives (or false positives in some
orderings).
 The utility was designed for sequential execution.

### Verification

Version under test:
 `bun:test` shipped with Bun 1.3.
x.

Reproduce by adding `expect.assertions(n)` to any test in a
file with two or more concurrent tests,
 then running:

```bash
mise run test -- path/to/file.test.ts
```

Failure depends on scheduler order;
 reruns fluctuate.

### Verified workaround

Remove all `expect.assertions(...)` calls and replace with
explicit assertions that validate the exact behaviour expected:

```ts
// Before; unreliable under concurrency
test('example', () => {
  expect.assertions(2,);
  expect(getValue(),).toBe(5,);
  expect(getError(),).toBeNull();
});

// After; reliable regardless of concurrency
test('example', () => {
  const value = getValue();
  const error = getError();
  expect(value,).toBe(5,);
  expect(error,).toBeNull();
});
```

Tradeoff:
 loses the "every code path made it to its
assertion" guarantee that `expect.assertions(n)` provides.
Compensate by structuring tests so each branch is a separate
`test` block with its own explicit assertions,
 or by adding a
sentinel assertion at the end of an early-return branch.

Search for existing usage to migrate:

```bash
rg 'expect\.assertions' . --type ts
```

### What does not work

- Disabling concurrency for one file (`describe.serial` or
  similar):
   `bun:test` does not expose a per-file
  serialisation hook;
   concurrency is controlled at the runner
  level.
- Wrapping in a mutex:
   the assertions counter is internal to
  `bun:test`;
   user-space synchronisation cannot affect it.

### Why we do not file this upstream

1. **Is it really upstream's fault?
   ** Yes;
    the assertion-count
   counter should be per-test,
    not shared state.
2. **Can upstream fix it?
   ** Yes;
    key the counter by test
   identity.
3. **Are they supporting this use case?
   ** `expect.assertions`
   is a documented jest-API surface that `bun:test` aims to
   match.
4. **Will they likely fix it?
   ** Plausible;
    the fix is
   contained.
5. **Have we prototyped a minimal fix?
   ** No.

Decision:
 worth filing.
 Until then,
 the "remove
`expect.assertions(n)`" workaround is the working approach.

---

## Recommended commands (workspace-specific)

- Run a single test file:
  ```bash
  mise run test -- package/module/es/src/function.tryCatch.unit.test.ts
  ```
- Build and test together to avoid stale artifacts:
  ```bash
  mise run buildAndTest
  ```
