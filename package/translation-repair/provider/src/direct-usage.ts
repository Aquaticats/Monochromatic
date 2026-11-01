import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { openRouterChunksOf, } from './openrouter-chunk-scan.ts';

//region Direct usage
// WHAT DEEPSEEK'S AND XIAOMI'S STREAMS REPORT BESIDE THE TEXT, read off the
// wire for the spend line and the completion log line.
//
// SCANNED SEPARATELY FROM THE COMPLETION READER, for the reason
// `openrouter-cost.ts` gives: that reader is shared with every provider and
// reports text and the two token counts, and a provider-specific field on
// `ExtractedCompletion` would make every other reader carry a value it cannot
// fill. The chunk walk is `openrouter-chunk-scan.ts`'s, which reads any
// OpenAI-compatible stream and skips comment lines (`: keep-alive` on a
// DeepSeek stream, a `:` line on some Xiaomi ones) and the `[DONE]` sentinel.
//
// DEEPSEEK splits the prompt in its usage block: `prompt_cache_hit_tokens` and
// `prompt_cache_miss_tokens` (parent session's probe, 2026-10-22), priced apart
// (`direct-card-sides.ts`). XIAOMI reports the hit as
// `prompt_tokens_details.cached_tokens`, the field `openrouter-cached-tokens.ts`
// already reads off any stream of that shape.

/**
 What a reader answers where the stream did not carry the field it was asked for.
 */
export const NOT_REPORTED: unique symbol = Symbol('field the stream does not carry',);

/**
 Prompt tokens DeepSeek's usage block split by cache.

 @example
 ```ts
 const split: DeepSeekCacheSplit = { hitTokens: 40, missTokens: 2, };
 ```
 */
export type DeepSeekCacheSplit = {
  /**
   Prompt tokens served from DeepSeek's cache, absent where the block named none.
   */
  readonly hitTokens?: number;

  /**
   Prompt tokens not served from the cache, absent where the block named none.
   */
  readonly missTokens?: number;
};

/**
 Reads one count off a usage block, refusing anything but a whole number.

 @param usage - the usage block

 @param name - field wanted

 @returns The count, or {@link NOT_REPORTED} where the field is absent or mistyped

 @example
 ```ts
 const hits = wholeCountAt({ usage, name: 'prompt_cache_hit_tokens', },);
 ```
 */
function wholeCountAt(
  {
    usage,
    name,
  }: {
    readonly usage: Readonly<Record<string, unknown>>;
    readonly name: string;
  },
): number | typeof NOT_REPORTED {
  /**
   Count as reported, of unknown type until checked.
   */
  const count = usage[name];
  if ((typeof count) !== 'number')
    return NOT_REPORTED;
  if ((!Number.isSafeInteger(count,)) || (count < 0))
    return NOT_REPORTED;
  return count;
}

/**
 The cache split the last usage block of a DeepSeek stream reported.

 THE LAST BLOCK WINS, as for the cost on OpenRouter: the usage arrives once,
 on the final chunk.

 @param bodyText - whole drained `text/event-stream` body

 @returns The split, with a field absent where no block named it

 @example
 ```ts
 const { hitTokens, missTokens, } = deepSeekCacheSplitOf({ bodyText: reply.bodyText, },);
 ```
 */
export function deepSeekCacheSplitOf({ bodyText, }: { readonly bodyText: string; },): DeepSeekCacheSplit {
  /**
   Splits each usage-bearing chunk reported, in arrival order.
   */
  const splits = openRouterChunksOf({ bodyText, },)
    .flatMap(function splitOf(chunk,): readonly DeepSeekCacheSplit[] {
      /**
       Usage block, absent on every chunk but the last.
       */
      const { usage, } = chunk;
      if (!isJsonRecord(usage,))
        return [];
      /**
       Hit count, when the block named one.
       */
      const hitTokens = wholeCountAt({
        usage,
        name: 'prompt_cache_hit_tokens',
      },);
      /**
       Miss count, when the block named one.
       */
      const missTokens = wholeCountAt({
        usage,
        name: 'prompt_cache_miss_tokens',
      },);
      return [
        {
          // Conditional spreads keep an unnamed count absent instead of undefined.
          ...((hitTokens === NOT_REPORTED) ? {} : { hitTokens, }),
          ...((missTokens === NOT_REPORTED) ? {} : { missTokens, }),
        },
      ];
    },);
  return splits.at(-1,) ?? {};
}

/**
 The model build the stream says served it, as DeepSeek's `system_fingerprint`.

 EVERY DEEPSEEK REPLY CARRIES IT (parent session's probe, 2026-10-22), and it
 is the only per-call evidence of the build behind `deepseek-flash`, whose
 1017 claim otherwise rests on DeepSeek's documentation alone. The shared
 completion reader does not expose it, so it is read here and logged on the
 completion line.

 @param bodyText - whole drained `text/event-stream` body

 @returns The last non-empty fingerprint a chunk carried, or {@link NOT_REPORTED}

 @example
 ```ts
 const fingerprint = systemFingerprintOf({ bodyText: reply.bodyText, },);
 ```
 */
export function systemFingerprintOf({ bodyText, }: { readonly bodyText: string; },): string | typeof NOT_REPORTED {
  /**
   Fingerprints each chunk carried, in arrival order.
   */
  const fingerprints = openRouterChunksOf({ bodyText, },)
    .flatMap(function fingerprintOf(chunk,): readonly string[] {
      /**
       Fingerprint as reported, of unknown type until checked.
       */
      const fingerprint = chunk.system_fingerprint;
      if (((typeof fingerprint) === 'string') && (fingerprint !== ''))
        return [fingerprint,];
      return [];
    },);
  return fingerprints.at(-1,) ?? NOT_REPORTED;
}

//endregion Direct usage
