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
import {
  type BudgetView,
  createProviderBudgets,
} from '@monochromatic-dev/translation-repair-provider';
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
 Bedrock and OpenRouter wet. THE CLAUDE KEY READS DRY here and in the other views of this file, as a run without the key reads it,
 so the checker benches these cases name stay as they were before Haiku 5.5 was seated; the case that seats Haiku with the
 key wet uses {@link MIMO_UNSERVED_CLAUDE_WET}.
 */
const BEDROCK_AND_OPENROUTER: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: true,
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
  anthropic: true,
  hyper: true,
  openrouter: true,
};

/**
 Synthetic, Hyper and Bedrock wet and the Claude key wet, Xiaomi and OpenRouter dry: the view under which nobody serves
 the second measured checker (Mimo is served by those two alone), so the bench's third place goes to the next of the
 order a wet provider serves, which is Haiku 5.5 on the Claude key.
 */
const MIMO_UNSERVED_CLAUDE_WET: BudgetView = {
  neuralwatt: true,
  synthetic: false,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  anthropic: false,
  hyper: false,
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
  anthropic: 0,
  hyper: 0,
  openrouter: 0,
};

/**
 The hold the router registers on a refusal from a meter that reads dry.

 A READING UNDER A HOLD SLEEPS ITS SHORTEST HOLD ONCE WHEN A BENCH IT LEANS ON IS SHORT OF QUORUM, on the real clock
 (`run/src/corpus-run/run-seats-read.ts`, `readDrynessPastShortBench`), so a case that handed its client this figure
 and drove such a reading waited five real minutes in every gate (ledger B817). This file's views under this hold are
 short in one of them (the padded view, 3 of 8 wide seats against a quorum of 4 with the Claude key reading dry), and
 the describe took 300.3 s at `f2f77b0701` and at the tip of the second review. `viewClient` therefore reports every
 hold as ending within {@link SHORT_HOLD_MS}, whatever a case names.
 */
const DRY_HOLD_MS = 300_000;

/**
 A hold short enough for a case to wait it out. A case here needs a hold only to be running, so a case that does not
 report its hold through the cap names this one, and a case whose bench stays short can also end every hold at a
 named read (`holdEndsAtRead`).
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
  anthropic: true,
  hyper: true,
  openrouter: true,
};

/**
 Logger the hooks write to.
 */
const l = tagged({ tag: 'pass-reseat-test', },);

/**
 The holds a rig reports: each one a case names, ended within the window the rig allows.

 @param holds - holds as the case set them

 @param capMs - longest a reported hold lasts; {@link SHORT_HOLD_MS} for a case that waits on the real clock

 @returns Same providers held, none for longer than the cap

 @example
 ```ts
 const reported = endingHolds({ holds: { ...NO_HOLDS, synthetic: DRY_HOLD_MS, }, capMs: SHORT_HOLD_MS, },);
 // synthetic: 40
 ```
 */
function endingHolds(
  {
    holds,
    capMs,
  }: {
    readonly holds: typeof NO_HOLDS;
    readonly capMs: number;
  },
): typeof NO_HOLDS {
  return {
    neuralwatt: Math.min(holds.neuralwatt, capMs,),
    synthetic: Math.min(holds.synthetic, capMs,),
    bedrock: Math.min(holds.bedrock, capMs,),
    deepseek: Math.min(holds.deepseek, capMs,),
    xiaomi: Math.min(holds.xiaomi, capMs,),
    anthropic: Math.min(holds.anthropic, capMs,),
    hyper: Math.min(holds.hyper, capMs,),
    openrouter: Math.min(holds.openrouter, capMs,),
  };
}

/**
 Builds a client whose holds can change between chunks and which counts
 its dryness reads.

 @param view - dryness every read answers

 @param capMs - longest a hold the client reports lasts; the short window unless a case waits the hold
 out on a fake clock

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
    capMs = SHORT_HOLD_MS,
    holdEndsAtRead,
  }: {
    readonly view: BudgetView;
    readonly capMs?: number;
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
      providerHolds: () => endingHolds({ holds, capMs, },),
    },
  };
}

/**
 Meters that all read wet, one for each provider a case can need wet.
 */
const WET_METERS = {
  synthetic: {
    quotas: async () => ({
      weekly: { percentRemaining: 62, nextRegenAt: '2026-10-21T13:24:00.000Z', },
    }),
  },
  hyper: { credits: async () => ({ balance: 243, }), },
  bedrock: {
    credits: async () => ({
      creditUsd: 200,
      spentUsd: 1,
      reckonedUsd: 0,
      remainingUsd: 199,
      calls: 1,
    }),
  },
  xiaomi: { spend: async () => ({ spentUsd: 0, }), },
  anthropic: { spend: async () => ({ spentUsd: 0, billingRefused: false, }), },
  openrouter: {
    credits: async () => ({ purchasedUsd: 100, usedUsd: 40, remainingUsd: 60, }),
  },
};

