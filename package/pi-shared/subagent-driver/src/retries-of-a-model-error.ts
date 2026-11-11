/**
 What kind of failure a model error is, whether another attempt can fix it, and how long the driver waits before one.

 The shapes read here come from the feeds of the runs in `${HOME}/temp/agent/pi-runs` on 2026-11-09: of 556 run folders, six
 summaries ended `model-error`, and every assistant message that ended in an error carried one of four texts:

 - a Cloudflare `520` body, twice, which names `"retryable":true`, `"retry_after":60` and says "**Wait and retry.** Back off for
   at least 60 seconds." (`cargo-pnpm-search-shadow-open-issues-1`, `policy-and-candidate-route-findings-on-main-1`);
 - `Request timed out.`, once (`repo-read-and-index-route-findings-on-main-1`);
 - `Provider is not configured: neuralwatt`, three times, which names a provider this machine has no key for, so another attempt
   reaches the same answer (`toml-edit-wrapped-values-validated-1` and two others);
 - `This operation was aborted`, five times, which is the abort a coordinator asks for, so answering it again would fight the
   coordinator (`btrfs-ladder-host-run-1` and others).

 A shape no list here recognises counts as a failure another attempt cannot fix, which is how such a run ends today: only a
 shape that is known to pass is waited out.

 A 429 in any form is not the driver's to answer: Pi's own automatic retry makes its attempts at a rate limit, a concurrency
 limit and a quota, so the driver neither re-prompts one nor waits one out. Which layer owns which errors is in `README.md`.

 Every text is folded to lower case once, and each shape is then a plain substring of it, so a failure of a few kilobytes is
 read in one pass and nothing here can backtrack. A status code is read only where the text starts with one, which is where a
 response writes it: the same digits inside a body name a token count or a limit the provider states, and neither is transient.

 @module
 */

import {
  FEED_PREVIEW_CHARACTERS,
  MILLISECONDS_PER_SECOND,
  MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS,
} from './driver-constants.ts';
import { preview, } from './previews-of-pi-messages.ts';

/**
 What kind of failure a model error is. The kind decides both whether another attempt can fix it and how the run counts it.
 */
export type ModelErrorKind =
  /**
   A provider server error, a request timeout, or a dropped connection: another attempt can fix it.
   */
  | 'transient'
  /**
   A rate limit that passes, such as HTTP 429: Pi's own automatic retry answers one, so the driver makes no attempt at it. The
   coordinator measures these apart from other failures all the same.
   */
    | 'throttled'
  /**
   A quota or a plan limit: written as a rate limit, but nothing refills it, so Pi's own retry leaves it too and the run ends on
   it. It is counted apart all the same.
   */
    | 'quota-reached'
  /**
   Everything else, such as a missing key, an unknown provider or model, a context overflow, or an abort the coordinator asked
   for: another attempt cannot fix it.
   */
    | 'permanent';

/**
 The wait a provider asked for, when its failure names one, or that it named none.
 */
export type WaitAskedByProvider =
  | { readonly asked: false }
    | {
      readonly asked: true;
      readonly milliseconds: number
    };

/**
 The failures another attempt cannot fix, whatever else their text says: a credential Pi does not have, a provider or a model
 that does not exist, a context too large for the model, and an abort someone asked for.

 These read before the transient shapes below, because the words of a transient failure turn up inside them: `Invalid API key for
 upstream provider` holds `upstream`, and `Context length exceeds the limit of 500 tokens` holds a server status as a count. A
 rate limit reads before them all the same: `429 Too many requests for this API key` names the key it limited, and the limit still
 passes.
 */
const TERMINAL_CAUSE_PHRASES: readonly string[] = [
  'api key',
  'api_key',
  'unauthorized',
  'unauthenticated',
  'authentication failed',
  'not configured',
  'no key for',
  'unknown provider',
  'unknown model',
  'model not found',
  'provider not found',
  'context length',
  'context window',
  'context overflow',
  'maximum context',
  'prompt is too long',
  'too many tokens',
  'reduce the length',
  'was aborted',
  'aborted by',
  'request was aborted',
  'interrupted by user',
  'cancelled by user',
  'canceled by user',
];

/**
 A limit another attempt cannot refill: the account, the plan or the billing has nothing left, and waiting out a backoff does not
 change that.

 These read before the rate-limit shapes below, because several of them are sent as HTTP 429.
 */
