import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { NeuralWattDryness, } from './neuralwatt-return-hold.ts';

//region NeuralWatt return mark
// WHY NEURALWATT READS DRY AND WHEN A SPENT DAY STARTS AGAIN, on the budget layer's monotonic clock, kept beside the
// cached meter reading that named them. A reading replaces the mark, so a return never outlives the reading that named
// it, and past it the budget layer reads the meters again whatever the reading's age (`provider-budget.ts`). The mark
// WAITS FOR NOTHING (the owner's answer of 2026-10-29, "Never wait, relaunch"): it counts the distance down for the
// refusal that states it, and tells the budget layer when its cached dry reading is stale.

/**
 Logger root for the return mark.
 */
const root = contextRoot({ tag: 'translation-repair', },);

/**
 Milliseconds before the return at which it counts as passed.

 IT GUARDS THE CACHED READING, NOT A TIMER. Nothing arms a timer for the return (the owner's answer of 2026-10-29,
 "Never wait, relaunch"); the budget layer asks {@link NeuralWattReturnMark.passed} on each call that would otherwise be
 served the cached dry reading (`provider-budget.ts`, `currentReading`). A reading taken a hair before the boundary
 names a return that is a hair away, and without this margin the next call inside that hair would be served the dry
 reading that named it. With it the call reads the meters again, which costs one reading and replaces the mark.
 Past the boundary the same call reads again either way, so the margin only moves the re-read a hair earlier.

 THE VALUE IS THE ONE THE TIMER MEASURE GAVE, kept as a small margin and not re-derived for the cache: 400 timers
 requested at 7.4 ms fired early in 396 cases by up to 1.3 ms, because Node truncates fractional milliseconds
 (the review of 2026-10-22, keyless scratch measures), and two milliseconds covered the worst measured.
 */
const PASSED_SLACK_MS = 2;

/**
 The return the last meter reading named: none, or the instant on the monotonic clock the day starts again.

 @example
 ```ts
 const mark: ReturnMark = { kind: 'at', at: 49_000, dayStart: '2026-10-22T19:15:08.000Z', };
 ```
 */
type ReturnMark =
  | {
    /**
     The reading named a return.
     */
    readonly kind: 'at';

    /**
     Instant on the monotonic clock the day starts again.
     */
    readonly at: number;

    /**
     First instant of the spent day, which tells one return from the next.
     */
    readonly dayStart: string;
  }
    | {
    /**
     The reading named none.
     */
      readonly kind: 'none';
    };

/**
 The return of NeuralWatt's spent day, as the last reading named it.

 @example
 ```ts
 const mark: NeuralWattReturnMark = createNeuralWattReturnMark({ now: monotonicMs, },);
 ```
 */
export type NeuralWattReturnMark = {
  /**
   Replaces the mark with what a meter reading found, logging the wait when it names a return
   that is not the last one's.

   A SPENT DAY THAT THE LAST READING NAMED AND THAT STILL READS SPENT AFTER ITS RETURN HAS PASSED is the account
   lagging, not a new day: it names no return, so the waits do not wait a day for energy the account has not yet shown.

   @param dryness - why the reading is dry, or that it is not
   */
  readonly replace: (args: { readonly dryness: NeuralWattDryness; },) => void;

  /**
   Why NeuralWatt reads dry as the last reading found it, a spent day with the distance counted down on the clock.
   */
  readonly dryness: () => NeuralWattDryness;

  /**
   Whether the day has started again since the reading that named the return, so the budget layer reads the meters
   again whatever the age of the cached reading.
   */
  readonly passed: () => boolean;

  /**
   Milliseconds left until the day starts again, zero where no return is named or it has passed.
   */
  readonly leftMs: () => number;
};

/**
 Builds the mark of one budget layer.

 @param now - monotonic clock the budget layer reads

 @param l - logger the line is written to; the package's root by default

 @returns Mark that names no return until a reading names one

 @example
 ```ts
 const returns = createNeuralWattReturnMark({ now, },);
 ```
 */
export function createNeuralWattReturnMark(
  {
    now,
    l = root,
  }: {
    readonly now: () => number;
    readonly l?: Logger;
  },
): NeuralWattReturnMark {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: createNeuralWattReturnMark.name,
    l,
  },);

  /**
   The return the last reading named, and why the last reading was dry.
   */
  const held: {
    mark: ReturnMark;
    dryness: NeuralWattDryness
  } = {
    mark: { kind: 'none', },
    dryness: { kind: 'wet', },
  };

  /**
   Milliseconds left until the day starts again, zero where no return is named or it has passed.

   @returns Milliseconds left
   */
  function leftMs(): number {
    return (held.mark
      .kind
      === 'at')
      ? Math.max(
        0,
        held.mark
          .at
          - now(),
      )
      : 0;
  }

  /**
   Whether the day has started again since the reading that named the return.

   @returns Whether the return has passed, counting the last hair before it as passed
   */
  function passed(): boolean {
    return (held.mark
      .kind
      === 'at') && ((now() + PASSED_SLACK_MS)
        >= held.mark
          .at);
  }

  return {
    replace: function replace({ dryness, },): void {
      /**
       The mark before this reading.
       */
      const before = held.mark;
      /**
       Whether this reading names the same spent day as the last one, whose return has passed.
       */
      const lagging = (dryness.kind === 'spent-day') && (before.kind === 'at')
        && (before.dayStart === dryness.dayStart)
        && passed();
      if ((dryness.kind !== 'spent-day') || (dryness.returnsInMs === undefined)
        || lagging) {
        held.mark = { kind: 'none', };
        held.dryness = (lagging && (dryness.kind === 'spent-day'))
          ? {
            kind: 'spent-day',
            dayStart: dryness.dayStart,
          }
          : dryness;
        return;
      }
      held.mark = {
        kind: 'at',
        at: now() + dryness.returnsInMs,
        dayStart: dryness.dayStart,
      };
      held.dryness = dryness;
      if ((before.kind === 'at') && (before.dayStart === dryness.dayStart))
        return;
      rl.info(
        `neuralwatt: the day's energy is spent and its day starts again in ${String(dryness.returnsInMs,)}ms; `
          + 'held out until then, and read again then',
      );
    },

    dryness: function dryness(): NeuralWattDryness {
      return (held.dryness
        .kind
        === 'spent-day') && (held.mark
          .kind
          === 'at')
        ? {
          kind: 'spent-day',
          dayStart: held.dryness
            .dayStart,
          returnsInMs: leftMs(),
        }
        : held.dryness;
    },

    passed,

    leftMs,
  };
}

//endregion NeuralWatt return mark
