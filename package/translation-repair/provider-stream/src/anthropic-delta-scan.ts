import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  ChannelDelta,
  DeltaScanner,
  StreamChannel,
} from './stream-delta-scan.ts';
import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';

import { ssePayloadOf, } from './sse-data-line.ts';

//region Anthropic delta scan
// The SAME `DeltaScanner` the OpenAI-shaped path produces, fed by Anthropic
// Messages events instead.
//
// WHY NORMALIZE RATHER THAN GUARD TWICE. Everything downstream of a scanner
// consumes `ChannelDelta` and nothing else: `stream-degeneration.ts` counts
// repetition per channel, `stream-idle-guard.ts` watches the gap between
// deltas, `stream-runaway-watch.ts` decides when a call has stopped making
// progress, and `stream-overrun.ts` bounds volume. Every threshold in that set
// came from measurement, and the straggler and idle windows were re-derived
// after finding the median premise wrong by a factor of eighty. A second
// implementation of those guards would be unmeasured, and would drift from
// this one invisibly. One scanner interface, two wire formats.
//
// THE THINKING CHANNEL IS TYPED HERE RATHER THAN SNIFFED. Sniffing once cost 47
// percent of calls to a scanner that had to guess which of two field spellings
// carried reasoning, because the OpenAI-shaped provider names it
// `reasoning_content` on some models and `reasoning` on others. Anthropic
// sends a `thinking` content block with `thinking_delta` events, so the
// channel is declared by the wire and that whole class of blindness cannot
// recur through this path.
//
// `event:` LINES ARE IGNORED ON PURPOSE. Every Anthropic frame carries its own
// `type` inside the JSON payload, so reading the payload is both sufficient and
// robust against a server that reorders or omits the `event:` line.

/**
 Logger root for this scanner.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Sentinel OpenRouter's Messages endpoint appends after `message_stop`,
 captured 2026-09-03; it is not a frame and must not count as an unreadable
 one, for the same reason an empty keep-alive payload does not.
 */
const DONE_SENTINEL = '[DONE]';

/**
 Block type carrying the model's reasoning rather than its answer.
 */
const THINKING_BLOCK = 'thinking';

/**
 Reading given to a delta type this scanner does not carry.
 */
const UNREAD = 'unread';

/**
 How this scanner reads one delta type: the channel its text belongs to and
 the field the text rides in.

 @example
 ```ts
 const reading: DeltaReading = { channel: 'content', field: 'text', };
 ```
 */
type DeltaReading = {
  /**
   Channel the text is filed under.
   */
  readonly channel: StreamChannel;

  /**
   Field of the delta that carries the text.
   */
  readonly field: string;
};

/**
 Reading a delta gets, or that this scanner does not read its type.

 A THIRD STATE RATHER THAN A NULLISH UNION, so an unread delta is a named
 reading instead of an absence a caller has to remember to check.

 @example
 ```ts
 const routing: DeltaRouting = UNREAD;
 ```
 */
type DeltaRouting = DeltaReading | typeof UNREAD;

/**
 Delta types this scanner reads, each with its channel and its text field.

 `input_json_delta` IS THE ANSWER CHANNEL, which is the one mapping here that
 is not obvious. Under forced tool use the model's whole reply is the tool
 call's arguments, so those fragments are the content a consumer is waiting
 for. Routing them to `reasoning` would leave every schema'd call looking
 like a model that thought at length and answered nothing.

 ONE TABLE FOR BOTH (ledger B91). The channel and the field once sat in two
 maps, on the claim that one table could not hold two types sharing a
 channel under different fields; a table of pairs holds them. Split, a type
 with a channel and no field was possible, and a thinking block lent its
 channel to every type, so a type this scanner does not read was filed as
 reasoning and its text read from the field named by the empty string.

 A MAP, since the key is the type the provider's stream names: a plain object
 answered a delta typed `constructor` or `__proto__` with what every object
 inherits, and filed its text under that as a channel (ledger B77).
 */
const DELTA_READINGS: ReadonlyMap<string, DeltaReading> = new Map([
  [
    'text_delta',
    {
      channel: 'content',
      field: 'text',
    },
  ],
  [
    'thinking_delta',
    {
      channel: 'reasoning',
      field: 'thinking',
    },
  ],
  [
    'input_json_delta',
    {
      channel: 'content',
      field: 'partial_json',
    },
  ],
],);

