/**
 Tests for the editor width probe run over scripted stages and scripted
 process reaches, in which no model is called, no file is written and no
 corpus, commit or clock is read.

 THE ORDER IS UNDER TEST as much as the text: the header before the draw is
 read, the draw refused before the corpus is sampled, the control before the
 commit is read, and nothing spent after a refusal.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  RUN_MODELS,
  RUN_ROSTER,
  RunConfigError,
} from '@monochromatic-dev/translation-repair-run';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  type BenchSlice,
  runEditorWidthProbe,
  type WidthInputOutcome,
  type WidthRow,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { benchSliceOf, } from './editor-calibrate-rounds.test-fixture.ts';
import {
  widthInputOf,
  widthRowOf,
} from './editor-width-probe.test-fixture.ts';
import { scriptedClient, } from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/scripted-width-client.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

//region Fixtures

/**
 Seats of the narrow arm, which the header counts.
 */
const NARROW = String(RUN_MODELS.editorModelIds.length,);

/**
 Seats of the wide arm and of the panel, which the header counts.
 */
const WIDE = String(RUN_ROSTER.length,);

/**
 Corpus the scripted draw is asked to read, which names nothing real.
 */
const PIN: CorpusPin = {
  cloneDir: '/no/such/clone',
  commitSha: 'a'.repeat(40,),
};

/**
 Header the run prints before it reads the draw.
 */
const HEADER = `WIDTH narrow ${NARROW} against wide ${WIDE}, panel of ${WIDE} held fixed`;

/**
 What a run reached for, in order, with the arguments that matter.
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
   Slices the control was shown.
   */
  readonly controlSlices: BenchSlice[][];

  /**
   Pins the draw was asked to read.
   */
  readonly pins: CorpusPin[];
};

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
    controlSlices: [],
    pins: [],
  };
}

/**
 Runs the probe over scripted stages.

 @param typed - arguments after the command

 @param size - slices the scripted draw returns

 @param controlHeld - whether the scripted control holds

 @param keyless - whether building the client refuses as a process holding
 no provider key does

 @param reaches - record of what the run reached for, in order

 @throws whatever the run refuses with

 @example
 ```ts
 await probeWith({ typed: [], size: 4, controlHeld: true, keyless: false, reaches, },);
 ```
 */
