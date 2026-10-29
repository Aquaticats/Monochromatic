/**
 Renders an error value and its `.cause` chain plus any
 `AggregateError.errors` as an array of lines, one line per error
 in the walk. Each line carries the error's header AND its stack
 frames joined inline for grep-friendliness.

 An error whose assertion compared two values (`actual` and `expected`)
 also emits one line per difference beneath its own line, so a failing
 `toEqual` names what differs instead of leaving both values truncated.
 
 Stack frames are cleaned before joining:
 
 - the current working directory prefix is stripped so absolute
   paths render as cwd-relative
 - frames pointing into the harness's own bundle
   (`package/test-harness/core/dist/`) are filtered out, because they
   are noise for diagnosing user-test failures and the runtime's
   own tail dump still carries the unfiltered chain when needed
 
 The tagged logger prepends its tag prefix once per call. Callers
 typically use {@link formatFailure} to fuse a `FAIL` summary with
 the first error line into one record, so the tag lands only on the
 summary; subsequent cause lines are untagged because readers
 already know which suite/test the error belongs to from the
 summary's tag.
 
 @module
 */

import {
  type AssertionSite,
  readAssertionSites,
} from './assertion-source.ts';
import {
  isHarnessInternalFrame,
  readProperty,
} from './harness-frames.ts';
import {
  errorDifferenceLines,
} from './value-difference.ts';

//region Workspace prefix resolution

/**
 Resolves the monorepo root directory (with trailing slash) by
 calling `findRootCached` with `MISE_MONOREPO` from
 `@monochromatic-dev/fs-path-core`. The shared cached variant
 memoises the result process-wide, so no local cache is needed.
 Falls back to `process.cwd()` when the cached variant rejects
 (no `mise.toml` with `[monorepo]` found), and to the empty string
 outside Node or when `process.cwd()` is unavailable.
 The filesystem dependency stays behind the Node runtime check
 so neutral browser consumers can import this module.
 
 Imports from module-fs-path create a workspace cycle (module-test
 is in module-fs-path's devDependencies); accepted because the
 cycle is build-time only and does not affect runtime resolution.
 
 @returns trailing-slashed monorepo root, cwd prefix, or empty
   string when unavailable
 */
async function resolveWorkspacePrefix(): Promise<string> {
  if (((typeof process) === 'undefined') || ((typeof process.versions
    ?.node) !== 'string'))
    return '';
  try {
    /**
     Workspace filesystem discovery belongs only to the Node diagnostic path.
     */
    const {
      findRootCached,
      MISE_MONOREPO,
    } = await import('@monochromatic-dev/fs-path-core/ts');
    /**
     Captured root so the trailing slash can be appended exactly once before returning.
     */
    const root = await findRootCached({ marker: MISE_MONOREPO, },);
    return `${root}/`;
  }
  catch (error: unknown) {
    if (!Error.isError(error,))
      throw error;
    /**
     The cached variant rejected (no mise.toml with [monorepo],
     browser without OPFS, etc.) and caches the rejection, so the
     fallback path runs on every call. Compute the cwd-based prefix
     directly; `process.cwd()` is cheap (single syscall).
     */
    if ((typeof process) === 'undefined')
      return '';
    try {
      return `${process.cwd()}/`;
    }
    catch (cwdError: unknown) {
      if (!Error.isError(cwdError,))
        throw cwdError;
      return '';
    }
  }
}

//endregion Workspace prefix resolution

/**
 Safely stringifies an arbitrary value for inline display.
 Wraps `String(value)` so trapped getters or cross-realm objects
 cannot derail logging.
 
 @param value - value to render
 
 @returns single-line string description, or `<unrepresentable>`
   when stringification throws
 
 @mutates value - `String` may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.
 
 @example
 ```ts
 safeString('boom') // 'boom'
 safeString(42)     // '42'
 safeString(null)   // 'null'
 ```
 */
function safeString(value: unknown,): string {
  try {
    return String(value,);
  }
  catch (error: unknown) {
    if (!Error.isError(error,))
      throw error;
    return '<unrepresentable>';
  }
}

/**
 Extracts the message string from an error-like object.
 Falls back to `<unknown message>` when the field is missing or
 not a string.
 
 @param error - error-like object whose `.message` to read
 
 @returns message string or fallback marker
 
 @mutates error - `Reflect.get` may invoke getters or proxy traps on error-like value.
 
 @example
 ```ts
 readMessage(new Error('boom')) // 'boom'
 readMessage({})                // '<unknown message>'
 readMessage({ message: 42 })   // '<unknown message>'
 ```
 */
