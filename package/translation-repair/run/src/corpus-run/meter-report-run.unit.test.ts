/**
 Tests for the meter report's procedure: which logs it reads, how it merges
 their readings and what it says when they hold none.

 A READING IS ONE READING WHATEVER LOG HOLDS IT. Passing one log twice, or two
 logs that both carry a reading, must not double its weight, so exact repeats
 collapse; two readings that differ in any provider's state or in any number are
 two readings, however alike their stamps and their first two providers.

 A LOG WITH NO READING IS SAID, WITH ITS OWN EXIT CODE, and the count of
 records that would not read is printed beside it. The procedure returns the
 code and sets none on the process, which the entry file does;
 `meter-report.unit.test.ts` holds the code the built command exits with.

 Every log is written under a scratch directory and named by path, so nothing
 here reads a real run.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  mergeSamples,
  reportMeters,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from './command-line.test-fixture.ts';
import {
  ASKING_NOTE,
  BASE_AT,
  BOTH_DRY,
  EVERYTHING_WET,
  meterLine,
  NOTHING_RECORDED_LINE,
  sampleOf,
  SYNTHETIC_DRY,
} from './meter-report.test-fixture.ts';
import {
  captureCodedReport,
  captureReport,
  type CodedReportCapture,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/report-run-capture.test-fixture.ts';

/**
 Reading of every provider, each one wet, with the stamp and states the fixture lines carry.
 */
const FIRST_AT = BASE_AT;

/**
 Runs the report over the logs named, with its printing diverted and the
 process's exit code held.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @param typed - arguments typed after the runner

 @returns What it printed, the exit code it returned and the one it left on
 the process

 @example
 ```ts
 const captured = await reportedByMeters({ sinon: ctx.sinon, typed: [], },);
 ```
 */
async function reportedByMeters(
  {
    sinon,
    typed,
  }: {
    readonly sinon: DisposableSandbox;
    readonly typed: readonly string[];
  },
): Promise<CodedReportCapture> {
  return await captureCodedReport({
    sinon,
    run: async function reportedByMetersRun(): Promise<number> {
      return await reportMeters({
        line: lineOf({
          command: 'meter-report',
          typed,
        },),
      },);
    },
  },);
}

