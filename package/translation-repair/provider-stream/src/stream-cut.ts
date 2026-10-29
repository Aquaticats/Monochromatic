import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { wholeOpening, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { exchangeFailureText, } from './exchange-failure-text.ts';
import type { StreamProgress, } from './stream-idle-guard.ts';

//region Stream cut
// WHAT A CALL HAD ALREADY PRODUCED WHEN IT WAS CUT, and the one place that
// reports what a stream did.
//
// TWO LOSSES USED TO HAPPEN AT EVERY ABORT. `drainBody` accumulated its decoded
// chunks and the catch rethrew, dropping every one of them, so a call that had
// already delivered "It's a cat. It did a backflip. It cras" delivered nothing.
// And the progress sample was emitted after the try, so an aborted stream logged
// NO line at all.
//
// THE SECOND ONE INVALIDATED A MEASUREMENT rather than merely hiding one. Every
// latency figure read off a pass log was computed over surviving calls only. A
// 2026-08-17 reading of "1349 streams, mean firstByte 1822ms" excluded every
// abandoned call by construction, which is survivorship bias over exactly the
// population it was quoted to describe.
//
// WHAT THE PARTIAL TEXT SETTLES, immediately and with no further instrument: a
// call that never got a first byte leaves an empty string, and one cut off
// mid-reasoning leaves a truncated thinking block. Those want opposite remedies,
// the first to drop or replace the straggler and the second to allow it longer,
// and until now nothing on disk could tell them apart.

/**
 Logger root for stream reporting.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Most UTF-16 units of generated text to show in the log line, ending on a
 whole character (`wholeOpening`).

 ENOUGH TO SEE WHAT KIND OF TEXT IT IS and no more. The opening tells a
 thinking block from an answer from an empty cut, which is the whole diagnostic
 question, while a longer excerpt would put licensed corpus material into a run
 log that gets read, grepped and pasted into documents.
 */
const OPENING_CHARS = 80;

/**
 How a stream ended.

 FOUR VALUES rather than two, so a termination THIS SYSTEM CHOSE reads as
 its own outcome rather than as `cut`. A stall and a runaway call for
 opposite responses, a stall is worth retrying and a model that has begun
 repeating itself will repeat itself again, and a reader counting `cut`
 lines to measure stalls would otherwise count every deliberate termination
 among them, which is the same conflation `StreamDegenerateError` was given
 its own class to avoid, one layer further out.

 THE TWO CHOSEN ENDINGS STAY APART for the same reason they are apart from
 `cut`: `degenerate` is a stream that stopped saying anything new, and
 `overrun` is one that said far more than any legitimate call ever did.
 They are found by different evidence and pooling them would make either
 unreadable in a census of the other.
 */
export type StreamOutcome = 'completed' | 'cut' | 'degenerate' | 'overrun';

/**
 Raised when a stream was cut off, carrying what it had already delivered.

 WRAPS RATHER THAN REPLACES. The original failure is the `cause`, so a stall
 still reads as a stall and steering still reads as steering. The message
 states the cause through `exchangeFailureText`: a cause that declares its own
 sentence safe is repeated, a provider's status failure is named by class and
 HTTP status, and any other is named by class, since a stream's failure can be
 a runtime rejection whose message quotes what it refused. That makes this
 class safe to repeat, so it declares `messageNamesOnly`.

 @example
 ```ts
 throw new StreamCutShortError({
   label: 'hf:whiskers',
   partialText: 'It is a cat. It did a backflip. It cras',
   progress: { firstByteMs: 812, firstContentMs: 1_340, maxGapMs: 43, chars: 9_211, },
   cause: new Error('aborted',),
 },);
 ```
 */
export class StreamCutShortError extends Error {
  /**
   Declares this message safe to forward: it names the model, a count of
   characters and the cause by its own safe sentence, or by class and the
   HTTP status a provider answered with.
   */
  readonly messageNamesOnly: true = true;

  /**
   Model or endpoint whose stream was cut.
   */
  readonly label: string;

  /**
   Everything the stream delivered before it stopped, which is empty when it
   never produced a byte.
   */
  readonly partialText: string;

  /**
   What the stream did up to the cut.
   */
  readonly progress: StreamProgress;

  /**
   Decoded characters of the answer and of the reasoning the stream had produced, as its watch counted them; absent where
   the caller had no count. `partialText` is the stream as it arrived with its framing, about 27 times the reasoning in the
   cuts of Qwen on the run of TianqiChen66625, so an abandonment line reads these and names the wire length apart.
   */
  readonly generatedChars?: {
    readonly content: number;
    readonly reasoning: number;
  };

  /**
   @param label - model or endpoint

   @param partialText - text delivered before the cut

   @param progress - what the stream did

   @param generatedChars - decoded characters of the answer and of the reasoning the watch counted, where the caller had them

   @param cause - original failure, kept so its identity survives

   @example
   ```ts
   const error = new StreamCutShortError({ label, partialText, progress, generatedChars, cause, },);
   ```
   */
  constructor(
    {
      label,
      partialText,
      progress,
      generatedChars,
      cause,
    }: {
      readonly label: string;
      readonly partialText: string;
      readonly progress: StreamProgress;
      readonly generatedChars?: {
        readonly content: number;
        readonly reasoning: number;
      };
      readonly cause: unknown;
    },
  ) {
    super(
      `${label}: stream cut after ${String(partialText.length,)} ${
        wordForCount({
          count: partialText.length,
          one: 'character',
          many: 'characters',
        },)
      } (${exchangeFailureText({ error: cause, },)})`,
      { cause, },
    );
    this.name = 'StreamCutShortError';
    this.label = label;
    this.partialText = partialText;
    this.progress = progress;
    if (generatedChars !== undefined)
      this.generatedChars = generatedChars;
  }
}

/**
 Reports what one stream did, on the path that finished and the path that did
 not.

 ONE FUNCTION FOR BOTH, which is the point: two call sites drifted before, and
 the one that never logged was the one carrying the calls worth measuring.

 RETURNS THE LINE IT LOGS, so the formatting is testable directly rather than
 only by capturing a logger's side effect. The caller is not expected to use
 the return value; `void`-typed call sites remain valid.

 REPORTS GENERATED CHARACTERS, NOT RAW ONES, both for the per-channel count
 and for the opening excerpt. `progress.chars` already names itself `raw
 chars` and keeps counting wire bytes, envelope included; nothing else here
 reads the raw stream at all. Fed the raw text instead, the excerpt would
 always open with the server-sent-event envelope, `data: {"id":"` and
 whatever follows, because every frame's JSON wrapper is identical by
 construction, and the count would repeat `progress.chars` under a new
 name rather than saying how much the model actually produced.

 @param label - model or endpoint

 @param progress - what the stream did

 @param unreadableFrames - payload lines the scanner could not read

 @param outcome - whether the stream finished, was cut, or was ended by this
 system's own degeneration guard

 @param openingText - generated text delivered, combined across channels in
 arrival order, used only for its opening

 @param generatedChars - decoded characters produced on each channel, from
 the same detectors the degeneration guard already keeps running totals in

 @param servedBy - upstream the gateway named for this stream, so a cut or a
 slow finish can be pinned on one endpoint of a many-endpoint provider;
 empty, and left off the line, where the wire names none

 @returns Line that was logged

 @example
 ```ts
 reportStreamProgress({
   label,
   progress,
   unreadableFrames: 0,
   outcome: 'cut',
   openingText: watch.openingText(),
   generatedChars: { content: 40, reasoning: 0, },
 },);
 ```
 */
export function reportStreamProgress(
  {
    label,
    progress,
    unreadableFrames,
    outcome,
    openingText,
    generatedChars,
    servedBy = '',
  }: {
    readonly label: string;
    readonly progress: StreamProgress;
    readonly unreadableFrames: number;
    readonly outcome: StreamOutcome;
    readonly openingText: string;
    readonly generatedChars: {
      readonly content: number;
      readonly reasoning: number;
    };
    readonly servedBy?: string;
  },
): string {
  /**
   Opening of what the model generated, with newlines flattened so one
   stream is one line.
   */
  const opening = wholeOpening({
    text: openingText,
    units: OPENING_CHARS,
  },)
    .split('\n',)
    .join(' ',);

  /**
   Opening excerpt, shown on anything but a clean finish, where what arrived
   is the diagnosis. A degenerate ending gets one for the same reason a cut
   does: seeing what the model was saying when it started repeating is as
   diagnostic as seeing what it was saying when the connection dropped.
   */
  const excerpt = (outcome === 'completed') ? '' : `, opening ${JSON.stringify(opening,)}`;

  /**
   Upstream attribution, quoted the way the excerpt is because the name is
   the gateway's text and this line's grammar is comma-separated; absent
   where the wire named none, so single-upstream providers' lines keep
   their shape.
   */
  const attribution = (servedBy === '') ? '' : `, served by ${JSON.stringify(servedBy,)}`;

  /**
   Sample line, assembled before the call so the logger chain stays one step
   per line.
   */
  const sample = `stream ${label}: ${outcome}, elapsed ${String(progress.elapsedMs,)}ms, `
    + `firstByte ${String(progress.firstByteMs,)}ms, `
    + `firstContent ${String(progress.firstContentMs,)}ms, `
    + `maxGap ${String(progress.maxGapMs,)}ms, ${String(progress.chars,)} raw ${
      wordForCount({
        count: progress.chars,
        one: 'char',
        many: 'chars',
      },)
    }, `
    + `${String(unreadableFrames,)} unreadable ${
      wordForCount({
        count: unreadableFrames,
        one: 'frame',
        many: 'frames',
      },)
    }, `
    + `${String(generatedChars.content,)} content ${
      wordForCount({
        count: generatedChars.content,
        one: 'char',
        many: 'chars',
      },)
    }, `
    + `${String(generatedChars.reasoning,)} reasoning ${
      wordForCount({
        count: generatedChars.reasoning,
        one: 'char',
        many: 'chars',
      },)
    }${attribution}${excerpt}`;

  /**
   Logger tagged with this report.
   */
  const rl = tagged({
    tag: reportStreamProgress.name,
    l,
  },);
  rl.info(sample,);
  return sample;
}

//endregion Stream cut
