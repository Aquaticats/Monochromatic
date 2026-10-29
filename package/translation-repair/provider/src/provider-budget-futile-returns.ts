import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Provider budget futile returns
// A RETURN THAT SERVED NOTHING, kept apart from the cooldown give-up in `provider-budget-refusal-holds.ts` and read by the
// seat reader before it waits for a hold (the owner's answer of 2026-10-29, "Remember futile returns"; measured on
// XingZ6020, where Synthetic refused again each time it came back and the waits for it bought nothing).
//
// WHAT IS A RETURN, AND WHAT SERVED NOTHING. A hold a refusal set ends at an instant: that end is the provider's return.
// The return served nothing when the provider's next refusal arrives after that end and no call it answered
// (`noteServed`, which the router reports when a provider's reply comes back) came after the end. The refusal is the
// first answer the provider gave after the wait, so a further wait for the next hold buys the same answer.
//
// THE SAME EVIDENCE AS THE GIVE-UP, A WIDER REACH, A WEAKER CONSEQUENCE. The give-up reads only a refusal cooldown (a
// hold the layer guessed alone) and takes the provider out of routing for the rest of the entry. This reads every hold a
// refusal sets (a cooldown, a rate-limit backoff, a wait the refusal states), takes nothing out of routing, and ends
// when the provider answers a call, so the provider that comes back is waited for again.
//
// A HOLD THAT HELD FOR NO TIME HAS NO RETURN: a refusal on a wet meter with no other wet provider sets a hold of zero
// milliseconds, and a refusal after it is not a provider coming back.
//
// ONE ENTRY'S, like the give-up: a mark is forgotten when the next entry begins, and a hold that ran out before the entry
// began is not a return the entry waited for.

/**
 What the budgets know of a provider whose last return served nothing.

 @example
 ```ts
 const mark: FutileReturn = { holdMs: 300_000, refusedAfterMs: 1_204, };
 ```
 */
export type FutileReturn = {
  /**
   How long the hold was that ended and was followed by a refusal with nothing served in between.
   */
  readonly holdMs: number;

  /**
   How long after that hold ended the provider refused again.
   */
  readonly refusedAfterMs: number;
};

/**
 The hold a refusal set before the one being marked.
 */
export type EarlierHold = {
  /**
   How long the hold was set for.
   */
  readonly holdMs: number;

  /**
   When the hold ended, on the layer's clock.
   */
  readonly endsAt: number;
};

/**
 The returns of one budget layer that served nothing.

 @example
 ```ts
 const futile: FutileReturns = createFutileReturns({ l, entryWords, },);
 ```
 */
export type FutileReturns = {
  /**
   Marks a provider's return as one that served nothing when a refusal arrives after a hold ended with nothing served
   since.

   @param provider - provider that refused

   @param earlier - hold the refusal replaces

   @param at - when the refusal arrived, on the layer's clock

   @param servedAt - when the provider last answered a call, negative infinity when it never did

   @param entryStartedAt - when the entry in flight began, negative infinity when none was named

   @mutates futile - replaces the provider's mark when the earlier hold ended in this entry with nothing served after
   */
  readonly note: (
    args: {
      readonly provider: ProviderName;
      readonly earlier: EarlierHold;
      readonly at: number;
      readonly servedAt: number;
      readonly entryStartedAt: number;
    },
  ) => void;

  /**
   The marks as they stand: a provider is in the map when its last return was followed by a refusal and no served call.

   @returns Each marked provider with what its last return was, live (copy it before keeping it)
   */
  readonly current: () => ReadonlyMap<ProviderName, FutileReturn>;

  /**
   Forgets a provider's mark, which a call it answered ends.

   @param provider - provider that served

   @mutates futile - clears the provider's mark
   */
  readonly served: (args: { readonly provider: ProviderName; },) => void;

  /**
   Forgets every mark, for an entry that begins with every provider met fresh.

   @mutates futile - clears every mark
   */
  readonly forgetAll: () => void;
};

/**
 Builds the futile returns of one budget layer.

 @param l - logger the mark of a return is written to

 @param entryWords - entry in flight, worded for a log line ("entry XingZ6020", or "the run" where none is named)

 @returns Marks that begin empty

 @example
 ```ts
 const futile = createFutileReturns({ l, entryWords: () => 'the run', },);
 ```
 */
export function createFutileReturns(
  {
    l,
    entryWords,
  }: {
    readonly l: Logger;
    readonly entryWords: () => string;
  },
): FutileReturns {
  /**
   The last return of each provider that served nothing.
   */
  const marks = new Map<ProviderName, FutileReturn>();

  return {
    current: function current(): ReadonlyMap<ProviderName, FutileReturn> {
      return marks;
    },

    served: function served({ provider, },): void {
      marks.delete(provider,);
    },

    forgetAll: function forgetAll(): void {
      for (const provider of PROVIDER_ORDER) {
        marks.delete(provider,);
      }
    },

    note: function note(
      {
        provider,
        earlier,
        at,
        servedAt,
        entryStartedAt,
      },
    ): void {
      // A HOLD OF NO TIME HAS NO RETURN, and neither has a provider no refusal has held.
      if (earlier.holdMs <= 0)
        return;
      // A HOLD STILL RUNNING IS NOT ONE THAT RAN OUT: a refusal that arrives inside it came from a call in flight when it
      // was set, and says nothing of what the provider does after the wait.
      if (at < earlier.endsAt)
        return;
      // A HOLD THAT RAN OUT BEFORE THE ENTRY BEGAN WAS NOT WAITED OUT BY IT.
      if (earlier.endsAt < entryStartedAt)
        return;
      // A CALL SERVED AFTER THE HOLD ENDED IS THE PROVIDER COMING BACK, whatever it does later.
      if (servedAt >= earlier.endsAt)
        return;
      /**
       What this return was.
       */
      const mark: FutileReturn = {
        holdMs: earlier.holdMs,
        refusedAfterMs: at - earlier.endsAt,
      };
      marks.set(
        provider,
        mark,
      );
      /**
       Logger pre-tagged with this function's name.
       */
      const rl = tagged({
        tag: note.name,
        l,
      },);
      rl.warn(
        `${provider}: refused again ${String(mark.refusedAfterMs,)}ms after its ${String(mark.holdMs,)}ms hold ended, `
          + `with nothing served in between; its return served nothing, so ${entryWords()} does not wait for ${provider}'s `
          + `holds to end while a bench is short, until ${provider} serves a call, unless ${provider} is the only `
          + 'writing provider not dry or its return would leave no bench floor unmet where without it one is',
      );
    },
  };
}

//endregion Provider budget futile returns
