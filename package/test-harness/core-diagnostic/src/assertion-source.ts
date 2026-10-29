/**
 Reconstructs the assertion expression behind a failure so a value-only
 message like `expected 3 to equal 2` carries the code that produced
 it. Walks an error (and its `.cause` / `AggregateError.errors` tree)
 to its first non-harness stack frame, reads that source file, and
 renders the assertion as a single line.
 
 The reader is node-only: it reads source files off disk. In the
 browser build there is no filesystem, so {@link readAssertionSites}
 short-circuits to an empty map and failures render exactly as before
 (message plus frames, no source line).
 
 Why a backward window rather than the single frame line: this repo
 splits assertions across lines, so the throw originates on the
 matcher line (`.toBe(2,)`), not the `expect(...)` line that names the
 subject. Reading only the frame's line would show the expected value
 but not the subject that was wrong. {@link extractAssertionExpression}
 walks up to the nearest `expect(` so subject, matcher, and expected
 all appear.
 
 Absence (an unparseable frame, an unreadable file, an out-of-range
 line) is handled by guard-and-narrow: the frame loop `continue`s past
 it. No helper returns a `T | undefined`; the pure helpers are total.
 
 @module
 */

import {
  isHarnessInternalFrame,
  readProperty,
} from './harness-frames.ts';

//region Tuning constants

/**
 How many lines to walk back from the frame line looking for the
 opening `expect(`. Bounds the window so a frame that is not an
 assertion (a bare helper call) renders just its own line instead of
 scanning the whole file.
 */
const MAX_LOOKBACK_LINES = 12;

/**
 Longest rendered expression before truncation, so a sprawling
 multi-line assertion cannot blow up the single-line failure summary.
 */
const MAX_EXPRESSION_LENGTH = 200;

/**
 Marker appended when an expression is truncated at
 {@link MAX_EXPRESSION_LENGTH}.
 */
const TRUNCATION_MARKER = '...';

/**
 Substring that opens an assertion in this harness. Matches the global
 `expect(`, the scoped `ctx.expect(`, and any `.expect(` member form,
 since all contain this fragment.
 */
const ASSERTION_OPENER = 'expect(';

/**
 `lastIndexOf` / `indexOf` sentinel for "not found", named so the
 existence checks read as comparisons against a known marker.
 */
const NOT_FOUND = -1;

//endregion Tuning constants

//region Types

/**
 Rendered assertion site: a short `file:line` location plus the
 reconstructed expression.
 
 @internal
 */
export type AssertionSite = {
  /**
   `basename:line` of the assertion, kept short because the full path
   already appears in the rendered stack frame.
   */
  readonly location: string;
  /**
   Single-line assertion expression, e.g.
   `expect(errorSpy.callCount,).toBe(2,)`.
   */
  readonly expression: string;
};

//endregion Types

//region Pure string helpers

/**
 Returns the basename of a path by scanning for the last `/` or `\`.
 Avoids importing `node:path` so this module stays loadable in the
 browser build (where {@link readAssertionSites} no-ops but the module
 still evaluates).
 
 @param path - filesystem path
 
 @returns final path segment
 
 @example
 ```ts
 basename('package/foo/src/bar.ts') // 'bar.ts'
 ```
 */
export function basename(path: string,): string {
  /**
   Rightmost separator across POSIX and Windows forms; -1 when the path has no separator.
   */
  const lastSeparator = Math.max(
    path.lastIndexOf('/',),
    path.lastIndexOf('\\',),
  );
  return path.slice(lastSeparator + 1,);
}

/**
 Extracts the location-bearing substring from a frame: the text inside
 the last balanced `(...)` when the frame names a function
 (`at fn (/abs/file.ts:21:12)`), otherwise the frame with a leading
 `at ` stripped (`at /abs/file.ts:21:12`). A leading `file://` URL
 scheme is removed so the result is a plain filesystem path. Total: a
 frame with no recognisable shape returns its own trimmed text, which
 the caller's numeric guards then reject.
 
 @param frame - trimmed stack frame line
 
 @returns substring expected to hold `path:line:col`, scheme-stripped
 
 @internal
 
 @example
 ```ts
 extractLocationSubstring('at fn (file:///abs/file.ts:21:12)') // '/abs/file.ts:21:12'
 extractLocationSubstring('at /abs/file.ts:9:3')               // '/abs/file.ts:9:3'
 ```
 */
