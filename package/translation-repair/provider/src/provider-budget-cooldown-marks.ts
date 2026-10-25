import {
  type ProviderName,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Provider budget cooldown marks
// THE PROVIDERS AN ENTRY HAS GIVEN UP WAITING FOR, split out of `provider-budget.ts` at its line budget.
//
// A COOLDOWN IS A GUESS THAT THE PROVIDER COMES BACK. A refusal on a meter that says dry or cannot be read holds the
// provider out for the refusal cooldown (`REFUSAL_COOLDOWN_MS`, 300 s), and the seat reader waits that hold out when the
// provider's return would lift a short bench. XingZ6020 (2026-10-23) seated Synthetic while its meter read "unreadable,
// treating as spendable": every call was refused at once, each refusal set a fresh 300 s cooldown, and the reader
// waited it out for every slice, about 25 waits of 240 to 300 s after 00:34 UTC. The meter half was fixed (S1); the loop
// was not, and any provider that refuses right after a cooldown ran out would cost one full cooldown for every slice.
//
// THE EVIDENCE THAT THE GUESS WAS WRONG is the budget layer's own: the provider was held out for a cooldown, the hold
// ended, nothing it served came back after the end, and it refused again. That refusal is the first answer the provider
// gave after the wait, so a further wait for it buys the same answer. It is read from the holds and from the service
// the router reports (`noteServed`), not from a time window: a window would have to outlast the transport's retry
// ladder, whose length no measurement fixes, and a provider that served after its cooldown and refuses much later is a
// provider that came back, which the next cooldown is waited for as before.
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
   When the hold ends, on the layer's clock.
   */
  readonly endsAt: number;
};

/**
 What a refusal right after a cooldown tells the entry that met it.
 */
export type CooldownRepeat = {
  /**
   Length of the cooldown the provider's earlier refusal set, which ran out before this refusal.
   */
  readonly waitedOutMs: number;

  /**
   The entry that gives the provider up, none where the layer's caller named no entry.
   */
  readonly entryId: string | undefined;
};

/**
 The cooldown marks of one budget layer.

 @example
 ```ts
 const marks: CooldownMarks = createCooldownMarks({ now: monotonicMs, },);
 ```
 */
export type CooldownMarks = {
  /**
   Starts an entry: forgets every provider an entry that ended had given up on, and names the entry for the log.

   @param entryId - the entry that starts

   @mutates marks - clears the providers given up on

   @returns The providers that were given up on and are asked again, empty when none was
   */
  readonly beginEntry: (args: { readonly entryId: string; },) => readonly ProviderName[];

  /**
   Whether the entry has given up waiting for a provider's cooldowns.

   @param provider - provider to check

   @returns Whether it refused right after a cooldown ran out and the entry has not ended since
   */
  readonly givenUp: (args: { readonly provider: ProviderName; },) => boolean;

  /**
   Whether the hold last set on a provider is a refusal cooldown, and not a hold that names its own return.

   @param provider - provider to check

   @returns Whether its latest hold is a cooldown
   */
  readonly heldByCooldown: (args: { readonly provider: ProviderName; },) => boolean;

  /**
   Records the hold a refusal has just set, and tells whether the refusal came right after a cooldown ran out.

   @param provider - provider that refused

   @param holdMs - how long the refusal holds it out

   @param cooldown - whether the hold is a refusal cooldown

   @mutates marks - gives the provider up when this refusal is the first answer after a cooldown that ran out

   @returns The repeat, the first time it is found in an entry; none otherwise
   */
  readonly noteHold: (
    args: {
      readonly provider: ProviderName;
      readonly holdMs: number;
      readonly cooldown: boolean;
    },
  ) => CooldownRepeat | undefined;

  /**
   Records that a provider served a call, which ends the question whether its cooldown was a recovery.

   @param provider - provider that served
   */
  readonly noteServed: (args: { readonly provider: ProviderName; },) => void;
};

/**
 The hold before any refusal: none.

 @returns Nothing set

 @example
 ```ts
 const lastHold = providerRecord({ of: noHoldSetYet, },);
 ```
 */
function noHoldSetYet(): SetHold | undefined {
  return undefined;
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
 Builds the cooldown marks of one budget layer.

 @param now - the layer's clock, which the holds and the service are stamped on

 @returns Marks that give no provider up until a refusal right after a cooldown does

 @example
 ```ts
 const marks = createCooldownMarks({ now: monotonicMs, },);
 ```
 */
export function createCooldownMarks({ now, }: { readonly now: () => number; },): CooldownMarks {
  /**
   The last hold each provider's refusals set.
   */
  const lastHold = providerRecord({ of: noHoldSetYet, },);

  /**
   When each provider last served a call.
   */
  const servedAt = providerRecord({ of: neverServed, },);

  /**
   The entry the marks are for, none until one is named.
   */
  const entry: {
    id: string | undefined;
    startedAt: number;
  } = {
    id: undefined,
    startedAt: Number.NEGATIVE_INFINITY,
  };

  /**
   Providers the entry has given up waiting for.
   */
  const given = new Set<ProviderName>();

  return {
    beginEntry: function beginEntry({ entryId, },): readonly ProviderName[] {
      /**
       Providers given up on until this entry began.
       */
      const forgotten = [...given,];
      entry.id = entryId;
      entry.startedAt = now();
      given.clear();
      return forgotten;
    },

    givenUp: function givenUp({ provider, },): boolean {
      return given.has(provider,);
    },

    heldByCooldown: function heldByCooldown({ provider, },): boolean {
      return lastHold[provider]?.cooldown === true;
    },

    noteHold: function noteHold(
      {
        provider,
        holdMs,
        cooldown,
      },
    ): CooldownRepeat | undefined {
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
      if (!cooldown)
        return undefined;
      if (earlier?.cooldown !== true)
        return undefined;
      // A HOLD STILL RUNNING IS NOT ONE THAT RAN OUT: a refusal that arrives inside it came from a call in flight when
      // it was set, and says nothing of what the provider does after the wait.
      if (at < earlier.endsAt)
        return undefined;
      // A COOLDOWN THAT RAN OUT BEFORE THE ENTRY BEGAN WAS NOT WAITED OUT BY IT.
      if (earlier.endsAt < entry.startedAt)
        return undefined;
      // A CALL SERVED AFTER THE HOLD ENDED IS THE PROVIDER COMING BACK, whatever it does later.
      if (servedAt[provider] >= earlier.endsAt)
        return undefined;
      if (given.has(provider,))
        return undefined;
      given.add(provider,);
      return {
        waitedOutMs: earlier.holdMs,
        entryId: entry.id,
      };
    },

    noteServed: function noteServed({ provider, },): void {
      servedAt[provider] = now();
    },
  };
}

//endregion Provider budget cooldown marks