const QUOTA_REACHED_PHRASES: readonly string[] = [
  'quota',
  'out of budget',
  'billing',
  'usage limit reached',
  'usage_limit_exceeded',
  'available balance',
  'gousagelimiterror',
  'freeusagelimiterror',
];

/**
 A rate limit that passes: the provider asks the caller to slow down, and a later attempt may go through.

 This reads before the terminal causes, because a rate limit names what it limited: `429 Too many requests for this API key` is
 about the key being asked too often, not about a key that is wrong. A status the text starts with that refused the request
 reads before every phrase here all the same: a 401 whose body mentions rate limits is about the credential, and its own words
 about rate limits are what kept this list from being read first.
 */
const THROTTLE_PHRASES: readonly string[] = [
  'ratelimit',
  'rate limit',
  'rate_limit',
  'rate-limit',
  'too many requests',
  'throttl',
];

/**
 The status code of a throttle, read only where the failure text starts with one.
 */
const THROTTLE_CODE = '429';

/**
 The phrases a provider or a transport uses about a failure another attempt can fix, taken from the texts Pi itself treats as
 transient (`packages/ai/src/utils/retry.ts` in `earendil-works/pi`).
 */
const TRANSIENT_PHRASES: readonly string[] = [
  'overloaded',
  'server_busy',
  'servers are currently busy',
  'currently experiencing high demand',
  'model is at capacity',
  'server error',
  'server_error',
  'internal error',
  'internal_error',
  'service unavailable',
  'service is temporarily unavailable',
  'bad gateway',
  'gateway timeout',
  'timed out',
  'request timeout',
  'etimedout',
  'econnreset',
  'econnrefused',
  'connection reset',
  'connection lost',
  'connection was closed',
  'connection closed',
  'connection refused',
  'socket hang up',
  'other side closed',
  'fetch failed',
  'network error',
  'getaddrinfo',
  'enotfound',
  'eai_again',
  'stream was interrupted',
  'stream interrupted',
  'stream was disconnected',
  'stream ended',
  'stream disconnected',
  'ended without',
  'upstream request failed',
  'upstream_connect',
  'unable to forward upstream',
];

/**
 The lowest status a response writes when the request itself is what it refused, which is where an error status begins.
 */
const FIRST_ERROR_STATUS = 400;

/**
 The status codes of a failure another attempt can fix, read only where the failure text starts with one: that is where a response
 writes its status, before the body that explains it.

 Each says the same thing in its own way: the provider was too busy, the request took too long, the connection to the origin did
 not come up, or the answer did not arrive in time. A `429` is a rate limit rather than a server failure, so it is read as
 `THROTTLE_CODE` and not repeated here.
 */
const TRANSIENT_CODES: readonly string[] = [
  '408',
  '425',
  '500',
  '502',
  '503',
  '504',
  '520',
  '521',
  '522',
  '523',
  '524',
  '529',
];

/**
 The ways a failure writes the wait it asks for: the Cloudflare body above carries `"retry_after":60`, and an HTTP header carries
 `Retry-After: 60`.
 */
const WAIT_KEYS: readonly string[] = [
  'retry_after',
  'retry-after',
  'retry after',
];

/**
 What may stand between a wait's key and its number: whitespace of any kind, quotes, a colon and an equals sign. A body
 pretty-printed over several lines puts a line break between the key and its number, which is still the wait it asks for.
 */
const WAIT_SEPARATORS: ReadonlySet<string> = new Set([
  '"',
  '\'',
  ' ',
  '\t',
  '\n',
  '\r',
  ':',
  '=',
]);

/**
 Tells whether a character is an ASCII digit.

 @param character - one character of the text

 @returns whether it is a digit
 */
function isDigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Tells whether a character is an ASCII letter or digit, so a number that touches one is part of a longer word, count or request
 number.

 @param character - one character of the text

 @returns whether it is a letter or a digit
 */
function isLetterOrDigit(character: string,): boolean {
  return (isDigit(character,)) || ((character >= 'a') && (character <= 'z'));
}

/**
 Finds the status code a failure text starts with.

 A response writes its status before the body that explains it, so a number anywhere else is a count, a request number or a limit
 the provider states, and none of those is a status. Reading only the first word is what keeps `max_tokens must not exceed 500`
 from being read as a server error another attempt may fix.

 @param text - text folded to lower case

 @returns the digits of the status, or an empty string when the text starts with anything else
 */
