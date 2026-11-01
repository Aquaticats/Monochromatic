import {
  errorName,
  isAsciiLowerLetter,
} from '@monochromatic-dev/translation-repair-markup/ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  MalformedCompletionError,
  ssePayloadOf,
  ProviderHttpError,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';

/**
 Logger root for the Anthropic whole-message check.
 */
const l = contextRoot({ tag: 'translation-repair', },);

//region Anthropic whole message
// Whether one drained Anthropic Messages body ended the way a whole message
// does, and how each of its payloads reads as a frame. Split from
// `anthropic-completion.ts`, which folds the frames into an answer, so that the
// retry ladder in `hyper-client.ts` and the fold read a body by one rule.
//
// THE TERMINATOR IS `message_stop`, NOT `[DONE]`. Requiring it matters for the
// same reason `extractStreamedCompletion` requires its own: a stream that ended
// without one was cut off, and returning the truncated prefix would hand a
// validator a half-written JSON object and get it reported as a schema mismatch
// rather than as the transport failure it is.

/**
 Event ending a well-formed message.
 */
const TERMINATOR = 'message_stop';

/**
 Event a provider sends when the message fails partway, in place of the rest
 of it (the streaming documentation's "Error events").
 */
const ERROR_EVENT = 'error';

/**
 Longest error type a refusal repeats. The documented types are short
 protocol words; anything longer is read as text and not repeated.
 */
const ERROR_TYPE_LIMIT = 64;

/**
 Name a refusal gives an error event whose type is absent or is not a
 protocol word.
 */
const UNNAMED_ERROR = 'unnamed';

/**
 Reads one string field off a parsed object.

 @param fields - parsed object to read

 @param name - field wanted

 @returns Value, or empty when absent or not a string

 @example
 ```ts
 const kind = stringField({ fields: frame, name: 'type', },);
 ```
 */
export function stringField(
  {
    fields,
    name,
  }: {
    readonly fields: Readonly<Record<string, unknown>>;
    readonly name: string;
  },
): string {
  /**
   Raw value under that name, of unknown type.
   */
  const value = fields[name];

  if ((typeof value) !== 'string')
    return '';
  return value;
}

/**
 What one event payload reads as: a frame, or why it is not one.

 A DISCRIMINATED RESULT so the two readers of a body share one parse and
 still differ in what they do with a payload that is not a frame: the check
 for a whole message passes over it, and the fold refuses the body.

 @example
 ```ts
 const reading: FrameReading = { kind: 'not-object', };
 ```
 */
export type FrameReading =
  | {
    readonly kind: 'frame';

    /**
     Parsed frame, which carries its own `type`.
     */
    readonly frame: Readonly<Record<string, unknown>>;
  }
    | {
      readonly kind: 'not-json';

    /**
     Parse failure, kept as the cause of any refusal.
     */
      readonly cause: unknown;
    }
    | { readonly kind: 'not-object'; };

/**
 Reads one event payload as a frame.

 @param payload - one `data:` line's payload, already unwrapped

 @returns Frame, or why the payload is not one

 @example
 ```ts
 const reading = readFrame({ payload: '{"type":"ping"}', },);
 ```
 */
export function readFrame({ payload, }: { readonly payload: string; },): FrameReading {
  try {
    /**
     Whatever the payload parsed to, before any shape is assumed.
     */
    const parsed: unknown = JSON.parse(payload,);

    if (!isJsonRecord(parsed,))
      return { kind: 'not-object', };
    return {
      kind: 'frame',
      frame: parsed,
    };
  } catch (error) {
    l.debug(`anthropic stream payload did not parse: ${errorName({ error, },)}`,);
    return {
      kind: 'not-json',
      cause: error,
    };
  }
}

/**
 Whether a provider's error type reads as a protocol word: lower-case ASCII
 letters and underscores, as every documented type is spelled, and short.

 A TEST OF SHAPE RATHER THAN A LIST, so a type the provider adds later is
 still named, while text that is not a protocol word is never repeated in a
 refusal whose class promises to quote nothing from the body.

 @param text - error type as the frame gave it

 @returns Whether a refusal may repeat it

 @example
 ```ts
 isProtocolWord({ text: 'overloaded_error', },);
 ```
 */
function isProtocolWord({ text, }: { readonly text: string; },): boolean {
  if ((text.length === 0) || (text.length > ERROR_TYPE_LIMIT))
    return false;
  return Array.from(text,)
    .every(function isWordCharacter(character,): boolean {
      return (character === '_') || isAsciiLowerLetter({ character, },);
    },);
}