async function probeWith(
  {
    typed,
    size,
    controlHeld,
    keyless,
    reaches,
  }: {
    readonly typed: readonly string[];
    readonly size: number;
    readonly controlHeld: boolean;
    readonly keyless: boolean;
    readonly reaches: Reaches;
  },
): Promise<void> {
  await runEditorWidthProbe({
    line: lineOf({
      command: 'editor-width-probe',
      typed,
    },),
    runsDir: '/cattery/runs',
    newClient: function built(): ModelClient {
      reaches.events.push('client',);
      if (keyless)
        throw new RunConfigError({ variable: 'TRANSLATION_REPAIR_CATNIP_API_KEY', },);
      return scriptedClient({ script: {}, },);
    },
    drawSample: function drawn({
      count,
      pin,
    },): Promise<readonly BenchSlice[]> {
      reaches.events.push('draw',);
      reaches.counts.push(count,);
      reaches.pins.push(pin,);
      return Promise.resolve(Array.from(
        { length: size, },
        function sliceAt(_unused, index,): BenchSlice {
          return benchSliceOf({ entryId: 'mittens', index, },);
        },
      ),);
    },
    pin: PIN,
    readHeadSha: function head(): Promise<string> {
      reaches.events.push('head',);
      return Promise.resolve('f00dcafe1234',);
    },
    controlHolds: function control({ slices, },): Promise<boolean> {
      reaches.events.push('control',);
      reaches.controlSlices.push([...slices,],);
      return Promise.resolve(controlHeld,);
    },
    gather: function ready({ slice, },): Promise<WidthInputOutcome> {
      reaches.events.push('gather',);
      return Promise.resolve({
        kind: 'ready',
        input: widthInputOf({ entryId: slice.entryId, sliceIndex: slice.index, },),
      },);
    },
    runSlice: function row({ input, },): Promise<WidthRow> {
      reaches.events.push('slice',);
      return Promise.resolve(widthRowOf({
        entryId: input.entryId,
        sliceIndex: input.sliceIndex,
        comparison: 'same-text',
        narrowRepeatAgreed: true,
      },),);
    },
    writeReport: function write({ draw, },): Promise<string> {
      reaches.events.push('write',);
      return Promise.resolve(`/runs/editor-width-${draw}.md`,);
    },
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runEditorWidthProbe.name,
      concurrency: 1,
      children: [
        it({
          name: 'SPENDS draw A over the first and third slices of a sample of four, showing the control the whole '
            + 'sample, and reaches for each thing in order',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            await probeWith({ typed: ['4',], size: 4, controlHeld: true, keyless: false, reaches, },);

            expect(printed.lines,).toEqual([
              HEADER,
              'WIDTH sample 4, draw A 2, other half held back',
              'WIDTH control held; running draw A',
              'WIDTH mittens slice 0: same-text, repeat agreed, not-run',
              'WIDTH mittens slice 2: same-text, repeat agreed, not-run',
              'WIDTH wrote /runs/editor-width-a.md',
            ],);
            expect(reaches.events,).toEqual([
              'draw',
              'client',
              'control',
              'head',
              'gather',
              'slice',
              'write',
              'gather',
              'slice',
              'write',
              'write',
            ],);
            expect(reaches.counts,).toEqual([4,],);
            expect(reaches.pins,).toEqual([PIN,],);
            expect(reaches.controlSlices.map(function indices(slices,): readonly number[] {
              return slices.map(function indexOf(slice,): number {
                return slice.index;
              },);
            },),).toEqual([[0, 1, 2, 3,],],);
          },
        },),

        it({
          name: 'ASKS the draw for eighteen slices when the line names no count, and spends draw B on request',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            await probeWith({ typed: ['18', 'b',], size: 3, controlHeld: true, keyless: false, reaches, },);
            await probeWith({ typed: [], size: 2, controlHeld: true, keyless: false, reaches, },);

            expect(reaches.counts,).toEqual([18, 18,],);
            expect(printed.lines.filter(function drawLine(line,): boolean {
              return line.startsWith('WIDTH sample',) || line.startsWith('WIDTH control',);
            },),).toEqual([
              'WIDTH sample 3, draw B 1, other half held back',
              'WIDTH control held; running draw B',
              'WIDTH sample 2, draw A 1, other half held back',
              'WIDTH control held; running draw A',
            ],);
          },
        },),

        it({
          name: 'REFUSES a draw that is neither half after the header and before the sample is drawn',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            /**
             What the run refused with.
             */
            const refusal = await rejectionOf(async function drawC() {
              await probeWith({ typed: ['4', 'c',], size: 4, controlHeld: true, keyless: false, reaches, },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: editor width probe refused: draw must be \'a\' or \'b\', not \'c\'; spending '
                + 'draw A under another name would burn the held-back half of the sample without saying so',
            );
            expect(reaches.events,).toEqual([],);
            expect(printed.lines,).toEqual([HEADER,],);
          },
        },),

        it({
          name: 'REFUSES when the panel does not prefer intact text, spending no slice and reading no commit',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            /**
             What the run refused with when the control fails.
             */
            const refusal = await rejectionOf(async function controlFails() {
              await probeWith({ typed: ['4',], size: 4, controlHeld: false, keyless: false, reaches, },);
            },);

            expect(String(refusal,),).toBe(
              'StatedRefusalError: editor width probe refused: the panel did not prefer intact text over the same '
                + 'text with a sentence removed, so it cannot read the finer difference this draw asks about and '
                + 'the draw was not spent',
            );
            expect(reaches.events,).toEqual(['draw', 'client', 'control',],);
            expect(printed.lines,).toEqual([
              HEADER,
              'WIDTH sample 4, draw A 2, other half held back',
            ],);
          },
        },),

        it({
          name: 'REFUSES draw B of a sample of one slice before the control is bought, since it holds no slice',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            /**
             What the run refused with.
             */
            const refusal = await rejectionOf(async function emptyDrawB() {
              await probeWith({ typed: ['1', 'b',], size: 1, controlHeld: true, keyless: false, reaches, },);
            },);

            expect(String(refusal,),).toBe(
              'StatedRefusalError: editor width probe refused: draw B holds no slice of the 1 slice drawn, so '
                + 'spending it would buy the positive control and report nothing; ask for at least 2 slices',
            );
            expect(reaches.events,).toEqual(['draw',],);
            expect(printed.lines,).toEqual([
              HEADER,
              'WIDTH sample 1, draw B 0, other half held back',
            ],);
          },
        },),

        it({
          name: 'REFUSES draw A of a sample that holds no slice at all, with the count in the plural and one slice asked for',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            /**
             What the run refused with.
             */
            const refusal = await rejectionOf(async function emptySample() {
              await probeWith({ typed: [], size: 0, controlHeld: true, keyless: false, reaches, },);
            },);

            expect(String(refusal,),).toBe(
              'StatedRefusalError: editor width probe refused: draw A holds no slice of the 0 slices drawn, so '
                + 'spending it would buy the positive control and report nothing; ask for at least 1 slice',
            );
            expect(reaches.events,).toEqual(['draw',],);
            expect(printed.lines,).toEqual([
              HEADER,
              'WIDTH sample 0, draw A 0, other half held back',
            ],);
          },
        },),

        it({
          name: 'BUILDS THE CLIENT ONLY ONCE THE DRAW HOLDS A SLICE, so a process holding no key prints the header '
            + 'and the draw and then refuses with what building the client said, asking the panel nothing',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            /**
             What the run refused with when the client cannot be built.
             */
            const refusal = await rejectionOf(async function keylessProcess() {
              await probeWith({ typed: ['4',], size: 4, controlHeld: true, keyless: true, reaches, },);
            },);

            expect(refusal,).toBeInstanceOf(RunConfigError,);
            expect(String(refusal,),).toBe(
              'RunConfigError: TRANSLATION_REPAIR_CATNIP_API_KEY is not set; run under mise so sops injects it',
            );
            expect(reaches.events,).toEqual(['draw', 'client',],);
            expect(printed.lines,).toEqual([
              HEADER,
              'WIDTH sample 4, draw A 2, other half held back',
            ],);
          },
        },),

        it({
          name: 'REFUSES a slice count below one before the header, drawing nothing and building no client',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run reached for.
             */
            const reaches = noReaches();

            /**
             What the run refused with.
             */
            const refusal = await rejectionOf(async function noSlices() {
              await probeWith({ typed: ['0',], size: 4, controlHeld: true, keyless: false, reaches, },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe('StatedRefusalError: slices must be at least 1, and 0 is not',);
            expect(reaches.events,).toEqual([],);
            expect(printed.lines,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