function readMessage(error: object,): string {
  /**
   Defensively-read `.message`; held in a local so the getter fires at most once.
   */
  const message = readProperty({
    source: error,
    key: 'message',
  },);
  if ((typeof message) === 'string')
    return message;
  return '<unknown message>';
}

/**
 Extracts the class label from an error-like object's `.name`.
 Falls back to `Error` when missing or not a non-empty string.
 
 @param error - error-like object whose `.name` to read
 
 @returns class label such as `TypeError`, `AggregateError`, or
   `Error` as the default
 
 @mutates error - `Reflect.get` may invoke getters or proxy traps on error-like value.
 
 @example
 ```ts
 readErrorLabel(new TypeError('x')) // 'TypeError'
 readErrorLabel({})                 // 'Error'
 ```
 */
function readErrorLabel(error: object,): string {
  /**
   Defensively-read `.name`; held in a local so the getter fires at most once.
   */
  const label = readProperty({
    source: error,
    key: 'name',
  },);
  if (((typeof label) === 'string') && (label !== ''))
    return label;
  return 'Error';
}

/**
 Strips the resolved workspace prefix from a frame so absolute
 paths render as cwd-relative. No-op when the prefix is empty.
 
 @param frame - trimmed stack frame line
 
 @param workspacePrefix - resolved prefix from
   {@link resolveWorkspacePrefix}
 
 @returns frame with workspace prefix removed where present
 */
function stripWorkspacePrefix({
  frame,
  workspacePrefix,
}: {
  readonly frame: string;
  readonly workspacePrefix: string;
},): string {
  if (workspacePrefix === '')
    return frame;
  return frame.replaceAll(
    workspacePrefix,
    '',
  );
}

/**
 Splits an error's `.stack` into trimmed non-empty frames, dropping
 the first line when it duplicates the `ErrorName: message` header
 (the runtime convention on V8 and JavaScriptCore). Strips the
 workspace prefix from each frame and filters out frames pointing
 into the harness's own bundle.
 
 @param error - error-like object whose `.stack` to read
 
 @param message - already-extracted message used to detect the
   header-duplicate first line
 
 @param workspacePrefix - resolved prefix from
   {@link resolveWorkspacePrefix}
 
 @returns trimmed, prefix-stripped, harness-filtered stack frames;
   empty when `.stack` is missing or not a string
 
 @mutates error - `Reflect.get` may invoke getters or proxy traps on error-like value.
 
 @example
 ```ts
 await readStackFrames({
   error: new Error('boom'),
   message: 'boom',
   workspacePrefix: '/var/home/user/Monochromatic/',
 }) // ['at fn (package/foo/file.ts:9:19)']
 ```
 */
function readStackFrames({
  error,
  message,
  workspacePrefix,
}: {
  readonly error: object;
  readonly message: string;
  readonly workspacePrefix: string;
},): readonly string[] {
  /**
   Defensively-read `.stack`; held in a local so the getter fires at most once.
   */
  const stack = readProperty({
    source: error,
    key: 'stack',
  },);
  if ((typeof stack) !== 'string')
    return [];
  /**
   Raw newline-split stack lines, before header-line trimming and per-frame cleanup.
   */
  const rawLines = stack
    .split('\n',);
  /**
   V8 and JavaScriptCore prefix the stack with `ErrorName: message`
   on the first line. Drop it so the caller does not show the
   header twice; otherwise keep all lines.
   */
  const startIndex = (rawLines[0]
    !== undefined)
    && rawLines[0]
      .includes(message,)
    ? 1
    : 0;
  return rawLines
    .slice(startIndex,)
    .map(function trimLine(line,) {
      return line.trim();
    },)
    .filter(function nonEmpty(line,) {
      return line !== '';
    },)
    .filter(function notHarnessInternal(frame,) {
      /**
       Filter using the un-stripped path so the fragment matches
       the workspace-qualified `package/test-harness/core/dist/` and the
       node_modules-qualified `test-harness-core/dist/` consistently.
       */
      return !isHarnessInternalFrame(frame,);
    },)
    .map(function applyWorkspaceStrip(frame,) {
      return stripWorkspacePrefix({
        frame,
        workspacePrefix,
      },);
    },);
}

