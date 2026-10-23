import type { ModelReach, } from './budget-routing.ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Stream bound hold
// CLASS ONE HUNDRED FORTY-EIGHT. A call cut at its card's stream bound says
// one provider has stopped serving one model at its measured pace; Bedrock
// kept `google.gemma-4-26b-a4b` at its usual 1 s while `google.gemma-4-e2b`
// ran 219 to 263 s. So the hold is the pair's: the provider keeps its other
// models, and the model keeps its other providers. A model no other provider
// serves reads unreachable while the hold runs, so a round counts the seat
// out of its quorum instead of waiting on it (class twenty-six).

/**
 One provider and one model it serves.

 @example
 ```ts
 const pair: StreamBoundPair = { provider: 'bedrock', modelId: 'google.gemma-4-e2b', };
 ```
 */
export type StreamBoundPair = {
  /**
   Provider the call went to.
   */
  readonly provider: ProviderName;

  /**
   Model the call asked for.
   */
  readonly modelId: string;
};

/**
 Holds on one provider for one model, after a call there ran past its bound.

 @example
 ```ts
 const holds = createStreamBoundHolds({ holdMs: 60_000, now: monotonicMs, },);
 holds.hold({ provider: 'bedrock', modelId: 'google.gemma-4-e2b', },);
 ```
 */
export type StreamBoundHolds = {
  /**
   Starts, or restarts, one provider's hold on one model.
   */
  readonly hold: (args: StreamBoundPair,) => void;

  /**
   Milliseconds one provider's hold on one model has left, zero when free.
   */
  readonly remainingMs: (args: StreamBoundPair,) => number;
};

/**
 Builds the stream bound holds one router keeps.

 @param holdMs - how long a hold lasts

 @param now - clock the holds are read by, injectable for tests; the router passes `monotonicMs` (ledger B78)

 @returns Holds shared by every call the router makes

 @example
 ```ts
 const holds = createStreamBoundHolds({ holdMs: 60_000, now: monotonicMs, },);
 ```
 */
export function createStreamBoundHolds(
  {
    holdMs,
    now,
  }: {
    readonly holdMs: number;
    readonly now: () => number;
  },
): StreamBoundHolds {
  /**
   When each held pair's hold ends, by provider then model.
   */
  const until: Record<ProviderName, Map<string, number>> = {
    neuralwatt: new Map<string, number>(),
    synthetic: new Map<string, number>(),
    hyper: new Map<string, number>(),
    bedrock: new Map<string, number>(),
    deepseek: new Map<string, number>(),
    xiaomi: new Map<string, number>(),
    anthropic: new Map<string, number>(),
    openrouter: new Map<string, number>(),
  };
  return {
    hold: function hold(pair,): void {
      /**
       Holds on this pair's provider, by model.
       */
      const onProvider = until[pair.provider];
      onProvider.set(
        pair.modelId,
        now() + holdMs,
      );
    },
    remainingMs: function remainingMs(pair,): number {
      /**
       Holds on this pair's provider, by model.
       */
      const onProvider = until[pair.provider];
      /**
       When this pair's hold ends, absent when it was never held.
       */
      const ends = onProvider.get(pair.modelId,);
      if (ends === undefined)
        return 0;
      /**
       What is left of the hold, negative once it has ended.
       */
      const left = ends - now();
      if (left <= 0) {
        onProvider.delete(pair.modelId,);
        return 0;
      }
      return left;
    },
  };
}

/**
 A model's reach with the providers holding it out removed.
 */
export type ReachPastBoundHolds = {
  /**
   Providers that serve the model and hold nothing against it.
   */
  readonly reach: ModelReach;

  /**
   Longest hold among the serving providers taken out, zero when none was.
   */
  readonly heldMs: number;
};

/**
 Removes from a model's reach every provider holding it out on its stream
 bound.

 @param reach - providers that serve the model

 @param modelId - model being routed

 @param holds - the router's stream bound holds

 @returns Reach past the holds, and the longest hold taken out

 @example
 ```ts
 const { reach, heldMs, } = reachPastBoundHolds({ reach, modelId, holds, },);
 ```
 */
export function reachPastBoundHolds(
  {
    reach,
    modelId,
    holds,
  }: {
    readonly reach: ModelReach;
    readonly modelId: string;
    readonly holds: StreamBoundHolds;
  },
): ReachPastBoundHolds {
  /**
   Hold one serving provider has on the model, zero where it has none or
   does not serve the model.

   @param provider - provider read

   @returns Milliseconds left on its hold
   */
  function heldOn(provider: ProviderName,): number {
    if (!reach[provider])
      return 0;
    return holds.remainingMs({
      provider,
      modelId,
    },);
  }

  /**
   Whether one provider serves the model and holds nothing against it.

   @param provider - provider read

   @returns True where it may take the call
   */
  function open(provider: ProviderName,): boolean {
    return reach[provider] && (heldOn(provider,) === 0);
  }

  return {
    reach: {
      neuralwatt: open('neuralwatt',),
      synthetic: open('synthetic',),
      hyper: open('hyper',),
      bedrock: open('bedrock',),
      deepseek: open('deepseek',),
      xiaomi: open('xiaomi',),
      anthropic: open('anthropic',),
      openrouter: open('openrouter',),
    },
    heldMs: PROVIDER_ORDER.reduce(
      function longest(
        most,
        provider,
      ): number {
        return Math.max(
          most,
          heldOn(provider,),
        );
      },
      0,
    ),
  };
}

//endregion Stream bound hold
