/**
 Tests the per-slice re-seating hook against the bound its client carries.

 The hook reads the seats again while a hold runs, and a NeuralWatt return counts as a hold only when it lies inside
 what the pass has left: a return past it is not a hold, so the hook costs nothing and hands the slice the seating it
 had. The client here answers its holds the way the budgets do, by honouring the bound it is asked with.

 Fixtures are cat-themed invention; every view is scripted and no HTTP is made.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider';
import type { JudgeSeatPhase, } from '@monochromatic-dev/translation-repair-run';
import {
  type ProviderName,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { reseatHookFor, } from '../../dist/final/node/index.mjs';
import { metersAllWet, } from '@monochromatic-dev/translation-repair-provider/ts/meter-states.test-fixture.ts';

/**
 Every provider wet.
 */
const ALL_WET: BudgetView = {
  neuralwatt: false,
  synthetic: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  hyper: false,
  openrouter: false,
};

/**
 The seating a slice runs on before any hold has run.
 */
const UNSEATED = { seated: 'roster', };

/**
 The seating a reading hands the slices.
 */
const RESEATED = { seated: 'read under a hold', };

/**
 The hook over a client whose NeuralWatt return is `returnInMs` away, answering holds by the bound it is asked with.

 @param returnInMs - milliseconds until NeuralWatt's day starts again

 @param passLeftMs - what the pass has left

 @returns Hook and the dryness reads counted

 @example
 ```ts
 const { hook, reads, } = hookBefore({ returnInMs: 700, passLeftMs: 600, },);
 ```
 */
function hookBefore(
  {
    returnInMs,
    passLeftMs,
  }: {
    readonly returnInMs: number;
    readonly passLeftMs: number;
  },
) {
  const reads = { count: 0, };
  const hook = reseatHookFor({
    client: {
      providerDryness: async () => {
        reads.count += 1;
        return ALL_WET;
      },
      providerMeterStates: metersAllWet,
      providerHolds: ({ returnBoundMs = Number.POSITIVE_INFINITY, }: { readonly returnBoundMs?: number; } = {},) => ({
        neuralwatt: (returnInMs <= returnBoundMs) ? returnInMs : 0,
        synthetic: 0,
        bedrock: 0,
        deepseek: 0,
        xiaomi: 0,
        hyper: 0,
        openrouter: 0,
      }),
      passLeftMs: () => passLeftMs,
    },
    signal: new AbortController().signal,
    phase: 'lane contest',
    unseated: UNSEATED,
    seatingOf: () => ({ seating: RESEATED, line: 'slice re-seated under a hold', }),
    l: tagged({ tag: 'pass-reseat-hook-test', },),
  },);
  return { hook, reads, };
}

/**
 NeuralWatt and Bedrock wet and every other provider dry: the wide bench is 3 of 7 reachable against a quorum of 4,
 which DeepSeek back leaves as it is.
 */
const WIDE_ONE_SHORT: BudgetView = {
  neuralwatt: false,
  synthetic: true,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  hyper: true,
  openrouter: true,
};

/**
 How much of the DeepSeek hold of the case is left at the n-th time the holds are asked, in milliseconds: it shrinks by
 a second with every ask, as a real hold does between slices, so the sentence naming its length is a new one each time.
 The reading never waits for it, since DeepSeek back leaves the bench as short as it is.

 @param asks - how many times the holds have been asked, this time included

 @returns Milliseconds left, never below one
 */
function leftAfter({ asks, }: { readonly asks: number; },): number {
  return Math.max(
    1,
    10_000_000 - (asks * 1_000),
  );
}

/**
 The hook over a short bench with a hold that cannot cure it, whose hold the case ends by hand and which shrinks with
 every ask.

 @param lines - receives what the readings log

 @returns Hook and the switch that ends or starts the DeepSeek hold

 @example
 ```ts
 const { hook, hold, } = hookUnderDeepSeekHold({ lines: [], },);
 ```
 */
