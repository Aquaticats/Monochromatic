/**
 Turn provider quota facts into bounded retry windows, not estimates of free slots.
 @module
 */

import type { AssistantMessage, } from '@earendil-works/pi-ai';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { describeCaught, } from './error-class-name.ts';
import {
  digitsEnd,
  foldProviderText,
  HTTP_RATE_LIMIT_STATUS,
  isTransientStatus,
  leadingStatus,
} from './provider-wire-text.ts';

//region Provider error and timestamp formats

/**
 Failure fields used for evidence decisions, independent of mutability of the original Pi message.
 */
type FailureEvidence = Pick<Readonly<AssistantMessage>, 'errorMessage' | 'stopReason'>;

/**
 Logger for refusal-body parsing, which never logs refusal text or credentials.
 */
const l = tagged({ tag: 'rate-limit-fallback', },);

/**
 Milliseconds per second, for HTTP and header timestamps.
 */
const SECOND_MS = 1_000;

/**
 First HTTP status that refuses a request, where a `Retry-After` header applies.
 */
const HTTP_REFUSAL_START = 400;

/**
 Alibaba Cloud's suggested pause for a request-rate error, not a quota cap.

 Quoted fragments from the Token Plan FAQ, section "Rate limiting and concurrency" and its error
 table: the refusal is named `429 Requests rate limit exceeded`, and the advice is "Wait about 1
 minute before retrying" (https://www.alibabacloud.com/help/en/model-studio/token-plan-faq).
 */
const QWEN_REQUEST_RATE_WAIT_MS = 60_000;

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
 Whether the failure is a refusal the pool may hand to the next member because the provider is
 temporarily unable to serve it: HTTP 503, or one of Cloudflare's origin statuses 520 to 526.

 The status comes from Pi's adapter when it exposed one, and otherwise from the status the refusal's
 own text opens with, exactly as a 429's does.

 @param message - terminal assistant result, which never represents a successful reply here

 @param status - actual HTTP status when Pi's adapter exposed it

 @returns whether dispatch may try the next member

 @example
 ```ts
 isTransientRefusal({ message: { stopReason: 'error', errorMessage: '523: <html>' } }); // true
 ```
 */
export function isTransientRefusal({
  message,
  status,
}: {
  readonly message: FailureEvidence;
  readonly status?: number;
},): boolean {
  if (message.stopReason !== 'error')
    return false;
  /**
   Status the adapter exposed, or the one the refusal text opens with.
   */
  const refusalStatus = status ?? leadingStatus(message.errorMessage ?? '',);
  return (refusalStatus !== 0) && isTransientStatus(refusalStatus,);
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
    && ((status === HTTP_RATE_LIMIT_STATUS)
      || ((status === undefined) && (leadingStatus(message.errorMessage ?? '',) === HTTP_RATE_LIMIT_STATUS)));
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
 Longest delay a refusal body may set.

 The provider's own documented retry strategies name at most 60 seconds
 (`max_delay_s: 60` on its uncached-TPM case, https://docs.neuralwatt.com/guides/rate-limits.md,
 "Uncached TPM Limits"), so a longer suggestion is a wrong or hostile number rather than a hint,
 and holding a member past that would hide one that had recovered.
 */
const BODY_DELAY_CAP_MS = 60_000;

/**
 Delay a refusal asks for in its own body.

 NeuralWatt's recorded overage refusal carries `retry_strategy.suggested_initial_delay_s`
 (`OVERAGE_REFUSAL` in the quota suite, copied from the capture), and the provider's guide asks
 clients to honour it: "The error-specific `retry_strategy` provides additional guidance"
 (https://docs.neuralwatt.com/guides/rate-limits.md, "Retry advice"). The plugin prefers this
 provider-given number over its own documented-wording guess.

 @param message - terminal rate-limit result, whose text usually carries the JSON body

 @param now - time refusal arrived

 @returns retry time, or zero when the body names no usable delay
 */
function bodyDelay({
  message,
  now,
}: {
  readonly message: FailureEvidence;
  readonly now: number;
},): number {
  /**
   Logger naming this function, so an unreadable body is attributable.
   */
  const rl = tagged({
    tag: bodyDelay.name,
    l,
  },);
  /**
   Refusal text as the adapter reported it.
   */
  const text = message.errorMessage ?? '';
  if (!text.includes('{',))
    return 0;
  /**
   Start of the JSON body a refusal carries.
   */
  const bodyStart = text.indexOf('{',);
  try {
    /**
     Parsed body the refusal carried.
     */
    const body: unknown = JSON.parse(text.slice(bodyStart,),);
    /**
     Provider's retry guidance for this refusal.
     */
    const strategy = field({
      value: body,
      key: 'retry_strategy',
    },);
    /**
     Delay that guidance names, in seconds.
     */
    const seconds = finiteNumber(field({
      value: strategy,
      key: 'suggested_initial_delay_s',
    },),);
    return (Number.isFinite(seconds,) && (seconds > 0))
      ? now + Math.min(
        seconds * SECOND_MS,
        BODY_DELAY_CAP_MS,
      )
      : 0;
  } catch (error) {
    // A truncated or non-JSON body names no delay; only the error class is logged, because the
    // refusal text may carry provider or credential detail.
    rl.debug(`Refusal body unreadable: ${describeCaught(error,)}`,);
    return 0;
  }
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
   Explicit retry hint, read from any refused HTTP request: a 429 and a 503 or Cloudflare origin
   failure alike name when the request may return.
   */
  const retryAfter = status >= HTTP_REFUSAL_START ? headers.get('retry-after',) ?? '' : '';
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
 Retry time a refusal gives, preferring the provider's own number over a documented wording guess.

 The body's `retry_strategy.suggested_initial_delay_s` comes first, because the provider stated it for
 this refusal; Qwen's documented request-rate pause applies only when no body names a delay.

 @param provider - provider that refused

 @param message - terminal rate-limit result

 @param now - time refusal arrived

 @returns retry time, or zero when neither source applies

 @example
 ```ts
 refusalReset({ provider: 'qwen-token-plan-individual', message: { stopReason: 'error' }, now: 0 }); // 0
 ```
 */
export function refusalReset({
  provider,
  message,
  now,
}: {
  readonly provider: string;
  readonly message: FailureEvidence;
  readonly now: number;
},): number {
  /**
   Delay the refusal body itself names, which outranks any published wording.
   */
  const fromBody = bodyDelay({
    message,
    now,
  },);
  if (fromBody > now)
    return fromBody;
  return (provider === 'qwen-token-plan-individual')
    && foldProviderText(message.errorMessage ?? '',)
      .includes('requests rate limit exceeded',)
    ? now + QWEN_REQUEST_RATE_WAIT_MS
    : 0;
}

//endregion Response header evidence
