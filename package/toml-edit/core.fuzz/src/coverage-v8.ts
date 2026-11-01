/**
 Raw V8 coverage reader for the toml-edit fuzz coverage gate.
 
 Node v26 runs the package `.ts` source directly and its type stripping is
 position preserving (type syntax is blanked to whitespace, never shifted), so
 a `NODE_V8_COVERAGE` JSON range offset indexes the on-disk `.ts` source
 one-to-one. This module turns those raw ranges into a per-file covered-line
 count for the runtime package's implementation files (the coverage targets),
 excluding its conformance adapters and test files.
 
 The metric is V8 block coverage projected to lines: paint a per-character
 bitmap where the innermost range's count decides coverage (paint the longest
 range first so a never-taken block inside a covered function wins), then a
 source line counts as covered when it holds a non-whitespace character at a
 covered offset. Coverage is unioned at the line-set level so a file split
 across coverage processes stays the union of its runs, and no mutable bitmap
 or map is ever passed across a function boundary.
 
 @module
 */

import { relative, } from 'node:path';
import { fileURLToPath, } from 'node:url';

//region V8 JSON shapes

/**
 One V8 source range: a half-open character span and its execution count.
 */
export type V8Range = {
  readonly startOffset: number;
  readonly endOffset: number;
  readonly count: number;
};

/**
 One V8 function coverage entry: the function's own range plus block ranges.
 */
export type V8Function = {
  readonly functionName: string;
  readonly ranges: readonly V8Range[];
  readonly isBlockCoverage: boolean;
};

/**
 One covered script: its source URL and the function coverage entries.
 */
export type V8Script = {
  readonly url: string;
  readonly functions: readonly V8Function[];
};

/**
 One `NODE_V8_COVERAGE` output file: the array of covered scripts.
 */
export type V8CoverageFile = {
  readonly result: readonly V8Script[];
};

/**
 Whether parsed JSON has the coverage-file shape (a `result` array), narrowing
 from `unknown` without an `as` assertion.
 
 @param value - Parsed JSON to test before reading as coverage.
 
 @returns Whether `value` can be read as a coverage file.
 
 @example
 ```ts
 if (isCoverageFile(parsed)) {
   parsed.result.length;
 }
 ```
 */
export function isCoverageFile(value: unknown,): value is V8CoverageFile {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('result' in value)
    && Array.isArray(value.result,);
}

//endregion V8 JSON shapes

//region Public coverage shapes

/**
 Line coverage for one target file.
 */
export type FileCoverage = {
  /**
   Count of non-blank lines with a covered non-whitespace character.
   */
  readonly covered: number;
  /**
   Count of non-blank lines (the line denominator).
   */
  readonly total: number;
  /**
   One-based line numbers that hold code yet stayed uncovered.
   */
  readonly uncovered: readonly number[];
};

/**
 Per-target-file coverage, keyed by package-relative path (`src/foo.ts`).
 */
export type CoverageMap = Readonly<Record<string, FileCoverage>>;

//endregion Public coverage shapes

//region URL classification

/**
 Path fragments that mark a `src` `.ts` file as a conformance adapter or a
 test rather than a coverage target; the fuzz helpers themselves live in this
 sidecar and never match the runtime package root.
 */
const NON_TARGET_FRAGMENTS: readonly string[] = [
  '/conformance/',
  '.test.',
];

/**
 Classification of one V8 script URL against the coverage-target rule.
 */
export type UrlClass =
  | {
    readonly kind: 'target';
    readonly relPath: string;
    readonly absPath: string
  }
    | { readonly kind: 'other'; };

/**
 Classify a V8 script URL as a shipped-implementation coverage target or not.
 
 @returns Target descriptor for `src` implementation files under `packageRoot`;
   otherwise an `other` marker for dependencies, helpers, and test files.
 
 @example
 ```ts
 const cls = classifyUrl({ url: script.url, packageRoot, },);
 ```
 */
export function classifyUrl(
  {
    url,
    packageRoot,
  }: {
    readonly url: string;
    readonly packageRoot: string;
  },
): UrlClass {
  if (!url.startsWith('file:',)) return { kind: 'other', };
  /**
   Absolute filesystem path behind the `file:` URL.
   */
  const absPath = fileURLToPath(url,);
  /**
   Package-relative path used as the coverage key.
   */
  const rel = relative(
    packageRoot,
    absPath,
  );
  if (!rel.startsWith('src/',)) return { kind: 'other', };
  if (!rel.endsWith('.ts',)) return { kind: 'other', };
  if (NON_TARGET_FRAGMENTS.some(function present(fragment,) {
    return rel.includes(fragment,);
  },)) {
    return { kind: 'other', };
  }
  return {
    kind: 'target',
    relPath: rel,
    absPath,
  };
}

//endregion URL classification

//region Bitmap painting and line projection

