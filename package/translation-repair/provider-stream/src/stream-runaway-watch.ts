import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type ChannelDelta,
  STREAM_CHANNELS,
  type StreamChannel,
} from './stream-delta-scan.ts';
import {
  DEFAULT_WIRE_FORMAT,
  scannerFor,
  type StreamWireFormat,
} from './stream-wire-format.ts';
import {
  type DegenerationDetector,
  watchForDegeneration,
} from './stream-degeneration.ts';
import {
  type RecurrenceDetector,
  watchForRecurrence,
} from './stream-recurrence-watch.ts';

//region Stream runaway watch
// ONE THING THE DRAIN CAN CALL, so that reading a raw chunk and deciding
// whether the call has run away is a single step at the transport seam rather
// than three of them spread through the read loop.
//
// It owns the scanner and two detectors per channel: the windowed ratio from
// `stream-degeneration.ts` and the buffered recurrence check from
// `stream-recurrence-watch.ts`. Per channel, because a model that thinks in
// circles while writing a fine answer and one that writes in circles after
// thinking clearly are different failures, and pooling them would let either
// excuse the other. Per detector kind, because the two catch different
// periods: the ratio catches short cycling on a small sample, and the
// recurrence check catches the longer periods the ratio's window arithmetic
// is blind to.
//
// THE VERDICT IS A VALUE, not an exception. The drain decides what to do with
// it, which keeps the decision to end a call in the place that owns the
// connection rather than buried here.

/**
 Decimal places the distinct ratio is reported to.

 Enough to distinguish the degenerate range, which runs near 0.001, from the
 threshold at 0.1.
 */
const RATIO_DIGITS = 4;

/**
 Generated characters kept for the opening excerpt, combined across both
 channels in arrival order.

 WELL PAST WHAT A LOG LINE SHOWS. `stream-cut.ts` slices its own excerpt
 down to a much narrower width; this cap only has to stop the kept text
 from growing with a stream that runs forever, so it is set with generous
 margin rather than tuned to that narrower width, and the two stay free to
 change independently.
 */
const OPENING_TEXT_CAP = 200;

/**
 Answer characters one call may produce before the watch calls it a runaway.

 MEASURED, NOT CHOSEN, AND RE-MEASURED ONCE. The first bound was ten
 thousand, set on 545 completed calls whose largest legitimate emission was
 4,278 characters. That population carried no reading-lane call, and reading
 a picture is the one role here that legitimately emits at length. Pooling
 every log that records the two channels separately gives 1,887 real
 completions, whose largest legitimate emission is 11,392, a transcription.
 Ten thousand would have ended seven of those calls.

 Thirty-two thousand clears the largest legitimate emission by better than
 two and a half times, ends none of the 1,887, and still reaches a runaway
 about four times sooner than the repetition detectors, which need 131,072
 characters before a cycle becomes visible to them. Both recorded degenerate
 calls ran just past that threshold, and the two runaway cuts pulled
 9,699,969 and 11,366,983 raw characters off the wire before a different
 guard stopped them.

 A DEFAULT RATHER THAN A CONSTANT THE GUARD FREEZES IN: the measurement
 behind it comes from one bed, so a call site that knows its own role emits
 more should be able to say so, the way the exchange timeout already can.
 `doc/decision/translation-repair-runaway-call-termination.md` records what
 the number rests on.
 */
const CONTENT_OVERRUN_CAP = 32_000;

/**
 What the watch makes of a stream so far.

 @example
 ```ts
 const verdict: RunawayVerdict = { kind: 'continuing', };
 ```
 */
export type RunawayVerdict = {
  readonly kind: 'continuing';
} | {
  readonly kind: 'runaway';

  /**
   Channel that stopped saying anything new.
   */
  readonly channel: StreamChannel;

  /**
   Share of its recent windows that were distinct.
   */
  readonly distinctRatio: number;

  /**
   Characters that channel produced before the verdict, which is what the
   call cost before it was stopped.
   */
  readonly charsSeen: number;
} | {
  readonly kind: 'overrun';

  /**
   Channel that exceeded its bound.
   */
  readonly channel: StreamChannel;

  /**
   Characters that channel produced before the verdict.
   */
  readonly charsSeen: number;

  /**
   Bound that was exceeded, carried so a caller reports what was expected
   rather than only what arrived.
   */
  readonly cap: number;
};

/**
 A running watch over one call's raw stream.

 @example
 ```ts
 const watch = watchRunaway();
 const verdict = watch.notifyChunk({ chunk, },);
 ```
 */
