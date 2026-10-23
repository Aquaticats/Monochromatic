import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { ssePayloadOf, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  readFrame,
  stringField,
} from './anthropic-whole-message.ts';

//region Claude stream facts
// WHAT A DRAINED MESSAGES STREAM SAYS ABOUT HOW IT ENDED AND WHAT IT COST, beyond the answer text
// (`anthropic-completion.ts` reads that, for Hyper's client and this one). Two facts the shared reader folds away or
// sums: the refusal stop with its category, and the usage split into the four counts Anthropic bills at four rates.
//
// THE SHARED READER SUMS THE PROMPT (input, cache write and cache read together) because a caller of the other
// providers wants one prompt count. A price per count wants them apart, and changing the shared reader's return would
// reach Hyper's client, so this file reads the same frames again for the split.
//
// WHERE A REFUSAL'S DETAILS SIT ON A STREAM IS READ TOLERANTLY. The refusal page of the API reference
// (`https://platform.claude.com/docs/en/build-with-claude/refusals-and-fallback`, read 2026-10-22) shows
// `stop_details` on the whole message, with a `category` and an `explanation` that may each be null, and documents
// no streamed example. The streaming reference puts `stop_reason` in the `delta` of the `message_delta` frame, so the
// details are looked for in that delta, on the frame itself and in the `message` of `message_start`, the last one
// found wins; a `stop_reason` of `refusal` is a refusal whether or not any details arrive. The category is only ever
// logged, so a shape this reads wrongly costs a missing category on a log line and nothing else.
// UNVERIFIED AGAINST A LIVE STREAM, since no keyed call may be made from this session.

/**
 Prompt and completion counts a stream reported, each at the rate it is billed at.

 @example
 ```ts
 const usage: ClaudeUsageSplit = { input: 100, cacheWrite: 0, cacheRead: 0, output: 10, };
 ```
 */
export type ClaudeUsageSplit = {
  /**
   Prompt tokens past the last cache breakpoint.
   */
  readonly input: number;

  /**
   Prompt tokens written to the five-minute cache.
   */
  readonly cacheWrite: number;

  /**
   Prompt tokens read back from the cache.
   */
  readonly cacheRead: number;

  /**
   Completion tokens, thinking included.
   */
  readonly output: number;
};

/**
 What a stream said about its end and its cost.

 @example
 ```ts
 const facts: ClaudeStreamFacts = { stopReason: 'refusal', refusal: { category: 'cyber', }, };
 ```
 */
export type ClaudeStreamFacts = {
  /**
   Why the model stopped, as the last `message_delta` reported it; absent when none did.
   */
  readonly stopReason?: string;

  /**
   Present when the model stopped on a refusal.
   */
  readonly refusal?: {
    /**
     Policy area the refusal named; absent where the category was null or not given.
     */
    readonly category?: string;
  };

  /**
   Counts, present only when the stream reported both its prompt and its completion count.
   */
  readonly usage?: ClaudeUsageSplit;
};

/**
 Counts one pass accumulates, each series in arrival order.
 */
type Counts = {
  readonly input: number[];
  readonly cacheWrite: number[];
  readonly cacheRead: number[];
  readonly output: number[];
};

/**
 Usage fields read, each beside the series it lands in.
 */
const USAGE_FIELDS = [
  [
    'input_tokens',
    'input',
  ],
  [
    'cache_creation_input_tokens',
    'cacheWrite',
  ],
  [
    'cache_read_input_tokens',
    'cacheRead',
  ],
  [
    'output_tokens',
    'output',
  ],
] as const;

/**
 Records the counts one usage block carried, ignoring absent and null ones.

 @param holder - object that directly holds the `usage` block

 @param counts - accumulator to append to

 @example
 ```ts
 foldUsageInto({ holder: frame, counts, },);
 ```
 */
function foldUsageInto(
  {
    holder,
    counts,
  }: {
    readonly holder: Readonly<Record<string, unknown>>;
    readonly counts: Counts;
  },
): void {
  /**
   Usage block, absent while a frame carries none.
   */
  const { usage, } = holder;
  if (!isJsonRecord(usage,))
    return;
  for (const [field, series,] of USAGE_FIELDS) {
    /**
     Count this frame reported under that field, of unknown type.
     */
    const reported = usage[field];
    if ((typeof reported) === 'number')
      counts[series]
        .push(reported,);
  }
}