/**
 Delta types carrying the answer itself, which no enclosing block may demote
 to reasoning.

 THE ASYMMETRY IS THE POINT, and `text_delta` is deliberately NOT here. A
 provider has been seen sending plain text deltas inside a thinking block, so
 a `text_delta` must still yield to whatever the block declared. A tool-call
 argument fragment cannot be deliberation: it is the structured answer by
 construction, filling a schema this pipeline sent.

 CAPTURED FROM THE WIRE on 2026-08-25. `qwen3.8-max` on Charm Hyper
 opens index 1 as `tool_use`, then opens THE SAME INDEX again as `thinking`,
 and thereafter interleaves `thinking_delta` and `input_json_delta` under it.
 The block map keeps the later declaration, so the block-type override filed
 the whole reply as reasoning: 70 of 71 streams reported zero content while
 every one of them cast a ballot with a full prose reason.

 NOT COSMETIC. `stream-runaway-watch.ts` bounds the content channel and
 deliberately leaves reasoning alone, so an answer filed as reasoning escapes
 the volume cap and runs to the straggler deadline. That seat was cut 12 times
 in 71, the highest on the roster by two and a half times, and each cut is a
 lost voice on a panel already paid for.

 WHY THIS RATHER THAN FIXING THE BLOCK MAP: keeping the FIRST declaration
 would also route this capture correctly, but only because `tool_use` happened
 to arrive first. This holds whichever order the two declarations come in.
 */
const ANSWER_DELTAS: ReadonlySet<string> = new Set(['input_json_delta',],);

/**
 Reads one string field off a parsed object, ignoring anything else.

 @param fields - parsed object to read

 @param name - field wanted

 @returns Field value, or empty when absent or not a string

 @example
 ```ts
 const text = stringField({ fields: delta, name: 'text', },);
 ```
 */