/**
 Builds a client over the production budgets, so a hold a case sets folds into the dryness the way the run's router
 folds it and not the way a test double writes it.

 A PROVIDER THE VIEW READS DRY HAS NO METER, which the budgets read as dry, and a provider it reads wet has a wet meter.
 Every hold the budgets place lasts {@link SHORT_HOLD_MS}, and the stated wait a case names is cut to that by the
 budgets' own cap, so the reading that waits a hold out on the real clock waits no longer than a short window.

 @param view - dryness the meters give before any hold

 @returns Run client's three budget reads, and the budgets a case marks refusals on

 @throws {@link Error} when the view reads a provider wet that this file has no wet meter for

 @example
 ```ts
 const rig = budgetsClient({ view: MIMO_UNSERVED_CLAUDE_WET, },);
 await rig.budgets.markRefused({ provider: 'synthetic', signal, statedWaitMs: DRY_HOLD_MS, },);
 ```
 */
function budgetsClient(
  { view, }: { readonly view: BudgetView; },
) {
  /**
   Providers the view reads wet, each with the meter that says so.
   */
  const wet = Object.entries(view,)
    .filter(function isWet([, dry,],): boolean {
      return !dry;
    },)
    .map(function meterOf([provider,],): readonly [string, unknown] {
      if (!(provider in WET_METERS))
        throw new Error(`no wet meter is written for ${provider}`,);
      return [provider, WET_METERS[provider as keyof typeof WET_METERS],];
    },);
  /**
   Budgets whose holds all last the short window and whose clock stands still.
   */
  const budgets = createProviderBudgets({
    ...Object.fromEntries(wet,),
    maxHoldMs: SHORT_HOLD_MS,
    cooldownMs: SHORT_HOLD_MS,
    rateLimitBackoffMs: SHORT_HOLD_MS,
    now: () => 10_000,
  },);
  return {
    budgets,
    client: {
      providerDryness: budgets.read,
      providerMeterStates: budgets.readStates,
      providerHolds: budgets.holds,
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
          name: 'RE-SEATS THE REPAIR LANE before a chunk while a hold runs, so the checker seats a dry-out took are '
            + 'filled by the next measured checkers a wet provider serves (class one hundred three, zheermao7, '
            + '2026-09-23: ten of twelve checker rounds on one voice after Synthetic ran dry two minutes into the '
            + 'lane)',
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
          name: 'TESTS THE RIG, NOT PRODUCTION: viewClient REPORTS NO HOLD LONGER THAN THE SHORT WINDOW A CASE CAN '
            + 'WAIT OUT, however long the case names it (the 300 s sleep of the padded rig, measured at 300.3 s of '
            + 'the describe at f2f77b0701 and at the tip of the second review); this guards the fixture against a '
            + 'later edit and the case that waits a 300 s hold out in production code is the one on a fake clock',
          fn: async () => {
            /** Rig whose padded view is short of quorum, under the hold the router registers on a dry refusal. */
            const rig = viewClient({ view: NEURALWATT_AND_BEDROCK, },);
            rig.holds.synthetic = DRY_HOLD_MS;
            expect(rig.client.providerHolds().synthetic,).toBeGreaterThan(0,);
            expect(rig.client.providerHolds().synthetic,).toBeLessThanOrEqual(SHORT_HOLD_MS,);
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
          name: 'SEATS THE WIDE BENCH AT ITS QUORUM WITH THE CLAUDE KEY WET AND SHORT OF IT WITH THE KEY DRY, as the '
            + 'live roster seats Haiku 5.5 (the fleet models the Claude provider in both of its states): the padded '
            + 'view with the key wet reads 4 of 8 wide seats and waits for nothing, and with the key dry reads 3 of 8 '
            + 'against a quorum of 4, waits the hold out once and seats what it reads',
          fn: async () => {
            /** Rig of the padded view with the Claude key wet, under a running Synthetic hold. */
            const wet = viewClient({ view: { ...NEURALWATT_AND_BEDROCK, anthropic: false, }, },);
            wet.holds.synthetic = DRY_HOLD_MS;
            /** Rig of the padded view with the Claude key dry, under the same hold. */
            const dry = viewClient({ view: NEURALWATT_AND_BEDROCK, holdEndsAtRead: 2, },);
            dry.holds.synthetic = DRY_HOLD_MS;
            /** Lines the wet rig's hook wrote. */
            const wetLines: string[] = [];
            /** Lines the dry rig's hook wrote. */
            const dryLines: string[] = [];
            /** Roster the hook hands the driver, Claude key wet. */
            const wetSeating = await lanesHooksFor({
              client: wet.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
              l: capturingLogger({ messages: wetLines, },),
            },).beforeSlice({ lane: 'repair', },);
            /** Roster the hook hands the driver, Claude key dry. */
            const drySeating = await lanesHooksFor({
              client: dry.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
              l: capturingLogger({ messages: dryLines, },),
            },).beforeSlice({ lane: 'repair', },);
            /**
             The lines either hook wrote that say a bench was short of its quorum.
             */
            const shortLines = (lines: readonly string[],): readonly string[] => lines.filter(
              function isShort(line,): boolean {
                return line.includes('short of quorum',);
              },
            );

            expect(shortLines(wetLines,),).toEqual([],);
            expect(shortLines(dryLines,),).toHaveLength(1,);
            expect(shortLines(dryLines,)[0],).toContain('wide 3 of 8 reachable, quorum 4',);
            expect(shortLines(dryLines,)[0],).toContain(
              `waiting ${String(SHORT_HOLD_MS,)}ms for the shortest hold to end`,
            );
            // THE CLAUDE KEY IS NOT THE HOLD: the hold waited out is Synthetic's, and the dry arm seats the same bench
            // after it, with Haiku 5.5 unserved, where the wet arm seats Haiku first.
            expect(wetSeating.repairModels?.checkerModelIds,).toEqual([
              'claude-haiku-5-5',
              'google.gemma-4-31b',
              'hf:Qwen/Qwen3.8-27B',
            ],);
            expect(drySeating.repairModels?.checkerModelIds,).not.toContain('claude-haiku-5-5',);
            expect(dryLines.filter(function isChunkLine(line,): boolean {
              return line.includes('chunk re-seated under a hold',);
            },).join('\n',),).not.toContain('claude-haiku-5-5',);
          },
        },),
        it({
          name: 'SEATS HAIKU 5.5 ON THE CHECKER BENCH WHILE THE CLAUDE KEY IS WET AND SEATS THE NEXT MEASURED CHECKER '
            + 'INSTEAD WHEN IT IS DRY: with the second measured checker unserved, the pass-level hook that '
            + 're-reads the view takes the first three a wet provider serves, and the Claude provider is the fourth '
            + 'of the order that fills the place',
          fn: async () => {
            /** Rig whose Claude key is wet and whose second measured checker nobody serves, under a running hold. */
            const wet = viewClient({ view: MIMO_UNSERVED_CLAUDE_WET, holdEndsAtRead: 2, },);
            wet.holds.synthetic = DRY_HOLD_MS;
            /** The same rig with the Claude key dry. */
            const dry = viewClient({ view: { ...MIMO_UNSERVED_CLAUDE_WET, anthropic: true, }, holdEndsAtRead: 2, },);
            dry.holds.synthetic = DRY_HOLD_MS;
            /** Roster the hook hands the driver for the next chunk, Claude key wet. */
            const wetSeating = await lanesHooksFor({
              client: wet.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
            },).beforeSlice({ lane: 'repair', },);
            /** Roster the hook hands the driver for the next chunk, Claude key dry. */
            const drySeating = await lanesHooksFor({
              client: dry.client,
              signal: new AbortController().signal,
              entryId: 'mittens',
            },).beforeSlice({ lane: 'repair', },);

            expect(wetSeating.repairModels?.checkerModelIds,).toEqual([
              'hf:Qwen/Qwen3.8-27B',
              'hf:moonshotai/Kimi-K3',
              'claude-haiku-5-5',
            ],);
            expect(drySeating.repairModels?.checkerModelIds,).toEqual([
              'hf:Qwen/Qwen3.8-27B',
              'hf:moonshotai/Kimi-K3',
              'google.gemma-4-31b',
            ],);
          },
        },),
        it({
          name: 'DROPS HAIKU 5.5 FROM THE NEXT SEATING WHEN THE CLAUDE KEY GOES FROM WET TO HELD (R3 of the second '
            + 'review): the first seating, with the key wet, takes Haiku as the fourth of the order, and once the '
            + 'production budgets mark the key refused the router reads the key dry and the next seating has no Haiku '
            + 'in the repair checker bench; the same rig with no hold on the key seats Haiku again',
          fn: async () => {
            /** Rig over the production budgets, so a hold folds into the view as the router folds it. */
            const rig = budgetsClient({ view: MIMO_UNSERVED_CLAUDE_WET, },);
            /** Signal the refusals are marked under. */
            const { signal, } = new AbortController();
            await rig.budgets.markRefused({ provider: 'synthetic', signal, statedWaitMs: DRY_HOLD_MS, },);
            const hooks = lanesHooksFor({
              client: rig.client,
              signal,
              entryId: 'mittens',
            },);
            /** Seating while the Claude key is wet. */
            const wetSeating = await hooks.beforeSlice({ lane: 'repair', },);
            // THE NEGATIVE ARM: nothing else changed between the readings, so Haiku is seated again.
            /** Seating a second time with the Claude key still wet. */
            const stillWetSeating = await hooks.beforeSlice({ lane: 'repair', },);
            await rig.budgets.markRefused({ provider: 'anthropic', signal, statedWaitMs: DRY_HOLD_MS, },);
            /** Seating once the Claude key is held. */
            const heldSeating = await hooks.beforeSlice({ lane: 'repair', },);

            expect(wetSeating.repairModels?.checkerModelIds,).toContain('claude-haiku-5-5',);
            expect(stillWetSeating.repairModels?.checkerModelIds,).toContain('claude-haiku-5-5',);
            expect(heldSeating.repairModels?.checkerModelIds,).not.toContain('claude-haiku-5-5',);
            // THE FIFTH OF THE ORDER FILLS THE PLACE, and the bench rotates between chunks, so the case reads
            // membership and not the order of the three.
            expect(heldSeating.repairModels?.checkerModelIds,).toContain('google.gemma-4-31b',);
            expect(heldSeating.repairModels?.checkerModelIds?.length,).toBe(3,);
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
            + 'refiners the lanes phase leans on are short while the translate lane\'s translators and select judges '
            + 'are not, so the translate lane seats on one read and the repair lane waits the hold out and stops the '
            + 'entry',
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
            + 'slice the reading\'s writers, slate judges and naturalness roles, and keeps that roster once the hold '
            + 'has ended',
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
          name: 'COSTS NOTHING while nothing is held: no dryness read and no judges, so the slice keeps those the '
            + 'contest started on',
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
          name: 'COSTS NOTHING while nothing is held: no dryness read and no readers, so the picture keeps those the '
            + 'readings started on',
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

    // SEQUENTIAL: the case fakes the process-wide timer function.
    describe({
      name: `${lanesHooksFor.name} (a 300 s hold waited out on a fake clock)`,
      concurrency: 1,
      children: [
        it({
          name: 'WAITS A 300 S HOLD OUT THROUGH THE PRODUCTION WAIT AND NO SOONER: a view short of quorum under a '
            + 'Synthetic hold of 300,000 ms sleeps the hold in slices on a fake clock, is still waiting a millisecond '
            + 'before it ends, and reads again and seats once it has',
          fn: async (ctx) => {
            /** The fake clock the production wait sleeps on. */
            const clock = ctx.sinon.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout',], },);
            /** Rig whose padded view is short of quorum, reporting the hold whole. */
            const rig = viewClient({ view: NEURALWATT_AND_BEDROCK, capMs: DRY_HOLD_MS, holdEndsAtRead: 2, },);
            rig.holds.synthetic = DRY_HOLD_MS;
            /** Lines the hook wrote. */
            const lines: string[] = [];
            /** Whether the hook has returned. */
            const returned = { done: false, };

            /**
             Seats the next chunk on the fake clock and notes when the hook returns.

             @returns Roster the hook handed the driver
             */
            async function seated() {
              /** Roster the hook hands the driver. */
              const handed = await lanesHooksFor({
                client: rig.client,
                signal: new AbortController().signal,
                entryId: 'mittens',
                l: capturingLogger({ messages: lines, },),
              },).beforeSlice({ lane: 'repair', },);
              returned.done = true;
              return handed;
            }

            /** The hook's seating, started on the fake clock. */
            const seating = seated();
            await clock.tickAsync(DRY_HOLD_MS - 1,);
            /** Whether the hook had returned a millisecond before the hold ends. */
            const returnedEarly = returned.done;
            /** Reads taken a millisecond before the hold ends. */
            const readsEarly = rig.counter.reads;
            await clock.tickAsync(1,);
            /** Roster the hook handed the driver. */
            const roster = await seating;

            expect(returnedEarly,).toBe(false,);
            expect(readsEarly,).toBe(1,);
            expect(returned.done,).toBe(true,);
            expect(rig.counter.reads,).toBe(2,);
            expect(roster.repairModels,).toBeDefined();
            expect(
              lines.some(function waitsTheWholeHold(line,): boolean {
                return line.includes(`waiting ${String(DRY_HOLD_MS,)}ms for the shortest hold to end`,);
              },),
            ).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
