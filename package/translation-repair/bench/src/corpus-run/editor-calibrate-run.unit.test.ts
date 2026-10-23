/**
 Tests for the editor calibration run over scripted clients and scripted
 process reaches, in which no model is called and no environment, corpus or
 clock is read.

 THE ORDER IS UNDER TEST as much as the text: the count is read before the
 overlap, the overlap before the corpus draw, the windows after the header,
 and the client after the windows, so a refusal arrives with exactly the
 output that preceded it.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { RUN_ROSTER, } from '@monochromatic-dev/translation-repair-run';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  type CalibrationGrace,
  StatedRefusalError,
  type WriterGrace,
} from '@monochromatic-dev/translation-repair-glossary';
import {
  type BenchSlice,
  EDITOR_LANE_EDITORS,
  runEditorCalibrate,
} from '../../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_ONLY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { PINNED_EDITOR_BENCH_START, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/editor-bench-start.test-fixture.ts';
import { benchSliceOf, } from './editor-calibrate-rounds.test-fixture.ts';
import { shippingLaneClient, } from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/editor-lane-script.test-fixture.ts';
import { scriptedClient, } from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/scripted-width-client.test-fixture.ts';

//region Fixtures

/**
 Models the lane seats as editors and as judges, which the header counts.
 */
const EDITORS = String(EDITOR_LANE_EDITORS.length,);
const JUDGES = String(RUN_ROSTER.length,);

/**
 The one slice every sample holds.
 */
const SLICE: BenchSlice = benchSliceOf({ entryId: 'mittens', index: 0, },);

/**
 Corpus the scripted draw is asked to read, which names nothing real.
 */
const PIN: CorpusPin = {
  cloneDir: '/no/such/clone',
  commitSha: 'a'.repeat(40,),
};

/**
 Window the calibration adopts when nothing set one.
 */
const DEFAULT_GRACE: CalibrationGrace = {
  effectiveMs: 300_000,
  source: 'calibration-default',
};

/**
 Writer window that follows every other round, which prints no note; both
 windows are the hang stop, since the lane's rounds run under them.
 */
const ROUND_WINDOW_WRITERS: WriterGrace = {
  writerMs: HANG_STOP_MS,
  roundMs: HANG_STOP_MS,
  source: 'round-window',
};

/**
 Note a seat prints when it judged no round.
 */
const NO_ROUNDS_NOTE = '  NO ROUNDS. This seat judged nothing across the sample, so it has no standing. For the editor '
  + 'seat that means no slice carried an ACCEPTED issue: critics can raise claims and the panel can adjudicate '
  + 'them and the lane still report "nothing to edit", which is what one live slice did. For the refiner seat it '
  + 'means the naturalness lane proposed nothing. Draw more slices.';

/**
 Closing paragraph on the slices that reached a rewriter, for a sample of one that reached none.
 */
const REACH_LINE = '  reached a rewriter on 0 of 1 slice; 1 slice carried no paragraph over the eligibility floor, '
  + 'so no refiner was asked there and its silence is not evidence about any model';

/**
 Everything a run reached for, recorded in the order it reached.
 */
type Reaches = {
  /**
   Names of the reaches, in order.
   */
  readonly events: string[];

  /**
   Counts the draw was asked for.
   */
  readonly counts: number[];

  /**
   Fallbacks the overlap dial was read with.
   */
  readonly fallbacks: number[];

  /**
   Pins the draw was asked to read.
   */
  readonly pins: CorpusPin[];
};

/**
 Runs the calibration over scripted reaches, which record what it reached for.

 @param typed - arguments after the command

 @param overlap - slices in flight the scripted dial reads

 @param grace - window the scripted adoption returns

 @param writers - window the scripted writer read returns

 @param newClient - client builder the run is handed

 @param reaches - record of what the run reached for, in order

 @throws whatever the run refuses with

 @example
 ```ts
 await calibrateWith({ typed: [], overlap: 4, grace, writers, newClient, reaches, },);
 ```
 */
