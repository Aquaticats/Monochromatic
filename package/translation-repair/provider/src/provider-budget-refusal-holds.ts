import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  EVERY_PROVIDER_COUNTS,
  type WritingReach,
} from './budget-routing.ts';
import {
  createFutileReturns,
  type FutileReturn,
} from './provider-budget-futile-returns.ts';
import {
  createGiveUpAside,
  type UnmetFloors,
  writingProviderFloor,
} from './provider-budget-give-up-aside.ts';

//region Provider budget refusal holds
// THE HOLDS A REFUSAL SETS ON A PROVIDER, AND THE PROVIDERS AN ENTRY HAS GIVEN UP WAITING FOR, split out of
// `provider-budget.ts` at its line budget.
//
// A COOLDOWN IS A GUESS THAT THE PROVIDER COMES BACK. A refusal on a meter that says dry or cannot be read holds the
// provider out for the refusal cooldown (`REFUSAL_COOLDOWN_MS`, 300 s), and the seat reader waits that hold out when the
// provider's return would lift a short bench. XingZ6020 (2026-10-23) seated Synthetic while its meter read "unreadable,
// treating as spendable": every call was refused at once, each refusal set a fresh 300 s cooldown, and the reader
// waited it out for every slice: 58 distinct minutes between 00:34 and 06:05 UTC carry a seat-reader wait of 240 to
// 300 s, and 395 lines record a refusal (B965 has the command). The meter half was fixed (S1); the loop was not, and
// any provider that refuses right after a cooldown ran out would cost one full cooldown for every slice.
//
// THE EVIDENCE THAT THE GUESS WAS WRONG is the budget layer's own: the provider was held out for a cooldown, the hold
// ended, nothing it served came back after the end, and it refused again. That refusal is the first answer the provider
// gave after the wait, so a further wait for it buys the same answer. It is read from the holds and from the service
// the router reports (`noteServed`), not from a time window: a window would have to outlast the transport's retry
// ladder, whose length no measurement fixes, and a provider that served after its cooldown and refuses much later is a
// provider that came back, whose next cooldown is waited for as before.
//
// THE MARK IS ONE ENTRY'S. A pass shares one budget layer across its entries, and another entry meets the provider
// fresh: the owner's rule is that the entry stops waiting, not the run. `beginEntry` forgets every mark, and a layer
// whose caller never names an entry keeps one mark set for its whole life.
//
// A COOLDOWN THAT RAN OUT BEFORE THE ENTRY BEGAN IS NOT ONE THE ENTRY WAITED OUT. The entry is judged on its own wait:
// a refusal counts only against a cooldown that ended after the entry began, which the entry either waited for or let
// pass, and the provider's first refusal in a new entry only sets the next cooldown. One cooldown still running when the
// entry begins is waited for as any other.
//
// ONLY A COOLDOWN IS GIVEN UP. A hold that names its own return (a `Retry-After` header, a daily limit in the body, a
// NeuralWatt day start) and a payment refusal are statements about the provider and keep their handling, so the holds
// of such a refusal are never marked as cooldowns and a mark never cuts them.
//
// A GIVE-UP COSTS THE ENTRY NOTHING THAT WAITING WOULD HAVE BOUGHT (the lead's rulings of 2026-10-25, corrected in
// round 4 and open to the owner's veto). It is set aside, and the cooldown waited out and the provider asked again as
// before the give-up existed, only when the given-up provider is the only writing provider not dry, or when with it back
// no bench floor the entry needs would be left unmet while without it one is. Otherwise it holds
// (`provider-budget-give-up-aside.ts`). The decision is taken from each meter reading (`noteReading`) and from the
// holds as they stand, for each given-up provider.
//
// A RETURN THAT SERVED NOTHING IS MARKED BESIDE THE GIVE-UP, ON THE SAME EVIDENCE (`provider-budget-futile-returns.ts`):
// a hold ended in the entry, nothing the provider answered came after the end, and it refused again. The give-up reads
// that for a refusal cooldown and takes the provider out of routing; the mark reads it for every hold and takes nothing
// out, and a served call clears it. The seat reader skips a marked provider's holds, so a stage goes on short instead of
// waiting for a provider that has refused its last return. The mark yields to the give-up's two exceptions, decided by
// the same weighing (`waitingHelps`), so no wait that lets the entry finish is skipped.

