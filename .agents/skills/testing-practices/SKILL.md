---
name: testing-practices
description: Use when writing or reviewing TypeScript tests in this monorepo using the module-test harness.
---

# Testing practices

Fires when writing or modifying TypeScript tests,
 picking a test pattern,
 or reviewing
TypeScript test code in this monorepo.

The skill covers the module-test harness,
 the describe/it API,
 test file
conventions,
 coverage,
 parameterized tests,
 async patterns,
 and sinon
integration.

## Test framework

- TypeScript unit tests use `@monochromatic-dev/test-harness-core/ts`,
   a custom harness built on chai,
   sinon,
   and expect-type
- Unit tests import package behavior from built `dist`,
   never from sibling source files
- Browser and e2e tests use Playwright,
   executed inside a podman container
- All tests run through `mise run`;
   never invoke `bun test` or `playwright` directly
- Each test file is self-contained with top-level `await describe(...)`;
   no external test runner needed

## Running tests

```bash
# Build all packages, then run all tests (unit + browser + e2e)
mise run buildAndTest  # alias: bt

# Build, then run a specific unit test file
mise run buildAndTest -- package/async-time/core/src/wait.unit.test.ts

# Run tests only when dist is already fresh
mise run test          # alias: t

# Watch mode
mise run watch:test    # alias: tW
```

**Required for unit tests**:
 unit tests import package code from built `dist`.
Run `mise run buildAndTest` after source changes,
 including when narrowing to one file,
because `mise run test` alone can exercise stale artifacts.
Use a package-scoped test task only after the same verification sequence rebuilt its artifact.
Harness self-tests also import built output;
source/built interoperability tests are deliberate exceptions,
not permission to skip the harness build.

## Browser and e2e tests

Browser and e2e tests run inside a podman container image (`monochromatic-playwright`).
Build the image first with `mise run prepare:playwright`.

```bash
# Run browser tests (all browsers)
mise run test:browser

# Run browser tests for a specific browser
mise run test:browser:chromium
mise run test:browser:firefox

# Run e2e tests
mise run test:e2e
```

Playwright configuration lives in `playwright.browser.config.ts` and `playwright.e2e.config.ts`.
The browser test server (`playwright/serve.ts`) uses h3.

## Test file naming

- Unit tests:
   `{name}.unit.test.ts` -- co-located alongside source
- Browser tests:
   `{name}.browser.test.ts`
- E2E tests:
   `{name}.e2e.test.ts`

Test discovery uses `rg --files --glob '**/*.unit.test.*'` -- no configuration file needed.

## Test file setup

```ts
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { wait, } from '../dist/final/neutral/index.mjs';

await describe({
  name: wait.name,
  children: [
    it({
      name: 'resolves to undefined',
      fn: async () => {
        const result = await wait(1,);
        expect(result,).toBeUndefined();
      },
    },),
  ],
},);
```

Key differences from bun:
test / Jest:

- Import `describe`,
   `it`,
   `expect` from `@monochromatic-dev/test-harness-core/ts` (not `bun:test`)
- Import package code under test from built `dist`,
   not from sibling `src` files
- Use `it` (not `test`)
- `describe` and `it` take an options object `{ name, children/fn }`,
   not positional `(name, callback)` arguments
- `describe` returns a promise -- use `await describe(...)` at the top level
- Children are concurrent by default via `Promise.allSettled`

## Importing code under test

Treat direct imports from sibling source files as a test bug.
 Tests for exported
package behavior must import from the built `dist` entry point,
 for example
`../dist/final/neutral/index.mjs` from a `src/*.unit.test.ts` file.
 This verifies
the artifact users consume and catches export-map,
 build,
 and bundling errors.

Relative source imports are allowed only for fixtures,
 mock data,
 or test-only
helpers that are not part of package behavior.

## Describe name conventions

When a describe block names a specific **imported function**,
 use `functionName.name` instead of a string literal.
This keeps names in sync with refactors.

```ts
// Imported function -- use .name
describe({ name: wait.name, ... })

// Constant or descriptive category -- use string literal
describe({ name: 'SEVERITY_MAP', ... })
describe({ name: 'error code constants', ... })
describe({ name: 'valid fixtures', ... })
```

For the top-level describe in a file,
 use an empty string `name: ''` when the file tests a single module.
