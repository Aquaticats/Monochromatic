/**
 Tests for the slice cost report's command at its boundary.

 The built command is run as a child process whose environment carries no
 provider key, over a cost log the cases write into a scratch directory.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { reportSliceCost, } from '../../dist/final/node/index.mjs';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { SLICE_CACHE_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import {
  COST_LOG,
  NOTHING_NOTE,
  READ_NOTE,
  SPREAD_NOTE,
} from './slice-cost-report.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 What the command prints for `COST_LOG`, given the path it was named by.

 @param path - log path as named

 @returns Whole stdout

 @example
 ```ts
 const text = reportFor({ path, },);
 ```
 */
function reportFor({ path, }: { readonly path: string; },): string {
  return [
    path,
    '5 cost lines, 1 dropped',
    '',
    'WHAT A SLICE COSTS, BY THE SIZE OF ITS ORIGINAL',
    '  under     50 chars  slices    2  min/slice   0.75  ms/char  1125.0',
    '  under    200 chars  slices    1  min/slice   2.00  ms/char   800.0',
    '  under    any chars  slices    1  min/slice   6.00  ms/char   120.0',
    '',
    'SPREAD, WHICH THE BANDS AVERAGE AWAY',
    '  cheapest    40 chars    0.50 min  consolidation',
    '  dearest   3000 chars    6.00 min  translate',
    '  ratio     12.0x',
    SPREAD_NOTE,
    READ_NOTE,
    '',
    'BY LANE',
    '  repair        slices    1  total      1.0 min',
    '  translate     slices    2  total      8.0 min',
    '  consolidation slices    1  total      0.5 min',
    '',
  ].join('\n',);
}

