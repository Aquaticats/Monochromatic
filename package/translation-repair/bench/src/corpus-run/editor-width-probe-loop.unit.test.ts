/**
 Tests for the loop that spends a width draw over scripted stages, in which no
 model is called and no file is written: the line printed for each slice, and
 the report written after every slice and once more at the end.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type BenchSlice,
  runWidthDraw,
  type WidthInputOutcome,
  type WidthRow,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scriptedClient, } from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/scripted-width-client.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { benchSliceOf, } from './editor-calibrate-rounds.test-fixture.ts';
import {
  widthInputOf,
  widthRowOf,
} from './editor-width-probe.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 One report as the loop asked for it to be written.
 */
type Written = {
  /**
   Rows settled so far, copied at the moment of the write.
   */
  readonly rows: readonly WidthRow[];

  /**
   Slices without work, by the wall they hit.
   */
  readonly skipped: Readonly<Record<string, number>>;
};

/**
 Spends a draw over scripted stages.

 @param drawn - slices to spend, in order

 @param outcomes - what gathering each slice's work finds, in the same order

 @param rows - what running each ready slice at both widths contributes, in the order the ready slices come

 @param written - collects every report the loop asks for

 @returns Nothing; the loop's prints are read off the capture

 @example
 ```ts
 await spend({ drawn, outcomes, rows, written, },);
 ```
 */
async function spend(
  {
    drawn,
    outcomes,
    rows,
    written,
  }: {
    readonly drawn: readonly BenchSlice[];
    readonly outcomes: readonly WidthInputOutcome[];
    readonly rows: readonly WidthRow[];
    readonly written: Written[];
  },
): Promise<void> {
  /**
   Rows not yet handed out, the next one first.
   */
  const rowsLeft = [...rows,];

  /**
   Outcomes not yet handed out, the next one first.
   */
  const outcomesLeft = [...outcomes,];

  await runWidthDraw({
    client: scriptedClient({ script: {}, },),
    drawn,
    narrowEditorIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
    wideEditorIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_HYPER_OPENROUTER_UNMEASURED,],
    judgeModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
    signal: new AbortController().signal,
    l: tagged({ tag: 'editor-width-probe-loop-test', },),
    controlHeld: true,
    draw: 'b',
    headSha: 'f00dcafe1234',
    runsDir: '/cattery/runs',
    gather: function scriptedGather(): Promise<WidthInputOutcome> {
      /**
       Outcome scripted for this slice.
       */
      const outcome = outcomesLeft.shift();
      if (outcome === undefined)
        return Promise.reject(new Error('gather was asked about more slices than the case scripted',),);
      return Promise.resolve(outcome,);
    },
    runSlice: function scriptedSlice(): Promise<WidthRow> {
      /**
       Row scripted for this ready slice.
       */
      const row = rowsLeft.shift();
      if (row === undefined)
        return Promise.reject(new Error('runSlice was asked about more ready slices than the case scripted',),);
      return Promise.resolve(row,);
    },
    writeReport: function recordingWrite(input,): Promise<string> {
      written.push({
        rows: [...input.rows,],
        skipped: input.skipped,
      },);
      return Promise.resolve(`/runs/editor-width-${input.draw}.md`,);
    },
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 Slice the loop is handed, named by position.

 @param index - position within the invented entry

 @returns Slice of the entry `mittens`

 @example
 ```ts
 const slice = sliceAt({ index: 0, },);
 ```
 */
function sliceAt({ index, }: { readonly index: number; },): BenchSlice {
  return benchSliceOf({ entryId: 'mittens', index, },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runWidthDraw.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS a line per ready slice, saying FLIPPED for a narrow repeat that disagreed, and writes the '
            + 'report after each slice and once more at the end, naming the path',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Every report the loop asked for.
             */
            const written: Written[] = [];
            /**
             Row of the slice whose narrow arm agreed with itself.
             */
            const agreed = widthRowOf({ entryId: 'mittens', sliceIndex: 0, comparison: 'same-text', narrowRepeatAgreed: true, },);
            /**
             Row of the slice whose narrow arm disagreed with itself.
             */
            const flipped = widthRowOf({ entryId: 'mittens', sliceIndex: 1, comparison: 'differs', narrowRepeatAgreed: false, },);

            await spend({
              drawn: [sliceAt({ index: 0, },), sliceAt({ index: 1, },),],
              outcomes: [
                {
                  kind: 'ready',
                  input: widthInputOf({ entryId: 'mittens', sliceIndex: 0, },),
                },
                {
                  kind: 'ready',
                  input: widthInputOf({ entryId: 'mittens', sliceIndex: 1, },),
                },
              ],
              rows: [agreed, flipped,],
              written,
            },);

            expect(printed.lines,).toEqual([
              'WIDTH mittens slice 0: same-text, repeat agreed, not-run',
              'WIDTH mittens slice 1: differs, repeat FLIPPED, not-run',
              'WIDTH wrote /runs/editor-width-b.md',
            ],);
            expect(written,).toEqual([
              {
                rows: [agreed,],
                skipped: {},
              },
              {
                rows: [agreed, flipped,],
                skipped: {},
              },
              {
                rows: [agreed, flipped,],
                skipped: {},
              },
            ],);
          },
        },),

        it({
          name: 'COUNTS the slices that carried no work by the wall they hit, prints each, and republishes after each',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Every report the loop asked for.
             */
            const written: Written[] = [];

            await spend({
              drawn: [sliceAt({ index: 0, },), sliceAt({ index: 1, },), sliceAt({ index: 2, },),],
              outcomes: [
                {
                  kind: 'skipped',
                  refusal: 'no-claims',
                  entryId: 'mittens',
                  sliceIndex: 0,
                },
                {
                  kind: 'skipped',
                  refusal: 'no-claims',
                  entryId: 'mittens',
                  sliceIndex: 1,
                },
                {
                  kind: 'skipped',
                  refusal: 'no-envelopes',
                  entryId: 'mittens',
                  sliceIndex: 2,
                },
              ],
              rows: [],
              written,
            },);

            expect(printed.lines,).toEqual([
              'WIDTH mittens slice 0: no-claims',
              'WIDTH mittens slice 1: no-claims',
              'WIDTH mittens slice 2: no-envelopes',
              'WIDTH wrote /runs/editor-width-b.md',
            ],);
            expect(written,).toEqual([
              {
                rows: [],
                skipped: { 'no-claims': 1, },
              },
              {
                rows: [],
                skipped: { 'no-claims': 2, },
              },
              {
                rows: [],
                skipped: {
                  'no-claims': 2,
                  'no-envelopes': 1,
                },
              },
              {
                rows: [],
                skipped: {
                  'no-claims': 2,
                  'no-envelopes': 1,
                },
              },
            ],);
          },
        },),

        it({
          name: 'WRITES the report once with no row and prints only its path when the draw holds no slice',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Every report the loop asked for.
             */
            const written: Written[] = [];

            await spend({
              drawn: [],
              outcomes: [],
              rows: [],
              written,
            },);

            expect(printed.lines,).toEqual(['WIDTH wrote /runs/editor-width-b.md',],);
            expect(written,).toEqual([
              {
                rows: [],
                skipped: {},
              },
            ],);
          },
        },),
      ],
    },),
  ],
},);