The empty-name suite is invisible in output -- the filename already identifies what is being tested.
Nest the real function-named describes as children.

```ts
await describe({
  name: '',
  children: [
    describe({
      name: stripAnsi.name,
      children: [ ... ],
    }),
    describe({
      name: extractRuleName.name,
      children: [ ... ],
    }),
  ],
});
```

## Coverage requirements

Target 100% test coverage.
 When certain lines or branches cannot be tested (e.g. error handling for impossible states),
 use V8 ignore comments:

```ts
/* v8 ignore next -- @preserve */
if (impossibleCondition)
  throw new Error('This should never happen',);

// For multiple lines:
/* v8 ignore next 3 -- @preserve */
if (untestableCondition) {
  console.error('Untestable path',);
  return fallbackValue;
}
```

## Test structure

- Use descriptive test names that explain expected behavior
- Group related tests using nested `describe` blocks as children
- Keep `describe` names unique at the same scope within a file;
  duplicate names cause misattributed results
  (see [testing troubleshooting](../../../doc/troubleshooting/testing.md))
- Use `.map()` to generate parameterized `it` entries in the `children` array
- Test both happy path and error scenarios

### Parameterized tests with `.map()`

Since there is no `test.each`,
 generate `it` entries with `.map()` and spread into `children`:

```ts
await describe({
  name: shouldIgnoreFile.name,
  children: [
    ...IGNORED_EXTENSIONS.map(function mapExt(ext,) {
      return it({
        name: `returns true for ${String(ext,)} extension`,
        fn: async () => {
          expect(shouldIgnoreFile(`/some/path/file${String(ext,)}`,),).toBe(
            true,
          );
        },
      },);
    },),
    it({
      name: 'returns false for plain .ts files',
      fn: async () => {
        expect(shouldIgnoreFile('/some/path/file.ts',),).toBe(false,);
      },
    },),
  ],
},);
```

### Test timeouts

Pass `timeout` in the options object (milliseconds):

```ts
it({
  name: 'fetches embeddings from external API',
  fn: async () => {
    const result = await embed({ input: 'test', },);
    expect(result,).toBeDefined();
  },
  timeout: 30_000,
},);
```

Suites also accept `timeout` which applies to all children collectively.

### Skipping tests

Pass `skip: true` or `skip: 'reason string'`:

```ts
it({
  name: 'unix-only behavior',
  skip: process.platform === 'win32',
  fn: async () => { ... },
})
```

### Expected failures

Pass `fails: true` to mark a test that is expected to throw.
A throwing test is PASS;
 a passing test is FAIL:

```ts
it({
  name: 'rejects invalid input',
  fails: true,
  fn: async () => {
    throw new Error('expected',);
  },
},);
```

### Repeating tests (flake detection)

Pass `repeats: N` to run the test N additional times after the first:

```ts
it({
  name: 'not flaky',
  repeats: 2,  // runs 3 times total
  fn: async () => { ... },
})
```

### Sequential execution

Children run concurrently by default.
 Set `concurrency: 1` on describe
to run children one at a time in array order:

```ts
describe({
  name: 'ordered operations',
  concurrency: 1,
  children: [
    it({ name: 'step 1', fn: async () => { ... } }),
    it({ name: 'step 2', fn: async () => { ... } }),
  ],
})
```

Children are lazy descriptors and do not run until the parent dispatches
them,
 so `concurrency: 1` sequences execution without any wrapping.
 The
parent's effective concurrency is inherited by nested describes that
don't set their own,
 so a single `concurrency: 1` at the top sequences
all descendants.

### Region markers

Use `//region` and `//endregion` markers to organize test groups within a `describe` block:

```ts
await describe({
  name: wait.name,
  children: [
    //region Delay behavior

    it({
      name: 'resolves after the specified delay',
      fn: async () => {
        const DELAY = 20;
        const TOLERANCE = 5;

        const start = performance.now();
        await wait(DELAY,);
        const elapsed = performance.now() - start;

        expect(elapsed,).toBeGreaterThanOrEqual(DELAY - TOLERANCE,);
      },
    },),

    //endregion Delay behavior

    //region Return value

    it({
      name: 'returns undefined',
      fn: async () => {
        const result = await wait(1,);
        expect(result,).toBeUndefined();
      },
    },),
    //endregion Return value
  ],
},);
```

