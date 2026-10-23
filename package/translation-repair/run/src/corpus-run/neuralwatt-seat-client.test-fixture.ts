//region NeuralWatt seat client
// TEST SUPPORT, NOT PACKAGE SOURCE. The seat reader's client over the real budgets with NeuralWatt dry for a spent day,
// shared by the cases that read seats against the XingZ6019 shape.

import {
  createProviderBudgets,
  type NeuralWattEnergy,
} from '@monochromatic-dev/translation-repair-provider';
import type { RunClient, } from '../../dist/final/node/index.mjs';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';

/**
 Abort signal every read here carries.
 */
export const SIGNAL: AbortSignal = new AbortController().signal;

/**
 Milliseconds after 00:00 UTC the account's day starts at: 19:15:08 UTC.
 */
const DAY_START_MS_OF_DAY = 69_308_000;

/**
 The day whose energy is spent.
 */
export const SPENT_DAY = {
  dayStart: '2026-10-22T19:15:08.000Z',
  dailyWh: 550,
  spentWh: 550,
  reckonedWh: 0,
  reservedWh: 0,
  remainingWh: 0,
  calls: 40,
} as const;

/**
 The day that began at the return.
 */
const FRESH_DAY = {
  dayStart: '2026-10-23T19:15:08.000Z',
  dailyWh: 550,
  spentWh: 0,
  reckonedWh: 0,
  reservedWh: 0,
  remainingWh: 550,
  calls: 0,
} as const;

/**
 The year the stubbed wall clock reads.
 */
const WALL_YEAR = 2_026;

/**
 The month the stubbed wall clock reads: October, zero-based.
 */
const WALL_MONTH_INDEX = 9;

/**
 The day of month the stubbed wall clock reads.
 */
const WALL_DAY = 23;

/**
 Percent of Synthetic's week left when the case has it wet.
 */
const SYNTHETIC_WET_PERCENT = 50;

/**
 The anchor of the account's day, which is known.

 @returns Day start as a time of day

 @example
 ```ts
 const anchor = dayAnchor();
 ```
 */
function dayAnchor(): {
  readonly known: true;
  readonly msOfDay: number
} {
  return {
    known: true,
    msOfDay: DAY_START_MS_OF_DAY,
  };
}

/**
 What the seat reader takes of its client, and the energy reads counted.
 */
type NeuralWattSeatClient = {
  readonly reads: { count: number; };
  readonly client: Pick<
    RunClient,
    | 'neuralwattDryness'
      | 'neuralwattReturn'
      | 'passBoundOwner'
      | 'passLeftMs'
      | 'providerDryness'
      | 'providerHolds'
      | 'providerMeterStates'
    >;
};

/**
 The seat reader's client over the real budgets, with NeuralWatt dry for the spent day and
 Synthetic the only wet provider.

 @param msBeforeReturn - how long before NeuralWatt's day starts again the wall clock reads

 @param spent - first NeuralWatt reading, the spent day unless the case says otherwise

 @param syntheticWet - whether Synthetic, the only other configured provider, has budget

 @param passLeftMs - what the pass has left, which bounds the wait for a return; absent for a pass that bounds nothing

 @param boundOwner - whose bound passLeftMs is, the entry's own cap or the pass (default: the pass)

 @param lagging - whether the account still reads the spent day after the return (default: the new day shows)

 @param now - monotonic clock the budgets count a return's distance on, a frozen one for a case that waits for
 nothing (default: the real clock, for a case that sleeps)

 @returns Client the reader takes, and the energy reads counted

 @example
 ```ts
 const { client, } = clientBefore({ msBeforeReturn: 60, },);
 ```
 */
export function clientBefore(
  {
    msBeforeReturn,
    spent = SPENT_DAY,
    syntheticWet = true,
    lagging = false,
    passLeftMs,
    boundOwner,
    now,
  }: {
    readonly msBeforeReturn: number;
    readonly spent?: NeuralWattEnergy;
    readonly syntheticWet?: boolean;
    readonly lagging?: boolean;
    readonly passLeftMs?: number;
    readonly boundOwner?: 'entry' | 'pass';
    readonly now?: () => number;
  },
): NeuralWattSeatClient {
  /**
   Energy reads made so far.
   */
  const reads = { count: 0, };
  /**
   The real budgets over the stub meters.
   */
  const budgets = createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    neuralwatt: {
      energy: function energy(): Promise<NeuralWattEnergy> {
        reads.count += 1;
        return Promise.resolve(((reads.count === 1) || lagging) ? spent : FRESH_DAY,);
      },
      dayAnchor,
    },
    synthetic: {
      quotas: function quotas() {
        return Promise.resolve({
          weekly: {
            percentRemaining: syntheticWet ? SYNTHETIC_WET_PERCENT : 0,
            nextRegenAt: '2026-10-25T00:00:00.000Z',
          },
        },);
      },
    },
    wallNow: function wallNow() {
      return new Date(Date.UTC(
        WALL_YEAR,
        WALL_MONTH_INDEX,
        WALL_DAY,
        0,
        0,
        0,
        DAY_START_MS_OF_DAY - msBeforeReturn,
      ),);
    },
    returnMarginMs: 0,
    // Conditional spread keeps the real clock where the case sleeps for real.
    ...((now === undefined) ? {} : { now, }),
  },);
  return {
    reads,
    client: {
      providerDryness: function providerDryness() {
        return budgets.read({ signal: SIGNAL, },);
      },
      providerMeterStates: function providerMeterStates() {
        return budgets.readStates({ signal: SIGNAL, },);
      },
      providerHolds: budgets.holds,
      neuralwattReturn: budgets.neuralwattReturn,
      neuralwattDryness: budgets.neuralwattDryness,
      // Conditional spread keeps the client without a pass bound where the case names none.
      ...((passLeftMs === undefined)
        ? {}
        : {
          passLeftMs: function passLeft() {
            return passLeftMs;
          },
        }),
      // Conditional spread keeps the client without an owner where the case names none.
      ...((boundOwner === undefined)
        ? {}
        : {
          passBoundOwner: function passBoundOwner() {
            return boundOwner;
          },
        }),
    },
  };
}

//endregion NeuralWatt seat client
