/**
 Source-mapped coverage report for a deepmerge-ts build other than the npm
 release, such as `dist/fork-build/index.mjs` from the `fork:build` task.

 Projects V8 coverage of the built file onto its lines (`./coverage-v8.ts`),
 then maps each generated line through the build's source map to an original
 `src/*.ts` line. A source line counts when any generated line maps to it,
 and as covered when any covered generated line does. The report lists each
 source file's covered count and its uncovered lines as ranges, so a fix
 branch can see exactly which TypeScript the properties reach. Report only:
 the committed baseline gates the npm release, not local builds.

 Run by the `fuzz:coverage` task when `DEEPMERGE_FUZZ_TARGET` is set:

 ```sh
 node src/coverage-source.ts <coverageDir> <targetPath>
 ```

 @module
 */

import {
  readFile,
  realpath,
} from 'node:fs/promises';
import {
  SourceMap,
  type SourceMapPayload,
} from 'node:module';
import { pathToFileURL, } from 'node:url';

import { targetScripts, } from './coverage-report.ts';
import { coveredLines, } from './coverage-v8.ts';

/**
 Mapped and covered original lines of one source file, 1-based.
 */
export type SourceBucket = {
  readonly mapped: ReadonlySet<number>;
  readonly covered: ReadonlySet<number>;
};

/**
 Column value meaning a line holds no non-blank character.
 */
const BLANK_LINE = -1;

/**
 First non-blank column of a line, found by index scan.

 @param text - One generated line.

 @returns Column of the first non-whitespace character, or {@link BLANK_LINE}.

 @example
 ```ts
 firstNonBlankColumn('  x'); // 2
 ```
 */
export function firstNonBlankColumn(text: string,): number {
  for (let column = 0; column < text.length; column += 1) {
    if (text.charAt(column,)
      .trim()
      !== '')
      return column;
  }
  return BLANK_LINE;
}

/**
 Map generated lines to original lines and tally them per source.

 @param source - Built file text.

 @param map - Its source map.

 @param covered - Covered generated lines (0-based).

 @returns Buckets keyed by the map's source paths, lines 1-based.

 @example
 ```ts
 const tally = tallySources({ source, map, covered, });
 ```
 */
export function tallySources(
  {
    source,
    map,
    covered,
  }: {
    readonly source: string;
    readonly map: SourceMap;
    readonly covered: ReadonlySet<number>;
  },
): ReadonlyMap<string, SourceBucket> {
  /**
   Accumulators per source file, returned read-only.
   */
  const tally = new Map<string, {
    readonly mapped: Set<number>;
    readonly covered: Set<number>
  }>();
  for (const [line, text,] of source.split('\n',)
    .entries()) {
    /**
     First non-blank column, the position mapped for the whole line.
     */
    const column = firstNonBlankColumn(text,);
    if (column === BLANK_LINE)
      continue;
    /**
     Original position of this generated line, when the map has one.
     */
    const entry = map.findEntry(
      line,
      column,
    );
    if ('originalSource' in entry) {
      /**
       Accumulator for this line's source file.
       */
      const bucket = tally.get(entry.originalSource,) ?? {
        covered: new Set<number>(),
        mapped: new Set<number>(),
      };
      bucket.mapped
        .add(entry.originalLine + 1,);
      if (covered.has(line,))
        bucket.covered
          .add(entry.originalLine + 1,);
      tally.set(
        entry.originalSource,
        bucket,
      );
    }
  }
  return tally;
}

/**
 Collapse line numbers into `a-b` ranges for a compact report.

 @param lines - Line numbers, any order.

 @returns Comma-separated ranges in ascending order.

 @example
 ```ts
 ranges([7, 1, 2, 3,]); // '1-3, 7'
 ```
 */