/**
 Characters treated as whitespace, so a line of only these never counts as
 code: space, tab, carriage return, form feed, vertical tab.
 */
const WHITESPACE_CHARS = ' \t\r\f\v';

/**
 Paint a per-character covered bitmap from one script's ranges.
 
 Ranges nest; painting longest first lets the innermost range's count decide,
 so a never-taken block inside a covered function paints uncovered.
 
 @returns Bitmap of length `length` where `1` marks a covered character offset.
 */
function paintBitmap(
  {
    length,
    functions,
  }: {
    readonly length: number;
    readonly functions: readonly V8Function[];
  },
): Uint8Array {
  /**
   Every range across the script's functions, longest first.
   */
  const ranges = functions
    .flatMap(function rangesOf(fn,) { return fn.ranges; },)
    .toSorted(function byLengthDesc(
      a,
      b,
    ) {
      return (b.endOffset - b.startOffset) - (a.endOffset - a.startOffset);
    },);
  /**
   Covered bitmap, mutated as ranges paint over it from longest to shortest.
   */
  const bitmap = new Uint8Array(length,);
  for (const range of ranges) {
    /**
     Covered when the range executed at least once.
     */
    const fill = range.count > 0 ? 1 : 0;
    /**
     Clamp so a malformed range never writes past the source.
     */
    const end = Math.min(
      range.endOffset,
      length,
    );
    for (let offset = Math.max(
      range.startOffset,
      0,
    ); offset < end; offset += 1) {
      bitmap[offset] = fill;
    }
  }
  return bitmap;
}

/**
 Whether `line` holds at least one non-whitespace character (is a code line).
 
 @returns Whether the line denominator should count this line.
 */
function lineHasCode({ line, }: { readonly line: string; },): boolean {
  for (const char of line) {
    if (!WHITESPACE_CHARS.includes(char,)) return true;
  }
  return false;
}

/**
 Whether `line` holds a non-whitespace character at a covered bitmap offset.
 Indexed in code units so the offset aligns with V8's character offsets.
 
 @returns Whether the line counts as covered.
 */
function lineHasCoveredCode(
  {
    line,
    start,
    bitmap,
  }: {
    readonly line: string;
    readonly start: number;
    readonly bitmap: Uint8Array;
  },
): boolean {
  for (let index = 0; index < line.length; index += 1) {
    if ((!WHITESPACE_CHARS.includes(line[index] ?? ' ',)) && (bitmap[start + index] === 1)) return true;
  }
  return false;
}

/**
 Project one script's ranges onto the set of covered one-based line numbers.
 
 @returns Covered code-line numbers for this script.
 
 @example
 ```ts
 const covered = projectCovered({ source: 'x = 1\n', functions: [] });
 ```
 */
export function projectCovered(
  {
    source,
    functions,
  }: {
    readonly source: string;
    readonly functions: readonly V8Function[];
  },
): ReadonlySet<number> {
  /**
   Per-character covered bitmap, local to this projection.
   */
  const bitmap = paintBitmap({
    length: source.length,
    functions,
  },);
  /**
   Covered one-based line numbers accumulated over the source lines.
   */
  const covered = new Set<number>();
  source.split('\n',)
    .reduce(
    function fold(
      offset,
      line,
      index,
    ) {
      if (lineHasCoveredCode({
        line,
        start: offset,
        bitmap,
      },)) covered.add(index + 1,);
      return offset + line.length
        + 1;
    },
    0,
    );
  return covered;
}

/**
 Project a covered-line set onto per-file coverage counts.
 
 @returns Covered and total code-line counts plus the uncovered line numbers.
 
 @example
 ```ts
 const coverage = fileCoverageFrom({ source: 'x = 1\n', coveredLines: new Set([1]) });
 ```
 */
export function fileCoverageFrom(
  {
    source,
    coveredLines,
  }: {
    readonly source: string;
    readonly coveredLines: ReadonlySet<number>;
  },
): FileCoverage {
  /**
   One-based numbers of code lines that stayed uncovered, pushed in order.
   */
  const uncovered: number[] = [];
  /**
   Fold of covered and total code-line counts over the source lines.
   */
  const tally = source.split('\n',)
    .reduce(
    function fold(
      acc: {
        readonly covered: number;
        readonly total: number;
      },
      line,
      index,
    ) {
      if (!lineHasCode({ line, },)) return acc;
      /**
       One-based line number for this code line.
       */
      const lineNumber = index + 1;
      if (coveredLines.has(lineNumber,)) {
        return {
          covered: acc.covered + 1,
          total: acc.total + 1,
        };
      }
      uncovered.push(lineNumber,);
      return {
        covered: acc.covered,
        total: acc.total + 1,
      };
    },
    {
      covered: 0,
      total: 0,
    },
    );
  return {
    covered: tally.covered,
    total: tally.total,
    uncovered,
  };
}

//endregion Bitmap painting and line projection
