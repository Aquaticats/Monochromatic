/**
 Turn provider quota facts into bounded retry windows, not estimates of free slots.
 @module
 */

import type { AssistantMessage, } from '@earendil-works/pi-ai';
import {
  digitsEnd,
  foldProviderText,
} from './provider-wire-text.ts';

//region Provider error and timestamp formats

/**
 Milliseconds per second, for HTTP and header timestamps.
 */
const SECOND_MS = 1_000;

/**
 Alibaba Cloud's suggested pause for a request-rate error, not a quota cap.
 */
const QWEN_REQUEST_RATE_WAIT_MS = 60 * SECOND_MS;

/**
 Status text used by adapters when no structured status is available.
 */
const RATE_LIMIT_STATUS = '429';

/**
 Policy labels preserve single separators and the exact spacing of the possible-risk message.
 */
const POLICY_LABELS = [
  'content filter',
  'safety policy',
  'safety violation',
  'policy violation',
] as const;

/**
 Check the leading status token, without treating an unrelated mention of 429 as a failure status.

 @param value - terminal error text from the adapter

 @returns whether the message starts with HTTP 429
 */
function startsWithRateLimit(value: string,): boolean {
  /**
   Case-folded terminal message, with no leading whitespace removed.
   */
  const message = foldProviderText(value,);
  /**
   Status prefixes accepted by the adapter boundary.
   */
  const prefixes = ['status code:', 'status:', 'http',] as const;
  /**
   Matched prefix, absent if the message starts directly with the status.
   */
  const prefix = prefixes.find(function matchesPrefix(candidate,) {
    return message.startsWith(candidate,);
  },);
  /**
   Start of the horizontal whitespace following the prefix.
   */
  const afterPrefix = prefix?.length ?? 0;
  if (prefix === 'http' && message[afterPrefix] !== ' ' && message[afterPrefix] !== '\t')
    return false;
  for (let offset = afterPrefix; offset <= message.length; offset += 1) {
    if (prefix !== undefined && (message[offset] === ' ' || message[offset] === '\t'))
      continue;
    if (!message.startsWith(RATE_LIMIT_STATUS, offset,))
      return false;
    /**
     Boundary after the status, to exclude codes with additional digits or letters.
     */
    const boundary = message[offset + RATE_LIMIT_STATUS.length];
    return boundary === undefined || boundary === ':' || boundary === ' ' || boundary === '\t';
  }
  return false;
}

/**
 Match a model-service policy stop, independently of coincidental status or retry text.

 @param message - failed provider message

 @returns whether Pi should see a non-retryable policy error
 */
export function isPolicyStop(message: AssistantMessage,): boolean {
  if (message.stopReason !== 'error')
    return false;
  /**
   Case-folded text preserves the space in the possible-risk phrase.
   */
  const text = foldProviderText(message.errorMessage ?? '',);
  if (text.includes('possible risk',))
    return true;
  /**
   The other policy labels also accept a single underscore or hyphen separator.
   */
  const labels = text.replaceAll('_', ' ',).replaceAll('-', ' ',);
  return POLICY_LABELS.some(function matchesPolicy(label,) {
    return labels.includes(label,);
  },);
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
  status?: number;
},): boolean {
  return (message.stopReason === 'error')
    && ((status === 429) || ((status === undefined) && startsWithRateLimit(message.errorMessage ?? '',)));
}

/**
 Parse a reset from a provider-supplied date.

 @param value - candidate date

 @returns epoch milliseconds, or zero for unavailable evidence
 */
export function resetTime(value: unknown,): number {
  if ((typeof value) !== 'string')
    return 0;
  /**
   Parsed epoch time before rejecting unavailable date strings.
   */
  const time = Date.parse(value,);
  return Number.isFinite(time,) ? time : 0;
}

/**
 Read finite numeric JSON without coercing absent or null values to zero.

 @param value - untrusted JSON value

 @returns finite number, or NaN if the value is not a finite number
 */
export function finiteNumber(value: unknown,): number {
  return ((typeof value) === 'number') && Number.isFinite(value,) ? value : Number.NaN;
}

/**
 Read one property without accepting primitive data as a record.

 @param value - untrusted JSON value

 @param key - property to inspect

 @returns field if present
 */
export function field({
  value,
  key,
}: {
  value: unknown;
  key: string
},): unknown {
  if (((typeof value) !== 'object') || (value === null)
    || (!(key in value)))
    return undefined;
  return Reflect.get(
    value,
    key,
  );
}

/**
 Parse Synthetic's overlapping quota windows. A snapshot can defer a request only when it
 both reports exhaustion and gives a time to check again.

 @param quota - parsed response or response-header body

 @returns latest reported retry time among exhausted windows
 */
