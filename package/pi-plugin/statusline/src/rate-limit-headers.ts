/**
 Provider usage response header parser fan-in.
 
 @module
 */

import { parseAnthropicRateLimitSnapshots, } from './anthropic-rate-limit-headers.ts';
import { parseCodexRateLimitSnapshots, } from './codex-rate-limit-headers.ts';
import { normalizeHeaders, } from './rate-limit-parse-helpers.ts';
import { parseSyntheticRateLimitSnapshots, } from './synthetic-quota-headers.ts';
import type { RateLimitSnapshot, } from './rate-limit-types.ts';

/**
 Parses all supported provider usage header groups.
 
 Sources:
 - Anthropic token limit headers from Pi's Anthropic provider responses.
 - Codex subscription headers matching OpenAI Codex CLI `codex-api/src/rate_limits.rs`.
 - Synthetic quotas JSON from `@aliou/pi-synthetic`'s `x-synthetic-quotas` header.
 
 @param headers - provider response headers from Pi
 
 @param nowMs - wall-clock sample time in epoch milliseconds
 
 @returns parsed {@link RateLimitSnapshot} entries for complete and valid projectable usage windows
 
 @example
 ```ts
 parseRateLimitSnapshots({ headers, nowMs: Date.now() });
 ```
 */
function parseRateLimitSnapshots({
  headers,
  nowMs,
}: {
  readonly headers: Readonly<Record<string, string>>;
  readonly nowMs: number;
},): readonly RateLimitSnapshot[] {
  /**
   Headers keyed by lowercase name so providers may vary casing.
   */
  const normalizedHeaders = normalizeHeaders(headers,);

  return [
    ...parseAnthropicRateLimitSnapshots({
      headers: normalizedHeaders,
      nowMs,
    },),
    ...parseCodexRateLimitSnapshots({
      headers: normalizedHeaders,
      nowMs,
    },),
    ...parseSyntheticRateLimitSnapshots({
      headers: normalizedHeaders,
      nowMs,
    },),
  ];
}

export { normalizeHeaders, } from './rate-limit-parse-helpers.ts';

export { parseRateLimitSnapshots, };