function leadingStatusCode(text: string,): string {
  /**
   How many digits a status code has.
   */
  const statusDigits = 3;
  for (let at = 0; at < statusDigits; at += 1) {
    if (!isDigit(text.charAt(at,),)) {
      return '';
    }
  }
  // A letter or digit right after those three digits makes them part of a longer word or number, which is no status.
  return isLetterOrDigit(text.charAt(statusDigits,),) ? '' : text.slice(
    0,
    statusDigits,
  );
}

/**
 Tells whether a failure text starts with a status that refused the request itself.

 A status the text starts with is the answer the provider gave, and the body after it only explains that answer: a `400` whose
 body mentions an internal error is still a request the provider will not take, so no attempt at the same request reaches another
 answer.

 @param status - the digits the text starts with, or an empty string when it starts with anything else

 @returns whether the status is one the request itself earned
 */
function startsWithRefusedStatus(status: string,): boolean {
  return (status !== '')
    && (Number(status,) >= FIRST_ERROR_STATUS)
    && (!TRANSIENT_CODES.includes(status,))
    && (status !== THROTTLE_CODE);
}

/**
 Tells whether a text folded to lower case holds one of these phrases, or starts with one of these status codes.

 @param text - text folded to lower case

 @param phrases - phrases to look for

 @param statusCodes - status codes to look for at the start of the text

 @returns whether the text holds one of them
 */
function holdsOneOf({
  text,
  phrases = [],
  statusCodes = [],
}: {
  readonly text: string;
  readonly phrases?: readonly string[];
  readonly statusCodes?: readonly string[]
},): boolean {
  return phrases.some(function isInText(phrase,) {
    return text.includes(phrase,);
  },)
    || statusCodes.includes(leadingStatusCode(text,),);
}

/**
 Reads what kind of failure a model error is.

 @param problem - what the model said when its message ended in an error

 @returns the kind, which `permanent` is when no shape here recognises it

 @example
 ```ts
 const kind = kindOfModelError('Request timed out.',);
 ```
 */
export function kindOfModelError(problem: string,): ModelErrorKind {
  /**
   The failure, folded once so every shape below is a plain substring.
   */
  const text = problem.toLowerCase();
  // A status the text starts with is the answer the provider gave, and the body after it only explains that answer: a 401 whose
  // body sends the reader to a page about rate limits is still a credential the provider refused.
  if (startsWithRefusedStatus(leadingStatusCode(text,),)) {
    return 'permanent';
  }
  if (holdsOneOf({
    text,
    phrases: QUOTA_REACHED_PHRASES,
  },)) {
    return 'quota-reached';
  }
  if (holdsOneOf({
    text,
    phrases: THROTTLE_PHRASES,
    statusCodes: [THROTTLE_CODE,],
  },)) {
    return 'throttled';
  }
  if (holdsOneOf({
    text,
    phrases: TERMINAL_CAUSE_PHRASES,
  },)) {
    return 'permanent';
  }
  if (holdsOneOf({
    text,
    phrases: TRANSIENT_PHRASES,
    statusCodes: TRANSIENT_CODES,
  },)) {
    return 'transient';
  }
  return 'permanent';
}

/**
 Tells whether the driver itself answers a failure of this kind with another attempt.

 A 429 is Pi's own automatic retry's to answer, in every form it comes: a rate limit, a concurrency limit and a quota all reach
 the driver only once Pi's retry has given up on them, so the driver re-prompts none of them and waits none of them out.

 @param kind - what kind of failure a model error is

 @returns whether the driver answers it with another attempt

 @example
 ```ts
 const retried = driverRetries('transient',);
 ```
 */
export function driverRetries(kind: ModelErrorKind,): boolean {
  return kind === 'transient';
}

/**
 Tells whether a failure of this kind is one the coordinator measures apart from other failures.

 @param kind - what kind of failure a model error is

 @returns whether the run counts it as a rate limit

 @example
 ```ts
 const counted = isRateLimit('throttled',);
 ```
 */
export function isRateLimit(kind: ModelErrorKind,): boolean {
  return (kind === 'throttled') || (kind === 'quota-reached');
}

/**
 Finds where the separators between a wait's key and its number end.

 @param text - text folded to lower case

 @param from - where to start looking

 @returns the first index holding anything else, or the end of the text
 */
function firstIndexOutside({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number
},): number {
  for (let at = from; at < text.length; at += 1) {
    if (!WAIT_SEPARATORS.has(text.charAt(at,),)) {
      return at;
    }
  }
  return text.length;
}

