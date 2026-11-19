/**
 Turn provider quota facts into bounded retry windows, not estimates of free slots.
 @module
 */

import type { AssistantMessage, } from '@earendil-works/pi-ai';
import {
  digitsEnd,
  foldProviderText,
  HTTP_RATE_LIMIT_STATUS,
} from './provider-wire-text.ts';

//region Provider error and timestamp formats

/**
 Failure fields used for evidence decisions, independent of mutability of the original Pi message.
 */
type FailureEvidence = Pick<Readonly<AssistantMessage>, 'errorMessage' | 'stopReason'>;

/**
 Milliseconds per second, for HTTP and header timestamps.
 */
const SECOND_MS = 1_000;

/**
 Alibaba Cloud's suggested pause for a request-rate error, not a quota cap.

 Quoted fragments from the Token Plan FAQ, section "Rate limiting and concurrency" and its error
 table: the refusal is named `429 Requests rate limit exceeded`, and the advice is "Wait about 1
 minute before retrying" (https://www.alibabacloud.com/help/en/model-studio/token-plan-faq).
 */
const QWEN_REQUEST_RATE_WAIT_MS = 60_000;

/**
 Status text used by adapters when no structured status is available.
 */
const RATE_LIMIT_STATUS = String(HTTP_RATE_LIMIT_STATUS,);

/**
 Pi's OpenAI Responses adapter formats extracted statuses as `OpenAI API error (<status>):`.
 */
const OPENAI_RESPONSES_RATE_LIMIT = `openai api error (${RATE_LIMIT_STATUS}):`;

/**
 Distinguishes OpenAI's policy stop from a 429 that only mentions risk.
 */
const OPENAI_POLICY_OPENING = 'this content was flagged for possible ';

/**
 Policy labels that accept a single space, underscore, or hyphen as the separator.

 Stand-ins with no producer: neither an installed adapter nor any recorded provider message on this
 machine emits them. They stay as a conservative name for a content-stop refusal; the move on before
 output is gated on OpenAI's opening alone.
 */
const POLICY_LABELS = [
  'content filter',
  'safety policy',
  'safety violation',
  'policy violation',
] as const;

/**
 Check the leading status token, without treating an unrelated mention of 429 as a failure status.

 A recorded provider message begins `429 "…"` as well as `429: {…}`, so a space, tab, colon or the
 end of the text closes the status token; a digit or letter after it means the number is not a status.

 @param value - terminal error text from the adapter

 @returns whether the message starts with HTTP 429
 */
function startsWithRateLimit(value: string,): boolean {
  /**
   Case-folded terminal message, with no leading whitespace removed.
   */
  const message = foldProviderText(value,);
  if (message.startsWith(OPENAI_RESPONSES_RATE_LIMIT,))
    return true;
  if (!message.startsWith(RATE_LIMIT_STATUS,))
    return false;
  /**
   Character after the status token, absent when the status is the whole message.
   */
  const boundary = message[RATE_LIMIT_STATUS.length];
  return (boundary === undefined) || (boundary === ':')
    || (boundary === ' ')
    || (boundary === '\t');
}

/**
 Whether the failure opens with OpenAI's own policy-stop sentence.

 Only this sentence moves a request on to the next member: another vendor's content stop, and a
 refusal that merely quotes the sentence, end the request at the member that produced them. The
 match is anchored because OpenAI's stock refusal begins with the sentence in both recorded captures
 (`policy-stop-fixture.ts`), while a 429 body or a quoting refusal carries it later in the text.

 @param message - failed provider message

 @returns whether the folded text starts with OpenAI's stock refusal

 @example
 ```ts
 hasOpenAiPolicyOpening({ stopReason: 'error', errorMessage: 'This content was flagged for possible risk' }); // true
 ```
 */
export function hasOpenAiPolicyOpening(message: FailureEvidence,): boolean {
  return (message.stopReason === 'error')
    && foldProviderText(message.errorMessage ?? '',)
      .startsWith(OPENAI_POLICY_OPENING,);
}

/**
 Match policy text without misreading an HTTP 429 that merely mentions a policy or risk.

 @param message - failed provider message

 @param rateLimited - whether the adapter text or response supplied a 429 status

 @returns whether Pi should see a non-retryable policy error

 @example
 ```ts
 isPolicyStop({ message: { stopReason: 'error', errorMessage: 'Possible risk' }, rateLimited: false });
 ```
 */
export function isPolicyStop({
  message,
  rateLimited,
}: {
  readonly message: FailureEvidence;
  readonly rateLimited: boolean;
},): boolean {
  if (message.stopReason !== 'error')
    return false;
  /**
   Case-folded text with original spacing for the OpenAI refusal wording.
   */
  const text = foldProviderText(message.errorMessage ?? '',);
  // The stock sentence still outranks an explicit HTTP 429 wherever it sits, so a 429 body that
  // quotes it is a non-retryable stop rather than a rate limit; only the move needs it at the start.
  if (text.includes(OPENAI_POLICY_OPENING,))
    return true;
  if (rateLimited)
    return false;
  if (text.includes('possible risk',))
    return true;
  /**
   The other policy labels also accept a single underscore or hyphen separator.
   */
  const labels = text.replaceAll(
    '_',
    ' ',
  )
    .replaceAll(
    '-',
    ' ',
    );
  return POLICY_LABELS.some(function matchesPolicy(label,) {
    return labels.includes(label,);
  },);
}