export type RunawayWatch = {
  /**
   Takes the next raw chunk and says whether the call has run away.
   */
  readonly notifyChunk: (input: { readonly chunk: string; },) => RunawayVerdict;

  /**
   Payload lines the scanner could not read, which should stay at zero and is
   worth reporting because a changed wire format would otherwise look like a
   stream that simply produced nothing.
   */
  readonly unreadableFrames: () => number;

  /**
   Reads how many generated characters have arrived on each channel so far.

   READS THE DETECTORS' OWN TOTALS rather than keeping a second tally: they
   already count every character `notifyChunk` routes to them, and a
   progress line asking the same question a second way would only invite the
   two counts to drift.
   */
  readonly generatedChars: () => {
    readonly content: number;
    readonly reasoning: number;
  };

  /**
   Reads the first generated characters seen so far, combined across both
   channels in arrival order.

   GENERATED TEXT, NOT THE WIRE. A raw excerpt always opens with the
   server-sent-event envelope, `data: {"id":"` and whatever follows,
   because every frame's JSON wrapper is identical by construction; this
   reads whichever channel the model actually started producing, thinking
   or answering, which is the question an opening excerpt exists to answer.
   */
  readonly openingText: () => string;

  /**
   Reads the upstream the gateway named for this stream, empty where the
   wire names none; read off the scanner so a cut stream is attributed
   from what had already arrived.
   */
  readonly servedBy: () => string;
};

/**
 Builds a watch that reads raw stream chunks and reports a runaway call.

 @returns Watch fed by `notifyChunk`

 @example
 ```ts
 const watch = watchRunaway();
 for (const chunk of chunks) {
   const verdict = watch.notifyChunk({ chunk, },);
   if (verdict.kind === 'runaway')
     break;
 }
 ```
 */
export function watchRunaway(
  {
    contentCap = CONTENT_OVERRUN_CAP,
    wireFormat = DEFAULT_WIRE_FORMAT,
  }: {
    readonly contentCap?: number;
    readonly wireFormat?: StreamWireFormat;
  } = {},
): RunawayWatch {
  /**
   Turns raw server-sent events into generated text, per channel.

   CHOSEN BY GRAMMAR since a second provider joined. Every threshold this watch applies was
   measured on one wire and applies to both, but only if the events are read
   at all: a stream drained with the wrong reader shows an empty answer
   channel, which every guard here reads as a perfectly well-behaved call.
   */
  const scanner = scannerFor({ wireFormat, },);

  /**
   One windowed-ratio detector per channel, judged separately.
   */
  const ratioDetectors: Record<StreamChannel, DegenerationDetector> = {
    content: watchForDegeneration(),
    reasoning: watchForDegeneration(),
  };

  /**
   One buffered-recurrence detector per channel, fed the same text as its
   ratio counterpart and judged separately from it: this catches the
   periods the ratio detector's window arithmetic is blind to.
   */
  const recurrenceDetectors: Record<StreamChannel, RecurrenceDetector> = {
    content: watchForRecurrence(),
    reasoning: watchForRecurrence(),
  };

  /**
   First generated characters seen, combined across channels in arrival
   order and capped at {@link OPENING_TEXT_CAP}.

   A RECORD RATHER THAN A LOOSE BINDING so the factory root holds no mutable
   variable, matching the rest of this module's state.
   */
  const opening = { text: '', };

  /**
   Reads whichever channel has gone wrong, if either has, by either
   detector, and then whether the answer channel has simply produced too
   much.

   REPETITION IS READ FIRST, but that ordering only decides a stream both
   checks have already flagged, which measurement says is rare. The two
   observed repetition endings on the ANSWER channel were called degenerate
   only after 131,078 content characters, so the volume bound at 10,000 now
   reaches them thirteen times earlier and they report `overrun` where they
   used to report `degenerate`. That is a deliberate trade: the same call is
   ended either way, far sooner, and the reasoning channel is untouched
   because no volume bound applies to it.

   @returns Verdict for the first watched channel that has run away

   @example
   ```ts
   const verdict = readChannels();
   ```
   */
  function readChannels(): RunawayVerdict {
    /**
     Channels reporting degeneration, in the order `STREAM_CHANNELS` gives.
     */
    const failing = STREAM_CHANNELS.flatMap(function judge(channel,): readonly RunawayVerdict[] {
      /**
       What this channel's ratio detector currently says.
       */
      const ratioVerdict = ratioDetectors[channel]
        .verdict();
      if (ratioVerdict.kind === 'degenerate')
        return [{
          kind: 'runaway' as const,
          channel,
          distinctRatio: ratioVerdict.distinctRatio,
          charsSeen: ratioVerdict.charsSeen,
        },];

      /**
       What this channel's recurrence detector currently says, read only
       when the ratio detector found nothing: a channel already flagged
       has nothing left to decide.
       */
      const recurrenceVerdict = recurrenceDetectors[channel]
        .verdict();
      if (recurrenceVerdict.kind === 'degenerate')
        return [{
          kind: 'runaway' as const,
          channel,
          // NOT A SAMPLED RATIO. This verdict comes from a verbatim
          // recurrence rather than window sampling, so no distinct-window
          // share was ever computed. Reported as 0, the most repetitive
          // value the field can hold, since the checked span IS a complete
          // duplicate of an earlier one: honest within the field's existing
          // meaning (lower reads as more repetitive) without inventing a
          // second field only this path would ever populate.
          distinctRatio: 0,
          charsSeen: recurrenceVerdict.charsSeen,
        },];

      return [];
    },);

    /**
     Repetition verdict, if either detector on either channel found one.
     */
    const [repeated,] = failing;
    if (repeated !== undefined)
      return repeated;

    /**
     Answer characters produced so far, which is the volume the bound reads.

     THE ANSWER CHANNEL ONLY. A reasoning bound was measured and refused: on
     every thinking model here reasoning precedes content, so a bound on
     silent reasoning fires mid-stream on calls that were about to answer,
     and at 40,000 it would have killed 22 of 545 completed calls.
     */
    const contentChars = ratioDetectors.content
      .charsSeen();
    if (contentChars >= contentCap)
      return {
        kind: 'overrun',
        channel: 'content',
        charsSeen: contentChars,
        cap: contentCap,
      };

    return { kind: 'continuing', };
  }

  return {
    notifyChunk({ chunk, },): RunawayVerdict {
      /**
       Generated text this chunk completed, on either channel.
       */
      const deltas = scanner.feed({ chunk, },);

      deltas.forEach(function route(delta: ChannelDelta,): void {
        // Both detector kinds owning this channel see every piece of text,
        // in the same arrival order: each judges independently, and a
        // channel neither detector has flagged is still continuing.
        ratioDetectors[delta.channel]
          .notifyText({ text: delta.text, },);
        recurrenceDetectors[delta.channel]
          .notifyText({ text: delta.text, },);

        /**
         Characters kept so far, read once so the cap check names a single
         value rather than a member chain.
         */
        const keptSoFar = opening.text
          .length;

        // Stops appending once the cap is reached, so a call that never ends
        // never grows this string past OPENING_TEXT_CAP.
        if (keptSoFar < OPENING_TEXT_CAP)
          opening.text += delta.text;
      },);
      return readChannels();
    },

    unreadableFrames(): number {
      return scanner.unreadableFrames();
    },

    generatedChars(): {
      readonly content: number;
      readonly reasoning: number;
    } {
      return {
        content: ratioDetectors.content
          .charsSeen(),
        reasoning: ratioDetectors.reasoning
          .charsSeen(),
      };
    },

    openingText(): string {
      return opening.text;
    },

    servedBy(): string {
      return scanner.servedBy();
    },
  };
}