export function extractLocationSubstring(frame: string,): string {
  /**
   Closing paren of the location group; -1 when the frame is the bare `at <path>` form.
   */
  const closeParen = frame.lastIndexOf(')',);
  /**
   Opening paren matched to `closeParen`; checked for ordering so a stray `)` does not produce a negative slice.
   */
  const openParen = frame.lastIndexOf('(',);
  /**
   Body before scheme stripping: parenthesised group when present, else the frame minus a leading `at `.
   */
  const body = (closeParen > openParen) && (openParen !== NOT_FOUND)
    ? frame.slice(
      openParen + 1,
      closeParen,
    )
    : (frame.startsWith('at ',)
      ? frame.slice('at '.length,)
      : frame);

  return body.startsWith('file://',)
    ? body.slice('file://'.length,)
    : body;
}

/**
 True when every character of a non-empty string is an ASCII digit.
 Used to validate stack-frame line / column segments without a regex.
 
 @param value - candidate numeric segment
 
 @returns whether `value` is a non-empty run of digits
 
 @internal
 
 @example
 ```ts
 isIntegerString('21') // true
 isIntegerString('')   // false
 isIntegerString('a1') // false
 ```
 */
export function isIntegerString(value: string,): boolean {
  if (value === '')
    return false;
  for (const char of value)
    if ((char < '0') || (char > '9'))
      return false;
  return true;
}

//endregion Pure string helpers

//region Source-window extraction

/**
 Walks upward from `targetIndex` to `lowerBound` looking for the line
 that opens the assertion ({@link ASSERTION_OPENER}). Returns that
 line's index, or `targetIndex` when none is found in range.
 
 @param sourceLines - file split into lines
 
 @param targetIndex - 0-based matcher line index
 
 @param lowerBound - lowest index the walk may inspect
 
 @returns index where the assertion window begins
 */
function findOpenerIndex({
  sourceLines,
  targetIndex,
  lowerBound,
}: {
  readonly sourceLines: readonly string[];
  readonly targetIndex: number;
  readonly lowerBound: number;
},): number {
  for (let index = targetIndex; index >= lowerBound; index -= 1) {
    /**
     Current candidate line, captured so the opener check reads it once.
     */
    const line = sourceLines[index];
    if ((line !== undefined) && line.includes(ASSERTION_OPENER,))
      return index;
  }
  return targetIndex;
}

/**
 Reconstructs the assertion expression ending on `lineNumber` by
 walking backward (up to {@link MAX_LOOKBACK_LINES}) to the nearest
 line that opens an `expect(`, then joining that window into one line.
 Total: an out-of-range `lineNumber` yields `''`, which the caller
 treats as "no expression"; a target with no opener in range yields
 just that line trimmed.
 
 @param sourceLines - file split into lines (no trailing-newline element required)
 
 @param lineNumber - 1-based line the failing frame points at (the matcher line)
 
 @returns single-line expression, or `''` when `lineNumber` is out of range
 
 @internal
 
 @example
 ```ts
 extractAssertionExpression({
   sourceLines: ['expect(errorSpy.callCount,)', '  .toBe(2,);'],
   lineNumber: 2,
 }) // 'expect(errorSpy.callCount,) .toBe(2,);'
 ```
 */
export function extractAssertionExpression({
  sourceLines,
  lineNumber,
}: {
  readonly sourceLines: readonly string[];
  readonly lineNumber: number;
},): string {
  /**
   0-based index of the matcher line the stack frame blamed.
   */
  const targetIndex = lineNumber - 1;
  if ((targetIndex < 0) || (targetIndex >= sourceLines.length))
    return '';

  /**
   Lowest index the backward walk may reach, clamped to the start of file.
   */
  const lowerBound = Math.max(
    0,
    targetIndex - MAX_LOOKBACK_LINES,
  );
  /**
   Index where the window starts; slides up to the nearest `expect(` opener, else stays on the target line.
   */
  const startIndex = findOpenerIndex({
    sourceLines,
    targetIndex,
    lowerBound,
  },);

  /**
   Window lines trimmed and joined into one expression, blank lines dropped so wrapped calls read cleanly.
   */
  const joined = sourceLines
    .slice(
    startIndex,
    targetIndex + 1,
    )
    .map(function trimLine(line,) {
      return line.trim();
    },)
    .filter(function nonEmpty(line,) {
      return line !== '';
    },)
    .join(' ',);

  return joined.length > MAX_EXPRESSION_LENGTH
    ? `${joined.slice(
      0,
      MAX_EXPRESSION_LENGTH - TRUNCATION_MARKER.length,
    )}${TRUNCATION_MARKER}`
    : joined;
}

