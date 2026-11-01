import {
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from './provider-name.ts';
import type { RosterModelId, } from './roster-id.ts';

//region Provider router slots
// HOW MANY CALLS ARE IN FLIGHT PER MODEL ON EACH PROVIDER THAT LIMITS THEM.
//
// COUNTED HERE RATHER THAN ASKED OF THE CLIENT, because saturation is what
// the routing policy calls the state of every slot being busy, and the client
// that owns those slots does not expose their occupancy. The router is the
// layer that dispatches, so it is the one that knows.
//
// ONLY A PROVIDER WITH A PER-MODEL LIMIT IS EVER SATURATED. No live provider
// carries one today: Synthetic granted five concurrent calls per model until
// 2026-10-19, when its rate-limit page stopped naming a concurrency limit
// (`SYNTHETIC_PER_MODEL_CONCURRENCY` is now unbounded); Charm Hyper has no
// such ceiling by owner confirmation, and OpenRouter routes each call to one of
// many upstreams and states none. A provider without a limit takes no slot and
// always reads as having room; the ceiling arithmetic stays for a provider that
// states one again, and the tests run it on a fixture limit.
//
// SPLIT FROM `provider-router.ts` at its line budget, along the seam the
// saturation race of 2026-08-24 made visible: the count must move in the same
// synchronous step as the routing decision, so the ledger is a synchronous
// object the router reads and writes with nothing awaited in between.

/**
 Concurrent calls each provider grants one model.

 POSITIVE INFINITY MEANS NO LIMIT, the same reading `HYPER_PER_MODEL_CONCURRENCY`
 and `OPENROUTER_PER_MODEL_CONCURRENCY` give it: a provider that states no
 ceiling takes no slot and is never saturated.

 @example
 ```ts
 const limits: SlotLimits = { synthetic: 5, hyper: Number.POSITIVE_INFINITY, openrouter: Number.POSITIVE_INFINITY, };
 ```
 */
export type SlotLimits = ProviderRecord<number>;

/**
 Whether a limit is a real ceiling rather than the no-limit reading.

 @param limit - slots a provider grants one model

 @returns Whether calls on it should be counted

 @example
 ```ts
 const counted = isCeiling({ limit: 5, },);
 ```
 */
function isCeiling(
  { limit, }: { readonly limit: number; },
): boolean {
  return Number.isFinite(limit,);
}

/**
 Fresh per-model count for one provider.

 @returns Empty count map

 @example
 ```ts
 const counts = freshCounts();
 ```
 */
function freshCounts(): Map<RosterModelId, number> {
  return new Map<RosterModelId, number>();
}

/**
 In-flight accounting for the providers that limit per-model concurrency.

 @example
 ```ts
 const ledger: SlotLedger = createSlotLedger({ limits: { synthetic: 5, }, },);
 ```
 */
export type SlotLedger = {
  /**
   Which providers have every slot for this model taken right now.
   */
  readonly saturated: (args: { readonly modelId: RosterModelId; },) => ProviderRecord<boolean>;

  /**
   Whether a provider limits this ledger at all, so a caller knows whether
   taking a slot is a real act.
   */
  readonly limits: (args: { readonly provider: ProviderName; },) => boolean;

  /**
   Takes one slot on a provider for a model, synchronously.
   */
  readonly take: (args: {
    readonly provider: ProviderName;
    readonly modelId: RosterModelId;
  },) => void;

  /**
   Hands back the slot {@link SlotLedger.take} took, on scope exit.

   A DISPOSABLE RATHER THAN A `finally`, so the release cannot be skipped by
   an early return added later and does not need the caller to remember it.
   */
  readonly held: (args: {
    readonly provider: ProviderName;
    readonly modelId: RosterModelId;
  },) => Disposable;
};

/**
 Builds the ledger over the providers that limit per-model concurrency.

 @param limits - concurrent calls per model each limiting provider grants

 @returns Ledger the router reads and writes at the decision

 @example
 ```ts
 const ledger = createSlotLedger({ limits: { synthetic: 5, }, },);
 ```
 */
export function createSlotLedger(
  { limits, }: { readonly limits: SlotLimits; },
): SlotLedger {
  /**
   Calls in flight per provider, per model.
   */
  const inFlight = providerRecord({ of: freshCounts, },);

  /**
   Adjusts the in-flight count for one model on one provider.

   @param provider - provider whose count moves

   @param modelId - model whose count moves

   @param by - change to apply

   @example
   ```ts
   count({ provider: 'synthetic', modelId, by: 1, },);
   ```
   */
  function count(
    {
      provider,
      modelId,
      by,
    }: {
      readonly provider: ProviderName;
      readonly modelId: RosterModelId;
      readonly by: number;
    },
  ): void {
    /**
     This provider's per-model counts.
     */
    const counts = inFlight[provider];
    counts.set(
      modelId,
      (counts.get(modelId,) ?? 0) + by,
    );
  }

  return {
    saturated: function saturated({ modelId, },): ProviderRecord<boolean> {
      return providerRecord({
        of: function full(provider,): boolean {
          /**
           This provider's limit, infinite where it has none.
           */
          const limit = limits[provider];
          if (!isCeiling({ limit, },))
            return false;
          /**
           This provider's per-model counts.
           */
          const counts = inFlight[provider];
          /**
           Calls in flight on this model here.
           */
          const busy = counts.get(modelId,) ?? 0;
          return busy >= limit;
        },
      },);
    },

    limits: function limitsProvider({ provider, },): boolean {
      return isCeiling({ limit: limits[provider], },);
    },

    take: function take({
      provider,
      modelId,
    },): void {
      if (!isCeiling({ limit: limits[provider], },))
        return;
      count({
        provider,
        modelId,
        by: 1,
      },);
    },

    held: function held({
      provider,
      modelId,
    },): Disposable {
      return {
        [Symbol.dispose]: function release(): void {
          if (!isCeiling({ limit: limits[provider], },))
            return;
          count({
            provider,
            modelId,
            by: -1,
          },);
        },
      };
    },
  };
}

//endregion Provider router slots