/**
 The last hold a refusal set on one provider.
 */
type SetHold = {
  /**
   Whether the hold is a refusal cooldown: timed by the layer's own guess and naming no return.
   */
  readonly cooldown: boolean;

  /**
   How long the hold was set for.
   */
  readonly holdMs: number;

  /**
   When the hold ends, on the layer's clock; negative infinity for a provider no refusal has held.
   */
  readonly endsAt: number;
};

/**
 The refusal holds of one budget layer.

 @example
 ```ts
 const refusals: RefusalHolds = createRefusalHolds({ now: monotonicMs, l, },);
 ```
 */
export type RefusalHolds = {
  /**
   Records the hold a refusal has just set, and gives the provider up for the entry when the refusal came right after a
   cooldown ran out.

   @param provider - provider that refused

   @param holdMs - how long the refusal holds it out

   @param cooldown - whether the hold is a refusal cooldown, and not a hold that names its own return

   @mutates refusals - replaces the provider's hold, and gives the provider up when this refusal is the first answer
   after a cooldown that ran out in the entry with nothing served since
   */
  readonly hold: (
    args: {
      readonly provider: ProviderName;
      readonly holdMs: number;
      readonly cooldown: boolean;
    },
  ) => void;

  /**
   How much longer a refusal holds a provider out, whatever the entry has given up.

   @param provider - provider to check

   @returns Milliseconds of hold left, zero when it is not held
   */
  readonly left: (args: { readonly provider: ProviderName; },) => number;

  /**
   How much of the provider's hold a caller is to wait for: all of it, except a cooldown the entry gave up on while the
   give-up holds, whose end is no return since the provider refused at once after the last one. A hold that names its
   own return keeps counting, and so does every hold once the give-up is set aside.

   @param provider - provider to check

   @returns Milliseconds of hold left that are worth waiting for
   */
  readonly waitedLeft: (args: { readonly provider: ProviderName; },) => number;

  /**
   Whether a refusal keeps a provider out of routing: a hold is running, or the entry has given the provider up and
   the give-up holds.

   @param provider - provider to check

   @returns Whether the provider reads dry for this reason
   */
  readonly keepsOut: (args: { readonly provider: ProviderName; },) => boolean;

  /**
   Whether the entry has given up waiting for a provider's cooldowns, whether or not the give-up is set aside.

   @param provider - provider to check

   @returns Whether it refused right after a cooldown ran out and the entry has not ended since
   */
  readonly givenUp: (args: { readonly provider: ProviderName; },) => boolean;

  /**
   What the budgets know of a provider whose last return served nothing and whose holds the seat reader is therefore not
   to wait for: the hold ended in this entry, no call the provider answered came after the end, it refused again, and
   waiting for it is not what lets the entry finish (the provider is not the only writing provider not dry, and its
   return would not leave every bench floor met where without it one is not).

   @returns Each provider whose last return served nothing and whose holds are not worth waiting for, with that return;
   a provider that has none, served a call since, or must be waited for is not in it
   */
  readonly futileReturns: () => ReadonlyMap<ProviderName, FutileReturn>;

  /**
   Records which providers the last meter reading found dry apart from refusal holds and give-ups, and decides again
   for every provider given up whether its give-up is set aside.

   @param dry - per provider, whether its meter, a refused credential or a payment refusal keeps it dry

   @mutates refusals - replaces the reading the give-up decision is taken from
   */
  readonly noteReading: (args: { readonly dry: ProviderRecord<boolean>; },) => void;

  /**
   Records that a provider served a call, which ends the question whether its cooldown was a recovery.

   @param provider - provider that served
   */
  readonly noteServed: (args: { readonly provider: ProviderName; },) => void;

  /**
   Starts an entry: forgets every provider an entry that ended had given up on, and names the entry for the log.

   @param entryId - the entry that starts

   @mutates refusals - clears the providers given up on
   */
  readonly beginEntry: (args: { readonly entryId: string; },) => void;
};

