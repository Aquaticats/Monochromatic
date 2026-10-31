/**
 Tests for the spend report's procedure: which logs it reads, how it counts
 what they held and what it says when they cannot answer.

 A LOG WITH NO RECORDS IS NOT A RUN THAT SPENT NOTHING. A log written before
 the spend line existed carries none, so the report says it found nothing to
 total and prints no total at all: a zero total beside that sentence is the
 very figure it warns against reading.

 EVERY COUNT NOUN IS SHOWN IN THE SINGULAR AND THE PLURAL, and a count of lines
 is the lines a log holds, not the empty piece after its closing newline.

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
import { reportSpendCost, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from './command-line.test-fixture.ts';
import { writeLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/report-log-write.test-fixture.ts';
import {
  captureReport,
  type ReportCapture,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/report-run-capture.test-fixture.ts';
import {
  CUT_SPEND_LINE,
  HYPER_CHEAP,
  HYPER_QUIET,
  HYPER_RECKONED,
  SUBSCRIPTION_CALL,
} from './spend-report.test-fixture.ts';

/**
 What the report says when its logs carry no spend record.
 */
const NOTHING_RECORDED_LINE = 'NOTHING RECORDED. These logs carry no SPEND line, which is not the same as a run '
  + 'that spent nothing: every log written before spend-line.ts landed carries none. Check the run date against '
  + 'that landing before reading this as a free run.';

/**
 What the report says of a reckoned call, after the count and its verb.
 */
const RECKONED_REASON = 'written as reckonings, an attempt abandoned before it finished or a Bedrock attempt at '
  + 'its bound, so the tokens and cost this report shows for such calls are estimates or bounds rather than what '
  + 'the wire said (ledger P14)';

/**
 Runs the report over the logs named, with its printing diverted and its exit
 code held.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @param typed - arguments typed after the runner

 @returns What it printed and the exit code it left

 @example
 ```ts
 const captured = await reportedBySpend({ sinon: ctx.sinon, typed: [], },);
 ```
 */
async function reportedBySpend(
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
    run: async function reportedBySpendRun(): Promise<void> {
      await reportSpendCost({
        line: lineOf({
          command: 'spend-report',
          typed,
        },),
      },);
    },
  },);
}