/**
 Identify a terminal HTTP rate limit without treating text that merely mentions 429 as a status.

 @param message - terminal assistant result, which never represents a successful reply here

 @param status - actual HTTP status when Pi's adapter exposed it

 @returns whether dispatch may consider another provider

 @example
 ```ts
 isRateLimit({ message: { stopReason: 'error', errorMessage: '429: {"message":"Too many requests"}' } }); // true
 ```
 */
export function isRateLimit({
  message,
  status,
}: {
  readonly message: FailureEvidence;
  readonly status?: number;
},): boolean {
  return (message.stopReason === 'error')
    && ((status === HTTP_RATE_LIMIT_STATUS) || ((status === undefined) && startsWithRateLimit(message.errorMessage ?? '',)));
}

/**
 Parse a reset from a provider-supplied date.

 @param value - candidate date

 @returns epoch milliseconds, or zero for unavailable evidence

 @example
 ```ts
 resetTime('1970-01-01T00:00:01Z'); // 1000
 ```
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

 @example
 ```ts
 finiteNumber('7'); // NaN
 ```
 */
export function finiteNumber(value: unknown,): number {
  return ((typeof value) === 'number') && Number.isFinite(value,) ? value : Number.NaN;
}

/**
 Read one property without accepting primitive data as a record.

 @param value - untrusted JSON value

 @param key - property to inspect

 @returns field if present

 @example
 ```ts
 field({ value: { remaining: 0 }, key: 'remaining' }); // 0
 ```
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

 @example
 ```ts
 syntheticReset({ planWeeklyLimit: { percentRemaining: 1 } }); // 0
 ```
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
 Milliseconds per minute in a reset duration.
 */
const MINUTE_MS = 60_000;

/**
 Milliseconds per hour in a reset duration.
 */
const HOUR_MS = 3_600_000;

/**
 Milliseconds per day in a reset duration.
 */
const DAY_MS = 86_400_000;

/**
 Per-unit multipliers for durations such as OpenAI's `6m0s`.
 */
const RESET_UNITS = {
  ms: 1,
  s: SECOND_MS,
  m: MINUTE_MS,
  h: HOUR_MS,
  d: DAY_MS,
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
  const scan = {
    offset: 0,
    total: 0,
  };
  while (scan.offset < value.length) {
    /**
     End of the mandatory integer part.
     */
    const wholeEnd = digitsEnd({
      value,
      offset: scan.offset,
    },);
    if (wholeEnd === scan.offset)
      return 0;
    /**
     End of the number, including any fractional digits.
     */
    const amountEnd = value[wholeEnd] === '.'
      ? digitsEnd({
        value,
        offset: wholeEnd + 1,
      },)
      : wholeEnd;
    if ((value[wholeEnd] === '.') && (amountEnd === (wholeEnd
      + 1)))
      return 0;
    /**
     Unit suffix, with milliseconds taking precedence over minutes.
     */
    const unit = value.startsWith(
      'ms',
      amountEnd,
    ) ? 'ms' : value[amountEnd];
    if ((unit !== 'ms') && (unit !== 's')
      && (unit !== 'm')
      && (unit !== 'h')
      && (unit !== 'd'))
      return 0;
    scan.total += Number(value.slice(
      scan.offset,
      amountEnd,
    ),) * RESET_UNITS[unit];
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

 @example
 ```ts
 headerReset({ provider: 'synthetic', headers: new Headers({ 'Retry-After': '1' }), now: 0, status: 429 });
 ```
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
  const retryAfter = status === HTTP_RATE_LIMIT_STATUS ? headers.get('retry-after',) ?? '' : '';
  /**
   Retry-After converted to an absolute timestamp.
   */
  const retryReset = (retryAfter.length > 0) && (digitsEnd({
    value: retryAfter,
    offset: 0,
  },) === retryAfter.length)
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

 The wording is documented, not captured: no recorded `modelError` on this machine carries it, and no
 installed source builds it, so a change in Qwen's wording silently disables this rule. The rationale
 is the Token Plan FAQ sentence quoted above `QWEN_REQUEST_RATE_WAIT_MS`.

 @param provider - provider that refused

 @param message - terminal rate-limit result

 @param now - time refusal arrived

 @returns retry time, or zero when the guide does not apply

 @example
 ```ts
 recommendedReset({ provider: 'openai', message: { stopReason: 'error' }, now: 0 }); // 0
 ```
 */
export function recommendedReset({
  provider,
  message,
  now,
}: {
  readonly provider: string;
  readonly message: FailureEvidence;
  readonly now: number;
},): number {
  return (provider === 'qwen-token-plan-individual')
    && foldProviderText(message.errorMessage ?? '',)
      .includes('requests rate limit exceeded',)
    ? now + QWEN_REQUEST_RATE_WAIT_MS
    : 0;
}

//endregion Response header evidence
