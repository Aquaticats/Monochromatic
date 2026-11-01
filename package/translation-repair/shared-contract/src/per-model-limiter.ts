import pLimit, { type LimitFunction, } from 'p-limit';

import type { RosterModelId, } from './roster-id.ts';

//region Per-model limiter
// One concurrency limiter per model, created on first use. Each provider
// client (Bedrock, Hyper, OpenRouter, Synthetic) kept its own copy of this
// lookup (audit area six, 2026-09-28); one copy now serves all four.

/**
 Lookup handing each model its own limiter, created the first time the model
 is called and kept for the client's lifetime; bounded by catalog size.

 @param perModelConcurrency - concurrent requests granted to each model

 @returns Function returning the limiter for one model

 @example
 ```ts
 const limiterFor = perModelLimiter({ perModelConcurrency: 4, },);
 const limit = limiterFor('minimax-m3',);
 ```
 */
export function perModelLimiter(
  { perModelConcurrency, }: { readonly perModelConcurrency: number; },
): (modelId: RosterModelId,) => LimitFunction {
  /**
   Limiters keyed by model, created lazily.
   */
  const limiters = new Map<RosterModelId, LimitFunction>();

  /**
   Returns the limiter for one model, creating it on first use.

   @param modelId - model the exchange goes to

   @returns Limiter granting the model `perModelConcurrency` slots

   @example
   ```ts
   const limit = limiterFor('minimax-m3',);
   ```
   */
  return function limiterFor(modelId: RosterModelId,): LimitFunction {
    /**
     Existing limiter when this model was called before.
     */
    const existing = limiters.get(modelId,);
    if (existing !== undefined)
      return existing;

    /**
     Fresh limiter for first use of this model.
     */
    const created = pLimit(perModelConcurrency,);
    limiters.set(
      modelId,
      created,
    );
    return created;
  };
}

//endregion Per-model limiter