/**
 Finds the stop details a frame carries, in the places they may sit.

 @param frame - parsed frame

 @returns The details object in a list of one, or an empty list where the frame carries none

 @example
 ```ts
 const details = stopDetailsOf({ frame, },);
 ```
 */
function stopDetailsOf(
  { frame, }: { readonly frame: Readonly<Record<string, unknown>>; },
): readonly Readonly<Record<string, unknown>>[] {
  for (const holder of [
    frame.delta,
    frame,
    frame.message,
  ]) {
    if (isJsonRecord(holder,) && isJsonRecord(holder.stop_details,))
      return [holder.stop_details,];
  }
  return [];
}

/**
 Reads a drained Messages stream for its stop and its counts.

 @param bodyText - whole drained `text/event-stream` body

 @returns The facts, each absent where the stream said nothing

 @example
 ```ts
 const { refusal, usage, } = claudeStreamFactsOf({ bodyText, },);
 ```
 */
export function claudeStreamFactsOf({ bodyText, }: { readonly bodyText: string; },): ClaudeStreamFacts {
  /**
   Counts accumulated over the body.
   */
  const counts: Counts = {
    input: [],
    cacheWrite: [],
    cacheRead: [],
    output: [],
  };
  /**
   Stop reasons seen, in arrival order; the last one is the stream's.
   */
  const stopReasons: string[] = [];
  /**
   Stop details seen, in arrival order; the last one is the stream's.
   */
  const detailsSeen: Readonly<Record<string, unknown>>[] = [];

  for (const line of bodyText.split('\n',)) {
    /**
     Event payload of this line; empty lines hold none.
     */
    const payload = ssePayloadOf({ line, },);
    if (payload === '')
      continue;
    /**
     What the payload reads as.
     */
    const reading = readFrame({ payload, },);
    if (reading.kind !== 'frame')
      continue;
    /**
     Parsed frame.
     */
    const { frame, } = reading;
    /**
     Which frame this is.
     */
    const kind = stringField({
      fields: frame,
      name: 'type',
    },);
    if (kind === 'message_start') {
      /**
       Message envelope the opening frame carries.
       */
      const { message, } = frame;
      if (isJsonRecord(message,))
        foldUsageInto({
          holder: message,
          counts,
        },);
      detailsSeen.push(...stopDetailsOf({ frame, },),);
    }
    if (kind === 'message_delta') {
      foldUsageInto({
        holder: frame,
        counts,
      },);
      /**
       Delta the frame carries, which holds the stop reason.
       */
      const { delta, } = frame;
      if (isJsonRecord(delta,)) {
        /**
         Reason the delta reports, empty while the model is still going.
         */
        const reason = stringField({
          fields: delta,
          name: 'stop_reason',
        },);
        if (reason !== '')
          stopReasons.push(reason,);
      }
      detailsSeen.push(...stopDetailsOf({ frame, },),);
    }
  }

  /**
   Reason the stream stopped on, empty when none was reported.
   */
  const stopReason = stopReasons.at(-1,) ?? '';

  /**
   Details the stream ended with, when it sent any.
   */
  const details = detailsSeen.at(-1,);

  /**
   Category the details named, when they named one.
   */
  const category = (details === undefined)
    ? ''
    : stringField({
      fields: details,
      name: 'category',
    },);

  /**
   Whether the stream reported both its prompt and its completion count.
   */
  const counted = (counts.input
    .length
    > 0) && (counts.output
      .length
      > 0);

  return {
    ...((stopReason === '') ? {} : { stopReason, }),
    ...((stopReason === 'refusal')
      ? { refusal: (category === '') ? {} : { category, }, }
      : {}),
    ...(counted
      ? {
        usage: {
          input: counts.input
            .at(-1,)
            ?? 0,
          cacheWrite: counts.cacheWrite
            .at(-1,)
            ?? 0,
          cacheRead: counts.cacheRead
            .at(-1,)
            ?? 0,
          output: counts.output
            .at(-1,)
            ?? 0,
        },
      }
      : {}),
  };
}

//endregion Claude stream facts