export function ranges(lines: readonly number[],): string {
  /**
   Ascending copy of the input.
   */
  const sorted = lines.toSorted(function ascending(
    left,
    right,
  ) {
    return left - right;
  },);
  /**
   First line of each run: its predecessor is not one less.
   */
  const starts = sorted.filter(function startsRun(
    line,
    index,
  ) {
    return sorted[index - 1] !== (line
      - 1);
  },);
  /**
   Last line of each run: its successor is not one more.
   */
  const ends = sorted.filter(function endsRun(
    line,
    index,
  ) {
    return sorted[index + 1] !== (line
      + 1);
  },);
  return starts
    .map(function describeRun(
      start,
      index,
    ) {
      /**
       End of this run; runs pair up by index.
       */
      const end = ends[index] ?? start;
      return start === end ? String(start,) : `${String(start,)}-${String(end,)}`;
    },)
    .join(', ',);
}

/**
 Whether parsed JSON has the source map fields `SourceMap` reads.

 @param value - Parsed map file.

 @returns Whether `value` can be handed to `SourceMap`.

 @example
 ```ts
 isSourceMapPayload({ version: 3, sources: [], names: [], mappings: '', }); // true
 ```
 */
function isSourceMapPayload(value: unknown,): value is SourceMapPayload {
  return ((typeof value) === 'object')
    && (value !== null)
    && ((typeof Reflect.get(
      value,
      'mappings',
    )) === 'string')
    && Array.isArray(Reflect.get(
      value,
      'sources',
    ),);
}

/**
 Print the source-mapped report for one built target.

 @param coverageDir - `NODE_V8_COVERAGE` output directory.

 @param targetPath - Built ESM file with a sibling `.map`.

 @throws When the target was never loaded by the tests or its map is invalid.

 @example
 ```ts
 await runSourceReport({ coverageDir, targetPath, });
 ```
 */
export async function runSourceReport(
  {
    coverageDir,
    targetPath,
  }: {
    readonly coverageDir: string;
    readonly targetPath: string;
  },
): Promise<void> {
  /**
   Real path, as the ESM loader records it.
   */
  const path = await realpath(targetPath,);
  /**
   Coverage entries of the target across processes.
   */
  const scripts = await targetScripts({
    coverageDir,
    targetUrl: pathToFileURL(path,)
      .href,
  },);
  if (scripts.length === 0)
    throw new Error(`No coverage recorded for ${path}; did DEEPMERGE_FUZZ_TARGET point at it?`,);
  /**
   Built file text.
   */
  const source = await readFile(
    path,
    'utf8',
  );
  /**
   Covered generated lines, unioned across processes.
   */
  const covered = new Set(scripts.flatMap(function scriptLines(script,) {
    return [...coveredLines({
      script,
      source,
    },),];
  },),);
  /**
   Parsed map beside the target.
   */
  const payload: unknown = JSON.parse(await readFile(
    `${path}.map`,
    'utf8',
  ),);
  if (!isSourceMapPayload(payload,))
    throw new Error(`${path}.map is not a source map`,);
  /**
   Per-source tally, reported in path order.
   */
  const tally = tallySources({
    covered,
    map: new SourceMap(payload,),
    source,
  },);
  for (const file of [...tally.keys(),].toSorted()) {
    /**
     Bucket of this source file.
     */
    const bucket = tally.get(file,);
    if (bucket === undefined)
      throw new Error(`tally lost ${file}`,);
    /**
     Mapped lines no covered generated line reaches.
     */
    const uncovered = [...bucket.mapped,].filter(function isUncovered(line,) {
      return !bucket.covered
        .has(line,);
    },);
    console.log(`${file}: ${String(bucket.covered
      .size,)}/${String(bucket.mapped
        .size,)} mapped lines`,);
    if (uncovered.length > 0)
      console.log(`  uncovered: ${ranges(uncovered,)}`,);
  }
}

if (import.meta.main) {
  /**
   Positional script arguments.
   */
  const [
    coverageDir,
    targetPath,
  ] = process.argv
    .slice(2,);
  if ((coverageDir === undefined) || (targetPath === undefined))
    throw new Error('usage: node src/coverage-source.ts <coverageDir> <targetPath>',);
  await runSourceReport({
    coverageDir,
    targetPath,
  },);
}