/**
 Names the failure an error event reported, for the refusal that ends the
 call.

 @param frame - parsed error frame

 @returns Error type, or that the frame named none a refusal may repeat

 @example
 ```ts
 const named = errorTypeOf({ frame, },);
 ```
 */
function errorTypeOf(
  { frame, }: { readonly frame: Readonly<Record<string, unknown>>; },
): string {
  /**
   Error descriptor the frame carried.
   */
  const { error, } = frame;
  if (!isJsonRecord(error,))
    return UNNAMED_ERROR;

  /**
   Type the descriptor names, empty when it names none.
   */
  const named = stringField({
    fields: error,
    name: 'type',
  },);

  return isProtocolWord({ text: named, },)
    ? named
    : UNNAMED_ERROR;
}

/**
 Bad Request.
 */
const HTTP_BAD_REQUEST = 400;

/**
 Unauthorized.
 */
const HTTP_UNAUTHORIZED = 401;

/**
 Payment Required.
 */
const HTTP_PAYMENT_REQUIRED = 402;

/**
 Forbidden.
 */
const HTTP_FORBIDDEN = 403;

/**
 Not Found.
 */
const HTTP_NOT_FOUND = 404;

/**
 Content Too Large.
 */
const HTTP_CONTENT_TOO_LARGE = 413;

/**
 Error types the provider documents for a request it refuses, with the HTTP
 status each carries outside a stream (the errors page, "HTTP errors",
 platform.claude.com/docs/en/api/errors). A request refused for what it holds
 is refused again however often it is sent, so an error event of one of these
 types ends the attempt as the same refusal over plain HTTP does: the retry
 ladder returns a 4xx status unretried.

 NOT HERE, so retried as before: `rate_limit_error` (429), `api_error` (500),
 `timeout_error` (504) and `overloaded_error` (529), which the documentation
 describes as transient, a type it adds later or one that names none this
 package can read, and `conflict_error` (409), whose retry succeeds once the
 conflicting change is gone.
 */
const REFUSED_REQUEST_STATUS: ReadonlyMap<string, number> = new Map([
  [
    'invalid_request_error',
    HTTP_BAD_REQUEST,
  ],
  [
    'authentication_error',
    HTTP_UNAUTHORIZED,
  ],
  [
    'billing_error',
    HTTP_PAYMENT_REQUIRED,
  ],
  [
    'permission_error',
    HTTP_FORBIDDEN,
  ],
  [
    'not_found_error',
    HTTP_NOT_FOUND,
  ],
  [
    'request_too_large',
    HTTP_CONTENT_TOO_LARGE,
  ],
],);

/**
 Raised when a success-status Messages stream carried an error event naming a
 request the provider refuses, which no retry changes.

 THE HTTP REFUSAL IT STANDS FOR: a subclass of {@link ProviderHttpError}
 carrying the status the same refusal has outside a stream, so a caller
 branching on the status (a 402 holds the provider out) reads it as it reads
 the plain HTTP one, and the retry ladder, which returns a non-retryable
 status unretried, ends the attempt on it.

 @example
 ```ts
 throw new StreamErrorEventError({ provider: 'hyper', errorType: 'invalid_request_error', status: 400, },);
 ```
 */
export class StreamErrorEventError extends ProviderHttpError {
  /**
   Declares this message safe to forward: it names a documented error type and
   a status, never the provider's text.
   */
  readonly messageNamesOnly: true = true;

  /**
   Error type the event named, one of the documented refusal types.
   */
  readonly errorType: string;

  /**
   Names the refusal the stream carried.

   @param provider - provider whose stream carried the error event, which the event itself does not name

   @param errorType - documented refusal type the error event named

   @param status - status the same refusal carries outside a stream

   @example
   ```ts
   new StreamErrorEventError({ provider: 'hyper', errorType: 'billing_error', status: 402, },);
   ```
   */
  public constructor(
    {
      provider,
      errorType,
      status,
    }: {
      readonly provider: ProviderName;
      readonly errorType: string;
      readonly status: number;
    },
  ) {
    super({
      provider,
      status,
      bodyText: '',
      excerpt: 'withheld',
      summary: `Anthropic Messages stream carried an error event of type ${errorType}, a refusal of the request `
        + 'itself that repeating it would meet again; it arrived under a success status and is reported with the '
        + `status ${String(status,)} the same refusal carries outside a stream`,
    },);
    this.name = 'StreamErrorEventError';
    this.errorType = errorType;
  }
}

