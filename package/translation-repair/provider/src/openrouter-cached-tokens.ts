import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { openRouterChunksOf, } from './openrouter-chunk-scan.ts';

//region OpenRouter cached tokens
// HOW MUCH OF THE PROMPT THE UPSTREAM SERVED FROM ITS CACHE, read off the
// wire so the effect of price-sorted routing can be measured rather than
// assumed. Every stage's sheet is 9 to 14 KB and identical across that stage's
// calls, and the DeepSeek family's endpoints cache a prompt prefix without
// being asked (OpenRouter's prompt-caching page, read 2026-09-09: "automated
// and does not require any additional configuration", cache reads "0.1x the
// price of the original input pricing"), reporting the hit in
// `usage.prompt_tokens_details.cached_tokens`. Scanned beside the cost, for
// the reason `openrouter-cost.ts` gives for not widening the shared reader.

/**
 Reading given when no chunk carried a cached-token count.
 */
export const CACHED_UNREPORTED = 'unreported';

/**
 Prompt tokens the final chunk says were served from cache.

 THE LAST COUNT WINS, as the cost does: the usage block arrives once, on the
 final chunk.

 @param bodyText - whole drained `text/event-stream` body

 @returns Cached prompt tokens, or that no chunk reported a count

 @example
 ```ts
 const cached = openRouterCachedTokensOf({ bodyText: reply.bodyText, },);
 ```
 */
export function openRouterCachedTokensOf(
  { bodyText, }: { readonly bodyText: string; },
): number | typeof CACHED_UNREPORTED {
  /**
   Counts each chunk reported, in arrival order.
   */
  const counts = openRouterChunksOf({ bodyText, },)
    .flatMap(function cachedOf(chunk,): readonly number[] {
      /**
       Usage block, absent on every chunk but the last.
       */
      const { usage, } = chunk;
      if (!isJsonRecord(usage,))
        return [];
      /**
       Prompt detail block, absent on upstreams that report no cache.
       */
      const details = usage.prompt_tokens_details;
      if (!isJsonRecord(details,))
        return [];
      /**
       Count as reported, of unknown type until checked.
       */
      const cached = details.cached_tokens;
      if ((typeof cached) !== 'number')
        return [];
      if (!Number.isSafeInteger(cached,))
        return [];
      return [cached,];
    },);

  return counts.at(-1,) ?? CACHED_UNREPORTED;
}

//endregion OpenRouter cached tokens
