import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream/ts';

//region Decision context refusal
// LEDGER P13 (the whole-package audit, 2026-09-28): OpenRouter's decisions
// reference documents a 413 for an oversized payload and says nothing of a
// state past the model's context. Probed that day with a cat-themed state of
// 45,046 characters, the evidence first and then last: both came back HTTP
// 400 with the error type `max_tokens_exceeded`, so the endpoint refuses a
// state it cannot hold and never decides on a truncated one. The endpoint is
// therefore the only tokenizer this package needs: nothing estimates a
// state's size before it is sent.
//
// WHY IT CAN HAPPEN: `ce824d933` put the house rules in every select state,
// 4,085 tokens on the same probe, and 4 of the 13,878 Jev prompts in the run
// logs (largest 30,203) would now pass the 32,000-token context.

/**
 Status the endpoint refuses an over-long state with.
 */
const HTTP_BAD_REQUEST = 400;

/**
 Error type the endpoint names in the body of that refusal.
 */
const OVER_CONTEXT_ERROR_TYPE = 'max_tokens_exceeded';

/**
 Whether thrown failure is the decisions endpoint refusing a state past the
 seat's context, which no retry and no later round can change.

 @param error - whatever the typed exchange threw

 @returns Whether the endpoint refused the state for its length

 @example
 ```ts
 if (isDecisionStateOverContext({ error, },)) l.warn('state past the seat\'s context, seat out of reach',);
 ```
 */
export function isDecisionStateOverContext(
  { error, }: { readonly error: unknown; },
): boolean {
  if (!(error instanceof ProviderHttpError))
    return false;
  if (error.status !== HTTP_BAD_REQUEST)
    return false;
  return error.bodyExcerpt
    .includes(OVER_CONTEXT_ERROR_TYPE,);
}

//endregion Decision context refusal