function stringField(
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
 Where a frame said its block sits, or that it named no position.

 A DISCRIMINATED RESULT rather than `number | undefined`, because this repo
 models absence without nullish unions and index zero is a real position that
 a falsy check would read as absence.

 @example
 ```ts
 const at: FrameIndex = { present: true, index: 0, };
 ```
 */
type FrameIndex =
  | {
    readonly present: true;

    /**
     Position the frame named.
     */
    readonly index: number;
  }
    | { readonly present: false; };

/**
 Reads the position a frame named for its block.

 @param fields - parsed frame to read

 @returns Position, or that the frame named none

 @example
 ```ts
 const at = frameIndex({ fields: frame, },);
 ```
 */
function frameIndex(
  { fields, }: { readonly fields: Readonly<Record<string, unknown>>; },
): FrameIndex {
  /**
   Raw value under `index`, of unknown type.
   */
  const { index: value, } = fields;

  if ((typeof value) !== 'number')
    return { present: false, };
  return {
    present: true,
    index: value,
  };
}

/**
 How a delta is read, preferring the channel its block declared.

 A `text_delta` INSIDE A THINKING BLOCK is reasoning despite its type, which
 is why the block's declaration outranks the delta's channel. Providers have
 been observed to send plain text deltas inside a thinking block, and reading
 only the delta type would file that as the answer.

 {@link ANSWER_DELTAS} IS THE EXCEPTION, and its own note carries the wire
 capture that made it necessary: a block declaration cannot demote a tool-call
 argument fragment, because that fragment is the answer by construction.

 A TYPE THIS SCANNER DOES NOT READ STAYS UNREAD, inside a thinking block too
 (ledger B91): the block names a channel, never a field to read text from.

 @param deltaType - `type` of the delta object

 @param blockType - type the enclosing block declared, empty when unknown

 @returns Channel and field to read this delta by, or that this type is not read

 @example
 ```ts
 const routing = routingFor({ deltaType: 'text_delta', blockType: 'thinking', },);
 ```
 */
function routingFor(
  {
    deltaType,
    blockType,
  }: {
    readonly deltaType: string;
    readonly blockType: string;
  },
): DeltaRouting {
  /**
   How this scanner reads the type, absent for a type it does not read.
   */
  const reading = DELTA_READINGS.get(deltaType,);
  if (reading === undefined)
    return UNREAD;
  if ((blockType === THINKING_BLOCK) && (!ANSWER_DELTAS.has(deltaType,))) {
    return {
      channel: 'reasoning',
      field: reading.field,
    };
  }
  return reading;
}

/**
 Parses one event payload, reporting rather than raising on malformed JSON.

 A DISCRIMINATED RESULT rather than a nullable frame, matching `readPayload`
 in `stream-delta-scan.ts`. Returning the caught error as the value would pass
 {@link isJsonRecord}, which narrows only that a value is a non-null object other than an array,
 so an unreadable line would be read as an empty frame instead of counted.

 @param payload - one `data:` line's payload, already unwrapped

 @returns Parsed frame, or that it could not be read

 @example
 ```ts
 const parsed = readPayload({ payload: '{"type":"ping"}', },);
 ```
 */
function readPayload(
  { payload, }: { readonly payload: string; },
):
  | {
    readonly ok: true;
    readonly frame: Readonly<Record<string, unknown>>;
  }
    | { readonly ok: false; }
  {
  try {
    /**
     Whatever that payload parsed to, before any shape is assumed.
     */
    const frame: unknown = JSON.parse(payload,);

    if (!isJsonRecord(frame,))
      return { ok: false, };
    return {
      ok: true,
      frame,
    };
  } catch (error) {
    l.debug(`anthropic stream frame did not parse: ${errorName({ error, },)}`,);
    return { ok: false, };
  }
}

/**
 A running scanner over one Anthropic Messages stream body.

 Produces the same {@link DeltaScanner} the OpenAI-shaped path produces, so
 every stream guard consumes both wire formats without knowing which it has.

 @returns Scanner fed by `feed`

 @example
 ```ts
 const scanner = scanAnthropicDeltas();
 for (const chunk of chunks)
   for (const delta of scanner.feed({ chunk, },))
     detectors[delta.channel].notifyText({ text: delta.text, },);
 ```
 */
export function scanAnthropicDeltas(): DeltaScanner {
  /**
   Partial line held from an earlier chunk, the unreadable tally, and the
   type each open block declared.

   A RECORD RATHER THAN LOOSE BINDINGS so the factory root holds no mutable
   variable, matching `scanStreamDeltas`.
   */
  const state = {
    carry: '',
    unreadable: 0,
    blockTypes: new Map<number, string>(),
  };

  /**
   Records what an opening block declared, so its deltas can be attributed.

   A START THIS SCANNER CANNOT READ IS COUNTED (ledger B93): one naming no
   index, carrying no block, or whose block names no type string. A start
   is read for its declaration alone, so such a frame yields nothing it was
   read for.

   A START AT AN INDEX SUPERSEDES WHAT WAS DECLARED THERE, readable or not.
   The block there is now this one, so an unreadable start leaves its index
   with no known type, and the deltas that follow are read by their own
   type rather than under the block before it.

   @param frame - parsed `content_block_start` frame

   @example
   ```ts
   openBlock({ frame, },);
   ```
   */
  function openBlock(
    { frame, }: { readonly frame: Readonly<Record<string, unknown>>; },
  ): void {
    /**
     Position this block occupies in the message.
     */
    const at = frameIndex({ fields: frame, },);
    if (!at.present) {
      l.debug('anthropic stream content_block_start frame names no index',);
      state.unreadable += 1;
      return;
    }

    /**
     Block descriptor the frame carried.
     */
    const { content_block: block, } = frame;
    if (!isJsonRecord(block,)) {
      l.debug('anthropic stream content_block_start frame carries no block object',);
      state.unreadable += 1;
      state
        .blockTypes
        .delete(at.index,);
      return;
    }

    /**
     Type the block declared, of unknown type until checked.
     */
    const { type: blockType, } = block;
    if ((typeof blockType) !== 'string') {
      l.debug('anthropic stream content_block_start block names no type string',);
      state.unreadable += 1;
      state
        .blockTypes
        .delete(at.index,);
      return;
    }

    state
      .blockTypes
      .set(
      at.index,
      blockType,
      );
  }

  /**
   Reads one delta frame into whatever generated text it carried.

   A DELTA FRAME THIS SCANNER IS MEANT TO READ AND CANNOT IS COUNTED
   (ledger B93): one with no delta object, and a delta of a type it reads
   whose text field is absent or not a string. Passing those over left the
   tally at zero for frames the completion reader refuses. An empty
   fragment is read, not unreadable, and a type this scanner does not read
   is passed over whatever it holds, as the streaming documentation asks
   of types added later.

   @param frame - parsed `content_block_delta` frame

   @returns Deltas it carried, empty for a delta type this does not read

   @example
   ```ts
   const deltas = readDelta({ frame, },);
   ```
   */
  function readDelta(
    { frame, }: { readonly frame: Readonly<Record<string, unknown>>; },
  ): readonly ChannelDelta[] {
    /**
     Delta descriptor the frame carried.
     */
    const { delta, } = frame;
    if (!isJsonRecord(delta,)) {
      l.debug('anthropic stream content_block_delta frame carries no delta object',);
      state.unreadable += 1;
      return [];
    }

    /**
     Kind of delta this is, which names both channel and text field.
     */
    const deltaType = stringField({
      fields: delta,
      name: 'type',
    },);

    /**
     Position this delta belongs to, used to recover its block's type.
     */
    const at = frameIndex({ fields: frame, },);

    /**
     Type the enclosing block declared, empty where nothing declared one.
     */
    const declared = at.present
      ? state
        .blockTypes
        .get(at.index,)
      : '';

    /**
     That type, with an unopened block reading as no declaration at all.
     */
    const blockType = declared ?? '';

    /**
     Channel and field to read this delta by, or that this type is not read here.
     */
    const routing = routingFor({
      deltaType,
      blockType,
    },);
    if (routing === UNREAD)
      return [];

    /**
     Text this delta carried, of unknown type until checked.
     */
    const text = delta[routing.field];
    if ((typeof text) !== 'string') {
      l.debug(`anthropic stream ${deltaType} carries no ${routing.field} string`,);
      state.unreadable += 1;
      return [];
    }
    if (text === '')
      return [];

    return [{
      channel: routing.channel,
      text,
    },];
  }

  /**
   Reads one complete line, returning whatever text it carried.

   @param line - one line, without its newline

   @returns Deltas it carried, empty for every non-delta frame

   @example
   ```ts
   const deltas = readLine({ line: 'data: {"type":"ping"}', },);
   ```
   */
  function readLine(
    { line, }: { readonly line: string; },
  ): readonly ChannelDelta[] {
    /**
     Payload this line carries, empty for a comment or another field.
     */
    const payload = ssePayloadOf({ line, },);

    if (payload === '')
      return [];
    if (payload === DONE_SENTINEL)
      return [];

    /**
     Parsed frame, or a note that this line could not be read.
     */
    const parsed = readPayload({ payload, },);

    if (!parsed.ok) {
      state.unreadable += 1;
      return [];
    }

    /**
     Which Anthropic frame this is.
     */
    const frameType = stringField({
      fields: parsed.frame,
      name: 'type',
    },);

    if (frameType === 'content_block_start') {
      openBlock({ frame: parsed.frame, },);
      return [];
    }
    if (frameType === 'content_block_delta')
      return readDelta({ frame: parsed.frame, },);
    return [];
  }

  return {
    feed({ chunk, },): readonly ChannelDelta[] {
      /**
       Everything unparsed so far, including this chunk.
       */
      const pending = state.carry + chunk;

      /**
       Lines the pending text splits into; the last is kept for next time.
       */
      const lines = pending.split('\n',);
      // A SPLIT ALWAYS YIELDS ONE PIECE AT LEAST, the empty text included, so
      // there is always a last line to hold back.
      state.carry = nonNullishOrThrow(lines.pop(),);

      return lines.flatMap(function ofLine(line,): readonly ChannelDelta[] {
        return readLine({ line, },);
      },);
    },

    unreadableFrames(): number {
      return state.unreadable;
    },

    // CHARM HYPER FRONTS ONE UPSTREAM AND NAMES NONE. OpenRouter's Messages
    // endpoint does name one, on `message_start`, but that endpoint is used
    // for measurement only (`doc/planning/translation-repair-openrouter-2026-09-03.md`),
    // so this scanner reports the absence rather than reading a field no
    // production stream carries.
    servedBy(): string {
      return '';
    },
  };
}

//endregion Anthropic delta scan
