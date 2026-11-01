import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { openRouterChunksOf, } from './openrouter-chunk-scan.ts';
import { openRouterEndpointOf, } from './openrouter-endpoint.ts';
import { openRouterErrorFinishOf, } from './openrouter-error-finish.ts';

//region OpenRouter in-stream error
// A PROVIDER FAILURE THAT ARRIVES AS A SUCCESS. OpenRouter answers a chat
// completion with HTTP 200 and starts the event stream before the upstream
// has answered; when the upstream then fails, the gateway cannot change the
// status it already sent, so it writes one chunk carrying an `error` object
// (`{"code":504,"message":"error code: 504","metadata":{"error_type":
// "timeout"}}`, captured 2026-09-04) and closes the stream without `[DONE]`.
//
// WHAT THAT LOOKED LIKE BEFORE THIS READER. `requireStreamTerminator` saw
// no terminator and threw "stream ended without its [DONE] terminator; the
// reply was cut off": a true statement about the framing and a wrong
// diagnosis of the cause. On 2026-09-04, 114 of the day's 115 such failures
// were MiniMax M3 served by ModelRun, each a body of 846 characters that
// "completed" after about 10.5 seconds with no content, and a direct probe
// of that endpoint reproduced the frame on the fourth of six calls. The
// retry ladder handled every one as a truncated reply, which is the right
// action for the wrong reason, and the run log named no endpoint and no
// code, so the endpoint's failure rate (119 of 300 streams that day) had to
// be reconstructed from the raw-character count.
//
// NAMES, NEVER THE BODY. The gateway's error message is free text from the
// upstream; this reader carries the numeric code, the gateway's error type
// and the endpoint's display name, which is what an operator needs to act
// and nothing a run log must not hold.
//
// A SECOND SHAPE, 2026-09-08: a stream that closes its choice with
// `finish_reason: "error"`, carries no error object and does send `[DONE]`.
// `openrouter-error-finish.ts` records that class; this reader asks it when
// no chunk carried an error object, so both shapes throw the same failure.

/**
 Field OpenRouter puts the upstream's failure in, on the chunk that ends
 a failed stream.
 */
const ERROR_KEY = 'error';

/**
 Field inside the error object naming the failure class.
 */
const CODE_KEY = 'code';

/**
 Field inside the error object carrying the gateway's own metadata.
 */
const METADATA_KEY = 'metadata';

/**
 Field inside the metadata naming the failure kind.
 */
const ERROR_TYPE_KEY = 'error_type';

/**
 Value written where the wire carried no usable field.
 */
const UNNAMED = 'unnamed';

/**
 Failure kind written when a choice stopped on an error finish and the wire
 forwarded no native reason for it.
 */
const ERROR_FINISH_KIND = 'error-finish';

/**
 Lowest status of the 4xx class, which the gateway's error code mirrors
 (OpenRouter's errors page, read 2026-10-06: the code is the HTTP status the
 failure would have carried).
 */
const FIRST_CLIENT_ERROR_CODE = 400;

/**
 First status past the 4xx class.
 */
const FIRST_SERVER_ERROR_CODE = 500;

/**
 Request Timeout, a 4xx code that is weather rather than a refusal: the
 retry ladder retries it over plain HTTP, so it stays retried here.
 */
const REQUEST_TIMEOUT_CODE = 408;

/**
 Too Many Requests, a 4xx code that is weather rather than a refusal, kept
 retried here as the ladder retries it over plain HTTP.
 */
const TOO_MANY_REQUESTS_CODE = 429;

/**
 What one stream's error chunk said, reduced to names.

 @example
 ```ts
 const found: StreamErrorReading = { found: true, code: 504, errorType: 'timeout', endpoint: 'ModelRun', };
 ```
 */
export type StreamErrorReading =
  | {
    readonly found: true;

    /**
     Numeric failure class the gateway reported, or that it reported none.
     */
    readonly code: number | typeof UNNAMED;

    /**
     Gateway's failure kind, or that it named none.
     */
    readonly errorType: string;

    /**
     Upstream the gateway named as serving the call, or that it named none.
     */
    readonly endpoint: string;
  }
    | { readonly found: false; };

