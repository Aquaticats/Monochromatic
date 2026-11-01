/**
 Stack-frame classification shared by the failure formatter
 ({@link ../format-error.ts}) and the assertion-site source reader
 ({@link ../assertion-source.ts}). Holds the fragments that identify
 the harness's own dispatch machinery plus the getter-safe property
 reader both modules need.
 
 Split into its own module so `assertion-source.ts` can reuse the
 classifier without importing `format-error.ts`, which would create a
 cycle (`format-error.ts` already imports `assertion-source.ts` to
 enrich failures with their source line).
 
 @module
 */

//region Getter-safe property read

/**
 Reads a property from an error-like object, returning `undefined`
 when the key is absent or when accessing it throws. Error objects
 can carry a throwing getter (or a `Proxy` `get` trap) on `.message`,
 `.cause`, `.stack`, and similar; reading them directly would let the
 getter's throw escape mid-walk. `Reflect.get` returns `undefined`
 for an absent key, and the `try` swallows a throwing read, so callers
 degrade gracefully (treat the field as missing) instead of
 propagating the trap.
 
 @param source - object to read from

 @param key - property key to read; symbols pass through since record
   walks read symbol keys

 @returns property value, or `undefined` when absent or unreadable

 @mutates source - `Reflect.get` may invoke caller-defined getters or proxy traps.

 @example
 ```ts
 readProperty({ source: new Error('boom'), key: 'message', }) // 'boom'
 readProperty({ source: {}, key: 'message', })                // undefined
 ```
 */
export function readProperty({
  source,
  key,
}: {
  readonly source: object;
  readonly key: string | symbol;
},): unknown {
  try {
    return Reflect.get(
      source,
      key,
    );
  }
  catch (error: unknown) {
    if (!Error.isError(error,))
      throw error;
    return undefined;
  }
}

//endregion Getter-safe property read

//region Harness frame fragments

/**
 Substrings identifying stack frames inside the harness's bundled
 output or the vendored assertion stack. Any frame containing one of
 these is dispatch machinery, never the failing user code, so it is
 dropped from rendered traces.
 
 Three categories:
 
 - Runner, assertion, sandbox, and diagnostic bundles, under workspace
   paths (`package/test-harness/core-expect/dist/`) and installed package paths
   (`test-harness-core-expect/dist/`). The source-export counterpart lives in
   {@link HARNESS_SOURCE_FRAGMENTS}, which needs the test-file guard.
 
 - The vendored assertion stack. `expect()` lands in `chai`, which
   dispatches through `chai-as-promised` and `sinon-chai`, and `sinon`
   spies feed the sinon-chai matchers. These package paths identify
   assertion dispatch rather than the user's failing assertion.
 
 - `p-limit`, the concurrency limiter `describe.ts` uses to fan tests
   out. Its frames sit below the harness runner on every failure and
   carry no diagnostic value.
 
 Fragments match with substring `includes`, collapsing pnpm's virtual
 store layout
 (`node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js`) and the
 flat / hoisted layout (`node_modules/chai/index.js`) under one entry
 per package. The trailing slash on each package fragment prevents
 `chai/` matching `chai-as-promised/` and `sinon/` matching
 `sinon-chai/`.
 */
export const HARNESS_INTERNAL_FRAGMENTS: readonly string[] = [
  'package/test-harness/core/dist/',
  'test-harness-core/dist/',
  'package/test-harness/core-expect/dist/',
  'test-harness-core-expect/dist/',
  'package/test-harness/core-sandbox/dist/',
  'test-harness-core-sandbox/dist/',
  'package/test-harness/core-diagnostic/dist/',
  'test-harness-core-diagnostic/dist/',
  'node_modules/chai/',
  'node_modules/chai-as-promised/',
  'node_modules/sinon-chai/',
  'node_modules/sinon/',
  'node_modules/p-limit/',
];

/**
 Substrings identifying frames inside the harness's own **source**
 dispatch files (`it.ts`, `describe.ts`, `descriptor.ts`,
 `expect*.ts`, `format-error.ts`, ...). Consumers import the harness
 via `/ts` exports, which map to `src/`, so those dispatch frames
 must also be hidden. Both workspace and installed package forms
 are listed for the runner and every extracted responsibility.
 
 Unlike {@link HARNESS_INTERNAL_FRAGMENTS} these match only when the
 frame is **not** a test file: the harness's own test suite lives
 under `package/test-harness/core/src/` too, and those frames are the user
 code when test-harness/core tests itself. {@link isTestFileFrame} gates
 the match.
 */
export const HARNESS_SOURCE_FRAGMENTS: readonly string[] = [
  'package/test-harness/core/src/',
  'test-harness-core/src/',
  'package/test-harness/core-expect/src/',
  'test-harness-core-expect/src/',
  'package/test-harness/core-sandbox/src/',
  'test-harness-core-sandbox/src/',
  'package/test-harness/core-diagnostic/src/',
  'test-harness-core-diagnostic/src/',
];

//endregion Harness frame fragments

//region Frame classification

/**
 True when the frame points at a test file (`*.test.ts`,
 `*.unit.test.ts`, `*.property.unit.test.ts`, ...). Used to spare the
 harness's own test frames from the source-dispatch filter: those
 frames are the user code when test-harness/core tests itself.
 
 @param frame - trimmed stack frame line
 
 @returns whether the frame's path contains a `.test.` segment
 
 @example
 ```ts
 isTestFileFrame('at fn (package/foo/src/a.unit.test.ts:9:1)') // true
 isTestFileFrame('at fn (package/test-harness/core/src/it.ts:9:1)')  // false
 ```
 */
export function isTestFileFrame(frame: string,): boolean {
  return frame.includes('.test.',);
}

/**
 True when the frame belongs to the harness's dispatch machinery and
 should be dropped from a rendered trace. A frame qualifies when it
 matches a bundle / vendored-assertion fragment unconditionally, or a
 source-dispatch fragment while not being a test file.
 
 The failing user code is the interesting site; everything this
 function flags is the plumbing under it.
 
 @param frame - trimmed stack frame line
 
 @returns whether the frame should be dropped
 
 @example
 ```ts
 isHarnessInternalFrame('at toBe (package/test-harness/core/src/expect-matchers-core.ts:70:10)') // true
 isHarnessInternalFrame('at fn (package/test-harness/core/src/it.unit.test.ts:21:12)')           // false
 isHarnessInternalFrame('at userFn (package/foo/src/bar.ts:9:19)')                         // false
 ```
 */
export function isHarnessInternalFrame(frame: string,): boolean {
  if (HARNESS_INTERNAL_FRAGMENTS.some(function frameContainsBundle(fragment,) {
    return frame.includes(fragment,);
  },))
    return true;

  if ((!isTestFileFrame(frame,)) && HARNESS_SOURCE_FRAGMENTS.some(function frameContainsSource(fragment,) {
    return frame.includes(fragment,);
  },))
    return true;

  return false;
}

//endregion Frame classification
