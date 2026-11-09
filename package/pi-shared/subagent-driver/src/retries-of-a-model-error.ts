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

 Every text is folded to lower case once, and each shape is then a plain substring of it, so a failure of a few kilobytes is
 read in one pass and nothing here can backtrack.

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
   A rate limit that passes, such as HTTP 429: another attempt can fix it, and the coordinator measures these apart from other
   failures.
   */
    | 'throttled'
  /**
   A quota or a plan limit: written as a rate limit, but another attempt cannot refill it, so it ends the run and is counted apart
   all the same.
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

 These read before every other shape below, because the words of a transient failure turn up inside them: `Invalid API key for
 upstream provider` holds `upstream`, and `Context length exceeds the limit of 500 tokens` holds a server status as a count.
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
 The status code of a throttle, which counts only as a whole number.
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
 The server status codes of a failure another attempt can fix, each of which counts only as a whole number.
 */
const TRANSIENT_CODES: readonly string[] = [
  '500',
  '502',
  '503',
  '504',
  '520',
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
 What may stand between a wait's key and its number, besides spaces: quotes, a colon and an equals sign.
 */
const WAIT_SEPARATORS: ReadonlySet<string> = new Set([
  '"',
  '\'',
  ' ',
  '\t',
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
 Tells whether a character is an ASCII letter or digit, so a number that touches one is part of a longer word.

 @param character - one character of the text

 @returns whether it is a letter or a digit
 */
function isLetterOrDigit(character: string,): boolean {
  return (isDigit(character,)) || ((character >= 'a') && (character <= 'z'));
}

/**
 Tells whether a text holds a number as a whole word, so a request number or a token count carrying the same digits does not
 read as a status code.

 @param text - text folded to lower case

 @param number - the digits to look for

 @returns whether they stand alone
 */
function holdsNumberAlone({
  text,
  number,
}: {
  readonly text: string;
  readonly number: string
},): boolean {
  for (let at = text.indexOf(number,); at !== (-1); at = text.indexOf(
    number,
    at + 1,
  )) {
    /**
     What touches the digits on either side, which is nothing at either end of the text.
     */
    const before = text.charAt(at - 1,);
    /**
     What touches the digits on the right.
     */
    const after = text.charAt(at + number.length,);
    if ((!isLetterOrDigit(before,)) && (!isLetterOrDigit(after,))) {
      return true;
    }
  }
  return false;
}

/**
 Tells whether a text folded to lower case holds one of these phrases, or one of these numbers standing alone.

 @param text - text folded to lower case

 @param phrases - phrases to look for

 @param numbers - numbers to look for as whole words

 @returns whether the text holds one of them
 */
function holdsOneOf({
  text,
  phrases = [],
  numbers = [],
}: {
  readonly text: string;
  readonly phrases?: readonly string[];
  readonly numbers?: readonly string[]
},): boolean {
  return phrases.some(function isInText(phrase,) {
    return text.includes(phrase,);
  },)
    || numbers.some(function standsAlone(number,) {
      return holdsNumberAlone({
        text,
        number,
      },);
    },);
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
  if (holdsOneOf({
    text,
    phrases: TERMINAL_CAUSE_PHRASES,
  },)) {
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
    numbers: [THROTTLE_CODE,],
  },)) {
    return 'throttled';
  }
  if (holdsOneOf({
    text,
    phrases: TRANSIENT_PHRASES,
    numbers: TRANSIENT_CODES,
  },)) {
    return 'transient';
  }
  return 'permanent';
}

/**
 Tells whether another attempt may fix a failure of this kind.

 @param kind - what kind of failure a model error is

 @returns whether the run answers it with another attempt

 @example
 ```ts
 const retried = anotherAttemptMayFix('transient',);
 ```
 */
export function anotherAttemptMayFix(kind: ModelErrorKind,): boolean {
  return (kind === 'transient') || (kind === 'throttled');
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