/**
 Reading given when no chunk carried an error object.
 */
export const STREAM_ERROR_ABSENT: StreamErrorReading = { found: false, };

/**
 Raised when a success-status stream carried a provider failure instead of
 a completion.

 DISTINCT FROM `MalformedCompletionError`, which names a body this client
 cannot read; this body was read fine and says the upstream failed. Both
 ride the retry ladder as thrown transport failures, because the ladder
 retries whatever `verify` throws.
 */
export class InStreamProviderError extends Error {
  /**
   Declares this message safe to forward: it carries a code, a failure kind
   and an endpoint's display name, never the upstream's text.
   */
  readonly messageNamesOnly: true = true;

  /**
   Numeric failure class the gateway reported, or that it reported none.
   */
  readonly code: number | typeof UNNAMED;

  /**
   Upstream the gateway named as serving the call, or that it named none.
   */
  readonly endpoint: string;

  /**
   Names the failure the stream carried.

   @param code - numeric failure class, or that none was reported

   @param errorType - gateway's failure kind, or that none was named

   @param endpoint - upstream display name, or that none was named

   @example
   ```ts
   throw new InStreamProviderError({ code: 504, errorType: 'timeout', endpoint: 'ModelRun', },);
   ```
   */
  constructor(
    {
      code,
      errorType,
      endpoint,
    }: {
      readonly code: number | typeof UNNAMED;
      readonly errorType: string;
      readonly endpoint: string;
    },
  ) {
    super(
      `stream carried a provider failure instead of a completion: code ${String(code,)}, `
        + `type ${errorType}, served by ${endpoint}; the gateway had already sent a success status`,
    );
    this.name = 'InStreamProviderError';
    this.code = code;
    this.endpoint = endpoint;
  }
}

/**
 Raised when a success-status stream carried an error chunk whose code is a
 4xx status the retry ladder does not retry over plain HTTP: the gateway
 refused the request itself, and repeating it meets the same refusal.

 THE HTTP REFUSAL IT STANDS FOR: a subclass of {@link ProviderHttpError}
 carrying the code as its status, so the ladder returns it unretried as it
 returns the plain HTTP reply, and a caller branching on the status (a 402
 holds the provider out) reads it as it reads the plain one. A 408 or 429
 chunk is not this class: those ride the ladder as before, as
 {@link InStreamProviderError}.

 @example
 ```ts
 throw new InStreamRefusalError({ provider: 'openrouter', code: 400, errorType: 'invalid_request', endpoint: 'ModelRun', },);
 ```
 */
export class InStreamRefusalError extends ProviderHttpError {
  /**
   Declares this message safe to forward: it carries a code, a failure kind
   and an endpoint's display name, never the upstream's text.
   */
  readonly messageNamesOnly: true = true;

  /**
   Upstream the gateway named as serving the call, or that it named none.
   */
  readonly endpoint: string;

  /**
   Names the refusal the stream carried.

   @param provider - provider whose stream carried the error chunk; the chunk names an upstream endpoint, not the
   provider that relayed it

   @param code - 4xx status the gateway reported, which is also this error's status

   @param errorType - gateway's failure kind, or that none was named

   @param endpoint - upstream display name, or that none was named

   @example
   ```ts
   throw new InStreamRefusalError({ provider: 'openrouter', code: 402, errorType: 'payment_required', endpoint: 'ModelRun', },);
   ```
   */
  constructor(
    {
      provider,
      code,
      errorType,
      endpoint,
    }: {
      readonly provider: ProviderName;
      readonly code: number;
      readonly errorType: string;
      readonly endpoint: string;
    },
  ) {
    super({
      provider,
      status: code,
      bodyText: '',
      excerpt: 'withheld',
      summary: `stream carried a refusal of the request itself instead of a completion: code ${String(code,)}, `
        + `type ${errorType}, served by ${endpoint}; the gateway had already sent a success status, `
        + 'and repeating the request meets the same refusal',
    },);
    this.name = 'InStreamRefusalError';
    this.endpoint = endpoint;
  }
}

