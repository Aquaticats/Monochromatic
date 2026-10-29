import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { parseModelJson, } from './model-content.ts';

import { ssePayloadOf, } from './sse-data-line.ts';

//region Stream delta scan
// PULLS GENERATED TEXT OUT OF A STREAM AS IT ARRIVES, which is what makes
// `watchForDegeneration` usable at all: the detector must be fed generated
// text, and the transport carries server-sent events whose JSON envelope is
// identical around every token. Fed the raw body the detector would condemn
// every healthy stream, because the envelope would dominate its sample.
//
// BOTH CHANNELS ARE SCANNED, and the reasoning channel is the one that matters
// most. A model can degenerate entirely inside its thinking, repeating "I will
// output." forever while emitting no answer at all. A scanner that read only
// `content` would hand the detector an empty string, which reads as a short
// reply rather than a runaway one, so the very worst case would be the one
// case nothing caught.
//
// THREE DELIVERY SHAPES EXIST HERE and all three are covered. THE PROVIDER DOES
// NOT SPELL THE THINKING CHANNEL THE SAME WAY FOR EVERY MODEL, which cost this
// scanner two models' entire thinking channels until it was measured. One
// streaming call per model on 2026-08-21 counted the frames each spelling
// arrives in:
//
// HALF THE ROSTER USES EACH SPELLING. Measured one call per model:
//
//   `reasoning_content`, which this scanner already read:
//     zai-org/GLM-5.2         328 of 329 frames
//     Qwen/Qwen3.6-27B        463 of 511 frames
//     moonshotai/Kimi-K3       79 of 148 frames
//
//   `reasoning`, which it did not:
//     zai-org/GLM-4.7-Flash   871 of 1029 frames
//     openai/gpt-oss-120b      46 of  206 frames
//     nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4
//                              43 of  232 frames
//
// Reading only `reasoning_content`, as this scanner did, handed the detector an
// empty string for every thinking token those last three models produced. Across
// every stream record on disk before the fix, 2731 of 5864 calls came from them,
// so this was not an edge of the roster: it was 47% of all traffic, thinking
// invisible from end to end. After the fix all six report thinking on every call,
// and the largest median belongs to a model that had reported none at all.
//
// The third shape is a `<think>` block embedded in `content`, which
// `model-content.ts` strips after the fact and which needs nothing special here,
// since such text arrives as content and is scanned as content.
//
// A RENAMED FIELD IS INVISIBLE TO THE UNREADABLE TALLY, which is why this went
// unseen for so long. That counter rises only when a payload fails to parse.
// These frames parsed perfectly and simply were not read, so the number built to
// make a changed wire format visible sat at zero throughout.
//
// NOTHING HERE THROWS. It runs inside the drain loop for every chunk of every
// call, so a frame it cannot read must not take down a stream that is working.
// Unreadable frames are COUNTED rather than swallowed, and the count is
// reported, so a provider that changes its wire format shows up as a rising
// number instead of as silence.

/**
 Payload the provider sends to mark the end of a stream, which is not JSON.
 */
const DONE_PAYLOAD = '[DONE]';

/**
 Field the answer channel arrives in, which every model observed here spells
 the same way.
 */
const CONTENT_KEY = 'content';

/**
 Fields the thinking channel arrives in, in precedence order.

 ORDERED RATHER THAN MERGED, so a provider that begins sending both spellings
 as aliases of one another contributes that text ONCE. Merging them would
 double every thinking character on such a model and push the degeneration
 detector toward a verdict on volume the model never produced.

 `reasoning_content` comes first because it is the spelling this repository's
 own notes already name, so a model sending both stays counted as before.
 */
const REASONING_KEYS = [
  'reasoning_content',
  'reasoning',
] as const;

/**
 Top-level field a gateway that fronts many upstreams uses to name the one
 that served this stream.

 OPENROUTER PUTS `"provider":"ModelRun"` ON EVERY CHUNK (captured
 2026-09-03); Synthetic and Charm Hyper front one upstream each and send no
 such field, so their streams read as naming none and their progress lines
 are unchanged. READ HERE, IN THE SCANNER THE DRAIN ALREADY RUNS ON EVERY
 CHUNK, because the streams worth attributing are the cut ones, and a cut
 stream never reaches the client's post-drain readers: the first live
 OpenRouter pass cut five streams across three models and nothing could
 say which endpoint had served any of them.
 */
const SERVED_BY_KEY = 'provider';

/**
 Every channel a piece of generated text can arrive on, in the order a
 verdict is reported for them.

 REASONING FIRST, because a runaway there is the case that produces no answer
 at all, so when both have gone wrong it is the more informative one to name.

 THE TYPE IS DERIVED FROM THIS LIST, so a channel cannot be added to the type
 without being placed in the order, and the watch that walks this list cannot
 leave it unwatched.
 */
export const STREAM_CHANNELS = [
  'reasoning',
  'content',
] as const;

/**
 Which channel a piece of generated text arrived on.

 KEPT APART rather than merged into one string, so a degeneration verdict can
 name the channel it happened in. "The model repeated itself while thinking"
 and "the model repeated itself in its answer" are different failures to
 diagnose, and the first is invisible in the answer.

 @example
 ```ts
 const channel: StreamChannel = 'reasoning';
 ```
 */