/**
 Recursively renders an error node and its descendants as one line
 per error, with stack frames joined inline after the header for
 grep-friendliness.
 
 Cycle-safe: visited error objects are tracked in a shared
 {@link WeakSet}; on revisit, emits `... (cycle)` and stops the
 recursion at that branch.
 
 @param headerPrefix - text prepended to the node's line
   (empty for the root, `Caused by: ` for a cause node, `[N/M] `
   for an aggregate member)
 
 @param value - error-like value at this node
 
 @param visited - shared cycle-detection set, mutated as the walk
   descends
 
 @param workspacePrefix - resolved prefix from
   {@link resolveWorkspacePrefix}
 
 @returns one line for this node followed by lines for all of its
   descendants in walk order (cause first, then aggregate members)
 
 @mutates value - `Reflect.get` and `String` may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf` on error-like values.
 */
function formatNode({
  headerPrefix,
  value,
  visited,
  workspacePrefix,
  sites,
}: {
  readonly headerPrefix: string;
  readonly value: unknown;
  readonly visited: WeakSet<object>;
  readonly workspacePrefix: string;
  readonly sites: WeakMap<object, AssertionSite>;
},): readonly string[] {
  if (((typeof value) !== 'object') || (value === null))
    return [`${headerPrefix}Threw non-Error value: ${safeString(value,)}`,];

  if (visited.has(value,))
    return [`${headerPrefix}... (cycle)`,];

  visited.add(value,);

  /**
   Extracted message kept in a local so it can be reused for header construction and stack-header trimming.
   */
  const message = readMessage(value,);
  /**
   Class label rendered ahead of the message; held in a local for the same reason as `message`.
   */
  const label = readErrorLabel(value,);
  /**
   Cleaned stack frames produced once per node so the cause and aggregate branches both see the same set.
   */
  const frames = readStackFrames({
    error: value,
    message,
    workspacePrefix,
  },);
  /**
   Join frames with a single space so the whole error fits on one
   line: `Error: boom at fn (file:9:19) at runFnOnce (...)`. Empty
   stack contributes no trailing whitespace.
   */
  const framesInline = frames.length
    > 0
    ? ` ${frames.join(' ',)}`
    : '';
  /**
   Assertion site for this node, when its first non-harness frame
   resolved to a readable source line. `value` is a non-null object
   here (guarded above), so the map lookup is well-typed.
   */
  const site = sites.get(value,);
  /**
   Rendered assertion expression spliced between message and frames,
   turning `expected 3 to equal 2` into a line that also shows
   `@console.unit.test.ts:308 expect(errorSpy.callCount,).toBe(2,)`.
   Empty when no source line was resolved.
   */
  const siteInline = site !== undefined
    ? `  @${site.location} ${site.expression}`
    : '';
  /**
   Composed header-plus-site-plus-frames string for this node, prepended to the descendants in the return list.
   */
  const line = `${headerPrefix}${label}: ${message}${siteInline}${framesInline}`;

  /**
   Difference lines for an error whose assertion compared two values,
   empty elsewhere, kept under the error's own line so the headline
   still reads first.
   */
  const differenceLines = errorDifferenceLines({ error: value, },)
    .map(function prefixDifference(differenceLine,) {
      return `  ${differenceLine}`;
    },);

  /**
   Cause value, defensively read, pulled out so the recursion only runs once when a cause exists.
   */
  const causeValue: unknown = readProperty({
    source: value,
    key: 'cause',
  },);
  /**
   Recursively rendered cause subtree, kept separate from `errorLines` so cause precedes aggregate members.
   */
  const causeLines = causeValue !== undefined
    ? formatNode({
      headerPrefix: 'Caused by: ',
      value: causeValue,
      visited,
      workspacePrefix,
      sites,
    },)
    : [];

  /**
   `errors` field, defensively read, pulled out so the array check and subsequent iteration both refer to the same captured value.
   */
  const errorsField: unknown = readProperty({
    source: value,
    key: 'errors',
  },);
  /**
   Recursively rendered aggregate members, appended after `causeLines` to keep walk order stable.
   */
  const errorLines: readonly string[] = Array.isArray(errorsField,)
    ? errorsField.flatMap(
      /**
       Formats one retained aggregate error member.
       
       @param member - Aggregate member inspected through error formatting.
       
       @param index - Aggregate member position.
       
       @returns formatted lines for aggregate member.
       
       @mutates member - `Reflect.get` and `String` may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.
       */
      function formatAggregateMember(
        member,
        index,
      ) {
        return formatNode({
          headerPrefix: `[${String(index + 1,)}/${String(errorsField.length,)}] `,
          value: member,
          visited,
          workspacePrefix,
          sites,
        },);
      },)
    : [];

  return [
    line,
    ...differenceLines,
    ...causeLines,
    ...errorLines,
  ];
}