function hookUnderDeepSeekHold({ lines, }: { readonly lines: string[]; },) {
  const hold = { running: true, asks: 0, };
  const hook = reseatHookFor({
    client: {
      providerDryness: async () => WIDE_ONE_SHORT,
      providerMeterStates: metersAllWet,
      providerHolds: () => {
        hold.asks += 1;
        return {
          neuralwatt: 0,
          synthetic: 0,
          bedrock: 0,
          deepseek: hold.running ? leftAfter({ asks: hold.asks, },) : 0,
          xiaomi: 0,
          hyper: 0,
          openrouter: 0,
        };
      },
    },
    signal: new AbortController().signal,
    phase: 'lane contest',
    unseated: UNSEATED,
    seatingOf: () => ({ seating: RESEATED, line: 'slice re-seated under a hold', }),
    l: levelCapturingLogger({ lines, },),
  },);
  return { hook, hold, };
}

/**
 The warnings among what a reading logged that say a bench was seated short without a wait.

 @param lines - what the readings logged

 @returns Warning lines

 @example
 ```ts
 const warnings = shortWarningsOf({ lines, },);
 ```
 */
function shortWarningsOf({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines.filter(function isShortWarning(line,): boolean {
    return line.startsWith('warn',) && line.includes('short of quorum',);
  },);
}

/**
 Bedrock the only wet provider: the consolidation's slate and wide benches are both short, and NeuralWatt reads dry.
 */
const SLATE_AND_WIDE_SHORT: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  hyper: true,
  openrouter: true,
};

/**
 Bedrock and Synthetic wet: only the consolidation's slate bench is short, and NeuralWatt reads dry.
 */
const SLATE_SHORT: BudgetView = { ...SLATE_AND_WIDE_SHORT, synthetic: false, };

/**
 DeepSeek and Xiaomi the only wet providers: the translators of the page consistency, a phase that goes on without its
 writers, stand below the floor of two.
 */
const WRITERS_BELOW_FLOOR: BudgetView = {
  ...SLATE_AND_WIDE_SHORT,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
};

/**
 The hook for a consolidation whose NeuralWatt reads dry on a spent day with a return named past what the pass has
 left, under stated holds that shrink with every ask: the day-end shape, where every figure in a warning's text moves
 from one slice to the next.

 @param lines - receives what the readings log

 @param phase - phase the hook re-seats for

 @returns Hook and the state the case moves: the view the readings find and the providers held

 @example
 ```ts
 const { hook, state, } = hookOverDayEnd({ lines: [], },);
 ```
 */
function hookOverDayEnd(
  {
    lines,
    phase = 'consolidation',
  }: {
    readonly lines: string[];
    readonly phase?: JudgeSeatPhase;
  },
) {
  const state: {
    view: BudgetView;
    held: readonly ProviderName[];
    asks: number;
    waitHold: boolean;
    waitReads: number;
  } = {
    view: SLATE_AND_WIDE_SHORT,
    held: [],
    asks: 0,
    waitHold: false,
    waitReads: 0,
  };
  const hook = reseatHookFor({
    client: {
      providerDryness: async () => {
        // A reading that waits for the short hold (5 ms) finds every bench back after the wait, and the hold ends.
        if (state.waitHold) {
          state.waitReads += 1;
          if (state.waitReads > 1) {
            state.waitHold = false;
            state.waitReads = 0;
            return ALL_WET;
          }
        }
        return state.view;
      },
      providerMeterStates: metersAllWet,
      providerHolds: () => {
        state.asks += 1;
        return providerRecord({
          of: function heldFor(provider,): number {
            if (state.waitHold && (provider === 'neuralwatt'))
              return 5;
            return state.held.includes(provider,) ? leftAfter({ asks: state.asks, },) : 0;
          },
        },);
      },
      neuralwattDryness: () => ({
        kind: 'spent-day',
        dayStart: '2026-10-22T19:15:08.000Z',
        returnsInMs: Math.max(
          1,
          18_000_000 - (state.asks * 1_000),
        ),
      }),
      passLeftMs: () => Math.max(
        1,
        3_600_000 - (state.asks * 1_000),
      ),
    },
    signal: new AbortController().signal,
    phase,
    unseated: UNSEATED,
    seatingOf: () => ({ seating: RESEATED, line: 'slice re-seated under a hold', }),
    l: levelCapturingLogger({ lines, },),
  },);
  return { hook, state, };
}