//endregion Source-window extraction

//region Error-tree assertion-site reading

/**
 True when running under node (has a filesystem). The browser build
 has no `node:fs`, so the reader must not attempt the dynamic import
 there.
 
 @returns whether `process.versions.node` is present
 */
function isNodeRuntime(): boolean {
  return ((typeof process) !== 'undefined')
    && ((typeof process.versions) === 'object')
    && ((typeof process.versions
      .node) === 'string');
}

/**
 Scans one error's `.stack` for the first non-harness frame whose
 source file can be read, extracts the assertion window from it, and
 records the result under `node` in `sites`. Records nothing when no
 frame yields a readable source line. Every absence path is a
 `continue` guard, so there is no optional return.
 
 @param node - error-like node whose `.stack` to scan and key to record under
 
 @param sites - shared map mutated with this node's site when one resolves
 
 @param readFile - `node:fs/promises` `readFile`, injected so the
   dynamic import happens once in {@link readAssertionSites}
 
 @mutates node - `Reflect.get` may invoke getters or proxy traps on error-like value.
 */
async function recordSiteForError({
  node,
  sites,
  readFile,
}: {
  readonly node: object;
  readonly sites: WeakMap<object, AssertionSite>;
  readonly readFile: (
    path: string,
    encoding: 'utf8',
  ) => Promise<string>;
},): Promise<void> {
  /**
   Defensively-read `.stack`; a hostile getter degrades to no site rather than throwing.
   */
  const stack = readProperty({
    source: node,
    key: 'stack',
  },);
  if ((typeof stack) !== 'string')
    return;

  for (const rawLine of stack.split('\n',)) {
    /**
     Trimmed frame, skipped unless it is an `at ...` frame outside the harness's own machinery.
     */
    const frame = rawLine.trim();
    if ((!frame.startsWith('at ',)) || isHarnessInternalFrame(frame,))
      continue;

    /**
     Scheme-stripped `path:line:col` text; numeric guards below reject anything without a line number.
     */
    const locationText = extractLocationSubstring(frame,);
    /**
     Final colon, separating the column (or line, when no column) from the rest.
     */
    const lastColon = locationText.lastIndexOf(':',);
    if (lastColon === NOT_FOUND)
      continue;

    /**
     Everything before the final colon, re-split to tell `path:line:col` from `path:line`.
     */
    const beforeLast = locationText.slice(
      0,
      lastColon,
    );
    /**
     Tail after the final colon: the column in `path:line:col`, or the line in `path:line`.
     */
    const lastSegment = locationText.slice(lastColon + 1,);
    /**
     Penultimate colon, separating the line number from the path when a column is present.
     */
    const penultimateColon = beforeLast.lastIndexOf(':',);
    /**
     Segment between the two trailing colons: the line number in the `path:line:col` shape.
     */
    const penultimateSegment = penultimateColon === NOT_FOUND
      ? ''
      : beforeLast.slice(penultimateColon + 1,);

    /**
     Whether the frame carries `path:line:col` (two trailing integer segments).
     */
    const hasColumn = isIntegerString(penultimateSegment,) && isIntegerString(lastSegment,);
    if ((!hasColumn) && (!isIntegerString(lastSegment,)))
      continue;

    /**
     Source path, taken before the line:col tail (column form) or before the single line tail.
     */
    const path = hasColumn
      ? beforeLast.slice(
        0,
        penultimateColon,
      )
      : beforeLast;
    /**
     1-based line number the frame blamed, from the line segment of whichever shape matched.
     */
    const lineNumber = Number(hasColumn ? penultimateSegment : lastSegment,);

    /**
     Source text of the candidate file; an unreadable path (synthetic frame, deleted file) skips to the next frame.
     */
    // oxlint-disable-next-line no-await-in-loop -- frames are tried in stack order and the first readable one wins; the read must resolve before deciding whether to try the next frame
    const source = await readSourceText({
      readFile,
      path,
    },);
    if (source === '')
      continue;

    /**
     File split into lines once, reused for the range guard and the window extraction.
     */
    const sourceLines = source.split('\n',);
    if (lineNumber > sourceLines.length)
      continue;

    /**
     Reconstructed assertion expression; empty when the blamed line is blank.
     */
    const expression = extractAssertionExpression({
      sourceLines,
      lineNumber,
    },);
    if (expression === '')
      continue;

    sites.set(
      node,
      {
        location: `${basename(path,)}:${String(lineNumber,)}`,
        expression,
      },
    );
    return;
  }
}

