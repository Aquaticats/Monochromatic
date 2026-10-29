/**
 Tests that the half of the short-bench reading that takes no wait seats what is reachable and says why: one line per
 state of the held providers and the unmet benches, the running holds it skipped, and the stop of an entry whose writing
 bench stands under its floor unless the phase goes on without its writers.

 Fixtures are cat-themed invention; every view and hold is scripted and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider';
import {
  type ProviderName,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  benchesOf,
  type JudgeSeatPhase,
  judgeSeatsFor,
  phaseBenches,
  seatWithoutWaiting,
  unreachableWritingBenches,
  type WarnedState,
  WritingBenchUnreachableError,
} from '../../dist/final/node/index.mjs';

//region No-wait reading tests

/**
 The XingZ6020 view: NeuralWatt, DeepSeek and Xiaomi wet, every other provider dry. The late bench of the lane
 contest reads 1 of 7.
 */
const SYNTHETIC_HELD: BudgetView = {
  neuralwatt: false,
  synthetic: true,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  anthropic: true,
  hyper: true,
  openrouter: true,
};

/**
 Bedrock alone wet: the lanes' editors and refiners read 0 of 3, under the floor of a pair.
 */
const BEDROCK_ALONE: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  hyper: true,
  openrouter: true,
};

/**
 Every provider dry.
 */
const ALL_DRY: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  hyper: true,
  openrouter: true,
};

/**
 Holds of none of the providers.
 */
const NO_HOLDS = providerRecord({ of: (): number => 0, },);

/**
 What one call wrote and returned.
 */
type Called = {
  readonly lines: readonly string[];
  readonly waitMs: number;
  readonly unreachableWriters: readonly string[];
};

/**
 Calls the no-wait half the way the short-bench reading does, deriving the benches and the writing benches below the
 floor from the view.

 @param phase - phase the reading is for

 @param view - dryness as first read

 @param running - providers held, none by default

 @param goesOn - whether the phase goes on without its writers

 @param warned - memory of the line last written, absent for a caller that reads once

 @param lines - receives what the call logs

 @returns What the call returned, with the lines it wrote

 @example
 ```ts
 const called = callOf({ phase: 'lane contest', view: SYNTHETIC_HELD, goesOn: false, lines: [], },);
 ```
 */
function callOf(
  {
    phase,
    view,
    running = NO_HOLDS,
    goesOn,
    warned,
    lines,
  }: {
    readonly phase: JudgeSeatPhase;
    readonly view: BudgetView;
    readonly running?: Record<ProviderName, number>;
    readonly goesOn: boolean;
    readonly warned?: WarnedState;
    readonly lines: string[];
  },
): Called {
  /**
   Benches as first read.
   */
  const benches = benchesOf({ seats: judgeSeatsFor({ dry: view, }), },);
  /**
   Benches the phase leans on.
   */
  const names = phaseBenches({ phase, },);
  const { waitMs, unreachableWriters, } = seatWithoutWaiting({
    client: {
      providerDryness: () => Promise.resolve(view,),
      providerHolds: () => running,
      providerMeterStates: () => Promise.reject(new Error('the no-wait half reads no meter',),),
    },
    phase,
    l: levelCapturingLogger({ lines, },),
    ...((warned === undefined) ? {} : { warned, }),
    dry: view,
    running,
    holds: NO_HOLDS,
    meterDry: new Set(),
    names,
    benches,
    unreachable: unreachableWritingBenches({
      benches,
      names,
      dry: view,
    },),
    goesOn,
  },);
  return {
    lines,
    waitMs,
    unreachableWriters,
  };
}

