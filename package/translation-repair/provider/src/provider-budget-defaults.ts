import { tagged, } from '@monochromatic-dev/module-logger/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { MeterState, } from './provider-meters.ts';

//region Provider budget starting values
// SPLIT OUT OF `provider-budget.ts` at its line budget: the values a budgets instance holds before any meter has
// answered, the clock it reads by default, and the error for a hold bound that cannot bound a hold.

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
 The end of a refusal hold before any refusal: never.

 @returns Zero, the stamp for no hold

 @example
 ```ts
 const heldUntil = providerRecord({ of: holdEndsNever, },);
 ```
 */
export function holdEndsNever(): number {
  return 0;
}

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