/**
 Reads a file, returning `''` instead of throwing when the path is
 unreadable (synthetic test frame, deleted source, permission error).
 Empty is a safe sentinel: a real source file that an assertion lives
 in is never zero bytes, and {@link recordSiteForError} treats `''` as
 "try the next frame".
 
 @param readFile - injected `node:fs/promises` `readFile`
 
 @param path - filesystem path to read
 
 @returns file contents, or `''` on any read error
 */
async function readSourceText({
  readFile,
  path,
}: {
  readonly readFile: (
    path: string,
    encoding: 'utf8',
  ) => Promise<string>;
  readonly path: string;
},): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error: unknown) {
    if (!Error.isError(error,))
      throw error;
    return '';
  }
}

/**
 Walks a thrown value's error tree (`.cause` chain plus
 `AggregateError.errors`) and reads the assertion site for each node,
 keyed by the error object so {@link ../format-error.ts} can splice the
 site into that node's rendered line. Iterative work-stack walk
 (cause/aggregate form a tree, but a deep cause chain is a degenerate
 spine, so no recursion); cycle-safe via a {@link WeakSet}.
 
 No-ops to an empty map in the browser build, where there is no
 filesystem to read source from.
 
 @param value - thrown value of unknown shape
 
 @returns map from each error node to its rendered assertion site
 
 @internal
 
 @example
 ```ts
 const sites = await readAssertionSites(caughtAssertionError);
 sites.get(caughtAssertionError)?.expression // 'expect(x,).toBe(2,)'
 ```
 */
export async function readAssertionSites(
  value: unknown,
): Promise<WeakMap<object, AssertionSite>> {
  /**
   Result map; stays empty in the browser and when no node yields a readable source line.
   */
  const sites = new WeakMap<object, AssertionSite>();
  if (!isNodeRuntime())
    return sites;

  /**
   `node:fs/promises` `readFile`, imported dynamically so the browser bundle never pulls a node builtin at load.
   */
  const { readFile, } = await import('node:fs/promises');

  /**
   Cycle guard so a self-referential `.cause` cannot loop the walk.
   */
  const visited = new WeakSet<object>();
  /**
   Flattened list of error nodes, collected iteratively before reading so the reads can run concurrently.
   */
  const nodes: object[] = [];
  /**
   Work stack seeded with the root thrown value; tree children are pushed as they are discovered.
   */
  const pending: unknown[] = [value,];

  while (pending.length > 0) {
    /**
     Next node to inspect; non-objects and already-visited nodes are skipped.
     */
    const node: unknown = pending.pop();
    if (((typeof node) !== 'object') || (node === null)
      || visited.has(node,))
      continue;
    visited.add(node,);
    nodes.push(node,);

    /**
     Cause subtree, pushed so a wrapped assertion failure still resolves its source line.
     */
    const cause = readProperty({
      source: node,
      key: 'cause',
    },);
    if (cause !== undefined)
      pending.push(cause,);

    /**
     Aggregate members, pushed so each failure in an `AggregateError` resolves its own source line.
     */
    const errors = readProperty({
      source: node,
      key: 'errors',
    },);
    if (Array.isArray(errors,))
      for (const member of errors)
        pending.push(member,);
  }

  await Promise.all(nodes.map(
    /**
     Records assertion site for one retained error node.
     
     @param node - Error node whose stack getter may be invoked.
     
     @returns completion after optional source read.
     
     @mutates node - `Reflect.get` may invoke getters or proxy traps on error-like value.
     */
    function recordNode(node,) {
      return recordSiteForError({
        node,
        sites,
        readFile,
      },);
    },),);

  return sites;
}

//endregion Error-tree assertion-site reading
