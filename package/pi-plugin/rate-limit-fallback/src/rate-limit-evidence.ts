/**
 Turn provider quota facts into bounded retry windows, not estimates of free slots.
 @module
 */

import type { AssistantMessage, } from '@earendil-works/pi-ai';

//region Provider error and timestamp formats

/** Milliseconds per second, for HTTP and header timestamps. */
const SECOND_MS = 1_000;

/** Alibaba Cloud's suggested pause for a request-rate error, not a quota cap. */
const QWEN_REQUEST_RATE_WAIT_MS = 60 * SECOND_MS;

/** Pi's adapters retain an HTTP status at the start of many terminal error messages. */
const HTTP_429 = /^(?:HTTP[ \t]+|status(?: code)?:[ \t]*)?429(?:[ \t:]|$)/iu;

/** Safety and content-policy errors must not be interpreted as temporary capacity failures. */
const POLICY_STOP = /possible risk|content[_ -]filter|safety[_ -](?:policy|violation)|policy[_ -]violation/iu;

/**
 Match a model-service policy stop, independently of coincidental status or retry text.
 @param message - failed provider message
 @returns whether Pi should see a non-retryable policy error
 */
export function isPolicyStop(message: AssistantMessage,): boolean {
  return message.stopReason === 'error' && POLICY_STOP.test(message.errorMessage ?? '',);
}

/**
 Identify a terminal HTTP rate limit without treating text that merely mentions 429 as a status.
 @param message - terminal assistant result, which never represents a successful reply here
 @param status - actual HTTP status when Pi's adapter exposed it
 @returns whether dispatch may consider another provider
 */
export function isRateLimit({
  message,
  status,
}: {
  message: AssistantMessage;
  status: number | undefined;
},): boolean {
  return message.stopReason === 'error'
    && (status === 429 || (status === undefined && HTTP_429.test(message.errorMessage ?? '',)));
}

/**
 Parse a reset from a provider-supplied date.
 @param value - candidate date
 @returns epoch milliseconds, or zero for unavailable evidence
 */
export function resetTime(value: unknown,): number {
  if (typeof value !== 'string')
    return 0;
  const time = Date.parse(value,);
  return Number.isFinite(time,) ? time : 0;
}

/**
 Read finite numeric JSON without coercing absent or null values to zero.
 @param value - untrusted JSON value
 @returns usable number if present
 */
export function finiteNumber(value: unknown,): number | undefined {
  return typeof value === 'number' && Number.isFinite(value,) ? value : undefined;
}

/**
 Read one property without accepting primitive data as a record.
 @param value - untrusted JSON value
 @param key - property to inspect
 @returns field if present
 */
export function field({ value, key, }: { value: unknown; key: string; },): unknown {
  if (typeof value !== 'object' || value === null || !(key in value))
    return undefined;
  return Reflect.get(value, key,);
}

/**
 Parse Synthetic's overlapping quota windows. A snapshot can defer a request only when it
 both reports exhaustion and gives a time to check again.
 @param quota - parsed response or response-header body
 @returns latest reported retry time among exhausted windows
 */
export function syntheticReset(quota: unknown,): number {
  const subscription = field({ value: quota, key: 'subscription', },);
  const requests = finiteNumber(field({ value: subscription, key: 'requests', },),);
  const limit = finiteNumber(field({ value: subscription, key: 'limit', },),);
  const subscriptionReset = requests !== undefined && limit !== undefined && limit > 0 && requests >= limit
    ? resetTime(field({ value: subscription, key: 'renewsAt', },),)
    : 0;
  const weekly = field({ value: quota, key: 'planWeeklyLimit', },)
    ?? field({ value: quota, key: 'weeklyTokenLimit', },);
  const weeklyReset = field({ value: weekly, key: 'percentRemaining', },) === 0
    ? resetTime(field({ value: weekly, key: 'nextRegenAt', },),)
    : 0;
  return Math.max(subscriptionReset, weeklyReset,);
}

//endregion Provider error and timestamp formats

//region Response header evidence

/** Per-unit multipliers for durations such as OpenAI's `6m0s`. */
const RESET_UNITS = {
  ms: 1,
  s: SECOND_MS,
  m: 60 * SECOND_MS,
  h: 60 * 60 * SECOND_MS,
  d: 24 * 60 * 60 * SECOND_MS,
} as const;

/**
 Parse an OpenAI duration without accepting a partially parsed value.
 @param value - documented reset duration
 @returns duration in milliseconds, or zero if malformed
 */
function resetDuration(value: string,): number {
  const token = /(\d+(?:\.\d+)?)(ms|s|m|h|d)/gyu;
  let cursor = 0;
  let duration = 0;
  while (cursor < value.length) {
    const match = token.exec(value,);
    if (match === null)
      return 0;
    const unit = match[2];
    if (unit !== 'ms' && unit !== 's' && unit !== 'm' && unit !== 'h' && unit !== 'd')
      return 0;
    duration += Number(match[1],) * RESET_UNITS[unit];
    cursor = token.lastIndex;
  }
  return Number.isFinite(duration,) ? duration : 0;
}

/**
 Use the response's published timing when Pi exposes it.
 @param provider - provider whose quota contract applies
 @param headers - case-insensitive response headers, scoped to this attempt
 @param now - time response arrived
 @param status - HTTP status, so Retry-After is interpreted only on a refusal
 @returns latest reset time named by a depleted limit, or zero
 */
export function headerReset({
  provider,
  headers,
  now,
  status,
}: {
  provider: string;
  headers: Headers;
  now: number;
  status: number;
},): number {
  const retryAfter = status === 429 ? headers.get('retry-after',) ?? '' : '';
  const retryReset = /^\d+$/u.test(retryAfter,)
    ? now + Number(retryAfter,) * SECOND_MS
    : resetTime(retryAfter,);
  const validRetry = Number.isFinite(new Date(retryReset,).getTime(),) ? retryReset : 0;
  if (provider !== 'openai')
    return validRetry;
  const requests = headers.get('x-ratelimit-remaining-requests',) === '0'
    ? now + resetDuration(headers.get('x-ratelimit-reset-requests',) ?? '',)
    : 0;
  const tokens = headers.get('x-ratelimit-remaining-tokens',) === '0'
    ? now + resetDuration(headers.get('x-ratelimit-reset-tokens',) ?? '',)
    : 0;
  const reset = Math.max(validRetry, requests, tokens,);
  return Number.isFinite(new Date(reset,).getTime(),) ? reset : validRetry;
}

/**
 Apply Qwen's documented request-rate recommendation only if no response reset was available.
 @param provider - provider that refused
 @param message - terminal rate-limit result
 @param now - time refusal arrived
 @returns retry time, or zero when the guide does not apply
 */
export function recommendedReset({
  provider,
  message,
  now,
}: {
  provider: string;
  message: AssistantMessage;
  now: number;
},): number {
  return provider === 'qwen-token-plan-individual'
    && /requests rate limit exceeded/iu.test(message.errorMessage ?? '',)
    ? now + QWEN_REQUEST_RATE_WAIT_MS
    : 0;
}

//endregion Response header evidence