/**
 Renders a thrown value and its `.cause` chain plus any
 `AggregateError.errors` as an array of single-line strings, one
 per error in the walk. Each line is
 `[prefix]<label>: <message> <frames>` so the whole error sits on
 one line and `grep` matches by message, class, or frame.
 
 Walks the cause chain depth-first, then expands aggregate members
 after a node's cause. Cycle-safe via a {@link WeakSet} of visited
 error objects; on revisit, emits `... (cycle)` and stops descending
 at that branch.
 
 Non-Error throws (`throw 'oops'`, `throw 42`, `throw null`) render
 as a single `Threw non-Error value: ...` line. Missing `.message`
 renders as `<unknown message>`. Missing `.stack` contributes no
 frames. Property reads are getter-safe (see {@link readProperty}): a
 `.message`/`.name`/`.cause`/`.stack`/`.errors` accessor that throws
 degrades to the missing-field fallback rather than propagating, so the
 walk never rejects on a hostile error object.
 
 @param value - thrown value of unknown shape
 
 @returns single-line strings; see {@link formatFailure} for the
   common pattern of fusing first line with outcome summary
 
 @internal
 
 @mutates value - `Reflect.get` and `String` may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf` on thrown values.
 
 @example
 ```ts
 try {
   throw new Error('outer', { cause: new Error('root', ), },);
 }
 catch (caught) {
   for (const line of await formatErrorDeep(caught,))
     l.error(line,);
 }
 // emits:
 //   Error: outer at fn (file:9:19) at runFnOnce (...)
 //   Caused by: Error: root at root (file:5:1) at ...
 ```
 */
export async function formatErrorDeep(value: unknown,): Promise<readonly string[]> {
  /**
   Shared cycle-detection set so a self-referential `.cause` does not recurse forever.
   */
  const visited = new WeakSet<object>();
  /**
   Workspace prefix (for frame stripping) and assertion sites (for the
   source line) resolved together: both are needed before the walk and
   neither depends on the other, so they run concurrently.
   */
  const [workspacePrefix, sites,] = await Promise.all([
    resolveWorkspacePrefix(),
    readAssertionSites(value,),
  ],);
  return formatNode({
    headerPrefix: '',
    value,
    visited,
    workspacePrefix,
    sites,
  },);
}

/**
 Fuses outcome summary with formatted error chain into a
 single multi-line string ready for one error-level logger call.
 The first error line (header plus inline stack frames) lands on the
 summary line for grep-friendliness; subsequent cause lines
 follow as newline-separated continuation. The tagged logger then
 prefixes its tag onto the summary line only; continuation lines
 remain untagged.
 
 @param summary - message body following dedicated `[FAIL]` tag
 
 @param value - thrown value to format
 
 @returns multi-line string suitable for error logger
 
 @internal
 
 @mutates value - `Reflect.get` and `String` may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf` on thrown values.
 
 @example
 ```ts
 try {
   throw new Error('outer', {
     cause: new Error('inner', { cause: new Error('root', ), }, ),
   },);
 }
 catch (caught) {
   failLogger.error(await formatFailure({
     summary: `(${String(durationMs,)}ms)`,
     value: caught,
   },),);
 }
 // emits (with outcome tag applied only to line 1):
 //   [FAIL] (5ms) Error: outer at fn (file:9:19) at runFnOnce (...)
 //   Caused by: Error: inner at otherFn (...) at ...
 //   Caused by: Error: root at root (...) at ...
 ```
 */
export async function formatFailure({
  summary,
  value,
}: {
  readonly summary: string;
  readonly value: unknown;
},): Promise<string> {
  /**
   Walked error chain reused across the empty-check and the summary fusion.
   */
  const lines = await formatErrorDeep(value,);
  if (lines.length
    === 0)
    return summary;
  /**
   First line is fused onto the summary so the tagged logger's prefix only renders once.
   */
  const [first, ...rest] = lines;
  return [
    `${summary} ${first}`,
    ...rest,
  ]
    .join('\n',);
}
