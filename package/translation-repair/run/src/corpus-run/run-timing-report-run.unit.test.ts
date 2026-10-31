/**
 Tests for the run timing report's procedure: which logs it reads, what it
 prints about them and what it says when a log cannot answer.

 A LOG THAT CANNOT ANSWER IS SAID, NOT PASSED OVER. A log with no round line,
 a completion line with no duration, one with no stamp and a log with no timed
 call each name what is missing and that it is not the same as a run that
 never waited or made one call at a time, so each is a case here.

 EVERY COUNT NOUN IS SHOWN IN THE SINGULAR AND THE PLURAL, since the report
 once printed a count before a fixed plural.

 Every log is written under a scratch directory and named by path, so nothing
 here reads a real run.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { reportRunTiming, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from './command-line.test-fixture.ts';
import { writeLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/report-log-write.test-fixture.ts';
import {
  captureReport,
  type ReportCapture,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/report-run-capture.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  ROUND_STOOD_LINE,
  timedCallLine,
  UNSTAMPED_CALL_LINE,
  UNTIMED_CALL_LINE,
} from './run-timing-report.test-fixture.ts';

/**
 What the report says of a log that holds no round line.
 */
const NO_ROUND_LINE = 'NO ROUND LINE. This log predates round lines and call durations, so how long each fan-out '
  + 'took and how much of that was spent waiting after quorum are both unrecorded. That is not the same as a run '
  + 'that never waited.';

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
 Runs the report over the logs named, with its printing diverted and its exit
 code held.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @param typed - arguments typed after the runner

 @returns What it printed and the exit code it left

 @example
 ```ts
 const captured = await reportedByTiming({ sinon: ctx.sinon, typed: [], },);
 ```
 */
async function reportedByTiming(
  {
    sinon,
    typed,
  }: {
    readonly sinon: DisposableSandbox;
    readonly typed: readonly string[];
  },
): Promise<ReportCapture> {
  return await captureReport({
    sinon,
    run: async function reportedByTimingRun(): Promise<void> {
      await reportRunTiming({
        line: lineOf({
          command: 'run-timing-report',
          typed,
        },),
      },);
    },
  },);
}