/**
 Reads the failure a stream's error chunk carried, if any chunk carried one.

 THE FIRST ERROR CHUNK WINS, as the endpoint reader's first name does: the
 gateway writes one and closes. A choice that stopped on an error finish
 with no error object beside it counts as a failure too, with no code and
 the upstream's own reason as its kind.

 @param bodyText - whole drained `text/event-stream` body

 @returns Code, kind and endpoint of the failure, or that none was carried

 @example
 ```ts
 const reading = openRouterStreamErrorOf({ bodyText: reply.bodyText, },);
 ```
 */
export function openRouterStreamErrorOf(
  { bodyText, }: { readonly bodyText: string; },
): StreamErrorReading {
  /**
   Error objects the chunks carried, in arrival order.
   */
  const errors = openRouterChunksOf({ bodyText, },)
    .flatMap(function errorOf(chunk,): readonly Readonly<Record<string, unknown>>[] {
      /**
       Whatever sits at the field, of unknown type until checked.
       */
      const error = chunk[ERROR_KEY];
      return isJsonRecord(error,) ? [error,] : [];
    },);

  /**
   Upstream named on the chunks, or that none was.
   */
  const endpoint = openRouterEndpointOf({ bodyText, },);

  /**
   Upstream's display name as the reading carries it.
   */
  const endpointName = endpoint.reported ? endpoint.name : UNNAMED;

  /**
   First error object, or none.
   */
  const [first,] = errors;
  if (first === undefined) {
    /**
     Whether a choice stopped on an error finish with no error object beside it.
     */
    const finish = openRouterErrorFinishOf({ bodyText, },);
    if (!finish.found)
      return STREAM_ERROR_ABSENT;
    return {
      found: true,
      code: UNNAMED,
      errorType: finish.nativeReason ?? ERROR_FINISH_KIND,
      endpoint: endpointName,
    };
  }

  /**
   Numeric code, or that none was reported.
   */
  const code = first[CODE_KEY];

  /**
   Gateway metadata, of unknown shape until checked.
   */
  const metadata = first[METADATA_KEY];

  /**
   Failure kind, or that none was named.
   */
  const errorType = isJsonRecord(metadata,) ? metadata[ERROR_TYPE_KEY] : undefined;

  return {
    found: true,
    code: ((typeof code) === 'number') ? code : UNNAMED,
    errorType: ((typeof errorType) === 'string') ? errorType : UNNAMED,
    endpoint: endpointName,
  };
}

/**
 Refuses a success body whose stream carried a provider failure.

 ASKED BEFORE THE TERMINATOR CHECK, because such a stream also lacks its
 terminator and the terminator check would otherwise name the framing
 rather than the failure.

 @param provider - provider whose stream this is: OpenRouter, or a route that speaks the same chunk shape, carried
 onto the refusal an error chunk becomes

 @param bodyText - whole drained `text/event-stream` body

 @throws {@link InStreamRefusalError} when the error object's code is a 4xx
 status other than 408 and 429, which no retry changes

 @throws {@link InStreamProviderError} when a chunk carried any other error
 object or a choice stopped on an error finish

 @example
 ```ts
 requireNoStreamError({ provider: 'openrouter', bodyText, },);
 ```
 */
export function requireNoStreamError(
  {
    provider,
    bodyText,
  }: {
    readonly provider: ProviderName;
    readonly bodyText: string;
  },
): void {
  /**
   What the stream said about failing.
   */
  const reading = openRouterStreamErrorOf({ bodyText, },);
  if (reading.found) {
    if (((typeof reading.code) === 'number')
      && (reading.code >= FIRST_CLIENT_ERROR_CODE)
      && (reading.code < FIRST_SERVER_ERROR_CODE)
      && (reading.code !== REQUEST_TIMEOUT_CODE)
      && (reading.code !== TOO_MANY_REQUESTS_CODE))
      throw new InStreamRefusalError({
        provider,
        code: reading.code,
        errorType: reading.errorType,
        endpoint: reading.endpoint,
      },);
    throw new InStreamProviderError({
      code: reading.code,
      errorType: reading.errorType,
      endpoint: reading.endpoint,
    },);
  }
}

//endregion OpenRouter in-stream error
