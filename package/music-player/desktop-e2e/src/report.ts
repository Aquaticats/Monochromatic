/**
 The terminal output of a run: result lines, the per-check table and the summary file.
 
 Result lines are the exact output of the command, so they go to the standard streams directly; diagnostics go through
 the tagged logger instead.
 
 @example
 ```ts
 import { printLine, } from './report.ts';
 
 printLine('PASS start/page launch shows the player page',);
 ```
 
 @packageDocumentation
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

/**
 The recorded result of one check.
 */
export type Outcome = {
  readonly detail: string;
  readonly id: string;
  readonly ms: number;
  readonly ok: boolean;
  readonly skipped: boolean;
  readonly title: string;
};

/**
 Counts of the three kinds of outcome.
 */
export type Totals = {
  readonly failed: number;
  readonly passed: number;
  readonly skipped: number;
};

/**
 Width the check id is padded to in the per-check table.
 */
const ID_COLUMN_WIDTH = 34;

/**
 Name of the summary file inside the output directory.
 */
const SUMMARY_FILE = 'e2e-summary.json';

/**
 Spaces per level in the summary file.
 */
const SUMMARY_INDENT = 2;

/**
 Milliseconds per second.
 */
const MS_PER_SECOND = 1_000;

/**
 Writes one line to the standard output.
 
 @param line - text without the final newline
 
 @example
 ```ts
 printLine('74 passed',);
 ```
 */
export function printLine(line: string,): void {
  process.stdout
    .write(`${line}\n`,);
}

/**
 Writes one line to the standard error.
 
 @param line - text without the final newline
 
 @example
 ```ts
 printErrorLine('FAILED focus/ring-visible: the ring is off',);
 ```
 */
export function printErrorLine(line: string,): void {
  process.stderr
    .write(`${line}\n`,);
}

/**
 Counts the outcomes of each kind.
 
 @param outcomes - recorded results
 
 @returns passed, failed and skipped counts; a skipped check is neither passed nor failed
 
 @example
 ```ts
 totalsOf([],); // { passed: 0, failed: 0, skipped: 0 }
 ```
 */
export function totalsOf(outcomes: readonly Outcome[],): Totals {
  return {
    passed: outcomes.filter(function isPassed(outcome,): boolean {
      return outcome.ok && (!outcome.skipped);
    },)
      .length,
    failed: outcomes.filter(function isFailed(outcome,): boolean {
      return !outcome.ok;
    },)
      .length,
    skipped: outcomes.filter(function isSkipped(outcome,): boolean {
      return outcome.skipped;
    },)
      .length,
  };
}

/**
 Formats the per-check table: one line per outcome, in the order they ran.
 
 @param outcomes - recorded results
 
 @returns the table lines, status first, then the padded id, then the title
 
 @example
 ```ts
 tableLines([{ id: 'a/b', title: 'x', ok: true, skipped: false, detail: '', ms: 1, },],); // ['PASS  a/b ... x']
 ```
 */
export function tableLines(outcomes: readonly Outcome[],): readonly string[] {
  return outcomes.map(function lineOf(outcome,): string {
    /**
     The status word of the outcome.
     */
    const status = outcome.skipped ? 'SKIP' : outcome.ok ? 'PASS' : 'FAIL';
    return `${status}  ${outcome.id
      .padEnd(ID_COLUMN_WIDTH,)} ${outcome.title}`;
  },);
}

/**
 Formats the closing summary line.
 
 @param totals - counts of each kind
 
 @param seconds - how long the run took
 
 @returns for example `74 passed, 0 failed, 5 not injectable; 57.3 s`
 
 @example
 ```ts
 summaryLine({ totals: { passed: 1, failed: 0, skipped: 0, }, seconds: 2, },);
 ```
 */
export function summaryLine(
  {
    totals,
    seconds,
  }: {
    readonly seconds: number;
    readonly totals: Totals;
  },
): string {
  return `${totals.passed} passed, ${totals.failed} failed, ${totals.skipped} not injectable; ${seconds.toFixed(1,)} s`;
}

/**
 Prints the table, the summary line and the failures, and writes the summary file.
 
 @param outcomes - recorded results
 
 @param startedAt - start of the run, in milliseconds since the epoch
 
 @param finishedAt - end of the run, in milliseconds since the epoch
 
 @param outputRoot - directory the summary file goes into
 
 @returns the totals, so the caller can decide the exit status
 
 @example
 ```ts
 const totals = await writeReport({ outcomes, startedAt, finishedAt, outputRoot, },);
 ```
 */
export async function writeReport(
  {
    outcomes,
    startedAt,
    finishedAt,
    outputRoot,
  }: {
    readonly finishedAt: number;
    readonly outcomes: readonly Outcome[];
    readonly outputRoot: string;
    readonly startedAt: number;
  },
): Promise<Totals> {
  /**
   Counts of each kind.
   */
  const totals = totalsOf(outcomes,);

  /**
   Run time in seconds.
   */
  const seconds = (finishedAt - startedAt) / MS_PER_SECOND;
  printLine('\n==== per-shortcut results ====',);
  for (const line of tableLines(outcomes,))
    printLine(line,);
  printLine(`\n${summaryLine({
    totals,
    seconds,
  },)}`,);
  await mkdir(
    outputRoot,
    { recursive: true, },
  );
  await writeFile(
    join(
      outputRoot,
      SUMMARY_FILE,
    ),
    JSON.stringify(
      {
        startedAt,
        seconds,
        ...totals,
        outcomes,
      },
      undefined,
      SUMMARY_INDENT,
    ),
  );
  return totals;
}