async function calibrateWith(
  {
    typed,
    overlap,
    grace,
    writers,
    newClient,
    reaches,
    editorBenchStart,
  }: {
    readonly typed: readonly string[];
    readonly overlap: number;
    readonly grace: CalibrationGrace;
    readonly writers: WriterGrace;
    readonly newClient: () => ModelClient;
    readonly reaches: Reaches;
    readonly editorBenchStart?: number;
  },
): Promise<void> {
  await runEditorCalibrate({
    line: lineOf({
      command: 'editor-calibrate',
      typed,
    },),
    readOverlap: function overlapDial({ fallback, },): number {
      reaches.events.push('overlap',);
      reaches.fallbacks.push(fallback,);
      return overlap;
    },
    drawSample: function drawn({
      count,
      pin,
    },): Promise<readonly BenchSlice[]> {
      reaches.events.push('draw',);
      reaches.counts.push(count,);
      reaches.pins.push(pin,);
      return Promise.resolve([SLICE,],);
    },
    pin: PIN,
    adoptGrace: function adopted(): CalibrationGrace {
      reaches.events.push('grace',);
      return grace;
    },
    readWriterGrace: function writerWindow(): WriterGrace {
      reaches.events.push('writer-grace',);
      return writers;
    },
    newClient: function built(): ModelClient {
      reaches.events.push('client',);
      return newClient();
    },
    contestLedger: { kind: 'unkept', },
    ...((editorBenchStart === undefined) ? {} : { editorBenchStart, }),
  },);
}

/**
 Builds an empty record of reaches.

 @returns Record with nothing reached yet

 @example
 ```ts
 const reaches = noReaches();
 ```
 */