export type StreamChannel = typeof STREAM_CHANNELS[number];

/**
 One piece of generated text, and where it came from.

 @example
 ```ts
 const delta: ChannelDelta = { channel: 'reasoning', text: 'I will output.', };
 ```
 */
export type ChannelDelta = {
  /**
   Channel this text arrived on.
   */
  readonly channel: StreamChannel;

  /**
   Text itself, exactly as sent.
   */
  readonly text: string;
};

/**
 A running scanner over one stream's raw body.

 @example
 ```ts
 const scanner = scanStreamDeltas();
 const deltas = scanner.feed({ chunk, },);
 ```
 */
export type DeltaScanner = {
  /**
   Takes the next raw chunk and returns whatever generated text it completed.
   */
  readonly feed: (input: { readonly chunk: string; },) => readonly ChannelDelta[];

  /**
   Payload lines that could not be read, which should stay at zero.
   */
  readonly unreadableFrames: () => number;

  /**
   Upstream the gateway named on the first frame that named one, or empty
   for a wire whose frames name none.
   */
  readonly servedBy: () => string;
};

/**
 One parsed frame's `delta` object, or an empty stand-in for a frame that
 carries none.

 READONLY, because a writable index signature is not deeply readonly and this
 value crosses a function boundary that has no business mutating it.
 */
type DeltaFields = Readonly<Record<string, unknown>>;

/**
 Reads one string field off a delta object, ignoring anything else.

 @param fields - parsed `delta` object from a frame

 @param key - field to read

 @returns Its text, or empty when absent, null, or not a string

 @example
 ```ts
 const text = textField({ fields, key: 'reasoning_content', },);
 ```
 */
function textField(
  {
    fields,
    key,
  }: {
    readonly fields: DeltaFields;
    readonly key: string;
  },
): string {
  /**
   Whatever sits at that key, which may be absent or null.
   */
  const value = fields[key];
  return ((typeof value) === 'string') ? value : '';
}

/**
 Reads the thinking channel off a delta object, whichever field this model
 spells it in.

 FIRST NON-EMPTY WINS rather than first present: a model sending
 `reasoning_content` as an empty string alongside a populated `reasoning`
 would otherwise read as having produced no thinking at all, which is the
 failure this function exists to end.

 @param fields - parsed `delta` object from a frame

 @returns Thinking text, or empty when this frame carried none under any
 spelling

 @example
 ```ts
 const thinking = reasoningField({ fields, },);
 ```
 */
function reasoningField({ fields, }: { readonly fields: DeltaFields; },): string {
  for (const key of REASONING_KEYS) {
    /**
     Thinking text under this spelling, empty when absent.
     */
    const text = textField({
      fields,
      key,
    },);
    if (text !== '')
      return text;
  }
  return '';
}

/**
 Reads the `delta` object out of a parsed frame.

 GUARDS RATHER THAN ASSERTIONS, using the package's own `isJsonRecord` and
 `isJsonArray`: this walks a value the provider controls, so every step has to
 be checked rather than declared. An assertion here would state a shape the
 wire never promised.

 @param frame - parsed payload of one `data:` line, an object, since
 `readPayload` counts any other as unreadable

 @returns Its first choice's delta, or an empty object when the frame carries
 none, which usage-only frames legitimately do

 @example
 ```ts
 const fields = deltaOf({ frame, },);
 ```
 */
function deltaOf({ frame, }: { readonly frame: Readonly<Record<string, unknown>>; },): DeltaFields {
  /**
   Choices array, absent on the final usage frame.
   */
  const { choices, } = frame;
  if (!isJsonArray(choices,))
    return {};

  /**
   First choice, the only one requested.
   */
  const [first,] = choices;
  if (!isJsonRecord(first,))
    return {};

  /**
   Its delta, absent when the choice carries only a finish reason.
   */
  const { delta, } = first;
  if (!isJsonRecord(delta,))
    return {};
  return delta;
}

/**
 Reads the upstream's name off a parsed frame.

 @param frame - parsed payload of one `data:` line, an object, since
 `readPayload` counts any other as unreadable

 @returns Name as the gateway spelled it, or empty when the frame names none

 @example
 ```ts
 const name = servedByOf({ frame, },);
 ```
 */
function servedByOf({ frame, }: { readonly frame: Readonly<Record<string, unknown>>; },): string {
  /**
   Whatever sits at the field, of unknown type until checked.
   */
  const name = frame[SERVED_BY_KEY];
  return ((typeof name) === 'string') ? name : '';
}

/**
 What one payload line turned out to be.

 @example
 ```ts
 const read: ReadPayload = { ok: true, frame: { choices: [], }, };
 ```
 */
type ReadPayload = {
  readonly ok: true;

  /**
   Parsed frame, an object; every field's shape is the provider's and is
   checked where it is read.
   */
  readonly frame: Readonly<Record<string, unknown>>;
} | {
  readonly ok: false;

  /**
   Why it could not be read, kept so a future caller can report it. A parse
   refusal is named by class and quotes none of the frame V8 refused.
   */
  readonly reason: string;
};

