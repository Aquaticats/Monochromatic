/**
 Tests that the reports totalling a run's records read them from the run folder a pass keeps them in, and say so in
 words when the folder holds none.

 A PASS WRITES ITS RECORDS THROUGH THE SHARED LOGGER'S FILE SINK into `<run folder>/log/<day>/*.log.jsonl`
 (`typography/src/run-log.ts`), and no report reads a `.log` capture of the console any more. Each case here builds its
 run folder through that logger (`run-log-write.test-fixture.ts`), so a record is as the sink writes it.

 EACH READER HAS ITS DIFFERENTIAL CASE: the same logical lines read as the console text they were first a record of
 and as the records of a run folder give the same readings, and the report over the folder is the report the lines
 gave. A reader that gave a different report would have changed what a run says of itself.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  linesOfLog,
  readLogTexts,
  readMeterLog,
  readRunTiming,
  reportMeters,
  reportRunTiming,
  reportSpendCost,
  tallySpend,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  writeRecordFile,
  writeRunFolder,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/run-log-write.test-fixture.ts';
import {
  captureCodedReport,
  captureReport,
  type ReportCapture,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/report-run-capture.test-fixture.ts';
import { lineOf, } from './command-line.test-fixture.ts';
import {
  ASKING_NOTE,
  EVERYTHING_WET,
  SYNTHETIC_DRY,
} from './meter-report.test-fixture.ts';
import {
  ROUND_STOOD_LINE,
  timedCallLine,
} from './run-timing-report.test-fixture.ts';
import {
  CUT_SPEND_LINE,
  HYPER_CHEAP,
  SUBSCRIPTION_CALL,
} from './spend-report.test-fixture.ts';

/**
 A completion line ending ten seconds in, after a call of ten seconds.
 */
const FIRST_CALL = timedCallLine({
  stamp: '2026-08-25T10:00:10.000Z',
  elapsedMs: 10_000,
},);

/**
 A completion line ending a quarter minute in, after a call of ten seconds.
 */
const SECOND_CALL = timedCallLine({
  stamp: '2026-08-25T10:00:15.000Z',
  elapsedMs: 10_000,
},);

/**
 UTC day the folders of these cases keep their one record file under.
 */
const DAY = '2026-08-25';

/**
 Runs a report that prints and leaves an exit code over the paths named.

 @param sinon - calling case's own sandbox (`ctx.sinon`)
 @param run - report to run

 @returns What it printed and the exit code it left

 @example
 ```ts
 const captured = await printed({ sinon: ctx.sinon, run: async () => reportSpendCost({ line, },), },);
 ```
 */
async function printed(
  {
    sinon,
    run,
  }: {
    readonly sinon: DisposableSandbox;
    readonly run: () => Promise<void>;
  },
): Promise<ReportCapture> {
  return await captureReport({
    sinon,
    run,
  },);
}

