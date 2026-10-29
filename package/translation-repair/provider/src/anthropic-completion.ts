import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  readFrame,
  requireWholeAnthropicMessage,
  stringField,
} from './anthropic-whole-message.ts';
import {
  contextRoot,
  rendersAsNothing,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type ExtractedCompletion,
  MalformedCompletionError,
  ssePayloadOf,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';

/**
 Logger root for the Anthropic completion reader.
 */
const l = contextRoot({ tag: 'translation-repair', },);

//region Anthropic completion
// Reassembles one drained Anthropic Messages stream into the same
// `ExtractedCompletion` the OpenAI-shaped path produces, so `model-content.ts`
// reads an answer without knowing which provider served it.
//
// THE ANSWER IS USUALLY A TOOL CALL HERE. Charm Hyper produces schema-valid
// output only under tool use, so on most calls the model emits no text at all
// and the whole reply arrives as `input_json_delta` fragments of the tool's
// arguments. Those fragments ARE the answer, and concatenating them yields the
// JSON a validator then reads.
//
// THE BODY IS CHECKED WHOLE FIRST, by `requireWholeAnthropicMessage` in
// `anthropic-whole-message.ts`, which the retry ladder asks too: a terminator
// frame, and no error event.
//
// `[DONE]` IS SKIPPED, NOT REFUSED. OpenRouter's Messages endpoint appends an
// `event: data` frame carrying the OpenAI-style `data: [DONE]` sentinel after
// `message_stop` (captured 2026-09-03, `deepseek/deepseek-v4-flash-0731` via
// DigitalOcean), and Charm Hyper sends no such frame. The sentinel is not an
// event and carries nothing, so it is the one non-JSON payload this reader
// lets through; every other unreadable payload still refuses the body.
//
// THINKING IS DISCARDED HERE ON PURPOSE. `thinking_delta` is the model's
// private channel; `anthropic-delta-scan.ts` routes it to the guards that watch
// for a runaway, and this file reads only the answer.

/**
 Sentinel some gateways append after the terminator, carrying no event.

 SPELLED HERE RATHER THAN IMPORTED from `stream-completion.ts`: that file's
 constant is private to the OpenAI-shaped reader, and the two readers are
 kept independent so a change to one wire format cannot reach the other.
 */
const DONE_SENTINEL = '[DONE]';

/**
 Everything one pass over the body accumulates.
 */
type AnthropicFold = {
  /**
   Text fragments, in arrival order, from `text_delta` frames: the answer
   when the model answered in prose, and prose set aside when it also called
   the tool.
   */
  readonly textParts: string[];

  /**
   Tool-argument fragments, in arrival order, from `input_json_delta`
   frames: the answer whenever there are any.
   */
  readonly toolParts: string[];

  /**
   Why the model stopped, as `message_delta` reported it.
   */
  readonly stopReasons: string[];

  /**
   Prompt tokens after the last cache breakpoint, which is all Anthropic's
   `input_tokens` counts, in arrival order.
   */
  readonly inputTokens: number[];

  /**
   Prompt tokens written to the cache, in arrival order.
   */
  readonly cacheWriteTokens: number[];

  /**
   Prompt tokens read from the cache, in arrival order.
   */
  readonly cacheReadTokens: number[];

  /**
   Completion tokens, in arrival order.
   */
  readonly completionTokens: number[];
};

/**
 Usage fields this reader counts, each beside the series it lands in.

 THE PROMPT IS THREE OF THEM (ledger B88). Anthropic's `input_tokens` counts
 only the tokens after the last cache breakpoint; the prompt-caching
 documentation gives the prompt as that plus the tokens read from and
 written to the cache. Reading `input_tokens` alone reported a cached call
 as a short one.
 */
const USAGE_SERIES = [
  [
    'input_tokens',
    'inputTokens',
  ],
  [
    'cache_creation_input_tokens',
    'cacheWriteTokens',
  ],
  [
    'cache_read_input_tokens',
    'cacheReadTokens',
  ],
  [
    'output_tokens',
    'completionTokens',
  ],
] as const;

/**
 Records the token counts a usage block carried, ignoring absent ones.

 THE TWO FRAMES NEST IT DIFFERENTLY, which is why the holder is a parameter
 rather than read off the frame here. `message_delta` puts `usage` at the top
 level, while `message_start` puts it inside `message` alongside the model
 name and the null stop reason. Reading only the top level would silently drop
 every prompt-token count.

 A NULL IS ABSENT, not zero: OpenRouter's Messages endpoint opened with
 `cache_read_input_tokens: null` and closed with a count (captured
 2026-09-03), and the count is the one kept.

 @param holder - object that directly holds the `usage` block

 @param fold - accumulator to append to

 @example
 ```ts
 foldUsage({ holder: frame, fold, },);
 ```
 */
function foldUsage(
  {
    holder,
    fold,
  }: {
    readonly holder: Readonly<Record<string, unknown>>;
    readonly fold: AnthropicFold;
  },
): void {
  /**
   Usage block, absent while a frame carries none.
   */
  const { usage, } = holder;
  if (!isJsonRecord(usage,))
    return;

  for (const [field, series,] of USAGE_SERIES) {
    /**
     Count this frame reported under that field, of unknown type.
     */
    const reported = usage[field];
    if ((typeof reported) === 'number') {
      fold[series]
        .push(reported,);
    }
  }
}

/**
 Folds one `message_start` frame's usage, which it nests inside `message`.

 @param frame - parsed message-start frame

 @param fold - accumulator to append to

 @example
 ```ts
 foldStart({ frame, fold, },);
 ```
 */
function foldStart(
  {
    frame,
    fold,
  }: {
    readonly frame: Readonly<Record<string, unknown>>;
    readonly fold: AnthropicFold;
  },
): void {
  /**
   Message envelope this frame opens, which holds the usage block.
   */
  const { message, } = frame;
  if (!isJsonRecord(message,))
    return;

  foldUsage({
    holder: message,
    fold,
  },);
}

/**
 Reads the text an answer-carrying delta must hold.

 @param delta - delta descriptor of a frame that carries the answer

 @param kind - delta type, which names the field

 @param field - field the text rides in

 @returns Fragment, empty only where the provider sent it empty

 @throws {@link MalformedCompletionError} when the field is absent or not a string

 @example
 ```ts
 const fragment = answerFragmentOf({ delta, kind: 'text_delta', field: 'text', },);
 ```
 */
function answerFragmentOf(
  {
    delta,
    kind,
    field,
  }: {
    readonly delta: Readonly<Record<string, unknown>>;
    readonly kind: 'text_delta' | 'input_json_delta';
    readonly field: 'text' | 'partial_json';
  },
): string {
  /**
   Raw value under that field, of unknown type.
   */
  const value = delta[field];

  if ((typeof value) !== 'string') {
    throw new MalformedCompletionError({
      wireFormat: 'anthropic',
      detail: `${kind} carries no ${field} string`,
    },);
  }
  return value;
}

/**
 Folds one `content_block_delta` frame's answer text, if it carried any.

 READS BOTH `text_delta` AND `input_json_delta`, because a model asked for a
 tool answers in the second and a model asked for prose answers in the first,
 and this pipeline uses both shapes.

 A FRAGMENT THE FRAME SHOULD HOLD AND DOES NOT REFUSES THE BODY (ledger B89).
 The streaming documentation gives every such frame a `delta`, every
 `text_delta` a `text` and every `input_json_delta` a `partial_json`;
 folding past one returned the answer with a piece missing and nothing said,
 which a prose answer then carried into a page. A delta of a type this
 reader does not fold, a new one included, is passed over, and so is one
 naming no type, which cannot be told from a type added later.

 @param frame - parsed delta frame

 @param fold - accumulator to append to

 @throws {@link MalformedCompletionError} when the frame carries no delta, or an answer delta carries no text

 @example
 ```ts
 foldDelta({ frame, fold, },);
 ```
 */
function foldDelta(
  {
    frame,
    fold,
  }: {
    readonly frame: Readonly<Record<string, unknown>>;
    readonly fold: AnthropicFold;
  },
): void {
  /**
   Delta descriptor the frame carried.
   */
  const { delta, } = frame;
  if (!isJsonRecord(delta,)) {
    throw new MalformedCompletionError({
      wireFormat: 'anthropic',
      detail: 'content_block_delta frame carries no delta object',
    },);
  }

  /**
   Kind of delta, which names the field its text rides in.
   */
  const kind = stringField({
    fields: delta,
    name: 'type',
  },);

  if (kind === 'text_delta') {
    fold
      .textParts
      .push(answerFragmentOf({
        delta,
        kind,
        field: 'text',
      },),);
  }
  if (kind === 'input_json_delta') {
    fold
      .toolParts
      .push(answerFragmentOf({
        delta,
        kind,
        field: 'partial_json',
      },),);
  }
}

/**
 Folds one `message_delta` frame's stop reason and usage.

 @param frame - parsed message-delta frame

 @param fold - accumulator to append to

 @example
 ```ts
 foldMessageDelta({ frame, fold, },);
 ```
 */
function foldMessageDelta(
  {
    frame,
    fold,
  }: {
    readonly frame: Readonly<Record<string, unknown>>;
    readonly fold: AnthropicFold;
  },
): void {
  foldUsage({
    holder: frame,
    fold,
  },);

  /**
   Delta descriptor, which carries the stop reason on this frame kind.
   */
  const { delta, } = frame;
  if (!isJsonRecord(delta,))
    return;

  /**
   Why the model stopped, absent while it is still going.
   */
  const reason = stringField({
    fields: delta,
    name: 'stop_reason',
  },);

  if (reason !== '')
    fold
      .stopReasons
      .push(reason,);
}

/**
 Token counts as a READER sees them, with no way to append.

 A SEPARATE TYPE FROM {@link AnthropicFold} because the accumulator is
 deliberately mutable and this function only reads it. Taking the accumulator
 here would hand a reader the ability to change what it is reporting on.

 @example
 ```ts
 const counts: ReportedCounts = { inputTokens: [41,], cacheWriteTokens: [], cacheReadTokens: [], completionTokens: [12,], };
 ```
 */
type ReportedCounts = Readonly<Record<(typeof USAGE_SERIES)[number][1], readonly number[]>>;

/**
 Reads a count series as the total it reports.

 THE LAST REPORT, because `message_delta` counts are cumulative (the
 streaming documentation's warning) and may repeat or update what
 `message_start` reported. Zero where the stream reported none, which only
 a cache series may be read as: a stream reporting no cache field used no
 cache this reader can count. The input and output series are checked for
 a report before this is asked (ledger B94).

 @param series - one count series, in arrival order

 @returns Its last report, zero when it holds none

 @example
 ```ts
 const fresh = latestOf({ series: counts.inputTokens, },);
 ```
 */
function latestOf({ series, }: { readonly series: readonly number[]; },): number {
  return series.at(-1,) ?? 0;
}

/**
 Usage fragment for the result, present only when the stream reported both
 its prompt count and its completion count.

 BOTH OR NOTHING (ledger B94), as `readUsage` in `completion-shape.ts` reads
 the OpenAI-compatible wire. Usage was reported once any series held a
 report, and a series never reported read as zero, so a stream that sent
 its completion count alone came back with a prompt count of zero nobody
 measured. A cache series still reads as zero when absent, as
 {@link latestOf} explains.

 @param counts - token counts the body reported, read only

 @param l - logger of the reader that folded them

 @returns Spreadable fragment carrying usage, or nothing

 @example
 ```ts
 const fragment = usageOf({ counts: fold, l: rl, },);
 ```
 */
function usageOf(
  {
    counts,
    l: parentLogger,
  }: {
    readonly counts: ReportedCounts;
    readonly l: Logger;
  },
): Pick<ExtractedCompletion, 'usage'> {
  /**
   Logger pre-tagged with this function's name.
   */
  const ul = tagged({
    tag: usageOf.name,
    l: parentLogger,
  },);

  /**
   Every count series, named so no read is a three-step chain.
   */
  const {
    inputTokens,
    cacheWriteTokens,
    cacheReadTokens,
    completionTokens,
  } = counts;

  /**
   Whether the stream reported the count before any cache breakpoint.
   */
  const promptReported = inputTokens.length > 0;

  /**
   Whether the stream reported its completion count.
   */
  const completionReported = completionTokens.length > 0;

  if ((!promptReported) && (!completionReported))
    return {};
  if ((!promptReported) || (!completionReported)) {
    ul.debug(
      `usage left unreported: prompt count ${promptReported ? 'reported' : 'missing'}, `
        + `completion count ${completionReported ? 'reported' : 'missing'}`,
    );
    return {};
  }

  /**
   Prompt tokens the model read: those after the last cache breakpoint, plus
   those written to and read from the cache (ledger B88).
   */
  const prompt = latestOf({ series: inputTokens, },)
    + latestOf({ series: cacheWriteTokens, },)
    + latestOf({ series: cacheReadTokens, },);

  /**
   Completion tokens, whose last report is the running total.
   */
  const completion = latestOf({ series: completionTokens, },);

  return {
    usage: {
      prompt_tokens: prompt,
      completion_tokens: completion,
      total_tokens: prompt + completion,
    },
  };
}

/**
 Reassembles one drained Anthropic Messages body into a completion.

 @param provider - provider whose body this is, carried onto a refusal the body states

 @param bodyText - whole drained `text/event-stream` body

 @returns Answer text, stop reason, and usage

 @throws {@link MalformedCompletionError} when an event is not JSON, an error event arrived, or `message_stop` never did

 @example
 ```ts
 const extracted = extractAnthropicCompletion({ provider: 'hyper', bodyText: reply.bodyText, },);
 ```
 */
export function extractAnthropicCompletion(
  {
    provider,
    bodyText,
  }: {
    readonly provider: ProviderName;
    readonly bodyText: string;
  },
): ExtractedCompletion {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: extractAnthropicCompletion.name,
    l,
  },);

  /**
   Everything this pass over the body accumulates.
   */
  const fold: AnthropicFold = {
    textParts: [],
    toolParts: [],
    stopReasons: [],
    inputTokens: [],
    cacheWriteTokens: [],
    cacheReadTokens: [],
    completionTokens: [],
  };

  requireWholeAnthropicMessage({
    provider,
    bodyText,
  },);

  /**
   Body lines, folded into the answer.
   */
  const lines = bodyText.split('\n',);

  for (const rawLine of lines) {
    /**
     Event payload of this line; empty lines fold nothing.
     */
    const payload = ssePayloadOf({ line: rawLine, },);
    if (payload === '')
      continue;
    if (payload === DONE_SENTINEL)
      continue;

    /**
     What this payload reads as.
     */
    const reading = readFrame({ payload, },);

    if (reading.kind === 'not-json') {
      throw new MalformedCompletionError({
        wireFormat: 'anthropic',
        detail: 'stream event is not JSON',
        cause: reading.cause,
      },);
    }
    if (reading.kind === 'not-object')
      throw new MalformedCompletionError({
        wireFormat: 'anthropic',
        detail: 'stream event is not a JSON object',
      },);

    /**
     Parsed event payload.
     */
    const { frame, } = reading;

    /**
     Which frame this is.
     */
    const kind = stringField({
      fields: frame,
      name: 'type',
    },);

    if (kind === 'message_start')
      foldStart({
        frame,
        fold,
      },);
    if (kind === 'content_block_delta')
      foldDelta({
        frame,
        fold,
      },);
    if (kind === 'message_delta')
      foldMessageDelta({
        frame,
        fold,
      },);
  }

  /**
   Stop reason, when the stream reported one.
   */
  const stopReason = fold
    .stopReasons
    .at(-1,)
    ?? '';

  /**
   Tool arguments, whole, when the model called the tool at all.
   */
  const toolAnswer = fold
    .toolParts
    .join('',);

  /**
   Prose, whole, which is the answer only when no tool was called.
   */
  const prose = fold
    .textParts
    .join('',);

  // THE TOOL'S ARGUMENTS ARE THE ANSWER WHENEVER THERE ARE ANY. Under
  // `tool_choice: auto` a model may write a text block before the tool block;
  // folding both into one string handed the schema reader prose glued to JSON
  // and lost the voice, and a Hyper-only seat has no other stack for a second
  // opinion. The prose is set aside and its size logged, never its
  // content.
  //
  // ARGUMENTS THAT SHOW A READER NOTHING ARE NONE (ledger B40): asked of an
  // empty-string comparison, a tool call carrying only spaces or a zero-width
  // space was handed to the schema reader in place of a prose answer.
  /**
   Whether the tool call carried an answer.
   */
  const toolAnswered = !rendersAsNothing({ text: toolAnswer, },);
  if (toolAnswered && (prose !== ''))
    rl.info(
      `tool answer kept, ${String(prose.length,)} ${
        wordForCount({
          count: prose.length,
          one: 'character',
          many: 'characters',
        },)
      } of prose set aside`,
    );

  return {
    text: toolAnswered ? toolAnswer : prose,
    ...((stopReason === '') ? {} : { finishReason: stopReason, }),
    ...usageOf({
      counts: fold,
      l: rl,
    },),
  };
}

//endregion Anthropic completion
