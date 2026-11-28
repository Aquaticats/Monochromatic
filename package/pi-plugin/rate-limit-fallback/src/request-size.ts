/**
 How large a request is before a member is asked to hold it.
 @module
 */

import type { TranscriptContext, } from '@earendil-works/pi-ai';
import { estimateContextTokens, } from '@earendil-works/pi-ai/utils/estimate';

//region One request's size

/**
 Estimate the input tokens one request carries, with Pi's own estimator.

 The router needs a size before it sends anything, because which member can hold the request decides
 whether it is asked at all. Pi hands a provider the normalized transcript and nothing that counts it,
 so the count comes from `estimateContextTokens` (`@earendil-works/pi-ai` dist/utils/estimate.js):
 the last assistant answer's reported usage, plus 3.5 characters per token for every message after it,
 and 3.5 characters per token for the whole transcript when no answer reported usage yet. Pi itself
 decides compaction and each request's output cap from the same function
 (`dist/api/simple-options.js` `clampMaxTokensToContext`), so the pool's view of a request is the one
 Pi already acts on.

 No margin is added between that estimate and a member's window. An over-estimate would pass over a
 member that could have held the request, which is the capacity this change exists to keep, and an
 under-estimate ends in the member's own rejection: under a virtual selection Pi then compacts and
 retries the turn once (`README.md`, "What the route gives Pi", verified by
 `src/verify-stand-in-member-extension.ts`).

 The measurement a margin would have cited is in `README.md`, "What the router knows of a request's
 size": over 35,773 requests in 200 of the driver's runs, in sessions Pi never compacted or edited,
 the estimate was within 1% of the count the member itself reported for 98.0% of them, and within 5%
 for 99.4% of them, the 95th percentile standing 0.2% above and the 5th 0.1% below. A margin of that
 size is a thousandth of a member's window: it would pass over members to answer requests they can
 hold, and the remainder is what the member's own rejection covers.

 @param context - normalized transcript Pi is about to send

 @returns estimated input tokens, the count compared with a member's own reported window

 @example
 ```ts
 estimateRequestTokens(context); // 1234
 ```
 */
export function estimateRequestTokens(context: TranscriptContext,): number {
  return estimateContextTokens(context,)
    .tokens;
}

//endregion One request's size
