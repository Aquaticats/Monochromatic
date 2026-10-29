/**
 Deterministic coverage-baseline gate. Reads a `NODE_V8_COVERAGE` directory
 produced by `coverage-driver.ts`, projects it to a covered-function count per
 package source file, then either freezes the baseline (`write`) or fails on any
 per-file regression (`check`). Counts, not percentages, so a file whose
 reachable functions shrink fails even if another grows.
 
 ```sh
 node coverage-report.ts <check|write> <coverageDir> <baselinePath>
 ```
 
 @module
 */

import {
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  sep,
} from 'node:path';
import { fileURLToPath, } from 'node:url';

//region V8 shapes

/**
 One coverage range; only its execution `count` matters here.
 */
type V8Range = {
  readonly count: number;
};

/**
 One function's coverage; `ranges[0]` spans the whole function.
 */
type V8Function = {
  readonly ranges: readonly V8Range[];
};

/**
 Coverage for one script (source file).
 */
type V8Script = {
  readonly url: string;
  readonly functions: readonly V8Function[];
};

/**
 One `NODE_V8_COVERAGE` JSON file.
 */
type V8Coverage = {
  readonly result: readonly V8Script[];
};

/**
 Covered-function count per package-relative path.
 */
type CoverageCounts = Readonly<Record<string, number>>;

//endregion V8 shapes

//region Projection

/**
 Package source marker; only files beneath it are gated.
 */
const SOURCE_MARKER = `${sep}package${sep}module${sep}jsonc-edit${sep}src${sep}`;

/**
 Tests whether a package-relative path is a gate target: a non-test source file
 outside the bench, fuzz, and conformance directories. The empty string (a
 script outside the package) is never a target.
 
 @param relativePath - Path beneath the package `src` directory.
 
 @returns `true` when the file should be gated.
 
 @example
 ```ts
 isTarget('parse.ts'); // => true
 ```
 */
function isTarget(relativePath: string,): boolean {
  return (relativePath !== '')
    && relativePath.endsWith('.ts',)
    && (!relativePath.includes('.test.',))
    && (!relativePath.includes('.bench.',))
    && (!relativePath.startsWith(`bench${sep}`,))
    && (!relativePath.startsWith(`fuzz${sep}`,))
    && (!relativePath.startsWith(`conformance${sep}`,));
}

/**
 Maps a `file://` script URL to its package-relative path, or the empty string
 for URLs outside this package (node internals, dependencies).
 
 @param url - Script URL from V8 coverage.
 
 @returns Package-relative path, or `''` when outside the package.
 
 @example
 ```ts
 packageRelative('file:///repo/packages/module/jsonc-edit/src/parse.ts');
 // => 'parse.ts'
 ```
 */
function packageRelative(url: string,): string {
  if (!url.startsWith('file://',))
    return '';
  /**
   Absolute filesystem path of the script.
   */
  const path = fileURLToPath(url,);
  /**
   Offset of the source marker within the path.
   */
  const markerIndex = path.indexOf(SOURCE_MARKER,);
  if (markerIndex === (-1))
    return '';
  return path.slice(markerIndex + SOURCE_MARKER.length,);
}

/**
 Counts functions in a script whose whole-function range executed.
 
 @param script - One script's coverage.
 
 @returns Number of executed functions.
 
 @example
 ```ts
 coveredFunctions({ url: 'x', functions: [{ ranges: [{ count: 1 }] }] }); // => 1
 ```
 */
function coveredFunctions(script: V8Script,): number {
  return script.functions
    .filter(function executed(fn,): boolean {
      return (fn.ranges[0]
        ?.count
        ?? 0) > 0;
    },)
    .length;
}

//endregion Projection

//region Aggregation

/**
 Reads and parses one `NODE_V8_COVERAGE` JSON file.
 
 @param path - Absolute path to a coverage JSON file.
 
 @returns Parsed coverage.
 
 @example
 ```ts
 await readCoverage('/tmp/cov/coverage-1.json');
 ```
 */
async function readCoverage(path: string,): Promise<V8Coverage> {
  /**
   Raw file contents.
   */
  const text = await readFile(
    path,
    'utf8',
  );
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- NODE_V8_COVERAGE writes V8 ScriptCoverage JSON
  return JSON.parse(text,) as V8Coverage;
}