export function syntheticReset(quota: unknown,): number {
  /**
   Provider quota block for the paid subscription window.
   */
  const subscription = field({
    value: quota,
    key: 'subscription',
  },);
  /**
   Usage evidence for the request-count window.
   */
  const requests = finiteNumber(field({
    value: subscription,
    key: 'requests',
  },),);
  /**
   Published request allowance for the subscription window.
   */
  const limit = finiteNumber(field({
    value: subscription,
    key: 'limit',
  },),);
  /**
   Renewal time only when the subscription allowance is depleted.
   */
  const subscriptionReset = (limit > 0)
    && (requests >= limit)
    ? resetTime(field({
      value: subscription,
      key: 'renewsAt',
    },),)
    : 0;
  /**
   Weekly block under the current or previous Synthetic field name.
   */
  const weekly = field({
    value: quota,
    key: 'planWeeklyLimit',
  },)
    ?? field({
      value: quota,
      key: 'weeklyTokenLimit',
    },);
  /**
   Regeneration time only when weekly quota is depleted.
   */
  const weeklyReset = field({
    value: weekly,
    key: 'percentRemaining',
  },) === 0
    ? resetTime(field({
      value: weekly,
      key: 'nextRegenAt',
    },),)
    : 0;
  return Math.max(
    subscriptionReset,
    weeklyReset,
  );
}

//endregion Provider error and timestamp formats

//region Response header evidence

/**
 Per-unit multipliers for durations such as OpenAI's `6m0s`.
 */
const RESET_UNITS = {
  ms: 1,
  s: SECOND_MS,
  m: 60 * SECOND_MS,
  h: 60 * 60
    * SECOND_MS,
  d: 24 * 60
    * 60
    * SECOND_MS,
} as const;

/**
 Parse an OpenAI duration without accepting a partially parsed value.

 @param value - documented reset duration

 @returns duration in milliseconds, or zero if malformed
 */
function resetDuration(value: string,): number {
  /**
   Position of the next component and total milliseconds of all accepted components.
   */
  const scan = { offset: 0, total: 0, };
  while (scan.offset < value.length) {
    /**
     End of the mandatory integer part.
     */
    const wholeEnd = digitsEnd({ value, offset: scan.offset, },);
    if (wholeEnd === scan.offset)
      return 0;
    /**
     End of the number, including any fractional digits.
     */
    const amountEnd = value[wholeEnd] === '.'
      ? digitsEnd({ value, offset: wholeEnd + 1, },)
      : wholeEnd;
    if (value[wholeEnd] === '.' && amountEnd === wholeEnd + 1)
      return 0;
    /**
     Unit suffix, with milliseconds taking precedence over minutes.
     */
    const unit = value.startsWith('ms', amountEnd,) ? 'ms' : value[amountEnd];
    if (unit !== 'ms' && unit !== 's' && unit !== 'm' && unit !== 'h' && unit !== 'd')
      return 0;
    scan.total += Number(value.slice(scan.offset, amountEnd,),) * RESET_UNITS[unit];
    scan.offset = amountEnd + unit.length;
  }
  return Number.isFinite(scan.total,) ? scan.total : 0;
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
  /**
   Explicit retry hint, valid only on a refused HTTP request.
   */
  const retryAfter = status === 429 ? headers.get('retry-after',) ?? '' : '';
  /**
   Retry-After converted to an absolute timestamp.
   */
  const retryReset = retryAfter.length !== 0 && digitsEnd({ value: retryAfter, offset: 0, },) === retryAfter.length
    ? now + (Number(retryAfter,)
      * SECOND_MS)
    : resetTime(retryAfter,);
  /**
   Retry timestamp constrained to representable dates.
   */
  const validRetry = Number.isFinite(new Date(retryReset,).getTime(),) ? retryReset : 0;
  if (provider !== 'openai')
    return validRetry;
  /**
   Usage evidence for the request-count window.
   */
  const requests = headers.get('x-ratelimit-remaining-requests',) === '0'
    ? now + resetDuration(headers.get('x-ratelimit-reset-requests',) ?? '',)
    : 0;
  /**
   Reset for the token-count window, when depleted.
   */
  const tokens = headers.get('x-ratelimit-remaining-tokens',) === '0'
    ? now + resetDuration(headers.get('x-ratelimit-reset-tokens',) ?? '',)
    : 0;
  /**
   Latest deadline among depleted windows, before validating its date range.
   */
  const reset = Math.max(
    validRetry,
    requests,
    tokens,
  );
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
  return (provider === 'qwen-token-plan-individual')
    && foldProviderText(message.errorMessage ?? '',).includes('requests rate limit exceeded',)
    ? now + QWEN_REQUEST_RATE_WAIT_MS
    : 0;
}

//endregion Response header evidence