/**
 Raised when a call is ended because it stopped saying anything new.

 ITS OWN CLASS, so a lost voice can be recorded with this cause rather than
 folded in with a stall or with steering. They call for opposite responses: a
 stall is worth retrying, and a model that has begun repeating itself will
 repeat itself again.

 @example
 ```ts
 throw new StreamDegenerateError({ label, channel: 'reasoning', distinctRatio: 0.02, charsSeen: 400_000, rawChars: 36_000_000, },);
 ```
 */
export class StreamDegenerateError extends Error {
  /**
   Declares this message safe to forward: it names the model, the channel and two measurements of the stream.
   */
  readonly messageNamesOnly: true = true;

  /**
   Model or endpoint whose stream ran away.

   CARRIED AS A PROPERTY, not only baked into the message, for the same
   reason `StreamCutShortError` carries it: attributing a stream to the
   endpoint rather than the model makes a per-model figure unreadable, and
   every chat-completions call shares one endpoint across the whole roster.
   */
  readonly label: string;

  /**
   Channel that stopped saying anything new.
   */
  readonly channel: StreamChannel;

  /**
   Share of recent windows that were distinct when the call was ended.
   */
  readonly distinctRatio: number;

  /**
   Characters that channel produced before it was ended.
   */
  readonly charsSeen: number;

  /**
   Raw wire characters the stream delivered before the call was ended,
   envelope included: the unit the abandoned-spend reckoning divides by its
   measured raw-characters-per-token ratio (ledger P7). `charsSeen` counts
   one channel's decoded characters, about a hundredth of it.
   */
  readonly rawChars: number;

  /**
   @param label - what was being called, for the message

   @param channel - channel that ran away

   @param distinctRatio - share of recent windows that were distinct

   @param charsSeen - characters produced on that channel

   @param rawChars - raw wire characters delivered before the end

   @example
   ```ts
   const error = new StreamDegenerateError({
     label: 'critic',
     channel: 'reasoning',
     distinctRatio: 0.02,
     charsSeen: 400_000,
     rawChars: 36_000_000,
   },);
   ```
   */
  constructor(
    {
      label,
      channel,
      distinctRatio,
      charsSeen,
      rawChars,
    }: {
      readonly label: string;
      readonly channel: StreamChannel;
      readonly distinctRatio: number;
      readonly charsSeen: number;
      readonly rawChars: number;
    },
  ) {
    super(
      `${label}: ended a runaway call, ${channel} channel repeated itself at `
        + `${distinctRatio.toFixed(RATIO_DIGITS,)} distinct over ${String(charsSeen,)} ${
          wordForCount({
            count: charsSeen,
            one: 'character',
            many: 'characters',
          },)
        }`,
    );
    this.name = 'StreamDegenerateError';
    this.label = label;
    this.channel = channel;
    this.distinctRatio = distinctRatio;
    this.charsSeen = charsSeen;
    this.rawChars = rawChars;
  }
}

//endregion Stream runaway watch