/**
 Parses one payload, reporting failure as a value.

 NEVER THROWS, because this runs on every chunk of every call and one
 unreadable frame must leave a working stream working.

 A PAYLOAD THAT PARSES TO ANYTHING BUT AN OBJECT IS UNREADABLE TOO: an array,
 a number or `null` carries no field this scanner reads, so it is counted
 with the frames that did not parse, as the Anthropic scanner
 (`anthropic-delta-scan.ts`) counts it, rather than read as a frame naming
 nothing.

 @param payload - text after the `data:` prefix

 @returns Parsed frame, or why there is none

 @example
 ```ts
 const read = readPayload({ payload: '{"choices":[]}', },);
 ```
 */
function readPayload({ payload, }: { readonly payload: string; },): ReadPayload {
  /**
   Parse of the payload, whose refusal is data that names its class and
   quotes none of the frame V8 refused.
   */
  const attempt = parseModelJson({ text: payload, },);
  if (!attempt.parsed) {
    return {
      ok: false,
      reason: attempt.detail,
    };
  }
  /**
   Whatever the payload parsed to, before any shape is assumed.
   */
  const frame: unknown = attempt.value;
  if (!isJsonRecord(frame,)) {
    return {
      ok: false,
      reason: 'parsed to something other than a JSON object',
    };
  }
  return {
    ok: true,
    frame,
  };
}

/**
 Builds a scanner that turns raw stream chunks into generated text.

 INCREMENTAL BY LINE, because a chunk boundary falls wherever the network puts
 it and routinely lands in the middle of a frame. Whatever follows the last
 newline is carried forward rather than parsed, so no frame is read twice and
 none is read in halves.

 NO REGEX, per `RG1`: the rule is "split on newlines, keep lines starting with
 a fixed prefix", which string methods state directly.

 @returns Scanner fed by `feed`

 @example
 ```ts
 const scanner = scanStreamDeltas();
 for (const chunk of chunks)
   for (const delta of scanner.feed({ chunk, },))
     detectors[delta.channel].notifyText({ text: delta.text, },);
 ```
 */
export function scanStreamDeltas(): DeltaScanner {
  /**
   Partial line held back from an earlier chunk, plus the unreadable tally.

   A RECORD RATHER THAN LOOSE BINDINGS so the factory root holds no mutable
   variable.
   */
  const state = {
    carry: '',
    unreadable: 0,
    servedBy: '',
  };

  /**
   Reads one complete line, returning whatever text it carried.

   @param line - one line, without its newline

   @returns Deltas it carried, empty for comments and the done marker

   @example
   ```ts
   const deltas = readLine({ line: 'data: {"choices":[]}', },);
   ```
   */
  function readLine({ line, }: { readonly line: string; },): readonly ChannelDelta[] {
    /**
     Payload this line carries, empty for a comment or another field.
     */
    const payload = ssePayloadOf({ line, },);
    if (payload === DONE_PAYLOAD)
      return [];

    // An empty payload is a legitimate keep-alive rather than a frame nobody
    // could read, and counting it would inflate the tally that exists to make a
    // changed wire format visible.
    if (payload === '')
      return [];

    /**
     Parsed frame, or a note that it could not be read.
     */
    const parsed = readPayload({ payload, },);

    if (!parsed.ok) {
      state.unreadable += 1;
      return [];
    }

    // The first frame that names an upstream settles it for the stream.
    if (state.servedBy === '')
      state.servedBy = servedByOf({ frame: parsed.frame, },);

    /**
     Delta object this frame carried.
     */
    const fields = deltaOf({ frame: parsed.frame, },);

    /**
     Answer text, if any.
     */
    const content = textField({
      fields,
      key: CONTENT_KEY,
    },);

    /**
     Thinking text, if any, under whichever field this model spells it in.
     */
    const reasoning = reasoningField({ fields, },);

    return [
      ...((content === '') ? [] : [{
        channel: 'content' as const,
        text: content,
      },]),
      ...((reasoning === '') ? [] : [{
        channel: 'reasoning' as const,
        text: reasoning,
      },]),
    ];
  }

  return {
    feed({ chunk, },): readonly ChannelDelta[] {
      /**
       Everything unparsed so far, this chunk included.
       */
      const buffer = state.carry + chunk;

      /**
       Lines in it, the last of which may be incomplete.
       */
      const lines = buffer.split('\n',);

      // Hold the trailing fragment back until its newline arrives. A SPLIT
      // ALWAYS YIELDS ONE PIECE AT LEAST, the empty string included, so there
      // is always a last line here to hold back.
      state.carry = nonNullishOrThrow(lines.at(-1,),);

      /**
       Lines that are complete, the trailing fragment excluded.
       */
      const complete = lines.slice(
        0,
        -1,
      );

      return complete.flatMap(function read(line,): readonly ChannelDelta[] {
        return readLine({ line, },);
      },);
    },

    unreadableFrames(): number {
      return state.unreadable;
    },

    servedBy(): string {
      return state.servedBy;
    },
  };
}

//endregion Stream delta scan