await describe({
  name: 'reports over a run folder',
  concurrency: 1,
  children: [
    describe({
      name: readLogTexts.name,
      concurrency: 1,
      children: [
        it({
          name: 'READS the records of a run folder as the lines the console printed, one per record, whatever '
            + 'the path names: the folder, its log directory, its day directory or the file',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'report-run-folders-', },);
            /**
             Lines the one file holds.
             */
            const lines = [
              '[info] [2026-08-25T10:00:00.000Z] [translation-repair] purr',
              '[warn] [2026-08-25T10:00:01.500Z] [translation-repair] [Tabby] nap',
            ];
            /**
             Run folder.
             */
            const runDir = await writeRunFolder({
              sinon: ctx.sinon,
              dir: scratch.path,
              name: 'corpus-pass-tabby',
              day: DAY,
              lines,
            },);

            /**
             What each way of naming the records read.
             */
            const [byFolder, byLogDirectory, byDay,] = await Promise.all([
              readLogTexts({ paths: [runDir,], },),
              readLogTexts({ paths: [join(runDir, 'log',),], },),
              readLogTexts({ paths: [join(runDir, 'log', DAY,),], },),
            ],);

            expect(byFolder,).toEqual([`${lines.join('\n',)}\n`,],);
            expect(byLogDirectory,).toEqual(byFolder,);
            expect(byDay,).toEqual(byFolder,);
          },
        },),

        it({
          name: 'READS every process\'s file of a run folder in name order, and keeps a line a crash cut off or '
            + 'another writer left as the line it is',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'report-run-folders-', },);
            /**
             Run folder with its first process's file.
             */
            const runDir = await writeRunFolder({
              sinon: ctx.sinon,
              dir: scratch.path,
              name: 'corpus-pass-tabby',
              day: DAY,
              lines: [
                '[info] [2026-08-25T10:00:00.000Z] first process',
                'a foreign line',
                '{"level":"info","message":"cut off",',
              ],
            },);
            await writeRecordFile({
              sinon: ctx.sinon,
              path: join(
                runDir,
                'log',
                DAY,
                `${DAY}T09-00-00.000Z.00000000-0000-4000-8000-000000000002.log.jsonl`,
              ),
              lines: ['[error] [2026-08-25T10:30:00.000Z] second process',],
            },);

            expect(await readLogTexts({ paths: [runDir,], },),).toEqual([
              '[info] [2026-08-25T10:00:00.000Z] first process\na foreign line\n{"level":"info","message":"cut off",\n'
                + '[error] [2026-08-25T10:30:00.000Z] second process\n',
            ],);
          },
        },),

        it({
          name: 'REFUSES a folder that holds no record file, naming it and what to name instead, so an empty report '
            + 'is never taken for a quiet run',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'report-run-folders-', },);
            /**
             Run folder of a pass that never started.
             */
            const empty = join(
              scratch.path,
              'corpus-pass-nothing',
            );
            await mkdir(join(empty, 'log',), { recursive: true, },);

            /**
             What the read threw.
             */
            const refusal = await rejectionOf(async function readsEmpty(): Promise<unknown> {
              return await readLogTexts({ paths: [empty,], },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toContain(`${empty} holds no record file (log/<day>/*.log.jsonl)`,);
          },
        },),

        it({
          name: 'REFUSES a file that is not a record file, a `.log` capture of the console among them',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'report-run-folders-', },);
            /**
             Path of a text capture.
             */
            const capture = join(
              scratch.path,
              'corpus-pass-tabby.log',
            );

            /**
             What the read threw.
             */
            const refusal = await rejectionOf(async function readsCapture(): Promise<unknown> {
              return await readLogTexts({ paths: [capture,], },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toContain(
              `${capture} is neither a run folder nor a file ending .log.jsonl`,
            );
          },
        },),
      ],
    },),

    describe({
      name: reportMeters.name,
      concurrency: 1,
      children: [
        it({
          name: 'REPORTS the same window and providers from a run folder as from the lines it holds',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'report-run-folders-', },);
            /**
             Lines of two readings, the second with the first provider out.
             */
            const lines = [EVERYTHING_WET, SYNTHETIC_DRY,];
            /**
             Run folder holding them.
             */
            const runDir = await writeRunFolder({
              sinon: ctx.sinon,
              dir: scratch.path,
              name: 'corpus-pass-tabby',
              day: DAY,
              lines,
            },);

            // THE READINGS OF THE FOLDER ARE THOSE OF THE LINES.
            expect(
              readMeterLog({ text: (await readLogTexts({ paths: [runDir,], },))[0] ?? '', },),
            ).toEqual(readMeterLog({ text: `${lines.join('\n',)}\n`, },),);

            /**
             What the report printed over the folder, the code it returned and the code it left.
             */
            const captured = await captureCodedReport({
              sinon: ctx.sinon,
              run: async function reportsFolder(): Promise<number> {
                return await reportMeters({
                  line: lineOf({
                    command: 'meter-report',
                    typed: [runDir,],
                  },),
                },);
              },
            },);

            expect({
              returned: captured.returned,
              left: captured.left,
            },).toEqual({
              returned: 0,
              left: 'unset',
            },);
            expect(captured.lines.slice(0, 3,),).toEqual([
              'meter-report: logs=1 readings=2 unread=0',
              '  window 2026-08-25T10:00:00.000Z .. 2026-08-25T10:30:00.000Z (30m)',
              ASKING_NOTE,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: reportSpendCost.name,
      concurrency: 1,
      children: [
        it({
          name: 'REPORTS the same seats, line count and unreadable count from a run folder as from the lines it holds',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'report-run-folders-', },);
            /**
             Lines of a metered call, a subscription call, a cut record and a line of nothing.
             */
            const lines = [
              HYPER_CHEAP,
              SUBSCRIPTION_CALL,
              CUT_SPEND_LINE,
              'meow',
            ];
            /**
             Run folder holding them.
             */
            const runDir = await writeRunFolder({
              sinon: ctx.sinon,
              dir: scratch.path,
              name: 'corpus-pass-tabby',
              day: DAY,
              lines,
            },);

            // THE TALLY OF THE FOLDER IS THAT OF THE LINES.
            expect(
              tallySpend({ lines: linesOfLog({ text: (await readLogTexts({ paths: [runDir,], },))[0] ?? '', },), },),
            ).toEqual(tallySpend({ lines, },),);

            /**
             What the report printed over the folder.
             */
            const captured = await printed({
              sinon: ctx.sinon,
              run: async function reportsFolder(): Promise<void> {
                await reportSpendCost({
                  line: lineOf({
                    command: 'spend-report',
                    typed: [runDir,],
                  },),
                },);
              },
            },);

            expect(captured,).toEqual({
              lines: [
                'spend-report: 1 log, 4 lines, 2 seats',
                '1 line carried the marker and would not parse, so this report\'s totals are short by whatever '
                  + 'those calls cost',
                'metered seats, priced at rates read 2026-10-11:',
                '  gemma-4-26b-a4b-it: 8.64 credits (100.0%) over 1 call, in 1000000=1.96 out 1000000=6.68',
                'metered run total: 8.64 credits',
                'subscription seats, which bill no credits and are metered as a percentage of a weekly allowance on '
                  + 'the METERS line:',
                '  hf:zai-org/GLM-5.3-Flash: 1 call, in 300 out 40',
              ],
              exitCode: 'unset',
            },);
          },
        },),
      ],
    },),

    describe({
      name: reportRunTiming.name,
      concurrency: 1,
      children: [
        it({
          name: 'REPORTS the same rounds and calls in flight from a run folder as from the lines it holds',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'report-run-folders-', },);
            /**
             Lines of a round and two calls.
             */
            const lines = [
              ROUND_STOOD_LINE,
              FIRST_CALL,
              SECOND_CALL,
            ];
            /**
             Run folder holding them.
             */
            const runDir = await writeRunFolder({
              sinon: ctx.sinon,
              dir: scratch.path,
              name: 'corpus-pass-tabby',
              day: DAY,
              lines,
            },);

            // THE READINGS OF THE FOLDER ARE THOSE OF THE LINES.
            expect(
              readRunTiming({ lines: linesOfLog({ text: (await readLogTexts({ paths: [runDir,], },))[0] ?? '', },), },),
            ).toEqual(readRunTiming({ lines, },),);

            /**
             What the report printed over the folder.
             */
            const captured = await printed({
              sinon: ctx.sinon,
              run: async function reportsFolder(): Promise<void> {
                await reportRunTiming({
                  line: lineOf({
                    command: 'run-timing-report',
                    typed: [runDir,],
                  },),
                },);
              },
            },);

            expect(captured,).toEqual({
              lines: [
                'run-timing-report: 1 log, 3 lines',
                'rounds                 1, 1.52min in total',
                '  without quorum       0',
                '  waiting after quorum 30.00s, 32.8% of round time',
                '  voices never heard   1',
                'calls in flight        mean 1.33, peak 2',
                '  busy against span    20.00s of calls across 15.00s of run',
              ],
              exitCode: 'unset',
            },);
          },
        },),
      ],
    },),
  ],
},);