/**
 The hold before any refusal: a cooldown that ended at the beginning of time.

 @returns A hold that has run out and waits for nothing

 @example
 ```ts
 const lastHold = providerRecord({ of: noHoldSetYet, },);
 ```
 */
function noHoldSetYet(): SetHold {
  return {
    cooldown: false,
    holdMs: 0,
    endsAt: Number.NEGATIVE_INFINITY,
  };
}

/**
 The stamp of a provider that has served nothing: infinitely long ago.

 @returns Negative infinity

 @example
 ```ts
 const servedAt = providerRecord({ of: neverServed, },);
 ```
 */
function neverServed(): number {
  return Number.NEGATIVE_INFINITY;
}

/**
 Builds the refusal holds of one budget layer.

 @param now - the layer's clock, which the holds and the service are stamped on

 @param l - logger the decision to give a provider up, and the end of that decision, are written to

 @param writing - the providers whose wet reading keeps the run going, which decide whether a given-up provider is the
 only writing provider not dry (default: every provider)

 @param unmetFloors - the bench floors the entry's stages need, which decide whether a given-up provider's return would
 leave none unmet where without it one is (default: some writing provider can be asked)

 @returns Holds that give no provider up until a refusal right after a cooldown does

 @example
 ```ts
 const refusals = createRefusalHolds({ now: monotonicMs, l, },);
 ```
 */