await describe({
  name: reportRunTiming.name,
  concurrency: 1,
  children: [
    it({
      name: 'REPORTS the rounds and the calls in flight of the logs named, read as one run, and leaves no exit code',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-run-', },);
        /**
         Log of a round, and log of two calls.
         */
        const rounds = await writeLog({
          dir: scratch.path,
          name: 'rounds.log',
          lines: [ROUND_STOOD_LINE,],
        },);
        const calls = await writeLog({
          dir: scratch.path,
          name: 'calls.log',
          lines: [
            FIRST_CALL,
            SECOND_CALL,
          ],
        },);

        /**
         What the report printed and the code it left.
         */
        const captured = await reportedByTiming({
          sinon: ctx.sinon,
          typed: [rounds, calls,],
        },);

        expect(captured,).toEqual({
          lines: [
            'run-timing-report: 2 logs, 3 lines',
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

    it({
      name: 'SAYS NO ROUND LINE and NOTHING IN FLIGHT for a log with no timing line, naming one line in the singular',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-run-', },);
        /**
         Log written before the timing work.
         */
        const old = await writeLog({
          dir: scratch.path,
          name: 'old.log',
          lines: ['the cat slept',],
        },);

        /**
         What the report printed and the code it left.
         */
        const captured = await reportedByTiming({
          sinon: ctx.sinon,
          typed: [old,],
        },);

        expect(captured,).toEqual({
          lines: [
            'run-timing-report: 1 log, 1 line',
            NO_ROUND_LINE,
            'NOTHING IN FLIGHT: no call in this log carried a duration, so nothing can be counted in flight, '
              + 'which is not the same as a run that made one call at a time.',
          ],
          exitCode: 'unset',
        },);
      },
    },),

    it({
      name: 'SAYS NOTHING IN FLIGHT with the other reason when every timed call took no time',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-run-', },);
        /**
         Log of one call that took no time.
         */
        const instant = await writeLog({
          dir: scratch.path,
          name: 'instant.log',
          lines: [
            timedCallLine({
              stamp: '2026-08-25T10:00:10.000Z',
              elapsedMs: 0,
            },),
          ],
        },);

        /**
         What the report printed and the code it left.
         */
        const captured = await reportedByTiming({
          sinon: ctx.sinon,
          typed: [instant,],
        },);

        expect(captured,).toEqual({
          lines: [
            'run-timing-report: 1 log, 1 line',
            NO_ROUND_LINE,
            'NOTHING IN FLIGHT: every timed call in this log took no time, so no span holds a call in flight, '
              + 'which is not the same as a run that made one call at a time.',
          ],
          exitCode: 'unset',
        },);
      },
    },),

    it({
      name: 'COUNTS the completion lines with no duration or no stamp and says each is left out, at one of each',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-run-', },);
        /**
         Log of one call that can be timed, one that cannot and one with no stamp.
         */
        const mixed = await writeLog({
          dir: scratch.path,
          name: 'mixed.log',
          lines: [
            FIRST_CALL,
            UNTIMED_CALL_LINE,
            UNSTAMPED_CALL_LINE,
          ],
        },);

        /**
         What the report printed and the code it left.
         */
        const captured = await reportedByTiming({
          sinon: ctx.sinon,
          typed: [mixed,],
        },);

        expect(captured,).toEqual({
          lines: [
            'run-timing-report: 1 log, 3 lines',
            NO_ROUND_LINE,
            '1 completion line carries no elapsed field, predating call durations and leaving no interval to '
              + 'report. Any concurrency this report prints describes only the calls that could be timed.',
            'Completion lines whose stamp the logger did not write: 1. No instant places their calls, so any '
              + 'concurrency this report prints leaves them out.',
            'calls in flight        mean 1.00, peak 1',
            '  busy against span    10.00s of calls across 10.00s of run',
          ],
          exitCode: 'unset',
        },);
      },
    },),

    it({
      name: 'COUNTS the completion lines with no duration in the plural and agrees the verb with them',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-run-', },);
        /**
         Log of two calls with no duration.
         */
        const untimed = await writeLog({
          dir: scratch.path,
          name: 'untimed.log',
          lines: [
            UNTIMED_CALL_LINE,
            UNTIMED_CALL_LINE,
          ],
        },);

        /**
         What the report printed and the code it left.
         */
        const captured = await reportedByTiming({
          sinon: ctx.sinon,
          typed: [untimed,],
        },);

        expect(captured,).toEqual({
          lines: [
            'run-timing-report: 1 log, 2 lines',
            NO_ROUND_LINE,
            '2 completion lines carry no elapsed field, predating call durations and leaving no interval to '
              + 'report. Any concurrency this report prints describes only the calls that could be timed.',
            'NOTHING IN FLIGHT: no call in this log carried a duration, so nothing can be counted in flight, '
              + 'which is not the same as a run that made one call at a time.',
          ],
          exitCode: 'unset',
        },);
      },
    },),

    it({
      name: 'REFUSES a log that cannot be read, naming it and the code, before anything is printed',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-run-', },);
        /**
         Path where no log stands.
         */
        const missing = join(
          scratch.path,
          'missing.log.jsonl',
        );

        /**
         What the report threw.
         */
        const refusal = await rejectionOf(async function absent(): Promise<void> {
          await captureReport({
            sinon: ctx.sinon,
            run: async function readsAbsent(): Promise<void> {
              await reportRunTiming({
                line: lineOf({
                  command: 'run-timing-report',
                  typed: [missing,],
                },),
              },);
            },
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: the log ${missing} could not be read (ENOENT), so nothing was counted. `
            + 'Name a run folder a pass wrote, or one of its .log.jsonl record files, by a path that exists.',
        );
      },
    },),
  ],
},);
