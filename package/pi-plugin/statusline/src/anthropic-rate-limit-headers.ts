/**
 Anthropic token rate-limit response header parsing.
 
 @module
 */

import {
  PERCENT_BASE,
  RATE_LIMIT_WINDOW_SECONDS,
  type RateLimitHeaderFamily,
  type RateLimitSnapshot,
} from './rate-limit-types.ts';
import {
  INVALID_RATE_LIMIT_SNAPSHOT,
  INVALID_VALUE,
  clampNumber,
  createRateLimitSnapshot,
  isRateLimitSnapshot,
  parseNumberHeader,
  parseResetHeader,
  type InvalidRateLimitSnapshot,
} from './rate-limit-parse-helpers.ts';

//region Anthropic header families

/**
 Anthropic header groups that describe token usage capacity.
 
 Request-count headers are intentionally omitted because this extension ports
 usage-capacity projection warnings, not request throughput warnings.
 */
const ANTHROPIC_RATE_LIMIT_HEADER_FAMILIES: readonly RateLimitHeaderFamily[] = [
  {
    key: 'anthropic:tokens',
    label: 'anthropic tokens',
    limitHeader: 'anthropic-ratelimit-tokens-limit',
    remainingHeader: 'anthropic-ratelimit-tokens-remaining',
    resetHeader: 'anthropic-ratelimit-tokens-reset',
    windowSeconds: RATE_LIMIT_WINDOW_SECONDS,
  },
  {
    key: 'anthropic:input',
    label: 'anthropic input',
    limitHeader: 'anthropic-ratelimit-input-tokens-limit',
    remainingHeader: 'anthropic-ratelimit-input-tokens-remaining',
    resetHeader: 'anthropic-ratelimit-input-tokens-reset',
    windowSeconds: RATE_LIMIT_WINDOW_SECONDS,
  },
  {
    key: 'anthropic:output',
    label: 'anthropic output',
    limitHeader: 'anthropic-ratelimit-output-tokens-limit',
    remainingHeader: 'anthropic-ratelimit-output-tokens-remaining',
    resetHeader: 'anthropic-ratelimit-output-tokens-reset',
    windowSeconds: RATE_LIMIT_WINDOW_SECONDS,
  },
  {
    key: 'anthropic:unified',
    label: 'anthropic unified',
    limitHeader: 'anthropic-ratelimit-unified-tokens-limit',
    remainingHeader: 'anthropic-ratelimit-unified-tokens-remaining',
    resetHeader: 'anthropic-ratelimit-unified-tokens-reset',
    windowSeconds: RATE_LIMIT_WINDOW_SECONDS,
  },
  {
    key: 'anthropic:priority-input',
    label: 'anthropic priority input',
    limitHeader: 'anthropic-priority-input-tokens-limit',
    remainingHeader: 'anthropic-priority-input-tokens-remaining',
    resetHeader: 'anthropic-priority-input-tokens-reset',
    windowSeconds: RATE_LIMIT_WINDOW_SECONDS,
  },
  {
    key: 'anthropic:priority-output',
    label: 'anthropic priority output',
    limitHeader: 'anthropic-priority-output-tokens-limit',
    remainingHeader: 'anthropic-priority-output-tokens-remaining',
    resetHeader: 'anthropic-priority-output-tokens-reset',
    windowSeconds: RATE_LIMIT_WINDOW_SECONDS,
  },
];

//endregion Anthropic header families

//region Single family parsing

/**
 Parses one Anthropic header family into a usage snapshot.
 
 @param family - {@link RateLimitHeaderFamily} metadata
 
 @param headers - lowercase provider response headers
 
 @param nowMs - wall-clock sample time in epoch milliseconds
 
 @returns parsed snapshot, or invalid sentinel when any required header is unusable
 
 @example
 ```ts
 parseAnthropicRateLimitSnapshot({ family, headers, nowMs: Date.now() });
 ```
 */
function parseAnthropicRateLimitSnapshot({
  family,
  headers,
  nowMs,
}: Readonly<{
  family: RateLimitHeaderFamily;
  headers: Readonly<Record<string, string>>;
  nowMs: number;
}>,): RateLimitSnapshot | InvalidRateLimitSnapshot {
  /**
   Limiter capacity from `*-limit` header.
   */
  const limit = parseNumberHeader({
    headers,
    headerName: family.limitHeader,
  },);
  /**
   Remaining capacity from `*-remaining` header.
   */
  const rawRemaining = parseNumberHeader({
    headers,
    headerName: family.remainingHeader,
  },);
  /**
   Reset timestamp from `*-reset` header.
   */
  const resetAtMs = parseResetHeader({
    headers,
    headerName: family.resetHeader,
  },);

  if ((limit === INVALID_VALUE)
    || (rawRemaining === INVALID_VALUE)
    || (resetAtMs === INVALID_VALUE)
    || (limit <= 0))
    return INVALID_RATE_LIMIT_SNAPSHOT;

  /**

   Remaining capacity clamped because some headers are rounded.

   */
  const remaining = clampNumber({
    value: rawRemaining,
    min: 0,
    max: limit,
  },);
  /**
   Used capacity expressed as percentage of capacity.
   */
  const usedPercent = ((limit - remaining) / limit) * PERCENT_BASE;

  return createRateLimitSnapshot({
    key: family.key,
    label: family.label,
    resetAtMs,
    windowSeconds: family.windowSeconds,
    paceScale: 1,
    sampledAtMs: nowMs,
    usedPercent,
  },);
}

//endregion Single family parsing

//region Header collection parsing

/**
 Parses supported Anthropic rate-limit header groups.
 
 @param headers - lowercase provider response headers
 
 @param nowMs - wall-clock sample time in epoch milliseconds
 
 @returns parsed {@link RateLimitSnapshot} entries for complete and valid header groups
 
 @example
 ```ts
 parseAnthropicRateLimitSnapshots({ headers, nowMs: Date.now() });
 ```
 */
function parseAnthropicRateLimitSnapshots({
  headers,
  nowMs,
}: Readonly<{
  headers: Readonly<Record<string, string>>;
  nowMs: number;
}>,): readonly RateLimitSnapshot[] {
  return ANTHROPIC_RATE_LIMIT_HEADER_FAMILIES
    .map(function parseFamily(family,): RateLimitSnapshot | InvalidRateLimitSnapshot {
      return parseAnthropicRateLimitSnapshot({
        family,
        headers,
        nowMs,
      },);
    },)
    .filter(function keepSnapshot(snapshot,): snapshot is RateLimitSnapshot {
      return isRateLimitSnapshot(snapshot,);
    },);
}

//endregion Header collection parsing

export {
  ANTHROPIC_RATE_LIMIT_HEADER_FAMILIES,
  parseAnthropicRateLimitSnapshots,
};