export function createRefusalHolds(
  {
    now,
    l,
    writing = EVERY_PROVIDER_COUNTS,
    unmetFloors = writingProviderFloor({ writing, },),
  }: {
    readonly now: () => number;
    readonly l: Logger;
    readonly writing?: WritingReach;
    readonly unmetFloors?: UnmetFloors;
  },
): RefusalHolds {
  /**
   The last hold each provider's refusals set.
   */
  const lastHold: Record<ProviderName, SetHold> = providerRecord({ of: noHoldSetYet, },);

  /**
   When each provider last served a call.
   */
  const servedAt: Record<ProviderName, number> = providerRecord({ of: neverServed, },);

  /**
   The entry the holds are for, none until one is named, and when it began.
   */
  const entry: {
    id?: string;
    startedAt: number;
  } = { startedAt: Number.NEGATIVE_INFINITY, };

  /**
   Providers the entry has given up waiting for.
   */
  const given = new Set<ProviderName>();

  /**
   How much longer a refusal holds a provider out.

   @param provider - provider to check

   @returns Milliseconds of hold left

   @example
   ```ts
   const remaining = left({ provider: 'hyper', },);
   ```
   */
  function left({ provider, }: { readonly provider: ProviderName; },): number {
    /**
     When its hold ends.
     */
    const { endsAt, } = lastHold[provider];
    return Math.max(
      0,
      endsAt - now(),
    );
  }

  /**
   The entry the holds are for, worded for a log line.

   @returns "entry" and its name, or "the run" where no entry was named
   */
  function entryWords(): string {
    return (entry.id === undefined) ? 'the run' : `entry ${entry.id}`;
  }

  /**
   The decision, for each provider given up, whether the give-up is set aside because the provider is the only writing
   provider not dry or its return would leave no bench floor unmet where without it one is.
   */
  const aside = createGiveUpAside({
    unmetFloors,
    writing,
    given,
    left,
    entryWords,
    l,
  },);

  /**
   The returns that served nothing, marked on the same evidence as the give-up.
   */
  const futile = createFutileReturns({
    l,
    entryWords,
  },);

  return {
    left,

    noteReading: aside.noteReading,

    hold: function hold(
      {
        provider,
        holdMs,
        cooldown,
      },
    ): void {
      /**
       The hold this refusal replaces.
       */
      const earlier = lastHold[provider];
      /**
       When this refusal arrived.
       */
      const at = now();
      lastHold[provider] = {
        cooldown,
        holdMs,
        endsAt: at + holdMs,
      };
      // A RETURN THAT SERVED NOTHING IS MARKED FOR EVERY KIND OF HOLD, before the give-up reads the cooldowns alone.
      futile.note({
        provider,
        earlier,
        at,
        servedAt: servedAt[provider],
        entryStartedAt: entry.startedAt,
      },);
      if (!cooldown)
        return;
      if (!earlier.cooldown)
        return;
      // A HOLD STILL RUNNING IS NOT ONE THAT RAN OUT: a refusal that arrives inside it came from a call in flight when
      // it was set, and says nothing of what the provider does after the wait.
      if (at < earlier.endsAt)
        return;
      // A COOLDOWN THAT RAN OUT BEFORE THE ENTRY BEGAN WAS NOT WAITED OUT BY IT.
      if (earlier.endsAt < entry.startedAt)
        return;
      // A CALL SERVED AFTER THE HOLD ENDED IS THE PROVIDER COMING BACK, whatever it does later.
      if (servedAt[provider] >= earlier.endsAt)
        return;
      if (given.has(provider,))
        return;
      given.add(provider,);
      /**
       Logger pre-tagged with this function's name.
       */
      const rl = tagged({
        tag: hold.name,
        l,
      },);
      rl.warn(
        `${provider}: refused again right after its ${String(earlier.holdMs,)}ms refusal cooldown ran out, with `
          + `nothing served in between; ${entryWords()} `
          + `stops waiting for ${provider}'s cooldowns and treats it as unreachable until the entry ends, unless `
          + `${provider} is the only writing provider not dry or its return would leave no bench floor unmet where `
          + 'without it one is',
      );
      // THE GIVE-UP IS WEIGHED AT ONCE against the last reading, and the line that follows says which it is, so a
      // provider given up whose return waiting could use is set aside before anyone reads the dryness.
      aside.reconsider();
    },

    waitedLeft: function waitedLeft({ provider, },): number {
      /**
       Whether the hold in force is a refusal cooldown.
       */
      const { cooldown, } = lastHold[provider];
      if (given.has(provider,)
        && cooldown
        && (!aside.isAside({ provider, },)))
        return 0;
      return left({ provider, },);
    },

    keepsOut: function keepsOut({ provider, },): boolean {
      return (left({ provider, },) > 0) || (given.has(provider,) && (!aside.isAside({ provider, },)));
    },

    givenUp: function givenUp({ provider, },): boolean {
      return given.has(provider,);
    },

    futileReturns: function futileReturns(): ReadonlyMap<ProviderName, FutileReturn> {
      /**
       The marks that stand: the providers whose wait would not make the entry finishable.
       */
      const standing = new Map<ProviderName, FutileReturn>();
      for (const [provider, mark,] of futile.all()) {
        // THE MARK YIELDS WHERE WAITING IS WHAT LETS THE ENTRY FINISH, as the give-up does.
        if (!aside.waitingHelps({ provider, },))
          standing.set(
            provider,
            mark,
          );
      }
      return standing;
    },

    noteServed: function noteServed({ provider, },): void {
      servedAt[provider] = now();
      futile.served({ provider, },);
    },

    beginEntry: function beginEntry({ entryId, },): void {
      /**
       Providers given up on until this entry began.
       */
      const forgotten = [...given,];
      entry.id = entryId;
      entry.startedAt = now();
      given.clear();
      aside.forget();
      futile.forgetAll();
      if (forgotten.length > 0) {
        tagged({
          tag: beginEntry.name,
          l,
        },)
          .info(`entry ${entryId} begins: ${forgotten.join(', ',)} no longer given up on, so asked again`,);
      }
    },
  };
}

//endregion Provider budget refusal holds
