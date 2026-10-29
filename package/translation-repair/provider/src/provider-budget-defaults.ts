import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { NeuralWattReturn, } from './budget-routing.ts';
import type { NeuralWattClient, } from './neuralwatt-client.ts';
import type { MeterState, } from './provider-meters.ts';

//region Provider budget starting values
// SPLIT OUT OF `provider-budget.ts` at its line budget: the values a budgets instance holds before any meter has
// answered, the clock it reads by default, the error for a hold bound that cannot bound a hold, and what the budgets
// know of NeuralWatt's day without asking its client to decide it.

/**
 The reading before any meter has answered: nothing is dry.

 @returns Spendable

 @example
 ```ts
 const view = providerRecord({ of: spendableBeforeReading, },);
 ```
 */
export function spendableBeforeReading(): boolean {
  return false;
}

/**
 The level before any meter has answered: no number at all.

 @returns Empty

 @example
 ```ts
 const levels = providerRecord({ of: noLevelBeforeReading, },);
 ```
 */
export function noLevelBeforeReading(): string {
  return '';
}

/**
 Whether a provider last refused us for payment, and the meter level it
 read then.

 @example
 ```ts
 const mark: PaymentMark = { marked: true, level: 'openrouterUsd=0.01', };
 ```
 */
export type PaymentMark = {
  marked: boolean;
  level: string;
};

/**
 The mark before any provider has refused us for payment: none.

 @returns Unmarked

 @example
 ```ts
 const marks = providerRecord({ of: unmarkedPayment, },);
 ```
 */
export function unmarkedPayment(): PaymentMark {
  return {
    marked: false,
    level: '',
  };
}

/**
 The state before any meter has answered: nothing was read.

 @returns Unreadable

 @example
 ```ts
 const states = providerRecord({ of: unreadBeforeReading, },);
 ```
 */
export function unreadBeforeReading(): MeterState {
  return 'unreadable';
}

/**
 The hold of a run that holds no provider out for a reason its meter cannot show.

 @returns Not held

 @example
 ```ts
 const held = notHeldOut();
 ```
 */
export function notHeldOut(): boolean {
  return false;
}

/**
 The reading cache of a budgets instance, mutated as readings start.
 */
export type ReadingCache = {
  startedAt: number;
  inFlight: boolean;
  reading: Promise<{
    readonly view: ProviderRecord<boolean>;
    readonly states: ProviderRecord<MeterState>;
    readonly levels: ProviderRecord<string>;
  }>;
};

/**
 The cache of a budgets instance before any meter has answered: the last meter reading and when it was taken, stamped
 infinitely long ago until the first.

 THE PRE-READ VIEW IS NOT A PLACEHOLDER LIE. Before any meter has answered, nothing is known about any budget, and the
 policy for an unknown budget is already that it counts as spendable. A stamp infinitely long ago is stale on any clock,
 so the first call reads every meter regardless.

 NOT ZERO. The stamp was zero for never, which an epoch clock never reads; `monotonicMs` counts from near the process's
 start and reads zero in its first millisecond, so a reading taken then passed for no reading and every meter was read
 again inside the window (ledger B78). Nor can zero stand as an ordinary stamp: on this clock it is the process's start,
 so every call in the first `freshForMs` would take the pre-read view as fresh and read no meter, which failed nine of
 the budget layer's cases when tried.

 @returns Cache stamped infinitely long ago, with nothing in flight and the reading nothing is known from

 @example
 ```ts
 const cache = beforeAnyReading();
 ```
 */
export function beforeAnyReading(): ReadingCache {
  return {
    startedAt: Number.NEGATIVE_INFINITY,
    inFlight: false,
    reading: Promise.resolve({
      view: providerRecord({ of: spendableBeforeReading, },),
      states: providerRecord({ of: unreadBeforeReading, },),
      levels: providerRecord({ of: noLevelBeforeReading, },),
    },),
  };
}

/**
 Each provider's meter state in one reading.

 @param meters - every meter's record, keyed by provider

 @returns State of each meter

 @example
 ```ts
 const states = meterStatesOf({ meters, },);
 ```
 */
export function meterStatesOf(
  { meters, }: { readonly meters: ProviderRecord<{ readonly state: MeterState; }>; },
): ProviderRecord<MeterState> {
  return providerRecord({
    of: function stateOf(provider,): MeterState {
      /**
       This provider's meter record.
       */
      const meter = meters[provider];
      return meter.state;
    },
  },);
}

/**
 Each provider's meter level in one reading, the numbers it reported joined by spaces: what a payment refusal compares a
 later reading against.

 @param meters - every meter's record, keyed by provider

 @returns Joined numbers of each meter

 @example
 ```ts
 const levels = meterLevelsOf({ meters, },);
 ```
 */
