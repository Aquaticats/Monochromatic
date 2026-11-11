/**
 What kind of failure a model error is, whether another attempt can fix it, and what the driver sends to answer one.

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
 read in one pass and nothing here can backtrack. A status code is read only where the text starts with one, which is where a
 response writes it: the same digits inside a body name a token count or a limit the provider states, and neither is transient.

 The waits a failure asks for are read in `waits-a-model-error-names.ts`, and the characters both modules read are in
 `scanning-a-failure-text.ts`.

 @module
 */

import { FEED_PREVIEW_CHARACTERS, } from './driver-constants.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { leadingStatusCode, } from './scanning-a-failure-text.ts';

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
   A limit on how many requests the provider runs at once, sent as HTTP 429: another attempt can fix it, because a slot frees when
   another run of the provider's pool finishes, and the coordinator measures these apart from other failures.

   This is read apart from `throttled` because it names its own wait and its own ceiling, and because the wait it needs is the
   length of a run rather than a backoff: two recorded bodies, on 2026-11-11, asked for `suggested_initial_delay_s` 1 and 10 with
   `max_delay_s` 30 and 120.
   */
    | 'concurrency-limit'
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
 A limit on requests running at once, which a slot frees by passing: the provider says so in its own words, and the two bodies
 recorded on 2026-11-11 both carry it.

 A concurrency limit reads before the quota shapes below, because the provider explains the same 429 with its billing terms: the
 DeepSeek body says "Your subscription is in overage with auto-renew off" and links a page under `/billing`, and neither makes the
 slot that frees a quota that does not. See `doc/troubleshooting/` for the recorded bodies.
 */
const CONCURRENCY_LIMIT_PHRASES: readonly string[] = [
  '"limit_type":"concurrent',
  'concurrent limit',
  'concurrent request limit',
  'slots in use',
];

/**
 A limit another attempt cannot refill: the account, the plan or the billing has nothing left, and waiting out a backoff does not
 change that.

 These read before the rate-limit shapes below, because several of them are sent as HTTP 429.
 */
const QUOTA_REACHED_PHRASES: readonly string[] = [
  'subscription rate limit',
  'exceeded your subscription',
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
    phrases: CONCURRENCY_LIMIT_PHRASES,
  },)) {
    return 'concurrency-limit';
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
 Tells whether another attempt may fix a failure of this kind.

 @param kind - what kind of failure a model error is

 @returns whether the run answers it with another attempt

 @example
 ```ts
 const retried = anotherAttemptMayFix('transient',);
 ```
 */
export function anotherAttemptMayFix(kind: ModelErrorKind,): boolean {
  return (kind === 'transient') || (kind === 'throttled')
    || (kind === 'concurrency-limit');
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
  return (kind === 'throttled') || (kind === 'quota-reached')
    || (kind === 'concurrency-limit');
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