Region markers provide IDE folding and navigability in long test files.

### Assertion count verification

The `it` function provides a scoped `expect` via the test context parameter.
Use `ctx.expect.assertions(n)` to verify exactly N assertions run:

```ts
it({
  name: 'all assertions execute',
  fn: async ctx => {
    ctx.expect.assertions(2,);
    ctx.expect(await readFile(path,),).toBe('content',);
    ctx.expect(exists,).toBe(true,);
  },
},);
```

The global `expect` also works but does not support assertion counting.

### Sinon sandbox for spies/stubs

Each body attempt receives a fresh Sinon sandbox,
including repeats.
Completion or timeout closes its mutation authority before automatic restoration:

```ts
it({
  name: 'logs a message',
  fn: async ctx => {
    const spy = ctx.sinon.spy(console, 'log',);
    doSomething();
    expect(spy,).toHaveBeenCalledWith('hello',);
  },
},);
```

In Node,
keep tests concurrent when `ctx.sinon.stub(target, key)` or `spy(target, key)` replaces an own,
configurable,
writable function-valued data property.
Each attempt reads its own fake;
unstubbed readers see the original.
This includes `console.warn` and methods owned by a prototype.

For other shared state,
prefer test-local resources.
When sharing is unavoidable,
sequence every affected reader and writer under one common `concurrency: 1` ancestor.
Separate sequential sibling suites can still overlap.
This applies to shared fake timers,
`process.env`,
`process.argv`,
files,
whole-object replacements,
mocks,
and browser stubs.
These families do not gain context isolation from the method adapter.