await describe({
  name: 'meter report run',
  concurrency: 1,
  children: [
    describe({
      name: mergeSamples.name,
      concurrency: 1,
      children: [
        it({
          name: 'MERGES the readings of every log into one list in time order',
          fn: async () => {
            expect(mergeSamples({
              readings: [
                {
                  samples: [
                    sampleOf({ overrides: { at: FIRST_AT + 2_000, }, },),
                    sampleOf({ overrides: { at: FIRST_AT, }, },),
                  ],
                  skippedLines: 0,
                },
                {
                  samples: [sampleOf({ overrides: { at: FIRST_AT + 1_000, }, },),],
                  skippedLines: 0,
                },
              ],
            },),).toEqual([
              sampleOf({ overrides: { at: FIRST_AT, }, },),
              sampleOf({ overrides: { at: FIRST_AT + 1_000, }, },),
              sampleOf({ overrides: { at: FIRST_AT + 2_000, }, },),
            ],);
          },
        },),

        it({
          name: 'COLLAPSES a reading both logs carry, so naming one log twice cannot double its weight',
          fn: async () => {
            /**
             One log's whole reading.
             */
            const log = {
              samples: [
                sampleOf({ overrides: { levels: ['hyperBalance=2497',], }, },),
                sampleOf({ overrides: { at: FIRST_AT + 1_000, }, },),
              ],
              skippedLines: 0,
            };
            expect(mergeSamples({
              readings: [
                log,
                log,
              ],
            },),).toEqual(log.samples,);
          },
        },),

        it({
          name: 'KEEPS two readings that differ only in the third provider\'s state',
          fn: async () => {
            expect(mergeSamples({
              readings: [
                { samples: [sampleOf({ overrides: { openrouter: 'wet', }, },),], skippedLines: 0, },
                { samples: [sampleOf({ overrides: { openrouter: 'dry', }, },),], skippedLines: 0, },
              ],
            },),).toEqual([
              sampleOf({ overrides: { openrouter: 'wet', }, },),
              sampleOf({ overrides: { openrouter: 'dry', }, },),
            ],);
          },
        },),

        it({
          name: 'KEEPS two readings that differ only in the fourth provider\'s state',
          fn: async () => {
            expect(mergeSamples({
              readings: [
                { samples: [sampleOf({ overrides: { bedrock: 'wet', }, },),], skippedLines: 0, },
                { samples: [sampleOf({ overrides: { bedrock: 'absent', }, },),], skippedLines: 0, },
              ],
            },),).toEqual([
              sampleOf({ overrides: { bedrock: 'wet', }, },),
              sampleOf({ overrides: { bedrock: 'absent', }, },),
            ],);
          },
        },),

        it({
          name: 'KEEPS two readings that differ only in their numbers',
          fn: async () => {
            expect(mergeSamples({
              readings: [
                { samples: [sampleOf({ overrides: { levels: ['hyperBalance=2497',], }, },),], skippedLines: 0, },
                { samples: [sampleOf({ overrides: { levels: ['hyperBalance=2000',], }, },),], skippedLines: 0, },
              ],
            },),).toEqual([
              sampleOf({ overrides: { levels: ['hyperBalance=2497',], }, },),
              sampleOf({ overrides: { levels: ['hyperBalance=2000',], }, },),
            ],);
          },
        },),

        it({
          name: 'KEEPS readings whose numbers read the same joined, one field against two',
          fn: async () => {
            expect(mergeSamples({
              readings: [
                { samples: [sampleOf({ overrides: { levels: ['hyperNote=a,hyperNote=b',], }, },),], skippedLines: 0, },
                { samples: [sampleOf({ overrides: { levels: ['hyperNote=a', 'hyperNote=b',], }, },),], skippedLines: 0, },
              ],
            },),).toHaveLength(2,);
          },
        },),

        it({
          name: 'MERGES no log into no reading',
          fn: async () => {
            expect(mergeSamples({ readings: [], },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: reportMeters.name,
      concurrency: 1,
      children: [
        it({
          name: 'SAYS NOTHING WAS RECORDED and returns exit code 1, setting none on the process, for a log with no '
            + 'reading',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'meter-report-run-', },);
            /**
             Log with no meter record.
             */
            const bare = join(
              scratch.path,
              'bare.log',
            );
            await writeFile(
              bare,
              'the cat slept\n',
            );

            /**
             What the report printed, the code it returned and the code it left.
             */
            const captured = await reportedByMeters({
              sinon: ctx.sinon,
              typed: [bare,],
            },);

            expect(captured,).toEqual({
              lines: [
                'meter-report: logs=1 readings=0 unread=0',
                NOTHING_RECORDED_LINE,
              ],
              returned: 1,
              left: 'unset',
            },);
          },
        },),

        it({
          name: 'COUNTS the records that would not read and still says NOTHING WAS RECORDED, returning exit code 1 '
            + 'and setting none on the process, when no other reading stands',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'meter-report-run-', },);
            /**
             Log of two records cut off before their second state.
             */
            const cut = join(
              scratch.path,
              'cut.log',
            );
            await writeFile(
              cut,
              'x METERS synthetic=wet hyp\nx METERS synthetic=dry hyp\n',
            );

            /**
             What the report printed, the code it returned and the code it left.
             */
            const captured = await reportedByMeters({
              sinon: ctx.sinon,
              typed: [cut,],
            },);

            expect(captured,).toEqual({
              lines: [
                'meter-report: logs=1 readings=0 unread=2',
                NOTHING_RECORDED_LINE,
              ],
              returned: 1,
              left: 'unset',
            },);
          },
        },),

        it({
          name: 'REPORTS the window the readings cover and each provider in the order the record names them, '
            + 'returning exit code 0 and setting none on the process',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'meter-report-run-', },);
            /**
             Log of two readings, the second with the first provider out.
             */
            const log = join(
              scratch.path,
              'run.log',
            );
            await writeFile(
              log,
              `${EVERYTHING_WET}\n${SYNTHETIC_DRY}\n`,
            );

            /**
             What the report printed, the code it returned and the code it left.
             */
            const captured = await reportedByMeters({
              sinon: ctx.sinon,
              typed: [log,],
            },);

            expect({
              returned: captured.returned,
              left: captured.left,
            },).toEqual({
              returned: 0,
              left: 'unset',
            },);
            expect(captured.lines.slice(
              0,
              3,
            ),).toEqual([
              'meter-report: logs=1 readings=2 unread=0',
              '  window 2026-08-25T10:00:00.000Z .. 2026-08-25T10:30:00.000Z (30m)',
              ASKING_NOTE,
            ],);
            expect(captured.lines.filter(function isProvider(line,): boolean {
              return line.startsWith('\n',);
            },),).toEqual([
              '\nneuralwatt: wet=0 dry=0 unreadable=0',
              '\nsynthetic: wet=1 dry=1 unreadable=0',
              '\nbedrock: wet=0 dry=0 unreadable=0',
              '\ndeepseek: wet=0 dry=0 unreadable=0',
              '\nxiaomi: wet=0 dry=0 unreadable=0',
              '\nanthropic: wet=0 dry=0 unreadable=0',
              '\nhyper: wet=2 dry=0 unreadable=0',
              '\nopenrouter: wet=0 dry=0 unreadable=0',
            ],);
          },
        },),

        it({
          name: 'KEEPS two readings that share a stamp and the first two states and differ in the fourth provider\'s',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'meter-report-run-', },);
            /**
             Two logs, each with a reading at one stamp, one finding Bedrock wet and the other dry.
             */
            const wet = join(
              scratch.path,
              'wet.log',
            );
            const dry = join(
              scratch.path,
              'dry.log',
            );
            await writeFile(
              wet,
              `${meterLine({
                stamp: '2026-08-25T10:00:00.000Z',
                reading: {
                  synthetic: 'wet',
                  hyper: 'wet',
                  bedrock: 'wet',
                  levels: [],
                },
              },)}\n`,
            );
            await writeFile(
              dry,
              `${meterLine({
                stamp: '2026-08-25T10:00:00.000Z',
                reading: {
                  synthetic: 'wet',
                  hyper: 'wet',
                  bedrock: 'dry',
                  levels: [],
                },
              },)}\n`,
            );

            /**
             What the report printed, the code it returned and the code it left.
             */
            const captured = await reportedByMeters({
              sinon: ctx.sinon,
              typed: [wet, dry,],
            },);

            expect(captured.lines.at(0,),).toBe('meter-report: logs=2 readings=2 unread=0',);
            expect(captured.lines.includes('\nbedrock: wet=1 dry=1 unreadable=0',),).toBe(true,);
          },
        },),

        it({
          name: 'COUNTS a log named twice once, with both names counted as logs',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'meter-report-run-', },);
            /**
             Log of one reading named twice.
             */
            const log = join(
              scratch.path,
              'once.log',
            );
            await writeFile(
              log,
              `${BOTH_DRY}\n`,
            );

            /**
             What the report printed, the code it returned and the code it left.
             */
            const captured = await reportedByMeters({
              sinon: ctx.sinon,
              typed: [log, log,],
            },);

            expect(captured.lines.at(0,),).toBe('meter-report: logs=2 readings=1 unread=0',);
          },
        },),

        it({
          name: 'REFUSES a log that cannot be read, naming it and the code, before anything is printed',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'meter-report-run-', },);
            /**
             Path where no log stands.
             */
            const missing = join(
              scratch.path,
              'missing.log',
            );

            /**
             What the report threw.
             */
            const refusal = await rejectionOf(async function absent(): Promise<void> {
              await captureReport({
                sinon: ctx.sinon,
                run: async function readsAbsent(): Promise<void> {
                  await reportMeters({
                    line: lineOf({
                      command: 'meter-report',
                      typed: [missing,],
                    },),
                  },);
                },
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              `StatedRefusalError: the log ${missing} could not be read (ENOENT), so nothing was counted. `
                + 'Name a log a pass, probe or calibration wrote, by a path that exists.',
            );
          },
        },),
      ],
    },),
  ],
},);