await describe({
  name: reportSpendCost.name,
  concurrency: 1,
  children: [
    it({
      name: 'SAYS NOTHING WAS RECORDED and prints no total for a log with no spend line',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'spend-report-run-', },);
        /**
         Log written before the spend line existed.
         */
        const old = await writeLog({
          dir: scratch.path,
          name: 'old.log',
          lines: ['the cat slept',],
        },);

        /**
         What the report printed and the code it left.
         */
        const captured = await reportedBySpend({
          sinon: ctx.sinon,
          typed: [old,],
        },);

        expect(captured,).toEqual({
          lines: [
            'spend-report: 1 log, 1 line, 0 seats',
            NOTHING_RECORDED_LINE,
          ],
          exitCode: 'unset',
        },);
      },
    },),

    it({
      name: 'SAYS NOTHING WAS RECORDED and the lines that would not parse, plural, for logs of nothing but cut records',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'spend-report-run-', },);
        /**
         Log of two records cut off before their fields end.
         */
        const cut = await writeLog({
          dir: scratch.path,
          name: 'cut.log',
          lines: [
            CUT_SPEND_LINE,
            CUT_SPEND_LINE,
          ],
        },);

        /**
         What the report printed and the code it left.
         */
        const captured = await reportedBySpend({
          sinon: ctx.sinon,
          typed: [cut,],
        },);

        expect(captured,).toEqual({
          lines: [
            'spend-report: 1 log, 2 lines, 0 seats',
            NOTHING_RECORDED_LINE,
            '2 lines carried the marker and would not parse, so this report\'s totals are short by whatever '
              + 'those calls cost',
          ],
          exitCode: 'unset',
        },);
      },
    },),

    it({
      name: 'TOTALS two logs as one run and counts a line that would not parse in the singular',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'spend-report-run-', },);
        /**
         Log of a metered call, and log of a subscription call, a cut record and a line of nothing.
         */
        const metered = await writeLog({
          dir: scratch.path,
          name: 'metered.log',
          lines: [HYPER_CHEAP,],
        },);
        const rest = await writeLog({
          dir: scratch.path,
          name: 'rest.log',
          lines: [
            SUBSCRIPTION_CALL,
            CUT_SPEND_LINE,
            'meow',
          ],
        },);

        /**
         What the report printed and the code it left.
         */
        const captured = await reportedBySpend({
          sinon: ctx.sinon,
          typed: [metered, rest,],
        },);

        expect(captured,).toEqual({
          lines: [
            'spend-report: 2 logs, 4 lines, 2 seats',
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

    it({
      name: 'SAYS the calls written as reckonings in the plural, with the verb in the plural, and names them on the seat',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'spend-report-run-', },);
        /**
         Log of two reckoned calls.
         */
        const reckoned = await writeLog({
          dir: scratch.path,
          name: 'reckoned.log',
          lines: [
            HYPER_RECKONED,
            HYPER_RECKONED,
          ],
        },);

        /**
         What the report printed and the code it left.
         */
        const captured = await reportedBySpend({
          sinon: ctx.sinon,
          typed: [reckoned,],
        },);

        expect(captured,).toEqual({
          lines: [
            'spend-report: 1 log, 2 lines, 1 seat',
            'metered seats, priced at rates read 2026-10-11:',
            '  qwen3.8-max: 200.00 credits (100.0%) over 2 calls, in 2000000=80.00 out 1000000=120.00, '
              + '2 of them reckoned rather than reported',
            'metered run total: 200.00 credits',
            `RECKONED, NOT REPORTED: 2 calls were ${RECKONED_REASON}`,
          ],
          exitCode: 'unset',
        },);
      },
    },),

    it({
      name: 'SAYS the call written as a reckoning in the singular, with the verb in the singular',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'spend-report-run-', },);
        /**
         Log of one reckoned call.
         */
        const reckoned = await writeLog({
          dir: scratch.path,
          name: 'reckoned.log',
          lines: [HYPER_RECKONED,],
        },);

        /**
         What the report printed and the code it left.
         */
        const captured = await reportedBySpend({
          sinon: ctx.sinon,
          typed: [reckoned,],
        },);

        expect(captured,).toEqual({
          lines: [
            'spend-report: 1 log, 1 line, 1 seat',
            'metered seats, priced at rates read 2026-10-11:',
            '  qwen3.8-max: 100.00 credits (100.0%) over 1 call, in 1000000=40.00 out 500000=60.00, '
              + '1 of them reckoned rather than reported',
            'metered run total: 100.00 credits',
            `RECKONED, NOT REPORTED: 1 call was ${RECKONED_REASON}`,
          ],
          exitCode: 'unset',
        },);
      },
    },),

    it({
      name: 'SAYS the call that reported no usage block left a floor, with a seat priced at nothing',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'spend-report-run-', },);
        /**
         Log of one call that reported no usage.
         */
        const quiet = await writeLog({
          dir: scratch.path,
          name: 'quiet.log',
          lines: [HYPER_QUIET,],
        },);

        /**
         What the report printed and the code it left.
         */
        const captured = await reportedBySpend({
          sinon: ctx.sinon,
          typed: [quiet,],
        },);

        expect(captured,).toEqual({
          lines: [
            'spend-report: 1 log, 1 line, 1 seat',
            'metered seats, priced at rates read 2026-10-11:',
            '  qwen3.8-max: 0.00 credits (n/a) over 1 call, in 0=0.00 out 0=0.00',
            'metered run total: 0.00 credits',
            'FLOOR, NOT A TOTAL: 1 call reported no usage block, so its tokens are in no figure of this report',
          ],
          exitCode: 'unset',
        },);
      },
    },),

    it({
      name: 'REFUSES a log that cannot be read, naming it and the code, before anything is printed',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'spend-report-run-', },);
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
              await reportSpendCost({
                line: lineOf({
                  command: 'spend-report',
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

    it({
      name: 'REFUSES a log named twice, since every call in it would be totalled twice',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'spend-report-run-', },);
        /**
         Log named twice on the command line.
         */
        const twice = await writeLog({
          dir: scratch.path,
          name: 'twice.log',
          lines: [HYPER_CHEAP,],
        },);

        /**
         What the report threw.
         */
        const refusal = await rejectionOf(async function repeats(): Promise<void> {
          await captureReport({
            sinon: ctx.sinon,
            run: async function readsRepeated(): Promise<void> {
              await reportSpendCost({
                line: lineOf({
                  command: 'spend-report',
                  typed: [
                    twice,
                    twice,
                  ],
                },),
              },);
            },
          },);
        },);

        expect(String(refusal,),).toBe(
          `StatedRefusalError: the log ${twice} is named more than once, so its lines would be counted twice. `
            + 'Name each log once.',
        );
      },
    },),
  ],
},);