function noReaches(): Reaches {
  return {
    events: [],
    counts: [],
    fallbacks: [],
    pins: [],
  };
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runEditorCalibrate.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS the header, the window, one progress line, both empty standings and both closing '
            + 'paragraphs for a slice no critic raised a claim against, reaching for each thing once and in order',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            await calibrateWith({
              typed: [],
              overlap: 4,
              grace: DEFAULT_GRACE,
              writers: ROUND_WINDOW_WRITERS,
              newClient: function quiet(): ModelClient {
                return scriptedClient({ script: { critic_report: { issues: [], }, }, },);
              },
              reaches,
            },);

            expect(printed.lines,).toEqual([
              `editor-calibrate: 1 slice, ${EDITORS} models editing and ${JUDGES} judging each, 4 slices in flight`,
              'straggler window 300000ms (calibration default)',
              '  slice 1 of 1 (mittens chunk 0): 0 editor rounds, 0 refiner rounds (nothing eligible to rewrite), '
                + '0 editors shipping',
              '\nEDITOR standing over 0 judged rounds, from 0 of 1 slice',
              NO_ROUNDS_NOTE,
              '\nREFINER standing over 0 judged rounds, from 0 of 1 slice',
              NO_ROUNDS_NOTE,
              REACH_LINE,
              '\nEDITORS SHIPPED on 0 of 1 slice, 0 of them with no editor round judged at all',
              '  NOTHING SHIPPED. No slice in this sample carried an accepted issue.',
            ],);
            expect(reaches,).toEqual({
              events: ['overlap', 'draw', 'grace', 'writer-grace', 'client',],
              counts: [6,],
              fallbacks: [4,],
              pins: [PIN,],
            },);
          },
        },),

        it({
          name: 'ASKS the draw for the count on the command line, and names the override and the writer window '
            + 'when a launch set them',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            await calibrateWith({
              typed: ['3',],
              overlap: 1,
              grace: {
                effectiveMs: 1_234,
                source: 'override',
              },
              // THE HANG STOP FOR BOTH WINDOWS, since the lane's rounds run under
              // them; the note names whatever the dial set.
              writers: {
                writerMs: HANG_STOP_MS,
                roundMs: HANG_STOP_MS,
                source: 'writer-dial',
              },
              newClient: function quiet(): ModelClient {
                return scriptedClient({ script: { critic_report: { issues: [], }, }, },);
              },
              reaches,
            },);

            expect(reaches.counts,).toEqual([3,],);
            expect(printed.lines.slice(0, 3,),).toEqual([
              `editor-calibrate: 1 slice, ${EDITORS} models editing and ${JUDGES} judging each, 1 slice in flight`,
              'straggler window 1234ms (TRANSLATION_REPAIR_STRAGGLER_GRACE_MS override)',
              'WRITER GRACE OVERRIDDEN by TRANSLATION_REPAIR_WRITER_GRACE_MS: writer rounds (editor, refiner, '
                + 'translate, produceConsolidations, archive-block-review) abandon stragglers 60000ms after quorum rather than the 60000ms '
                + 'every other round waits',
            ],);
          },
        },),

        it({
          name: 'PRINTS every editor that wrote the shipped edit when every stage agrees on one edit, and both '
            + 'printers read the lone composite candidate no ballot was cast on as no judged round',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            await calibrateWith({
              typed: [],
              overlap: 4,
              grace: DEFAULT_GRACE,
              writers: ROUND_WINDOW_WRITERS,
              newClient: shippingLaneClient,
              reaches: noReaches(),
              editorBenchStart: PINNED_EDITOR_BENCH_START,
            },);

            /**
             The standing, the shipped heading and the credit lines.
             */
            const printedLines = printed.lines.slice(2,).filter(function standingOrShipping(line,): boolean {
              return line.includes('wrote shipping text',) || line.startsWith('\nEDITOR',)
                || line.startsWith('  NO JUDGED ROUNDS',);
            },);
            expect(printedLines.slice(0, 3,),).toEqual([
              '\nEDITOR standing over 0 judged rounds, from 0 of 1 slice; 1 round drew no ballot',
              '  NO JUDGED ROUNDS. The seat has no standing, since no ballot was cast on any round it produced: a '
                + 'slate of one candidate, which is what every producer proposing the same wording leaves, needs no '
                + 'vote, and a panel whose every judge abstained or failed casts none. Draw more slices.',
              '\nEDITORS SHIPPED on 1 of 1 slice, 1 of them with no editor round judged at all',
            ],);
            // THE SIX SEATS CREDITED ARE PINNED AGAIN (n166, Z3). The lane seats six editors from a bench the prompt rotates, so
            // which six of the seats that can hold an editor chair are credited moved whenever a sheet's wording did (n161:
            // the house tense rule). The rotation's start is injected, so no sheet text moves the seats. SONNET 5.5 TAKES THE
            // LAST SEAT SINCE 2026-10-22 (the lane seats the roster's writers and the run's named editors, and Haiku 5.5, seated
            // wide and not as an editor, holds no chair).
            expect(printedLines.slice(3,),).toEqual([
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_HYPER_ONLY,
              SEAT_HYPER_OPENROUTER_UNMEASURED,
              SEAT_BEDROCK_ONLY_TEXT,
              SEAT_OPENROUTER_ONLY,
              'claude-sonnet-5-5',
            ].map(function credited(modelId,): string {
              return `  ${modelId}: wrote shipping text on 1 of 1 slice`;
            },),);
            expect(printed.lines[2],).toBe(
              '  slice 1 of 1 (mittens chunk 0): 1 editor round, 0 refiner rounds (nothing eligible to rewrite), '
                + '6 editors shipping',
            );
          },
        },),

        it({
          name: 'REFUSES a count that is no number before the overlap is read, the sample drawn or anything printed',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            /**
             What the run refused with.
             */
            const refusal = await rejectionOf(async function countOfLetters() {
              await calibrateWith({
                typed: ['four',],
                overlap: 4,
                grace: DEFAULT_GRACE,
                writers: ROUND_WINDOW_WRITERS,
                newClient: shippingLaneClient,
                reaches,
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: slices must be a whole number written in digits, at most 9007199254740991, '
                + 'and "four" is not one',
            );
            expect(reaches.events,).toEqual([],);
            expect(printed.lines,).toEqual([],);
          },
        },),

        it({
          name: 'REFUSES in the overlap read before the sample is drawn, with nothing printed',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            /**
             What the run refused with when the overlap dial refuses.
             */
            const refusal = await rejectionOf(async function overlapRefused() {
              await runEditorCalibrate({
                line: lineOf({
                  command: 'editor-calibrate',
                  typed: [],
                },),
                readOverlap: function refusing(): number {
                  reaches.events.push('overlap',);
                  throw new StatedRefusalError({ says: 'the overlap dial is unreadable', },);
                },
                drawSample: function neverDrawn(): Promise<readonly BenchSlice[]> {
                  reaches.events.push('draw',);
                  return Promise.resolve([SLICE,],);
                },
                pin: PIN,
                adoptGrace: function neverAdopted(): CalibrationGrace {
                  reaches.events.push('grace',);
                  return DEFAULT_GRACE;
                },
                readWriterGrace: function neverRead(): WriterGrace {
                  reaches.events.push('writer-grace',);
                  return ROUND_WINDOW_WRITERS;
                },
                newClient: function neverBuilt(): ModelClient {
                  reaches.events.push('client',);
                  return shippingLaneClient();
                },
                contestLedger: { kind: 'unkept', },
              },);
            },);

            expect(String(refusal,),).toBe('StatedRefusalError: the overlap dial is unreadable',);
            expect(reaches.events,).toEqual(['overlap',],);
            expect(printed.lines,).toEqual([],);
          },
        },),

        it({
          name: 'REFUSES in the window adoption after the header is printed and before the client is built',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            /**
             What the run refused with when the window is unreadable.
             */
            const refusal = await rejectionOf(async function windowRefused() {
              await runEditorCalibrate({
                line: lineOf({
                  command: 'editor-calibrate',
                  typed: [],
                },),
                readOverlap: function dial(): number {
                  reaches.events.push('overlap',);
                  return 2;
                },
                drawSample: function drawn(): Promise<readonly BenchSlice[]> {
                  reaches.events.push('draw',);
                  return Promise.resolve([SLICE,],);
                },
                pin: PIN,
                adoptGrace: function refusing(): CalibrationGrace {
                  reaches.events.push('grace',);
                  throw new StatedRefusalError({ says: 'the straggler window is unreadable', },);
                },
                readWriterGrace: function neverRead(): WriterGrace {
                  reaches.events.push('writer-grace',);
                  return ROUND_WINDOW_WRITERS;
                },
                newClient: function neverBuilt(): ModelClient {
                  reaches.events.push('client',);
                  return shippingLaneClient();
                },
                contestLedger: { kind: 'unkept', },
              },);
            },);

            expect(String(refusal,),).toBe('StatedRefusalError: the straggler window is unreadable',);
            expect(reaches.events,).toEqual(['overlap', 'draw', 'grace',],);
            expect(printed.lines,).toEqual([
              `editor-calibrate: 1 slice, ${EDITORS} models editing and ${JUDGES} judging each, 2 slices in flight`,
            ],);
          },
        },),

        it({
          name: 'REFUSES in the client build after the header and the window lines are printed, asking no model',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            /**
             What the run refused with when no key is set.
             */
            const refusal = await rejectionOf(async function keyMissing() {
              await calibrateWith({
                typed: [],
                overlap: 4,
                grace: DEFAULT_GRACE,
                writers: ROUND_WINDOW_WRITERS,
                newClient: function withoutKey(): ModelClient {
                  throw new StatedRefusalError({ says: 'no provider key is set', },);
                },
                reaches,
              },);
            },);

            expect(String(refusal,),).toBe('StatedRefusalError: no provider key is set',);
            expect(reaches.events,).toEqual(['overlap', 'draw', 'grace', 'writer-grace', 'client',],);
            expect(printed.lines,).toEqual([
              `editor-calibrate: 1 slice, ${EDITORS} models editing and ${JUDGES} judging each, 4 slices in flight`,
              'straggler window 300000ms (calibration default)',
            ],);
          },
        },),
      ],
    },),
  ],
},);
