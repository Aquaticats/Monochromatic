import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { BedrockStreamEnd, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  MalformedCompletionError,
  ssePayloadOf,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { requireNoStreamError, } from './openrouter-stream-error.ts';
import { requireStreamTerminator, } from './stream-completion.ts';

//region Bedrock stream end
// HOW A BEDROCK STREAM SAYS IT IS WHOLE, which differs by route. The Gemma
// route ends on `data: [DONE]` as every other provider here does; the gpt-oss
// route ends on a chunk carrying `usage` and no choices, and sends no
// sentinel at all (measured 2026-09-07: six lines, the last a usage chunk).
// The shared reader requires the sentinel, on the grounds that a stream which
// stopped early comes back as 200; so this file asks the right question per
// route and, where the usage chunk is the terminator, appends the sentinel
// the reader expects once that chunk has been seen.

/**
 Logger root for the stream-end checks.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Terminal sentinel the shared reader requires.
 */
const DONE_LINE = 'data: [DONE]\n';

/**
 Whether one stream line is a chunk carrying a usage block.

 @param rawLine - one line of the drained body

 @returns Whether it parses as an object with a `usage` object

 @example
 ```ts
 isUsageChunk('data: {"choices":[],"usage":{"completion_tokens":10}}',);
 ```
 */
function isUsageChunk(rawLine: string,): boolean {
  /**
   Payload the line carries, read the way every other stream reader here
   reads it (`sse-data-line.ts`): this reader spelled the prefix with its
   space, so a usage chunk sent in the tight `data:{...}` form never counted
   and a whole stream was refused as cut off (audit area six, 2026-09-28).
   */
  const payload = ssePayloadOf({ line: rawLine, },);
  if (payload === '')
    return false;
  try {
    /**
     Parsed chunk, unknown until checked.
     */
    const chunk: unknown = JSON.parse(payload,);
    return isJsonRecord(chunk,) && isJsonRecord(chunk.usage,);
  } catch (error) {
    // A line that will not parse is not a usage chunk; the shared reader
    // names it when it folds the stream, so here it is only noted.
    l.debug(`stream line skipped while looking for the usage chunk: ${errorName({ error, },)}`,);
    return false;
  }
}

/**
 Refuses a body whose stream never announced it was whole, in the way this
 model's route announces it.

 @param bodyText - whole drained body, as the transport returned it

 @param streamEnd - how this model's stream ends

 @throws {@link import('./openrouter-stream-error.ts').InStreamRefusalError} when an error chunk carries a 4xx
 code the retry ladder does not retry

 @throws {@link import('./openrouter-stream-error.ts').InStreamProviderError} when any other error chunk arrived

 @throws {@link MalformedCompletionError} when the terminator never arrived

 @example
 ```ts
 requireBedrockStreamEnd({ bodyText, streamEnd: 'usage-chunk', },);
 ```
 */
export function requireBedrockStreamEnd(
  {
    bodyText,
    streamEnd,
  }: {
    readonly bodyText: string;
    readonly streamEnd: BedrockStreamEnd;
  },
): void {
  // THE FAILURE IS ASKED FIRST, as on the OpenRouter route. A stream that
  // failed mid-way carries an `error` chunk and neither terminator, so asking
  // only for the end named the framing ("cut off") where the wire named the
  // cause, and retried a refusal the ladder returns unretried over plain HTTP.
  // The chunk's shape is the OpenAI-compatible one, which this route speaks;
  // AWS's documentation for the route does not state an error frame (read
  // 2026-10-06), so this reads the convention and not a captured Bedrock frame.
  // THE PROVIDER IS FIXED HERE, not passed in: this function reads the Bedrock
  // route's streams and is called by the Bedrock client alone, so a refusal it
  // raises is Bedrock's.
  requireNoStreamError({
    provider: 'bedrock',
    bodyText,
  },);
  if (streamEnd === 'done-sentinel') {
    requireStreamTerminator({ bodyText, },);
    return;
  }

  /**
   Whether a usage chunk arrived anywhere in the stream.
   */
  const sawUsage = bodyText
    .split('\n',)
    .some(isUsageChunk,);
  if (!sawUsage) {
    throw new MalformedCompletionError({
      wireFormat: 'openai',
      detail: 'stream ended without its usage chunk; the reply was cut off',
    },);
  }
}

/**
 Body the shared reader accepts: the stream as it came, with the sentinel
 appended where this route ends on a usage chunk instead.

 @param bodyText - whole drained body, already checked whole

 @param streamEnd - how this model's stream ends

 @returns Body ending on the sentinel

 @example
 ```ts
 const extracted = extractStreamedCompletion({ bodyText: withDoneSentinel({ bodyText, streamEnd, },), },);
 ```
 */
export function withDoneSentinel(
  {
    bodyText,
    streamEnd,
  }: {
    readonly bodyText: string;
    readonly streamEnd: BedrockStreamEnd;
  },
): string {
  if (streamEnd === 'done-sentinel')
    return bodyText;
  return `${bodyText}\n${DONE_LINE}`;
}

//endregion Bedrock stream end