await describe({
  name: 'slice-cost-report run',
  concurrency: 1,
  children: [
    describe({
      name: reportSliceCost.name,
      children: [
        it({
          name: 'PRINTS THE REPORT OF A COST LOG, the count of lines and of dropped ones first and the lanes last',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'slice-cost-report-run-', },);

            /**
             The log the command is named.
             */
            const path = join(
              scratch.path,
              'pass.log.jsonl',
            );
            await writeFile(
              path,
              COST_LOG,
              'utf8',
            );

            await reportSliceCost({ line: lineOf({ command: 'slice-cost-report', typed: [path,], },), },);

            expect(`${printed.lines.join('\n',)}\n`,).toBe(reportFor({ path, },),);
          },
        },),
        it({
          name: 'SAYS THERE IS NOTHING TO READ where every cost line was cut short, and counts the dropped one',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'slice-cost-report-run-', },);

            /**
             A log whose one cost line was cut short.
             */
            const path = join(
              scratch.path,
              'pass.log.jsonl',
            );
            await writeFile(
              path,
              'SLICE-COST lane=repair chunk=5\n',
              'utf8',
            );

            await reportSliceCost({ line: lineOf({ command: 'slice-cost-report', typed: [path,], },), },);

            expect(printed.lines,).toEqual([`${path}\n0 cost lines, 1 dropped\n`, NOTHING_NOTE,],);
          },
        },),
        it({
          name: 'PRINTS ONE COST LINE AS "1 cost line"',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'slice-cost-report-run-', },);

            /**
             A log of one resumed slice, which prices nothing.
             */
            const path = join(
              scratch.path,
              'pass.log.jsonl',
            );
            await writeFile(
              path,
              'SLICE-COST lane=repair chunk=4 sourceChars=600 ms=60000 exit=resumed\n',
              'utf8',
            );

            await reportSliceCost({ line: lineOf({ command: 'slice-cost-report', typed: [path,], },), },);

            expect(printed.lines[0],).toBe(`${path}\n1 cost line, 0 dropped\n`,);
          },
        },),
        it({
          name: 'REFUSES A LOG NAMED AS AN EMPTY ARGUMENT as stated, with the usage line',
          fn: async () => {
            /**
             What the report rejected with.
             */
            const refusal = await rejectionOf({
              promise: reportSliceCost({ line: lineOf({ command: 'slice-cost-report', typed: [''], },), },),
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: name a run folder, not an empty argument: slice-cost-report <run folder or log file>',
            );
          },
        },),
        it({
          name: 'REFUSES A LOG THAT IS NOT THERE as stated, naming the path, rather than failing as a fault',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'slice-cost-report-run-', },);

            /**
             A path nothing is at.
             */
            const path = join(
              scratch.path,
              'no-such-pass.log.jsonl',
            );

            /**
             What the report rejected with.
             */
            const refusal = await rejectionOf({
              promise: reportSliceCost({ line: lineOf({ command: 'slice-cost-report', typed: [path,], },), },),
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              `StatedRefusalError: ${path} is not there; name a run folder or log file that exists: `
                + 'slice-cost-report <run folder or log file>',
            );
          },
        },),
        it({
          name: 'LETS A READ FAILURE THAT IS NOT A MISSING PATH ESCAPE as the fault it is: a directory named for the log',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'slice-cost-report-run-', },);

            /**
             A directory carrying the name of a record file.
             */
            const path = join(
              scratch.path,
              'nap.log.jsonl',
            );
            await mkdir(path,);

            /**
             What the report rejected with.
             */
            const failure = await rejectionOf({
              promise: reportSliceCost({ line: lineOf({ command: 'slice-cost-report', typed: [path,], },), },),
            },);

            expect(failure instanceof StatedRefusalError,).toBe(false,);
            // The reader streams the file, and a stream's failure names the code and the call and no path.
            expect(String(failure,),).toBe('Error: EISDIR: illegal operation on a directory, read',);
          },
        },),
      ],
    },),

    describe({
      name: 'slice-cost-report as built',
      children: [
        it({
          name: 'REPORTS A COST LOG and exits 0, its bands, its spread and its lanes, leaving the resumed slice '
            + 'and the cut line out of the pricing',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'slice-cost-report-built-', },);
            await using runs = await scratchDir({ prefix: 'slice-cost-report-built-runs-', },);

            /**
             The log the command is named.
             */
            const path = join(
              scratch.path,
              'pass.log.jsonl',
            );
            await writeFile(
              path,
              COST_LOG,
              'utf8',
            );

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'slice-cost-report',
              packageEntry: SLICE_CACHE_ENTRY,
              args: [path,],
              env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
            },);

            expect(run.stderr,).toBe('',);
            expect(run.stdout,).toBe(reportFor({ path, },),);
            expect(run.code,).toBe(0,);
          },
        },),
        it({
          name: 'SAYS THERE IS NOTHING TO READ YET for a log with no cost line, and exits 0',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'slice-cost-report-built-', },);
            await using runs = await scratchDir({ prefix: 'slice-cost-report-built-runs-', },);

            /**
             A log of something else.
             */
            const path = join(
              scratch.path,
              'pass.log.jsonl',
            );
            await writeFile(
              path,
              'a line about something else\n',
              'utf8',
            );

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'slice-cost-report',
              packageEntry: SLICE_CACHE_ENTRY,
              args: [path,],
              env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
            },);

            expect(run.stderr,).toBe('',);
            expect(run.stdout,).toBe(`${path}\n0 cost lines, 0 dropped\n\n${NOTHING_NOTE}\n`,);
            expect(run.code,).toBe(0,);
          },
        },),
        it({
          name: 'REFUSES A LOG THAT IS NOT THERE as stated and exits 6 with its line, which names the path',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'slice-cost-report-built-', },);
            await using runs = await scratchDir({ prefix: 'slice-cost-report-built-runs-', },);

            /**
             A path nothing is at.
             */
            const path = join(
              scratch.path,
              'no-such-pass.log.jsonl',
            );

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'slice-cost-report',
              packageEntry: SLICE_CACHE_ENTRY,
              args: [path,],
              env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
            },);

            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              `slice-cost-report: ${path} is not there; name a run folder or log file that exists: `
                + 'slice-cost-report <run folder or log file>\n',
            );
            expect(run.code,).toBe(REFUSED_AS_STATED,);
          },
        },),
        it({
          name: 'REFUSES A COMMAND LINE NAMING NO LOG as stated and exits 6 with its line',
          fn: async () => {
            await using runs = await scratchDir({ prefix: 'slice-cost-report-built-runs-', },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'slice-cost-report',
              packageEntry: SLICE_CACHE_ENTRY,
              args: [],
              env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
            },);

            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'slice-cost-report: this command needs <run folder or log file>. '
                + 'Usage: slice-cost-report <run folder or log file>\n',
            );
            expect(run.code,).toBe(REFUSED_AS_STATED,);
          },
        },),
        it({
          name: 'REFUSES A FLAG IT DOES NOT DECLARE as stated and exits 6 with its line',
          fn: async () => {
            await using runs = await scratchDir({ prefix: 'slice-cost-report-built-runs-', },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'slice-cost-report',
              packageEntry: SLICE_CACHE_ENTRY,
              args: ['--tabby', 'nap.log'],
              env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
            },);

            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'slice-cost-report: --tabby is not a flag this command reads. '
                + 'Usage: slice-cost-report <run folder or log file>\n',
            );
            expect(run.code,).toBe(REFUSED_AS_STATED,);
          },
        },),
      ],
    },),
  ],
},);
