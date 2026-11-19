/**
 Project caller options onto the request one pool member receives, without leaking pool credentials.
 @module
 */

import {
  clampThinkingLevel,
  type Api,
  type Model,
  type SimpleStreamOptions,
} from '@earendil-works/pi-ai';

/**
 Project caller options onto a physical request without leaking virtual-provider credentials.

 @param model - member to dispatch

 @param options - outer request options

 @returns credential-free options; registry adds credentials for this member

 @example
 ```ts
 const forwarded = memberOptions({ model: physical, options });
 ```
 */
export function memberOptions({
  model,
  options,
}: {
  model: Model<Api>;
  options?: SimpleStreamOptions;
},): SimpleStreamOptions {
  /**
   Separate virtual-provider credentials, caller retry policy and member-specific constraints from
   neutral options. Provider-level retries stay off: one attempt per member keeps the pool the owner
   of retry and fallback, and the pool itself never sleeps on a refused member.
   */
  const {
    apiKey,
    headers,
    env,
    maxTokens,
    reasoning,
    maxRetryDelayMs,
    ...neutral
  } = options ?? {};
  void apiKey;
  void headers;
  void env;
  void maxRetryDelayMs;
  /**
   Requested reasoning level after applying the physical model's capabilities.
   */
  const clamped = clampThinkingLevel(
    model,
    reasoning ?? 'off',
  );
  return {
    ...neutral,
    maxRetries: 0,
    ...(maxTokens === undefined ? {} : {
      maxTokens: model.maxTokens > 0 ? Math.min(
        maxTokens,
        model.maxTokens,
      ) : maxTokens,
    }),
    ...(clamped === 'off' ? {} : { reasoning: clamped, }),
  };
}