/**
 Refuses a body whose event stream did not end the way a whole message does:
 with a `message_stop` frame, and with no error event anywhere in it.

 SPLIT OUT SO THE RETRY LADDER CAN ASK IT TOO. A body that stops before
 `message_stop` is a transport failure wearing a success status: the HTTP
 exchange returned 200 and the message inside it is not whole. Reading it
 only after the retry had already returned meant the one failure this file
 calls a transport failure was the only one that never retried.

 ONE RULE IN ONE PLACE. `extractAnthropicCompletion` calls this rather than
 carrying its own copy, so the retry and the parse can never disagree about
 what a finished message looks like.

 THE TERMINATOR IS A FRAME OF THAT TYPE, read off the parse (ledger B87).
 Looking for the quoted word anywhere in a payload passed a body cut inside
 its last frame, after `{"type":"message_stop"` and before the closing
 brace, so the ladder returned it and the fold then refused it as not JSON,
 with no retry left to spend. It also passed any frame holding that word as
 a value, a tool's name for one.

 AN ERROR EVENT REFUSES THE MESSAGE even when a terminator follows it, and
 the refusal names its type. The provider said why it stopped; calling that
 a cut connection sent a reader to the network. Refused here, it is retried
 like a cut stream, which suits an overloaded or internal error. An error
 event naming a request the provider refuses (`REFUSED_REQUEST_STATUS`) is
 a {@link StreamErrorEventError} instead, carrying the status the same
 refusal has over plain HTTP, which the retry ladder does not retry.

 ONLY THE ENDING IS READ. A frame elsewhere in the body that does not parse
 is the fold's to refuse, after the ladder: a body whose terminator arrived
 was delivered whole, and a frame inside it that cannot be read is read here
 as a formatting defect a retry would repeat and pay for again (an
 inference, not measured). `requireStreamTerminator`, the OpenAI-shaped
 sibling, reads the same scope.

 @param provider - provider whose body this is, carried onto the refusal an error event becomes

 @param bodyText - whole drained body, as the transport returned it

 @throws {@link StreamErrorEventError} when an error event names a request the provider refuses

 @throws {@link MalformedCompletionError} when any other error event arrived or the terminator never did

 @example
 ```ts
 requireWholeAnthropicMessage({ provider: 'hyper', bodyText, },);
 ```
 */
export function requireWholeAnthropicMessage(
  {
    provider,
    bodyText,
  }: {
    readonly provider: ProviderName;
    readonly bodyText: string;
  },
): void {
  /**
   Frames the body carried, in arrival order; payloads that are not frames
   are passed over.
   */
  const frames = bodyText
    .split('\n',)
    .flatMap(function framesOf(rawLine,): readonly Readonly<Record<string, unknown>>[] {
      /**
       Payload of this line, empty for a line carrying no event.
       */
      const payload = ssePayloadOf({ line: rawLine, },);
      if (payload === '')
        return [];

      /**
       What that payload reads as.
       */
      const reading = readFrame({ payload, },);
      return (reading.kind === 'frame')
        ? [reading.frame,]
        : [];
    },);

  /**
   Type each frame declared, empty where it declared none.
   */
  const kinds = frames.map(function kindOf(frame,): string {
    return stringField({
      fields: frame,
      name: 'type',
    },);
  },);

  /**
   First error event, when the provider sent one.
   */
  const failure = frames.find(function isFailure(frame,): boolean {
    return stringField({
      fields: frame,
      name: 'type',
    },) === ERROR_EVENT;
  },);

  if (failure !== undefined) {
    /**
     Type the error event named, or that it named none a refusal may repeat.
     */
    const errorType = errorTypeOf({ frame: failure, },);
    /**
     Status the same refusal carries over plain HTTP, when the type names one.
     */
    const refusedStatus = REFUSED_REQUEST_STATUS.get(errorType,);
    if (refusedStatus !== undefined)
      throw new StreamErrorEventError({
        provider,
        errorType,
        status: refusedStatus,
      },);
    throw new MalformedCompletionError({
      wireFormat: 'anthropic',
      detail: `stream carried an error event (${errorType})`,
    },);
  }
  if (!kinds.includes(TERMINATOR,)) {
    throw new MalformedCompletionError({
      wireFormat: 'anthropic',
      detail: `stream ended without ${TERMINATOR}`,
    },);
  }
}

//endregion Anthropic whole message