/**
 Runs slices one after another through the hook.

 @param hook - hook under test

 @param count - how many slices

 @example
 ```ts
 await runSlices({ hook, count: 40, },);
 ```
 */
async function runSlices(
  {
    hook,
    count,
  }: {
    readonly hook: () => Promise<unknown>;
    readonly count: number;
  },
): Promise<void> {
  await Array.from({ length: count, },).reduce(async function nextSlice(previous: Promise<unknown>,): Promise<unknown> {
    await previous;
    return await hook();
  }, Promise.resolve(),);
}

await describe({
  name: reseatHookFor.name,
  concurrency: 1,
  children: [
    it({
      name: 'READS NOTHING FOR A RETURN PAST WHAT THE PASS HAS LEFT: 700 away with 600 left is no hold, so the slice '
        + 'keeps the seating it had and no dryness is read',
      fn: async () => {
        const { hook, reads, } = hookBefore({ returnInMs: 700, passLeftMs: 600, },);
        expect(await hook(),).toEqual(UNSEATED,);
        expect(reads.count,).toBe(0,);
      },
    },),

    it({
      name: 'READS THE SEATS FOR A RETURN INSIDE WHAT THE PASS HAS LEFT: 500 away with 600 left is a hold, so the '
        + 'slice takes the seating the reading gives',
      fn: async () => {
        const { hook, reads, } = hookBefore({ returnInMs: 500, passLeftMs: 600, },);
        expect(await hook(),).toEqual(RESEATED,);
        expect(reads.count,).toBe(1,);
      },
    },),

    it({
      name: 'READS NOTHING WHEN THE BOUND IS ZERO OR BELOW, the pass or the entry\'s cap spent, whatever '
        + 'return is named',
      fn: async () => {
        const { hook, reads, } = hookBefore({ returnInMs: 5, passLeftMs: -1, },);
        expect(await hook(),).toEqual(UNSEATED,);
        expect(reads.count,).toBe(0,);
      },
    },),

    it({
      name: 'WRITES ONE WARNING FOR A HOLD THAT CANNOT CURE THE SHORT BENCH, however many slices run under it: a '
        + 'DeepSeek hold over five slices says once that the bench is seated short, and says it again for a '
        + 'new hold after the first has ended',
      fn: async () => {
        const lines: string[] = [];
        const { hook, hold, } = hookUnderDeepSeekHold({ lines, },);
        expect(await hook(),).toEqual(RESEATED,);
        expect(await hook(),).toEqual(RESEATED,);
        expect(await hook(),).toEqual(RESEATED,);
        expect(await hook(),).toEqual(RESEATED,);
        expect(await hook(),).toEqual(RESEATED,);
        expect(shortWarningsOf({ lines, },),).toHaveLength(1,);

        // The hold ends, a slice runs free of it, and a later hold is a new one.
        hold.running = false;
        await hook();
        hold.running = true;
        await hook();
        await hook();
        expect(shortWarningsOf({ lines, },),).toHaveLength(2,);
      },
    },),

    it({
      name: 'KEYS THE ONE WARNING ON WHAT IT IS ABOUT, NOT ON ITS TEXT: with NeuralWatt dry on a spent day whose '
        + 'return is named past the pass, and every figure in the text shrinking from slice to slice, 40 slices under '
        + 'a Xiaomi hold write one warning; a change of the held providers, a change of the short benches, and the '
        + 'holds ending and coming back each write one more',
      fn: async () => {
        const lines: string[] = [];
        const { hook, state, } = hookOverDayEnd({ lines, },);
        state.held = ['xiaomi',];
        await runSlices({ hook, count: 40, },);
        expect(shortWarningsOf({ lines, },),).toHaveLength(1,);

        // A second provider held: new signature.
        state.held = ['xiaomi', 'deepseek',];
        await runSlices({ hook, count: 5, },);
        expect(shortWarningsOf({ lines, },),).toHaveLength(2,);

        // Back to the first set: another change.
        state.held = ['xiaomi',];
        await runSlices({ hook, count: 5, },);
        expect(shortWarningsOf({ lines, },),).toHaveLength(3,);

        // The same hold, fewer short benches (the slate alone): another change.
        state.view = SLATE_SHORT;
        await runSlices({ hook, count: 5, },);
        expect(shortWarningsOf({ lines, },),).toHaveLength(4,);

        // The hold ends, a slice runs free, and the same hold comes back: said again.
        state.held = [];
        await runSlices({ hook, count: 1, },);
        state.held = ['xiaomi',];
        await runSlices({ hook, count: 5, },);
        expect(shortWarningsOf({ lines, },),).toHaveLength(5,);
      },
    },),

    it({
      name: 'SAYS A SHORT BENCH AGAIN AFTER A CLEAN READING BETWEEN: under one unchanged Xiaomi hold the bench is '
        + 'short and warned once, every bench is back for five slices, and the bench falling short again writes one '
        + 'more',
      fn: async () => {
        const lines: string[] = [];
        const { hook, state, } = hookOverDayEnd({ lines, },);
        state.held = ['xiaomi',];
        await runSlices({ hook, count: 5, },);
        expect(shortWarningsOf({ lines, },),).toHaveLength(1,);

        state.view = ALL_WET;
        await runSlices({ hook, count: 5, },);
        expect(shortWarningsOf({ lines, },),).toHaveLength(1,);

        state.view = SLATE_AND_WIDE_SHORT;
        await runSlices({ hook, count: 5, },);
        expect(shortWarningsOf({ lines, },),).toHaveLength(2,);
      },
    },),

    it({
      name: 'SAYS A SHORT BENCH AGAIN AFTER A READING THAT WAITED: the same Xiaomi hold and the same short benches are '
        + 'warned once, a reading that waits 5 ms for a NeuralWatt hold finds them back, and the next reading finding '
        + 'them short again writes one more',
      fn: async () => {
        const lines: string[] = [];
        const { hook, state, } = hookOverDayEnd({ lines, },);
        state.view = SLATE_SHORT;
        state.held = ['xiaomi',];
        await runSlices({ hook, count: 3, },);
        expect(shortWarningsOf({ lines, },),).toHaveLength(1,);

        state.waitHold = true;
        await runSlices({ hook, count: 1, },);
        expect(state.waitHold,).toBe(false,);
        const afterWait = shortWarningsOf({ lines, },).length;

        await runSlices({ hook, count: 3, },);
        expect(shortWarningsOf({ lines, },).length - afterWait,).toBe(1,);
      },
    },),

    it({
      name: 'WRITES THE ERROR LINE OF A PHASE THAT GOES ON WITHOUT ITS WRITERS ONCE PER STATE: the page consistency, '
        + 'with its translators below the floor under an unchanged hold, names the bench once over five slices',
      fn: async () => {
        const lines: string[] = [];
        const { hook, state, } = hookOverDayEnd({ lines, phase: 'page consistency', },);
        state.view = WRITERS_BELOW_FLOOR;
        state.held = ['xiaomi',];
        await runSlices({ hook, count: 5, },);
        /**
         The error lines naming the bench below its floor.
         */
        const errors = lines.filter(function isBenchError(line,): boolean {
          return line.startsWith('error',) && line.includes('writing bench unreachable',);
        },);
        expect(errors,).toHaveLength(1,);
        expect(errors[0],).toContain('the phase goes on without its writers',);
      },
    },),
  ],
},);
