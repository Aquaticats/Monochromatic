import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type ExtractedCompletion,
  MalformedCompletionError,
  readFinishReason,
  readUsage,
  ssePayloadOf,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';


//region Streamed completion
// SSE reassembly for streamed chat completions. The provider is finicky without
// streaming, and streaming also defeats fetch's default headers timeout: headers
// arrive before thinking starts, and reasoning deltas keep the body alive. The
// transport drains the whole event stream to text; this module folds the events
// back into one completion. Reasoning deltas are dropped (content is the answer
// channel), refusal deltas accumulate into the first-class refusal field.

/**
 Terminal sentinel payload closing an OpenAI-compatible stream.
 */
const DONE_SENTINEL = '[DONE]';

/**
 Channels accumulated while folding one event stream.
 */
type StreamFold = {
  /**
   Content deltas in arrival order.
   */
  readonly contentParts: string[];

  /**
   Refusal deltas in arrival order.
   */
  readonly refusalParts: string[];

  /**
   Usage blocks seen; the last one wins.
   */
  readonly usageParts: ExtractedCompletion['usage'][];

  /**
   Finish reasons seen; the last one wins, as it closes the stream.
   */
  readonly finishReasons: string[];
};

/**
 Folds one parsed stream event into the accumulator.

 @param fold - accumulated channels

 @param chunk - parsed event payload

 @example
 ```ts
 foldChunk({ fold, chunk, },);
 ```
 */
function foldChunk(
  {
    fold,
    chunk,
  }: {
    readonly fold: StreamFold;
    readonly chunk: Readonly<Record<string, unknown>>;
  },
): void {
  /**
   Usage carried by this event, when present and well typed.
   */
  const { usage, } = readUsage({ parsed: chunk, },);
  if (usage !== undefined) {
    fold
      .usageParts
      .push(usage,);
  }

  /**
   Choices of this event; usage-only events carry an empty array.
   */
  const { choices, } = chunk;
  if (!isJsonArray(choices,))
    return;

  /**
   First choice of this event, when any.
   */
  const [first,] = choices;
  if (!isJsonRecord(first,))
    return;

  /**
   Why the model stopped, on the event that closes the stream.

   READ BEFORE THE DELTA CHECK, because the closing event of an
   OpenAI-compatible stream carries an EMPTY delta beside the reason. Folding
   it after that check would discard exactly the event worth reading.
   */
  const { finishReason, } = readFinishReason({ choice: first, },);
  if (finishReason !== undefined) {
    fold
      .finishReasons
      .push(finishReason,);
  }

  /**
   Delta block of the first choice.
   */
  const { delta, } = first;
  if (!isJsonRecord(delta,))
    return;

  if ((typeof delta.content) === 'string') {
    fold
      .contentParts
      .push(delta.content,);
  }
  if (((typeof delta.refusal) === 'string') && (delta.refusal !== '')) {
    fold
      .refusalParts
      .push(delta.refusal,);
  }
}

/**
 Refuses a body whose event stream never reached its terminator.

 SPLIT OUT SO THE RETRY LADDER CAN ASK IT TOO, on the same grounds as the
 Anthropic side: a stream that stopped early comes back as 200, so a check
 that runs after the retry returned is a check no retry ever sees.

 @param bodyText - whole drained body, as the transport returned it

 @throws {@link MalformedCompletionError} when the terminator never arrived

 @example
 ```ts
 requireStreamTerminator({ bodyText, },);
 ```
 */
export function requireStreamTerminator(
  { bodyText, }: { readonly bodyText: string; },
): void {
  /**
   Whether the terminal sentinel arrived anywhere in the stream.
   */
  const sawDone = bodyText
    .split('\n',)
    .some(function isDone(rawLine,): boolean {
      return ssePayloadOf({ line: rawLine, },) === DONE_SENTINEL;
    },);

  if (!sawDone) {
    throw new MalformedCompletionError({
      wireFormat: 'openai',
      detail: 'stream ended without its [DONE] terminator; the reply was cut off',
    },);
  }
}

/**
 Reassembles one drained SSE body into a completion.
 Requires the `[DONE]` terminator: a stream that ended without it was cut
 off, and silently returning truncated content would poison every consumer.

 @param bodyText - whole drained `text/event-stream` body

 @returns Reassembled content, refusal, and usage

 @throws {@link MalformedCompletionError} when an event is not JSON or the terminator is missing

 @example
 ```ts
 const extracted = extractStreamedCompletion({ bodyText: reply.bodyText, },);
 ```
 */
export function extractStreamedCompletion(
  { bodyText, }: { readonly bodyText: string; },
): ExtractedCompletion {
  /**
   Accumulated channels across every event.
   */
  const fold: StreamFold = {
    contentParts: [],
    refusalParts: [],
    usageParts: [],
    finishReasons: [],
  };

  requireStreamTerminator({ bodyText, },);

  /**
   Stream lines, folded into the answer.
   */
  const lines = bodyText.split('\n',);

  for (const rawLine of lines) {
    /**
     Event payload of this line; empty and sentinel lines fold nothing.
     */
    const payload = ssePayloadOf({ line: rawLine, },);
    if ((payload === '') || (payload === DONE_SENTINEL))
      continue;

    try {
      /**
       Parsed event payload.
       */
      const chunk: unknown = JSON.parse(payload,);
      if (!isJsonRecord(chunk,))
        throw new MalformedCompletionError({
          wireFormat: 'openai',
          detail: 'stream event is not a JSON object',
        },);
      foldChunk({
        fold,
        chunk,
      },);
    }
    catch (error) {
      if (error instanceof MalformedCompletionError)
        throw error;
      throw new MalformedCompletionError({
        wireFormat: 'openai',
        detail: 'stream event is not valid JSON',
        cause: error,
      },);
    }
  }

  /**
   Refusal accumulated across deltas, when the API refused.
   */
  const refusal = fold
    .refusalParts
    .join('',);

  /**
   Answer accumulated across content deltas.
   */
  const text = fold
    .contentParts
    .join('',);

  /**
   Last usage block of the stream, when any arrived.
   */
  const lastUsage = fold
    .usageParts
    .at(-1,);

  /**
   Reason closing the stream, when the provider sent one.
   */
  const finishReason = fold
    .finishReasons
    .at(-1,);

  return {
    text,
    // Conditional spreads keep absent channels absent. A refusal that shows a
    // reader nothing refused nothing (ledger B40): present, it is read as the
    // API's refusal ahead of every answer, so refusal deltas of spaces or a
    // zero-width space beside a JSON answer cost the voice.
    ...(rendersAsNothing({ text: refusal, },)
      ? {}
      : { refusal, }),
    ...(lastUsage === undefined
      ? {}
      : { usage: nonNullishOrThrow(lastUsage,), }),
    ...(finishReason === undefined
      ? {}
      : { finishReason, }),
  };
}

//endregion Streamed completion
