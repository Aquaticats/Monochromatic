/**
 Guards class one hundred three (zheermao7, 2026-09-23): the repair lane's
 checker bench was seated once at the lanes boundary, and the lanes phase's
 bench list never named the checkers, so when Synthetic's five-hour window
 ran out two minutes into the lane the two Synthetic-only checker seats were
 unreachable for the rest of it: ten of twelve checker rounds and their
 introduced-defect probes ran on one voice, short of quorum, while the
 substitute a dry reading seats (zheermao6, Synthetic dry from the start:
 two of three on every round) sat idle. The repair lane's per-chunk hook now
 re-reads the seats while a hold runs and hands the driver the roster to
 seat the next chunk with. Cat-themed invention throughout; no corpus
 content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { judgeSeatsFor, } from '@monochromatic-dev/translation-repair-run';
import { configuredConsolidationPolish, } from '@monochromatic-dev/translation-repair-consolidate';
import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider';
import {
  consolidationHooksFor,
  contestHooksFor,
  insertionHooksFor,
  lanesHooksFor,
  picturesHooksFor,
  preparationHooksFor,
} from '../../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { metersAllWet, } from '@monochromatic-dev/translation-repair-provider/ts/meter-states.test-fixture.ts';

/**
 The zheermao7 view at 23:15:27 UTC: Synthetic's window spent, Hyper dry,
 Bedrock and OpenRouter wet.
 */
const BEDROCK_AND_OPENROUTER: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  hyper: true,
  openrouter: false,
};

/**
 Synthetic, Hyper and OpenRouter dry, NeuralWatt and Bedrock wet: the view
 under which the lanes phase seats the measured checkers a wet provider
 serves first and pads the bench with two ranked seats nobody serves, as the
 zheermao20 chunks were padded with Qwen3.8-27B once Synthetic ran dry.
 */
const NEURALWATT_AND_BEDROCK: BudgetView = {
  neuralwatt: false,
  synthetic: true,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  hyper: true,
  openrouter: true,
};

/**
 No provider held out.
 */
const NO_HOLDS = {
  neuralwatt: 0,
  synthetic: 0,
  bedrock: 0,
  deepseek: 0,
  xiaomi: 0,
  hyper: 0,
  openrouter: 0,
};

/**
 A hold short enough for a case to wait it out. The router registers 300000 ms on a refusal from a meter
 that reads dry, and a seat reading sleeps the shortest hold it is told of when a bench the phase leans on
 is short of quorum, so a case that handed its client that figure and drove such a reading waited five real
 minutes in every gate (ledger B817). A case here needs a hold only to be running, so none carries more.
 */
const SHORT_HOLD_MS = 40;

/**
 Synthetic and Bedrock wet, Hyper and OpenRouter dry: the one view under which
 the lanes phase finds benches short (the editors and the refiners) and the
 translate lane finds none (measured 2026-09-28 over all sixteen views).
 */
const HYPER_AND_OPENROUTER_DRY: BudgetView = {
  neuralwatt: true,
  synthetic: false,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  hyper: true,
  openrouter: true,
};

/**
 Logger the hooks write to.
 */
const l = tagged({ tag: 'pass-reseat-test', },);

/**
 Builds a client whose holds can change between chunks and which counts
 its dryness reads.

 @param view - dryness every read answers

 @param holdEndsAtRead - read at which every running hold has ended, for a case whose bench stays short and whose
 wait would otherwise last as long as the hold does; a hold that never ends is not a hold

 @returns Client beside its read counter and its hold setter

 @example
 ```ts
 const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
 rig.holds.synthetic = SHORT_HOLD_MS;
 ```
 */
function viewClient(
  {
    view,
    holdEndsAtRead,
  }: {
    readonly view: BudgetView;
    readonly holdEndsAtRead?: number;
  },
) {
  /**
   Reads taken so far.
   */
  const counter = { reads: 0, };
  /**
   Holds as the router would report them now.
   */
  const holds = { ...NO_HOLDS, };
  return {
    counter,
    holds,
    client: {
      providerDryness: async (): Promise<BudgetView> => {
        counter.reads += 1;
        if (counter.reads === holdEndsAtRead)
          Object.assign(holds, NO_HOLDS,);
        return view;
      },
      providerMeterStates: metersAllWet,
      providerHolds: () => ({ ...holds, }),
    },
  };
}

/**
 A two-paragraph page the consolidation's naturalness roles read.
 */