/**
 Aggregates covered-function counts per gate-target file across every coverage
 JSON in a directory, keeping the maximum seen for each file.
 
 @param coverageDir - Directory written by `NODE_V8_COVERAGE`.
 
 @returns Covered-function count per package-relative path.
 
 @example
 ```ts
 await aggregate('/tmp/cov');
 ```
 */
async function aggregate(coverageDir: string,): Promise<CoverageCounts> {
  /**
   Coverage JSON file names in the directory.
   */
  const entries = await readdir(coverageDir,);
  /**
   Parsed coverage for every process, read concurrently.
   */
  const coverages = await Promise.all(
    entries
      .filter(function isJson(name: string,): boolean {
        return name.endsWith('.json',);
      },)
      .map(function readEntry(name: string,): Promise<V8Coverage> {
        return readCoverage(join(
          coverageDir,
          name,
        ),);
      },),
  );
  /**
   Accumulated maximum covered-function count per file.
   */
  const counts = new Map<string, number>();
  for (const coverage of coverages)
    for (const script of coverage.result) {
      /**
       Gate-target path for this script, if any.
       */
      const relativePath = packageRelative(script.url,);
      if (isTarget(relativePath,))
        counts.set(
          relativePath,
          Math.max(
            counts.get(relativePath,) ?? 0,
            coveredFunctions(script,),
          ),
        );
    }
  return Object.fromEntries(
    [...counts.entries(),].toSorted(function byKey(
      left,
      right,
    ): number {
      return left[0]
        .localeCompare(right[0],);
    },),
  );
}

//endregion Aggregation

//region Gate

/**
 Throws when any baseline file's covered-function count regressed.
 
 @param baseline - Committed baseline counts.
 
 @param current - Counts from this run.
 
 @throws Error listing every regressed file.
 
 @example
 ```ts
 checkBaseline({ baseline: { 'a.ts': 1 }, current: { 'a.ts': 1 } });
 ```
 */
function checkBaseline({
  baseline,
  current,
}: {
  readonly baseline: ReadonlyMap<string, number>;
  readonly current: ReadonlyMap<string, number>;
},): void {
  /**
   Human-readable regression lines.
   */
  const regressions = [...baseline.entries(),]
    .flatMap(function compare([file, expected],): readonly string[] {
    /**
     Covered functions for this file in the current run.
     */
      const got = current.get(file,) ?? 0;
      return (got < expected)
        ? [`${file}: ${String(got,)} < baseline ${String(expected,)}`,]
        : [];
    },);
  if (regressions.length > 0)
    throw new Error(`coverage regressed:\n  ${regressions.join('\n  ',)}`,);
  console.log(`coverage gate passed: ${String(baseline.size,)} files at or above baseline`,);
}

//endregion Gate

//region Entry

/**
 Run mode, coverage directory, and baseline path from the command line.
 */
const [mode, coverageDir, baselinePath] = process.argv
  .slice(2,);
if ((mode === undefined) || (coverageDir === undefined)
  || (baselinePath === undefined))
  throw new Error('usage: node coverage-report.ts <check|write> <coverageDir> <baselinePath>',);

/**
 Covered-function counts from this run.
 */
const current = await aggregate(coverageDir,);

/**
 Number of package source files the projection matched. A stale
 SOURCE_MARKER makes every projection miss; catching it here keeps write
 mode from freezing an empty baseline (the repo-wide packages/ to package/
 rename broke this gate exactly this way).
 */
const matchedFiles = Object.keys(current,)
  .length;
if (matchedFiles === 0)
  throw new Error('coverage projection matched no package source files; SOURCE_MARKER is likely stale',);

if (mode === 'write') {
  await writeFile(
    baselinePath,
    `${JSON.stringify(
      current,
      null,
      2,
    )}\n`,
  );
  console.log(`coverage baseline written: ${String(Object.keys(current,)
    .length,)} files`,);
}
else {
  /**
   Raw committed baseline contents.
   */
  const baselineText = await readFile(
    baselinePath,
    'utf8',
  );
  /* oxlint-disable typescript/no-unsafe-type-assertion -- the committed baseline is a path-to-count record */
  /**
   Committed baseline counts.
   */
  const baseline = JSON.parse(baselineText,) as CoverageCounts;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  checkBaseline({
    baseline: new Map(Object.entries(baseline,),),
    current: new Map(Object.entries(current,),),
  },);
}

//endregion Entry
