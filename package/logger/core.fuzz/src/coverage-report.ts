/**
 Coverage baseline gate and human summary for the module-logger fuzz campaign.
 
 Reads a `NODE_V8_COVERAGE` directory produced by `coverage-driver.ts`, projects
 it to per-file covered-line counts through `coverage-v8.ts`, then either freezes
 a baseline (`write`) or fails on any per-file regression against the committed
 baseline (`check`). The gate compares covered-line counts, not percentages, so
 a target file whose reachable lines shrink fails even if another file grows.
 
 Run as a script by the `fuzz:coverage` mise task:
 
 ```sh
 node coverage-report.ts <check|write> <coverageDir> <baselinePath>
 ```
 
 @module
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { dirname, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import { aggregateCoverage, } from './coverage-aggregate.ts';
import type { CoverageMap, } from './coverage-v8.ts';

//region Baseline shapes

/**
 Committed baseline: covered code-line count per package-relative target path.
 */
export type Baseline = Readonly<Record<string, number>>;

/**
 One per-file regression: covered lines fell below the baseline.
 */
export type Regression = {
  readonly file: string;
  readonly baseline: number;
  readonly current: number;
};

/**
 Whole numbers used for the human percentage; `100` is the percentage scale.
 */
const PERCENT_SCALE = 100;

/**
 Right-aligned column width for the percentage in the summary table.
 */
const PERCENT_WIDTH = 3;

/**
 Column width for the covered and total line counts in the summary table.
 */
const COUNT_WIDTH = 4;

/**
 Whether parsed JSON has the baseline shape (an object), narrowing from
 `unknown` without an `as` assertion.
 
 @param value - Parsed JSON to test before reading as a baseline.
 
 @returns Whether `value` can be read as a baseline.
 */
function isBaseline(value: unknown,): value is Baseline {
  return ((typeof value) === 'object') && (value !== null);
}

//endregion Baseline shapes

//region Baseline derivation and IO

/**
 Reduce a coverage map to its committed baseline form (covered counts only),
 with keys sorted so the on-disk baseline diffs cleanly.
 
 @returns Baseline keyed by sorted package-relative path.
 
 */
function toBaseline({ map, }: { readonly map: Readonly<Record<string, CoverageMap[string]>>; },): Baseline {
  return Object.fromEntries(
    Object.keys(map,)
      .toSorted()
      .map(function entry(file,) {
        return [
          file,
          (map[file] ?? {
            covered: 0,
            total: 0,
            uncovered: [],
          }).covered,
        ] as const;
      },),
  );
}

/**
 Serialize a baseline to stable JSON text (sorted keys, two-space indent,
 trailing newline) so a refreeze produces a minimal diff.
 
 @returns JSON text for the baseline file.
 
 @mutates baseline - `JSON.stringify` can invoke getters, proxy traps, `toJSON`, replacer, and coercion hooks.
 */
function serializeBaseline({ baseline, }: { baseline: Record<string, number>; },): string {
  return `${JSON.stringify(
    baseline,
    undefined,
    2,
  )}\n`;
}

/**
 Load a committed baseline from disk.
 
 @returns Parsed baseline; an empty baseline when the file is absent.
 */
async function loadBaseline({ baselinePath, }: { readonly baselinePath: string; },): Promise<Baseline> {
  try {
    /**
     Parsed baseline JSON, narrowed by assertion from `unknown`.
     */
    const parsed: unknown = JSON.parse(await readFile(
      baselinePath,
      'utf8',
    ),);
    if (!isBaseline(parsed,)) throw new Error('Malformed baseline JSON: expected an object',);
    return parsed;
  }
  catch (error: unknown) {
    if (Error.isError(error,)
      && ('code' in error)
      && (error.code === 'ENOENT'))
      return {};
    throw error;
  }
}

//endregion Baseline derivation and IO

//region Comparison

/**
 Find every target file whose covered-line count regressed below the baseline,
 including a baseline file the current run no longer covers at all.
 
 @returns Regressions in sorted path order; empty when nothing regressed.
 
 */
function findRegressions(
  {
    map,
    baseline,
  }: {
    readonly map: CoverageMap;
    baseline: Record<string, number>;
  },
): readonly Regression[] {
  return Object.keys(baseline,)
    .toSorted()
    .flatMap(function check(file,) {
      /**
       Baseline covered count for this file.
       */
      const baselineCovered = baseline[file] ?? 0;
      /**
       Current covered count, or zero when the run no longer reaches the file.
       */
      const currentCovered = (map[file] ?? {
        covered: 0,
        total: 0,
        uncovered: [],
      }).covered;
      if (currentCovered >= baselineCovered) return [];
      return [
        {
          file,
          baseline: baselineCovered,
          current: currentCovered,
        },
      ];
    },);
}

//endregion Comparison

//region Summary

/**
 Format one target file's coverage line for the human summary.
 
 @returns A single aligned summary line.
 */
