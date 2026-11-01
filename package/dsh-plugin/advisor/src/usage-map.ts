/**
 Token-usage projection from DSH `TokenUsage` to pi-ai usage records. @module
 */

import type {
  Usage,
} from '@earendil-works/pi-ai';
import type {
  TokenUsage,
} from '@deepseek-ai/dsh-llm';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Project one DSH token-usage record onto the pi-ai usage shape.

 Counters map field for field; totals fall back to the sum of disjoint
 counters when the provider reports no exact total. Cost stays zero-valued:
 the DSH seam consults no text pricing anywhere (Q12), so the shared ledger
 records counts only.

 @param usage - DSH usage counters from one request

 @returns pi-ai usage record

 @example
 ```ts
 const usage = mapDshUsageToPiUsage(chunk.usage);
 ```
 */
export function mapDshUsageToPiUsage(
  usage: ForeignBorrowed<TokenUsage>,
): Usage {
  /**
   Uncached input tokens from the provider counters.
   */
  const input = usage.inputTokens;
  /**
   Output tokens from the provider counters.
   */
  const output = usage.outputTokens;
  /**
   Cache-read input tokens, zero when the provider reports none.
   */
  const cacheRead = usage.cacheReadTokens ?? 0;
  /**
   Cache-write input tokens, zero when the provider reports none.
   */
  const cacheWrite = usage.cacheWriteTokens ?? 0;
  return {
    input,
    output,
    cacheRead,
    cacheWrite,
    totalTokens: usage.totalTokens
      ?? (input
        + output
        + cacheRead
        + cacheWrite),
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      total: 0,
    },
    ...(usage.reasoningTokens === undefined
      ? {}
      : { reasoning: usage.reasoningTokens, }),
  };
}
