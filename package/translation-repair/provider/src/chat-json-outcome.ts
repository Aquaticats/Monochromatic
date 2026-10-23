import { tagged, } from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  contextRoot,
  rendersAsNothing,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { stripChannelMarker, } from './channel-marker.ts';
import type {
  ChatJsonOutcome,
  ChatTextReply,
  ChatTextRequest,
  JsonSchemaResponseFormat,
} from './chat-contract.ts';
import {
  type EnumCaseFolded,
  foldEnumCase,
} from './enum-case-fold.ts';
import {
  parseAnswerJson,
  stripCodeFence,
  stripThinkBlock,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { detectRefusalShape, } from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Chat json outcome
// HOW A REPLY BECOMES AN OUTCOME, for every provider rather than for one.
//
// Lifted out of `synthetic-client.ts` whole when the second provider arrived.
// Nothing in this ladder is provider-specific: it reads text a model wrote and
// decides whether that text is an answer, a refusal, or a mismatch. Copying it
// into the second client would have been the more obvious move and the worse
// one, because every fix this ladder has taken was a real defect found live,
// and a copy takes none of the next ones.
//
// WHAT IT HAS LEARNED, each step standing for a measured failure:
//
//   The API's own refusal field outranks every heuristic over content.
//
//   The thinking channel is split off before anything judges the text, because
//   deliberation harmlessly contains refusal-like phrasing and would otherwise
//   be read as a refusal.
//
//   Provider token-limit markers outrank parsing because a syntactically
//   complete prefix does not prove that model generation completed.
//
//   Truncation inside unmarked thinking is its own outcome, since it sends a
//   reader to the token ceiling rather than to the prompt.
//
//   A channel marker is stripped AND logged, never silently dropped: the
//   2026-08-13 recurrence was diagnosable only because the raw opening had
//   been recorded.
//
//   The fence stripper runs twice, because it cannot see a fence hidden behind
//   a marker, and a reply of a marker then a fenced object would otherwise be
//   lost to the very defect just repaired.
//
//   Without a token-limit marker, content that parses and passes the guard
//   WINS even when it quotes refusal-like phrasing; the refusal scan runs
//   only on parse failure.
//
//   An abandoned opening ahead of the object is read past, never refused:
//   reasoning streams (Bedrock's gpt-oss-120b, OpenRouter's Makora route for
//   deepseek-v4-flash-0731) wrote `{ {   "choice`, `{"{"resolution` and
//   `{"best": 1{"best": 1, ...` on 2026-09-08, and the guard still judges
//   what was read (`json-false-start.ts`, the seventeenth class).
//
//   A value that differs from a listed `enum` or `const` value only in capitalization is read as the listed one when
//   the schema the caller sent lists exactly one such value, and logged (`enum-case-fold.ts`): the structured outputs
//   page does not guarantee the capitalization, and a guard compares a label exactly.

/**
 Logger root for the provider-neutral reply reader.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Usage fragment carried onto every outcome, present only when reported.

 @param reply - reply whose usage is forwarded

 @returns Spreadable fragment carrying usage, or nothing

 @example
 ```ts
 const spread = usageSpreadOf({ reply, },);
 ```
 */
function usageSpreadOf(
  { reply, }: { readonly reply: ChatTextReply; },
): Pick<ChatTextReply, 'usage'> {
  /**
   Usage as the provider reported it, absent when it did not.
   */
  const { usage, } = reply;

  return (usage === undefined) ? {} : { usage, };
}

/**
 The response format a request carried, as a fragment a reading spreads in.

 @param request - request whose reply is being read

 @returns Spreadable fragment carrying the request's response format, or nothing where it sent none

 @example
 ```ts
 const outcome = readJsonOutcome({ modelId, reply, validate, ...responseFormatSpreadOf({ request, },), },);
 ```
 */
export function responseFormatSpreadOf(
  { request, }: { readonly request: ForeignBorrowed<Pick<ChatTextRequest, 'responseFormat'>>; },
): Pick<ChatTextRequest, 'responseFormat'> {
  /**
   Format the request sent, absent when it asked for plain text.
   */
  const { responseFormat, } = request;

  return (responseFormat === undefined) ? {} : { responseFormat, };
}

/**
 Takes a parsed reply's capital-only differences from the values its request's schema lists back to the schema's
 spelling, where the request carried a schema.

 @param responseFormat - format the request sent, absent when it asked for plain text

 @param value - reply parsed from JSON

 @returns Value to read onward and each value taken back; where no format was sent, the value itself and no
 values taken back

 @example
 ```ts
 const { value, folds, } = foldedByFormat({ responseFormat, value: { verdict: 'Consistent', }, },);
 ```
 */
function foldedByFormat(
  {
    responseFormat,
    value,
  }: {
    readonly responseFormat?: ForeignBorrowed<JsonSchemaResponseFormat>;
    readonly value: unknown;
  },
): EnumCaseFolded {
  if (responseFormat === undefined)
    return {
      value,
      folds: [],
    };

  /**
   The envelope holding the schema the model was asked to answer in.
   */
  const { json_schema: jsonSchema, } = responseFormat;
  return foldEnumCase({
    schema: jsonSchema.schema,
    value,
  },);
}

/**
 Names why the model stopped, when the provider said, for a mismatch detail.

 A REPLY THAT STOPPED EARLY IS NOT A MALFORMED ONE, and the mismatch reason
 must preserve that distinction: token-limit markers are refused before
 parsing, while other stop reasons are retained here for content that does
 not parse.

 @param reply - reply whose stop reason is named

 @returns Clause to append to a detail, empty when the provider said nothing

 @example
 ```ts
 const stopped = stoppedNote({ reply, },);
 ```
 */
function stoppedNote(
  { reply, }: { readonly reply: ChatTextReply; },
): string {
  /**
   Reason as the provider delivered it, absent when it omitted one.
   */
  const { finishReason, } = reply;

  if (finishReason === undefined)
    return '';
  return ` (model stopped with finish_reason=${finishReason})`;
}

/**
 Whether the provider says generation ended at its token ceiling.

 Anthropic Messages reports `max_tokens`;
 OpenAI-compatible Chat Completions reports `length`.
 Either marker invalidates even parseable content because syntax does not
 prove every intended member was generated.

 @param reply - needed because provider protocol chooses the stop-reason spelling

 @returns Whether callers must refuse this answer regardless of parse result

 @example
 ```ts
 isTruncatingFinishReason({ reply: { text: '{}', finishReason: 'max_tokens', }, },);
 ```
 */
function isTruncatingFinishReason(
  { reply, }: { readonly reply: ChatTextReply; },
): boolean {
  return (reply.finishReason === 'max_tokens')
    || (reply.finishReason === 'length');
}

/**
 Reads one raw reply into the outcome a caller acts on.

 REFUSALS AND MISMATCHES ARE DATA, never exceptions: calling unreliable models
 is what this pipeline does, so an answer it cannot use is an ordinary result
 and only provider protocol failures throw. Nothing here throws at all.

 @param modelId - model that produced this reply, for the log lines only

 @param reply - raw text reply, whichever provider served it

 @param validate - caller's guard admitting parsed content

 @param responseFormat - format the caller sent with the request, when it sent one: a value of the reply that differs
 from a value its schema lists under `enum` or `const` only in capitalization is read as the listed value, and each
 such reading is logged as a warning (`enum-case-fold.ts` holds the rule and why). Absent, the reply is read as it came

 @returns Outcome as data: ok, refusal-shaped, or schema-mismatch

 @example
 ```ts
 const outcome = readJsonOutcome({ modelId, reply, validate: isVerdict, },);
 ```
 */
export function readJsonOutcome<ValueT,>(
  {
    modelId,
    reply,
    validate,
    responseFormat,
  }: {
    readonly modelId: string;
    readonly reply: ChatTextReply;
    readonly validate: (value: unknown,) => value is ValueT;
    readonly responseFormat?: JsonSchemaResponseFormat;
  },
): ChatJsonOutcome<ValueT> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readJsonOutcome.name,
    l,
  },);

  /**
   Usage spread carried onto every outcome for budget observability.
   */
  const usageSpread = usageSpreadOf({ reply, },);

  // The API's own refusal field outranks every content heuristic. Its text
  // stands for content that shows a reader nothing (ledger B40), so the record
  // keeps the words the model wrote rather than its padding.
  if (reply.refusal !== undefined) {
    rl.debug(`${modelId}: refusal-shaped (api-refusal-field)`,);
    return {
      kind: 'refusal-shaped',
      rawText: rendersAsNothing({ text: reply.text, },) ? reply.refusal : reply.text,
      marker: 'api-refusal-field',
      ...usageSpread,
    };
  }

  if (isTruncatingFinishReason({ reply, },)) {
    /**
     Provider marker retained as evidence without inspecting answer content.
     */
    const stopped = stoppedNote({ reply, },);
    rl.debug(`${modelId}: schema-mismatch (truncated completion)${stopped}`,);
    return {
      kind: 'schema-mismatch',
      rawText: reply.text,
      reason: 'truncated-completion',
      detail: `provider reported a truncating completion${stopped}`,
      ...usageSpread,
    };
  }

  /**
   Answer channel with any embedded thinking block split off;
   refusal scanning and parsing judge the answer,
   never the deliberation.
   */
  const {
    answer,
    truncatedThinking,
  } = stripThinkBlock({ text: reply.text, },);

  if (truncatedThinking) {
    rl.debug(`${modelId}: schema-mismatch (truncated thinking)`,);
    return {
      kind: 'schema-mismatch',
      rawText: reply.text,
      reason: 'truncated-thinking',
      detail: 'output was truncated inside its thinking block;'
        + ' raise or omit maxTokens (thinking tokens count against it)',
      ...usageSpread,
    };
  }

  /**
   Fence-stripped answer, with any truncated channel marker removed and
   reported rather than silently dropped.
   */
  const {
    content,
    marker,
  } = stripChannelMarker({ text: stripCodeFence({ text: answer, },), },);

  if (marker !== '')
    rl.info(`${modelId}: stripped channel marker ${JSON.stringify(marker,)} ahead of JSON`,);

  /**
   Parse attempt over the unwrapped answer, fence-stripped a SECOND time
   because the first pass was looking at the marker.
   */
  const attempt = parseAnswerJson({
    text: (marker === '') ? content : stripCodeFence({ text: content, },),
  },);

  if (!attempt.parsed) {
    /**
     Refusal classification of the unparseable answer.
     */
    const scan = detectRefusalShape({ text: answer, },);

    if (scan.refusalShaped) {
      rl.debug(`${modelId}: refusal-shaped (${scan.marker})`,);
      return {
        kind: 'refusal-shaped',
        rawText: reply.text,
        marker: scan.marker,
        ...usageSpread,
      };
    }

    /**
     Why the model stopped, when the provider said, named in the detail.
     */
    const stopped = stoppedNote({ reply, },);
    rl.debug(`${modelId}: schema-mismatch (unparseable)${stopped}`,);
    return {
      kind: 'schema-mismatch',
      rawText: reply.text,
      reason: 'unparseable-json',
      detail: `content is not valid JSON: ${attempt.detail}${stopped}`,
      ...usageSpread,
    };
  }

  if (attempt.abandoned > 0) {
    rl.warn(
      `${modelId}: json false start: read the object past an abandoned opening of ${
        String(attempt.abandoned,)
      } ${
        wordForCount({
          count: attempt.abandoned,
          one: 'char',
          many: 'chars',
        },)
      }`,
    );
  }
  if (attempt.trailing > 0) {
    rl.warn(
      `${modelId}: json trailing text: read the value before ${String(attempt.trailing,)} trailing ${
        wordForCount({
          count: attempt.trailing,
          one: 'char',
          many: 'chars',
        },)
      } (ledger P8)`,
    );
  }

  /**
   Parsed content with each capital-only difference from a listed value taken back to the schema's spelling, and
   those differences. Nothing is taken back where the caller sent no response format.
   */
  const {
    value: candidate,
    folds,
  } = foldedByFormat({
    ...((responseFormat === undefined) ? {} : { responseFormat, }),
    value: attempt.value,
  },);

  for (const fold of folds) {
    rl.warn(
      `${modelId}: enum capitalization: field ${fold.path} came as ${
        JSON.stringify(fold.received,)
      } and is read as ${JSON.stringify(fold.taken,)}, the spelling the schema lists`,
    );
  }

  if (!validate(candidate,)) {
    rl.debug(`${modelId}: schema-mismatch (guard rejected)`,);
    return {
      kind: 'schema-mismatch',
      rawText: reply.text,
      reason: 'caller-guard-rejected',
      detail: 'content parsed as JSON but failed the caller schema guard',
      ...usageSpread,
    };
  }

  return {
    kind: 'ok',
    value: candidate,
    rawText: reply.text,
    ...usageSpread,
  };
}

//endregion Chat json outcome