function formatFileLine(
  {
    file,
    coverage,
    baseline,
  }: {
    readonly file: string;
    readonly coverage: CoverageMap[string];
    readonly baseline: Baseline;
  },
): string {
  /**
   Integer percentage; a file with no code lines reads as fully covered.
   */
  const percent = coverage.total === 0
    ? PERCENT_SCALE
    : Math.round((coverage.covered / coverage.total) * PERCENT_SCALE,);
  /**
   Baseline covered count for the delta annotation, if the file is tracked.
   */
  const baselineCovered = baseline[file];
  /**
   Signed delta against the baseline, or a new-file marker.
   */
  const delta = baselineCovered === undefined
    ? ' (new)'
    : ((coverage.covered === baselineCovered)
      ? ''
      : ` (${coverage.covered > baselineCovered ? '+' : ''}${String(coverage.covered - baselineCovered,)})`);
  return `  ${String(percent,)
    .padStart(PERCENT_WIDTH,)}%  ${String(coverage.covered,)
      .padStart(COUNT_WIDTH,)}/${String(coverage.total,)
        .padEnd(COUNT_WIDTH,)}  ${file}${delta}`;
}

/**
 Build the full human summary: a per-file table plus a totals line.
 
 @returns Multi-line summary text for CI logs and local runs.
 
 */
function formatSummary(
  {
    map,
    baseline,
  }: {
    map: Record<string, CoverageMap[string]>;
    readonly baseline: Baseline;
  },
): string {
  /**
   Per-file lines in sorted path order.
   */
  const fileLines = Object.keys(map,)
    .toSorted()
    .map(function line(file,) {
      return formatFileLine({
        file,
        coverage: map[file] ?? {
          covered: 0,
          total: 0,
          uncovered: [],
        },
        baseline,
      },);
    },);
  /**
   Summed covered and total code lines across all target files.
   */
  const totals = Object.values(map,)
    .reduce(
    function sum(
      acc: {
        readonly covered: number;
        readonly total: number;
      },
      coverage,
    ) {
      return {
        covered: acc.covered + coverage.covered,
        total: acc.total + coverage.total,
      };
    },
    {
      covered: 0,
      total: 0,
    },
    );
  /**
   Overall integer percentage across target files.
   */
  const overall = totals.total === 0
    ? PERCENT_SCALE
    : Math.round((totals.covered / totals.total) * PERCENT_SCALE,);
  return [
    'module-logger fuzz coverage (runtime src files, V8 line coverage):',
    ...fileLines,
    `  ----`,
    `  ${String(overall,)
      .padStart(PERCENT_WIDTH,)}%  ${String(totals.covered,)
        .padStart(COUNT_WIDTH,)}/${String(totals.total,)
          .padEnd(COUNT_WIDTH,)}  ${String(Object.keys(map,)
            .length,)} files`,
  ].join('\n',);
}

//endregion Summary

//region CLI

/**
 Run the coverage gate as a script.
 
 `write` freezes the baseline from the current coverage and prints the summary.
 `check` prints the summary and throws when any target file regressed, so the
 mise task exits non-zero.
 
 @throws Error when `mode` is unknown, or when `check` finds a regression.
 */
async function main(): Promise<void> {
  /**
   Positional arguments: mode, coverage directory, baseline path.
   */
  const [mode, coverageDir, baselinePath,] = process.argv
    .slice(2,);
  if ((mode === undefined) || (coverageDir === undefined)
    || (baselinePath === undefined)) {
    throw new Error('usage: coverage-report.ts <check|write> <coverageDir> <baselinePath>',);
  }
  /**
   Runtime package root, resolved through the `/ts` subpath of
   `@monochromatic-dev/logger-core` (its `src/index.ts`, two levels down)
   so the gate measures the runtime package's `src`, never this sidecar.
   */
  const packageRoot = dirname(
    dirname(
      fileURLToPath(import.meta.resolve('@monochromatic-dev/logger-core/ts',),),
    ),
  );
  /**
   Current per-file coverage from the driver run.
   */
  const map = await aggregateCoverage({
    coverageDir,
    packageRoot,
  },);
  if (Object.keys(map,)
    .length
    === 0) {
    throw new Error(`No target coverage found under ${coverageDir}; did the driver run with NODE_V8_COVERAGE set?`,);
  }
  if (mode === 'write') {
    await writeFile(
      baselinePath,
      serializeBaseline({ baseline: toBaseline({ map, }), },),
    );
    console.log(formatSummary({
      map,
      baseline: {},
    },),);
    console.log(`\nWrote coverage baseline to ${baselinePath}`,);
    return;
  }
  if (mode !== 'check') {
    throw new Error(`Unknown mode ${mode}; expected 'check' or 'write'`,);
  }
  /**
   Committed baseline to gate against.
   */
  const baseline = await loadBaseline({ baselinePath, },);
  console.log(formatSummary({
    map,
    baseline,
  },),);
  /**
   Per-file regressions against the baseline.
   */
  const regressions = findRegressions({
    map,
    baseline,
  },);
  if (regressions.length > 0) {
    throw new Error(
      `Coverage regressed in ${String(regressions.length,)} file(s):\n${
        regressions
          .map(function describe(regression,) {
            return `  ${regression.file}: ${String(regression.baseline,)} -> ${String(regression.current,)} covered lines`;
          },)
          .join('\n',)
      }\nReach the lost lines again, or, if the drop is intended (for example a node release shifted V8 coverage), refreeze the baseline with the fuzz:coverage --write task.`,
    );
  }
}

if (process.argv[1] === import.meta.filename) await main();

//endregion CLI