Await or stop background work before finishing a test:
closing sandbox authority does not cancel application work.
For inherited/accessor/proxy targets,
descriptor mutation,
captured references,
call-sequence controllers,
or callback ownership,
read the [harness ownership contract](../../../package/test-harness/core/README.md#stubs-and-spies)
before choosing a fixture.

## Async error assertions

Await the async operation first,
 then assert on the caught error.
`.rejects` and `.resolves` are legacy APIs -- avoid them in new test code.

`.rejects.toThrow()` passes the rejected value to chai's `.throw()`,
which expects a function to call.
 The harness patches around this mismatch,
but the indirection obscures stack traces and is inherently fragile.
`.resolves.toBe(x)` adds nothing over `const v = await p; expect(v).toBe(x)`.

```ts
// Preferred: await first, assert on the error
it({
  name: 'rejects on missing file',
  fn: async () => {
    let caught: unknown;
    try {
      await readConfig('/nonexistent',);
    }
    catch (error) {
      caught = error;
    }
    expect(caught,).toBeInstanceOf(ConfigError,);
    expect((caught as Error).message,).toContain('not found',);
  },
},);

// Preferred: await first, assert on the resolved value
it({
  name: 'returns parsed config',
  fn: async () => {
    const result = await readConfig('/valid',);
    expect(result,).toHaveProperty('version', 2,);
  },
},);
```

## Type-level testing

Use `expectTypeOf` from `expect-type`,
 re-exported by `@monochromatic-dev/test-harness-core/ts`:

```ts
import {
  describe,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChatRole, } from '../dist/final/neutral/index.mjs';

await describe({
  name: 'ChatRole',
  children: [
    it({
      name: 'is exactly the role union',
      fn: async () => {
        expectTypeOf<ChatRole>().toEqualTypeOf<'system' | 'user' | 'assistant'>();
      },
    },),
  ],
},);
```

## Available matchers

The `expect` function provides Jest-compatible matchers backed by chai:

**Value matchers**:
 `toBe`,
 `toEqual`,
 `toStrictEqual`,
 `toContain`,
 `toContainEqual`,
`toMatch`,
 `toMatchObject`,
 `toSatisfy`,
 `toBeCloseTo`

**Type/state matchers**:
 `toBeDefined`,
 `toBeUndefined`,
 `toBeNull`,
 `toBeTruthy`,
`toBeFalsy`,
 `toBeNaN`,
 `toBeTypeOf`,
 `toBeInstanceOf`

**Comparison matchers**:
 `toBeGreaterThan`,
 `toBeGreaterThanOrEqual`,
`toBeLessThan`,
 `toBeLessThanOrEqual`

**Collection matchers**:
 `toHaveLength`,
 `toHaveProperty`

**Error matcher**:
 `toThrow`

**Spy matchers** (sinon-chai):
 `toHaveBeenCalled`,
 `toHaveBeenCalledTimes`,
`toHaveBeenCalledWith`,
 `toHaveBeenCalledExactlyOnceWith`,
`toHaveBeenLastCalledWith`,
 `toHaveBeenNthCalledWith`,
`toHaveReturned`,
 `toHaveReturnedTimes`,
 `toHaveReturnedWith`,
`toHaveLastReturnedWith`,
 `toHaveNthReturnedWith`

**Negation**:
 `expect(x).not.toBe(y)`

**Promise matchers** (legacy,
 avoid in new code):
 `expect(promise).rejects.toThrow()`,
 `expect(promise).resolves.toBe(42)`.
Prefer awaiting the promise first,
 then asserting on the result or caught error directly.

**Asymmetric matchers** (for use inside `toHaveBeenCalledWith`):
`expect.stringContaining`,
 `expect.stringMatching`,
 `expect.objectContaining`,
`expect.arrayContaining`,
 `expect.anything`,
 `expect.any`

## Test output format

Every log line carries the full suite hierarchy as a tag chain.
 The leftmost tag
is the outermost `describe`,
 the rightmost tag is the current `it` (or innermost
`describe`);
 empty-name suites contribute no tag.

Default verbosity is compact:
 each `describe` emits one info-level line listing
its fulfilled children's names.
 Per-test `PASS` lines are at `debug` (hidden
unless `MONOCHROMATIC_VERBOSE=true`).
 Failures remain at `error` and are always visible.

```text
[info]  [iso] [outer] [inner] [PASS] childA, childB, ... (Nms) -- happy-path enumeration
[info]  [iso] [outer] [inner] [PASS] childA                   -- mixed-result: passing siblings only
[error] [iso] [outer] [inner] [bad] [FAIL] ...                 -- per-test failure with full hierarchy
[error] [iso] [outer] [inner] [FAIL] ...                       -- suite rollup with failure
[debug] [iso] [outer] [inner] [test-name] [PASS] (Nms)         -- detail with MONOCHROMATIC_VERBOSE=true
```

Parent-children relationships are visible per line by default;
 no verbosity flag
needed.
 Levels:

- **`info`** -- per-suite `PASS childA, childB (Nms)` enumeration line and
  `SKIP` notices from `it`.
- **`error`** -- `FAIL` for tests and suites;
   always visible regardless of
  verbosity.
- **`debug`** -- per-test `PASS (Nms)`,
   per-suite `start (concurrency: N)`
  traces,
   and the rollup line for empty-name (invisible) suites.
   Hidden by
  default;
   set `MONOCHROMATIC_VERBOSE=true` or pass `--verbose` to surface.

The logger's environment switch is verified against
`package/logger/core/src/sink/console.ts:55` at source
`e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
The isolated verbosity-only control retains ten debug lifecycle lines from a failing child.
The subsequent both-stream capture passes an actual stdout/stderr positive control
and joins all 179 product source-case labels and nine separate harness-control labels
to named debug `PASS` records in successful-child stderr;
see `doc/audit/depot-local-node-capture-correction-2026-10-18.evidence.json`.
The earlier wrapper's empty successful-stderr fields remain unobserved placeholders.

When a failure surfaces,
 read the rightmost (deepest) tag plus the `FAIL` line
to identify the failing test;
 walk leftward through tags to follow the suite
nesting back to the file's top-level `describe`.
 The full `Error.cause` chain
mirrors the tag chain.

## Linting test code

### Testing intentional violations

When tests intentionally violate a lint rule to verify behavior:

```ts
// BAD: Adding data to satisfy the linter
expect(isError(new Error('test message',),),).toBe(true,);

// GOOD: Use disable comments for intentional violations
// oxlint-disable-next-line unicorn/error-message -- Testing error without message
expect(isError(new Error(),),).toBe(true,);
```

For known testing issues (duplicate describe blocks,
 missing test output,
 misattributed logs),
see [testing troubleshooting](../../../doc/troubleshooting/testing.md).