const PREPARED = prepareDocumentPair({
  sourceText: '猫在窗边睡着了。\n\n它梦见了鱼。',
  targetText: 'The cat fell asleep by the window.\n\nIt dreamed of fish.',
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${lanesHooksFor.name} (class one hundred three)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RE-SEATS THE REPAIR LANE before a chunk while a hold runs, so the checker seats a dry-out took '
            + 'are filled by the next measured checkers a wet provider serves (class one hundred three, '
            + 'zheermao7, 2026-09-23: ten of twelve checker rounds on one voice after Synthetic ran dry two '
            + 'minutes into the lane)',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            rig.holds.synthetic = SHORT_HOLD_MS;
            const hooks = lanesHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
            },);
            /**
             Roster the hook hands the driver for the next chunk.
             */
            const seating = await hooks.beforeSlice({ lane: 'repair', },);
            expect(seating.repairModels,).toBeDefined();
            expect(seating.repairModels?.checkerModelIds,).toEqual([
              SEAT_OPENROUTER_ONLY_CHECKER,
              SEAT_BEDROCK_ONLY_VISION_UNSEATED,
              SEAT_OPENROUTER_ONLY,
            ],);
            expect(seating.repairModels?.checkerModelIds,).not.toContain(SEAT_SYNTHETIC_VISION_WITHHELD,);
            expect(rig.counter.reads,).toBe(1,);
          },
        },),
        it({
          name: 'MARKS IN THE CHUNK LINE THE CHECKERS THAT ARE PADDING, ranked seats no wet provider serves, '
            + 'and says none when every checker is served (zheermao20: Qwen3.8-27B sat on the bench after '
            + 'Synthetic ran dry and neither line said so)',
          fn: async () => {
            /** Rig whose NeuralWatt and Bedrock alone are wet, under a running hold. */
            const padded = viewClient({ view: NEURALWATT_AND_BEDROCK, holdEndsAtRead: 2, },);
            padded.holds.synthetic = SHORT_HOLD_MS;
            /** Lines the padded rig's hook wrote. */
            const paddedLines: string[] = [];
            await lanesHooksFor({
              client: padded.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
              l: capturingLogger({ messages: paddedLines, },),
            },).beforeSlice({ lane: 'repair', },);
            /** Rig whose every checker a wet provider serves, under a running hold. */
            const served = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            served.holds.synthetic = SHORT_HOLD_MS;
            /** Lines the served rig's hook wrote. */
            const servedLines: string[] = [];
            await lanesHooksFor({
              client: served.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
              l: capturingLogger({ messages: servedLines, },),
            },).beforeSlice({ lane: 'repair', },);

            // The padded bench is short of quorum, so this reading waits the hold out once, which is why the
            // hold is the short one; the served bench is full, so its reading waits for nothing.
            /** Text the reading's warning gives for a wait out of the hold. */
            const waitedOut = `waiting ${String(SHORT_HOLD_MS,)}ms for the shortest hold to end`;
            expect({
              padded: paddedLines.some(function namesTheWait(line,): boolean {
                return line.includes(waitedOut,);
              },),
              served: servedLines.some(function namesAWait(line,): boolean {
                return line.includes('for the shortest hold to end',);
              },),
            },).toEqual({
              padded: true,
              served: false,
            },);
            expect(paddedLines.filter(function isChunkLine(line,): boolean {
              return line.includes('chunk re-seated under a hold',);
            },),).toEqual([
              'JUDGE SEATS phase=lanes chunk re-seated under a hold: '
                + 'checkers=google.gemma-4-31b,hf:Qwen/Qwen3.8-27B,xiaomi/mimo-v2.6-flash '
                + 'unserved-checkers=hf:Qwen/Qwen3.8-27B,xiaomi/mimo-v2.6-flash',
            ],);
            expect(servedLines.filter(function isChunkLine(line,): boolean {
              return line.includes('chunk re-seated under a hold',);
            },),).toEqual([
              `JUDGE SEATS phase=lanes chunk re-seated under a hold: checkers=${
                [SEAT_OPENROUTER_ONLY_CHECKER, SEAT_BEDROCK_ONLY_VISION_UNSEATED, SEAT_OPENROUTER_ONLY,].join(',',)
              } unserved-checkers=none`,
            ],);
          },
        },),
        it({
          name: 'COSTS NOTHING while nothing is held: no dryness read, nothing to seat, so a pass under wet '
            + 'providers asks its meters exactly as often as before',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            const hooks = lanesHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
            },);
            expect(await hooks.beforeSlice({ lane: 'repair', },),).toEqual({},);
            expect(rig.counter.reads,).toBe(0,);
          },
        },),
        it({
          name: 'KEEPS THE LATEST SEATING once the hold has ended, so the chunks after it are not seated on '
            + 'the roster read before the dry-out',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            rig.holds.synthetic = SHORT_HOLD_MS;
            const hooks = lanesHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
            },);
            /**
             Roster read under the hold.
             */
            const underHold = await hooks.beforeSlice({ lane: 'repair', },);
            rig.holds.synthetic = 0;
            /**
             Roster handed over once the hold has ended.
             */
            const afterHold = await hooks.beforeSlice({ lane: 'repair', },);
            expect(afterHold.repairModels,).toBeDefined();
            expect(afterHold,).toEqual(underHold,);
            expect(rig.counter.reads,).toBe(1,);
          },
        },),
        it({
          name: 'COSTS NOTHING on the translate lane either while nothing is held: no dryness read, nothing to seat',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            const hooks = lanesHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
            },);
            expect(await hooks.beforeSlice({ lane: 'translate', },),).toEqual({},);
            expect(rig.counter.reads,).toBe(0,);
          },
        },),
        it({
          name: 'RE-SEATS THE TRANSLATE LANE before a slice while a hold runs, as it does the repair lane, and keeps '
            + 'that roster once the hold has ended (ledger H5: the fourth stage of a family classes one hundred '
            + 'three, one hundred nine and one hundred thirteen fixed one at a time)',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            rig.holds.synthetic = SHORT_HOLD_MS;
            const hooks = lanesHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
            },);
            /**
             Roster the hook hands the translate lane under the hold.
             */
            const underHold = await hooks.beforeSlice({ lane: 'translate', },);
            rig.holds.synthetic = 0;
            /**
             Roster handed over once the hold has ended.
             */
            const afterHold = await hooks.beforeSlice({ lane: 'translate', },);
            /**
             What a reading of this view seats on the translate lane.
             */
            const { translateModels, } = judgeSeatsFor({ dry: BEDROCK_AND_OPENROUTER, },);
            expect({
              underHold,
              afterHold,
              reads: rig.counter.reads,
            },).toEqual({
              underHold: { translateModels, },
              afterHold: { translateModels, },
              reads: 1,
            },);
          },
        },),
        it({
          name: 'KEEPS EACH LANE\'S SEATING APART: a repair chunk re-seated under a hold hands the translate lane '
            + 'nothing once the hold has ended, since the translate lane has not read its own',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            rig.holds.synthetic = SHORT_HOLD_MS;
            const hooks = lanesHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
            },);
            /**
             Seating the repair lane took under the hold.
             */
            const repair = await hooks.beforeSlice({ lane: 'repair', },);
            rig.holds.synthetic = 0;
            /**
             Seating the translate lane is handed once the hold has ended.
             */
            const translate = await hooks.beforeSlice({ lane: 'translate', },);
            expect({
              repairSeated: repair.repairModels !== undefined,
              translate,
              reads: rig.counter.reads,
            },).toEqual({
              repairSeated: true,
              translate: {},
              reads: 1,
            },);
          },
        },),
        it({
          name: 'READS EACH LANE UNDER ITS OWN PHASE: with Hyper and OpenRouter dry and Hyper held, the editors and '
            + 'refiners the lanes phase leans on are short while the translate lane\'s translators and select '
            + 'judges are not, so the translate lane seats on one read and the repair lane waits the hold out '
            + 'and stops the entry',
          fn: async () => {
            const rig = viewClient({ view: HYPER_AND_OPENROUTER_DRY, holdEndsAtRead: 3, },);
            rig.holds.hyper = SHORT_HOLD_MS;
            const hooks = lanesHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
            },);
            /**
             Seating the translate lane takes under the hold.
             */
            const translate = await hooks.beforeSlice({ lane: 'translate', },);
            /**
             Dryness reads the translate lane's seating took.
             */
            const translateReads = rig.counter.reads;
            await expect(hooks.beforeSlice({ lane: 'repair', },),)
              .rejects
              .toThrow('writing bench unreachable at lanes: editors',);
            expect({
              translateSeated: translate.translateModels !== undefined,
              translateReads,
              repairReads: rig.counter.reads - translateReads,
            },).toEqual({
              translateSeated: true,
              translateReads: 1,
              repairReads: 2,
            },);
          },
        },),
      ],
    },),

    describe({
      name: `${consolidationHooksFor.name} (ledger H5)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RE-SEATS THE CONSOLIDATION before a slice while a hold runs, as it does both lanes, handing the '
            + 'slice the reading\'s writers, slate judges and naturalness roles, and keeps that roster once the '
            + 'hold has ended',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            rig.holds.synthetic = SHORT_HOLD_MS;
            /**
             Every line the hook wrote.
             */
            const lines: string[] = [];
            const hooks = consolidationHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              prepared: PREPARED,
              l: capturingLogger({ messages: lines, },),
            },);
            /**
             Seating the hook hands the slice under the hold.
             */
            const underHold = await hooks.beforeSlice();
            rig.holds.synthetic = 0;
            /**
             Seating handed over once the hold has ended.
             */
            const afterHold = await hooks.beforeSlice();
            /**
             What a reading of this view seats.
             */
            const seats = judgeSeatsFor({ dry: BEDROCK_AND_OPENROUTER, },);
            /**
             Roster the slice should run on, its naturalness roles as `pass-consolidate.ts` builds them.
             */
            const roster = {
              modelIds: seats.writers,
              judgeModelIds: seats.slateJudges,
              polishConfig: configuredConsolidationPolish({
                prepared: PREPARED,
                models: seats.repairModels,
                gateModelIds: seats.lateJudges,
              },),
            };
            /**
             Line naming the phase the reading waited on and the roster it seated.
             */
            const reseatLine = `JUDGE SEATS phase=consolidation slice re-seated under a hold: writers=${seats.writers.join(',',)} `
              + `slateJudges=${seats.slateJudges.join(',',)} polishGate=${seats.lateJudges.join(',',)}`;
            expect({
              underHold,
              afterHold,
              reads: rig.counter.reads,
              logged: lines.includes(reseatLine,),
            },).toEqual({
              underHold: { roster, },
              afterHold: { roster, },
              reads: 1,
              logged: true,
            },);
          },
        },),
        it({
          name: 'COSTS NOTHING while nothing is held: no dryness read and no roster, so the slice keeps the one '
            + 'the consolidation started on',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            const hooks = consolidationHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              prepared: PREPARED,
              l,
            },);
            expect(await hooks.beforeSlice(),).toEqual({},);
            expect(rig.counter.reads,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: `${contestHooksFor.name} (ledger X12)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RE-SEATS THE LANE CONTEST before a slice while a hold runs, as it does the lanes and the '
            + 'consolidation, handing the slice the reading\'s judges, and keeps them once the hold has ended',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            rig.holds.synthetic = SHORT_HOLD_MS;
            /**
             Every line the hook wrote.
             */
            const lines: string[] = [];
            const hooks = contestHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              l: capturingLogger({ messages: lines, },),
            },);
            /**
             Seating the hook hands the slice under the hold.
             */
            const underHold = await hooks.beforeSlice();
            rig.holds.synthetic = 0;
            /**
             Seating handed over once the hold has ended.
             */
            const afterHold = await hooks.beforeSlice();
            /**
             Judges a reading of this view seats.
             */
            const { lateJudges, } = judgeSeatsFor({ dry: BEDROCK_AND_OPENROUTER, },);
            expect({
              underHold,
              afterHold,
              reads: rig.counter.reads,
              logged: lines.includes(`JUDGE SEATS phase=lane contest slice re-seated under a hold: judges=${lateJudges.join(',',)}`,),
            },).toEqual({
              underHold: { modelIds: lateJudges, },
              afterHold: { modelIds: lateJudges, },
              reads: 1,
              logged: true,
            },);
          },
        },),
        it({
          name: 'COSTS NOTHING while nothing is held: no dryness read and no judges, so the slice keeps those '
            + 'the contest started on',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            const hooks = contestHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              l,
            },);
            expect(await hooks.beforeSlice(),).toEqual({},);
            expect(rig.counter.reads,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: `${picturesHooksFor.name} (ledger X12)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RE-SEATS THE PICTURE READINGS before a picture while a hold runs, as it does every phase that '
            + 'buys item by item, handing the picture the reading\'s readers, and keeps them once the hold has ended',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            rig.holds.synthetic = SHORT_HOLD_MS;
            /**
             Every line the hook wrote.
             */
            const lines: string[] = [];
            const hooks = picturesHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              l: capturingLogger({ messages: lines, },),
            },);
            /**
             Seating the hook hands the picture under the hold.
             */
            const underHold = await hooks.beforePicture();
            rig.holds.synthetic = 0;
            /**
             Seating handed over once the hold has ended.
             */
            const afterHold = await hooks.beforePicture();
            /**
             Readers a reading of this view seats.
             */
            const { readers, } = judgeSeatsFor({ dry: BEDROCK_AND_OPENROUTER, },);
            expect({
              underHold,
              afterHold,
              reads: rig.counter.reads,
              logged: lines.includes(`JUDGE SEATS phase=pictures picture re-seated under a hold: readers=${readers.join(',',)}`,),
            },).toEqual({
              underHold: { readerModelIds: readers, },
              afterHold: { readerModelIds: readers, },
              reads: 1,
              logged: true,
            },);
          },
        },),
        it({
          name: 'COSTS NOTHING while nothing is held: no dryness read and no readers, so the picture keeps those '
            + 'the readings started on',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            const hooks = picturesHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              l,
            },);
            expect(await hooks.beforePicture(),).toEqual({},);
            expect(rig.counter.reads,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: `${insertionHooksFor.name} (ledger X12)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RE-SEATS THE INSERTION ADMISSION before a candidate while a hold runs, under its own phase, handing '
            + 'the candidate the reading\'s roster, and keeps it once the hold has ended',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            rig.holds.synthetic = SHORT_HOLD_MS;
            /**
             Every line the hook wrote.
             */
            const lines: string[] = [];
            const hooks = insertionHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              l: capturingLogger({ messages: lines, },),
            },);
            /**
             Seating the hook hands the candidate under the hold.
             */
            const underHold = await hooks.beforeCandidate();
            rig.holds.synthetic = 0;
            /**
             Seating handed over once the hold has ended.
             */
            const afterHold = await hooks.beforeCandidate();
            /**
             Roster a reading of this view seats.
             */
            const { roster, } = judgeSeatsFor({ dry: BEDROCK_AND_OPENROUTER, },);
            expect({
              underHold,
              afterHold,
              reads: rig.counter.reads,
              logged: lines.includes(`JUDGE SEATS phase=insertion admission candidate re-seated under a hold: roster=${roster.join(',',)}`,),
            },).toEqual({
              underHold: { modelIds: roster, },
              afterHold: { modelIds: roster, },
              reads: 1,
              logged: true,
            },);
          },
        },),
        it({
          name: 'COSTS NOTHING while nothing is held: no dryness read and no roster, so the candidate keeps the one '
            + 'the admission started on',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            const hooks = insertionHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              l,
            },);
            expect(await hooks.beforeCandidate(),).toEqual({},);
            expect(rig.counter.reads,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: `${preparationHooksFor.name} (ledger X12)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RE-SEATS THE PREPARATION before an item while a hold runs, under its own phase, handing the item '
            + 'the reading\'s roster, and keeps it once the hold has ended, so every later stage stays re-seated',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            rig.holds.synthetic = SHORT_HOLD_MS;
            /**
             Every line the hook wrote.
             */
            const lines: string[] = [];
            const hooks = preparationHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              l: capturingLogger({ messages: lines, },),
            },);
            /**
             Seating the hook hands the item under the hold.
             */
            const underHold = await hooks.beforeItem();
            rig.holds.synthetic = 0;
            /**
             Seating handed over once the hold has ended.
             */
            const afterHold = await hooks.beforeItem();
            /**
             Roster a reading of this view seats.
             */
            const { roster, } = judgeSeatsFor({ dry: BEDROCK_AND_OPENROUTER, },);
            expect({
              underHold,
              afterHold,
              reads: rig.counter.reads,
              logged: lines.includes(`JUDGE SEATS phase=preparation stage re-seated under a hold: roster=${roster.join(',',)}`,),
            },).toEqual({
              underHold: { modelIds: roster, },
              afterHold: { modelIds: roster, },
              reads: 1,
              logged: true,
            },);
          },
        },),
        it({
          name: 'COSTS NOTHING while nothing is held: no dryness read and no roster, so the item keeps the one '
            + 'the preparation started on',
          fn: async () => {
            const rig = viewClient({ view: BEDROCK_AND_OPENROUTER, },);
            const hooks = preparationHooksFor({
              client: rig.client,
              signal: new AbortController().signal,
              l,
            },);
            expect(await hooks.beforeItem(),).toEqual({},);
            expect(rig.counter.reads,).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
