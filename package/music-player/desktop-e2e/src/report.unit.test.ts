/**
 Tests for the report: counts, the per-check table, the summary line and the summary file.
 
 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type Outcome,
  summaryLine,
  tableLines,
  totalsOf,
  writeReport,
} from './report.ts';

/**
 One passed, one failed and one skipped outcome.
 */
const OUTCOMES: readonly Outcome[] = [
  { id: 'a/pass', title: 'passes', ok: true, skipped: false, detail: 'fine', ms: 3, },
  { id: 'b/fail', title: 'fails', ok: false, skipped: false, detail: 'broken', ms: 4, },
  { id: 'c/skip', title: 'is skipped', ok: true, skipped: true, detail: 'cannot run here', ms: 0, },
];

await describe({
  name: totalsOf.name,
  children: [
    it({
      name: 'counts passed, failed and skipped; a skipped check is neither passed nor failed',
      fn: async ({ expect, }) => {
        expect(totalsOf(OUTCOMES,),).toEqual({ passed: 1, failed: 1, skipped: 1, },);
        expect(totalsOf([],),).toEqual({ passed: 0, failed: 0, skipped: 0, },);
      },
    },),
  ],
},);

await describe({
  name: tableLines.name,
  children: [
    it({
      name: 'writes the status word, the padded id and the title',
      fn: async ({ expect, }) => {
        expect(tableLines(OUTCOMES,),).toEqual([
          `PASS  ${'a/pass'.padEnd(34,)} passes`,
          `FAIL  ${'b/fail'.padEnd(34,)} fails`,
          `SKIP  ${'c/skip'.padEnd(34,)} is skipped`,
        ],);
      },
    },),
  ],
},);

await describe({
  name: summaryLine.name,
  children: [
    it({
      name: 'writes the counts and the run time with one decimal',
      fn: async ({ expect, }) => {
        expect(summaryLine({ totals: { passed: 74, failed: 0, skipped: 5, }, seconds: 57.34, },),).toBe(
          '74 passed, 0 failed, 5 not injectable; 57.3 s',
        );
      },
    },),
  ],
},);

await describe({
  name: writeReport.name,
  children: [
    it({
      name: 'writes the summary file with the counts and the outcomes, and returns the totals',
      fn: async ({ expect, }) => {
        const out = await mkdtemp(join(scratchRoot(), 'e2e-report-',),);
        const totals = await writeReport({ outcomes: OUTCOMES, startedAt: 1_000, finishedAt: 3_500, outputRoot: join(out, 'nested',), },);
        expect(totals,).toEqual({ passed: 1, failed: 1, skipped: 1, },);
        const summary: unknown = JSON.parse(await readFile(join(out, 'nested', 'e2e-summary.json',), 'utf8',),);
        expect(summary,).toEqual({ startedAt: 1_000, seconds: 2.5, passed: 1, failed: 1, skipped: 1, outcomes: OUTCOMES, },);
        await rm(out, { recursive: true, force: true, },);
      },
    },),
  ],
},);