await describe({
  name: `${seatWithoutWaiting.name}: the reading that takes no wait`,
  concurrency: 1,
  children: [
    it({
      name: 'SEATS WHAT IS REACHABLE AND SAYS WHY FOR A SHORT JUDGE BENCH: no wait, no writer left out, and one '
        + 'warning with the clause, the cure, the floor and the words that no return was named',
      fn: async () => {
        const { lines, waitMs, unreachableWriters, } = callOf({
          phase: 'lane contest',
          view: SYNTHETIC_HELD,
          goesOn: false,
          lines: [],
        },);
        expect(waitMs,).toBe(0,);
        expect(unreachableWriters,).toEqual([],);
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain(
          'warn JUDGE SEATS phase=lane contest short of quorum: late 1 of 7 reachable, quorum 4',
        );
        expect(lines[0],).toContain('late reads under its floor of 2 and no hold runs that could lift it',);
        expect(lines[0],).toContain(
          'no provider has named a return the pass can wait for, so the phase runs on what is reachable',
        );
      },
    },),

    it({
      name: 'SAYS THE SAME STATE ONCE FOR A CALLER THAT KEEPS ITS MEMORY, and again once the memory is forgotten, '
        + 'while a caller that keeps none says it every time',
      fn: async () => {
        const lines: string[] = [];
        const warned: WarnedState = { signature: '', };
        callOf({
          phase: 'lane contest',
          view: SYNTHETIC_HELD,
          goesOn: false,
          warned,
          lines,
        },);
        expect(lines.length,).toBe(1,);
        expect(warned.signature,).not.toBe('',);
        callOf({
          phase: 'lane contest',
          view: SYNTHETIC_HELD,
          goesOn: false,
          warned,
          lines,
        },);
        expect(lines.length,).toBe(1,);
        warned.signature = '';
        callOf({
          phase: 'lane contest',
          view: SYNTHETIC_HELD,
          goesOn: false,
          warned,
          lines,
        },);
        expect(lines.length,).toBe(2,);
        callOf({
          phase: 'lane contest',
          view: SYNTHETIC_HELD,
          goesOn: false,
          lines,
        },);
        callOf({
          phase: 'lane contest',
          view: SYNTHETIC_HELD,
          goesOn: false,
          lines,
        },);
        expect(lines.length,).toBe(4,);
      },
    },),

    it({
      name: 'NAMES THE RUNNING HOLDS IT DID NOT WAIT FOR: Xiaomi held and its return lifting no bench, so the line '
        + 'says it would lift no short bench over its floor or to its quorum, with the ceiling of each bench',
      fn: async () => {
        const { lines, } = callOf({
          phase: 'lane contest',
          view: SYNTHETIC_HELD,
          running: providerRecord({ of: (provider,) => (provider === 'xiaomi') ? 40 : 0, },),
          goesOn: false,
          lines: [],
        },);
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain(
          'late ceiling 1 of 7 with xiaomi back, floor 2 and quorum 4, so no return lifts it',
        );
        expect(lines[0],).toContain('held: xiaomi (',);
        expect(lines[0],).toContain('it would lift no short bench over its floor or to its quorum',);
      },
    },),

    it({
      name: 'STOPS THE ENTRY FOR A WRITING BENCH UNDER ITS FLOOR with no wait to take: one error line names the '
        + 'benches and says the entry stops, and the refusal carries the same clauses',
      fn: async () => {
        const lines: string[] = [];
        let refusal: unknown;
        try {
          callOf({
            phase: 'lanes',
            view: BEDROCK_ALONE,
            goesOn: false,
            lines,
          },);
        } catch (error) {
          refusal = error;
        }
        expect(refusal,).toBeInstanceOf(WritingBenchUnreachableError,);
        expect(lines.filter(function isError(line,): boolean {
          return line.startsWith(
            'error JUDGE SEATS phase=lanes writing bench unreachable: editors 0 of 4 reachable, floor 2',
          )
            && line.endsWith('stopping the entry',);
        },).length,).toBe(1,);
      },
    },),

    it({
      name: 'GOES ON WITHOUT THE WRITERS IN A PHASE THAT DOES, handing back the benches below the floor, saying so '
        + 'once for a caller that keeps its memory and never saying the entry stops',
      fn: async () => {
        const lines: string[] = [];
        const warned: WarnedState = { signature: '', };
        const first = callOf({
          phase: 'page consistency',
          view: ALL_DRY,
          goesOn: true,
          warned,
          lines,
        },);
        expect(first.unreachableWriters,).toEqual(['translators 0 of 8 reachable, floor 2',],);
        expect(first.waitMs,).toBe(0,);
        const errors = lines.filter(function isError(line,): boolean {
          return line.startsWith('error ',);
        },);
        expect(errors.length,).toBe(1,);
        expect(errors[0],).toContain('so the phase goes on without its writers',);
        expect(lines.some(function saysStop(line,): boolean {
          return line.includes('stopping the entry',);
        },),).toBe(false,);
        callOf({
          phase: 'page consistency',
          view: ALL_DRY,
          goesOn: true,
          warned,
          lines,
        },);
        expect(lines.filter(function isError(line,): boolean {
          return line.startsWith('error ',);
        },).length,).toBe(1,);
      },
    },),
  ],
},);

//endregion No-wait reading tests