/**
 Finds where a run of digits ends.

 @param text - text folded to lower case

 @param from - where to start looking

 @returns the first index that is no digit, or the end of the text
 */
function firstIndexNotDigit({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number
},): number {
  for (let at = from; at < text.length; at += 1) {
    if (!isDigit(text.charAt(at,),)) {
      return at;
    }
  }
  return text.length;
}

/**
 Reads the wait a provider asked for, when its failure names one, in seconds.

 @param problem - what the model said when its message ended in an error

 @returns the milliseconds to wait, or that the failure named none

 @example
 ```ts
 const asked = waitAskedByProvider('{"retry_after":60}',);
 ```
 */
export function waitAskedByProvider(problem: string,): WaitAskedByProvider {
  /**
   The failure, folded once so every key below is a plain substring.
   */
  const text = problem.toLowerCase();
  /**
   Where the earliest key ends, and nothing until one is found.
   */
  const afterKey = WAIT_KEYS.reduce(
    function earlierKeyEnd(
      earliest,
      key,
    ) {
    /**
     Where this key starts, when the text holds it.
     */
      const found = text.indexOf(key,);
      if (found === (-1)) {
        return earliest;
      }
    /**
     Where this key ends.
     */
      const end = found + key.length;
      return ((earliest === (-1)) || (end < earliest)) ? end : earliest;
    },
    -1,
  );
  if (afterKey === (-1)) {
    return { asked: false, };
  }
  /**
   Where the number starts, once every separator between it and its key is passed.
   */
  const start = firstIndexOutside({
    text,
    from: afterKey,
  },);
  /**
   Where the number ends.
   */
  const end = firstIndexNotDigit({
    text,
    from: start,
  },);
  if (end === start) {
    return { asked: false, };
  }
  /**
   The seconds the failure names.
   */
  const seconds = Number(text.slice(
    start,
    end,
  ),);
  return Number.isFinite(seconds,)
    ? {
      asked: true,
      milliseconds: seconds * MILLISECONDS_PER_SECOND,
    }
    : { asked: false, };
}

/**
 How long the driver waits before the next attempt: a delay that doubles with each attempt, raised to the wait the provider
 asked for when it named one, and never longer than the largest delay.

 The cap is what keeps the wait from growing without end: a failure naming no wait doubles from the first delay, and a provider
 asking for an hour is still waited out within the cap.

 @param problem - what the model said when its message ended in an error

 @param retryNumber - which attempt this delay comes before, counting from one

 @param baseSeconds - the first delay, in seconds

 @returns the milliseconds to wait

 @example
 ```ts
 const delay = retryDelayMilliseconds({ problem: 'Request timed out.', retryNumber: 2, baseSeconds: 2, },);
 ```
 */
export function retryDelayMilliseconds({
  problem,
  retryNumber,
  baseSeconds,
}: {
  readonly problem: string;
  readonly retryNumber: number;
  readonly baseSeconds: number
},): number {
  /**
   The delay the growth alone asks for, which a large attempt count pushes past every cap.
   */
  const grown = (baseSeconds * MILLISECONDS_PER_SECOND) * (2 ** (retryNumber - 1));
  /**
   The wait the provider asked for.
   */
  const asked = waitAskedByProvider(problem,);
  return Math.min(
    MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS,
    Math.max(
      grown,
      asked.asked ? asked.milliseconds : 0,
    ),
  );
}

/**
 Writes the message that answers a model error with another attempt.

 Pi drops an assistant message that ended in an error when it builds the request for the model
 (`packages/ai/src/api/transform-messages.ts` in `earendil-works/pi`), so the attempt after this message reads the last state the
 model produced successfully, which is what resuming the session by hand reads too.

 @param problem - what the model said when its message ended in an error

 @returns the prompt for the next attempt

 @example
 ```ts
 const prompt = continuationPromptOf({ problem: 'Request timed out.', },);
 ```
 */
export function continuationPromptOf({ problem, }: { readonly problem: string },): string {
  return [
    'The previous attempt ended before it finished, on a failure of the model provider rather than on anything you did.',
    `The provider said: ${preview({
      value: problem,
      length: FEED_PREVIEW_CHARACTERS.modelError,
    },)}`,
    'Continue the task from where you left off, and do not repeat work that already succeeded.',
  ].join('\n\n',);
}