export function meterLevelsOf(
  { meters, }: { readonly meters: ProviderRecord<{ readonly fields: readonly string[]; }>; },
): ProviderRecord<string> {
  return providerRecord({
    of: function levelOf(provider,): string {
      /**
       This provider's meter record.
       */
      const meter = meters[provider];
      return meter.fields
        .join(' ',);
    },
  },);
}

/**
 What the run knows of NeuralWatt's day, without asking the client to decide it.

 @param neuralwatt - NeuralWatt meter, absent where the run has no NeuralWatt key

 @returns Off without a meter, unanchored where the client has decided no day start, else the day start as a time of day

 @example
 ```ts
 const day = neuralwattReturnOf({ neuralwatt, },);
 ```
 */
export function neuralwattReturnOf(
  { neuralwatt, }: {
    readonly neuralwatt?: Pick<NeuralWattClient, 'energy'> & { readonly dayAnchor?: NeuralWattClient['dayAnchor']; };
  },
): NeuralWattReturn {
  if (neuralwatt === undefined)
    return { kind: 'off', };
  // A stub client that carries no anchor decided none.
  /**
   What the client has decided, without asking it to decide.
   */
  const anchor = neuralwatt.dayAnchor?.() ?? { known: false, };
  if (!anchor.known)
    return { kind: 'unanchored', };
  return {
    kind: 'at',
    msOfDay: anchor.msOfDay,
  };
}

/**
 Raised when the longest hold a budgets instance may state is not a finite whole number of
 milliseconds greater than zero.

 CHECKED AT RUN TIME, where the budgets are built, since the type says `number` and so admits `Infinity`, which brings
 back the unbounded hold the bound exists to end, and `NaN`, which made the cut return `NaN`. The value is quoted back
 because it is the thing to fix.

 @example
 ```ts
 throw new HoldBoundError({ maxHoldMs: Number.POSITIVE_INFINITY, },);
 ```
 */
export class HoldBoundError extends Error {
  /**
   Declares this message safe to print whole: the name of the setting and the number the caller gave it.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the setting and quotes the number it was given.

   @param maxHoldMs - value the caller passed, whatever number it was

   @example
   ```ts
   new HoldBoundError({ maxHoldMs: 0, },);
   ```
   */
  public constructor({ maxHoldMs, }: { readonly maxHoldMs: number; },) {
    super(
      `maxHoldMs must be a finite whole number of milliseconds greater than zero, received ${String(maxHoldMs,)}`,
    );
    this.name = 'HoldBoundError';
  }
}

/**
 The system clock, the wall clock outside a test.

 @returns Current instant

 @example
 ```ts
 const at = systemClock();
 ```
 */
export function systemClock(): Date {
  return new Date();
}

/**
 A wait a refusal stated for itself, and the provider that stated it.

 @example
 ```ts
 const stated: StatedWait = { provider: 'hyper', statedWaitMs: 3_000, };
 ```
 */
export type StatedWait = {
  readonly provider: ProviderName;
  readonly statedWaitMs: number;
};

/**
 Logger root for the budget layer.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Builds the cut that keeps a wait a refusal states for itself inside the longest hold a run can use.

 @param maxHoldMs - longest hold a wait may be cut to

 @returns Cut, which returns the stated wait or `maxHoldMs` where it is longer, and logs the first cut of each provider

 @example
 ```ts
 const cappedWait = createWaitCap({ maxHoldMs: 25_200_000, },);
 ```
 */
export function createWaitCap(
  { maxHoldMs, }: { readonly maxHoldMs: number; },
): (args: StatedWait,) => number {
  /**
   Providers whose stated wait was cut to `maxHoldMs` and logged, so the cut is written once for each provider and not
   on every refusal that repeats it.
   */
  const cutLogged = new Set<ProviderName>();

  /**
   Cuts a wait a refusal states for itself to the longest hold the run can use.

   @param provider - provider that stated the wait

   @param statedWaitMs - wait the refusal stated, in milliseconds

   @returns Stated wait, or `maxHoldMs` where it is longer
   */
  return function cappedWait(
    {
      provider,
      statedWaitMs,
    }: StatedWait,
  ): number {
    if (statedWaitMs <= maxHoldMs)
      return statedWaitMs;
    if (!cutLogged.has(provider,)) {
      cutLogged.add(provider,);
      /**
       Logger pre-tagged with this function's name.
       */
      const rl = tagged({
        tag: cappedWait.name,
        l,
      },);
      rl.warn(
        `${provider}: refusal stated a wait of ${String(statedWaitMs,)}ms, capped at ${String(maxHoldMs,)}ms, `
          + 'the longest hold this run can use',
      );
    }
    return maxHoldMs;
  };
}

//endregion Provider budget starting values
