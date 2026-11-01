import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { StreamCutShortError, } from './stream-cut.ts';
import { StreamDegenerateError, } from './stream-runaway-watch.ts';
import { StreamOverrunError, } from './stream-overrun.ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Abandon kind
// WHY A VOICE WAS LOST, in a form a run log can be grouped by.
//
// Every abandonment used to read the same: `abandoned 180000ms after quorum`,
// followed by whatever the platform called its abort. That is one line for at
// least three situations which want OPPOSITE remedies, and no run log could
// separate them:
//
//   a call that never got a first byte is queueing or admission, and wants the
//   straggler dropped or replaced;
//
//   a call cut off part way through was working, and wants a longer window or a
//   smaller slate;
//
//   a call we ended ourselves had stopped saying anything new, and wants
//   neither, because waiting longer buys more of the same.
//
// Keeping what an aborted stream delivered made the evidence available.
// This reads it. Without this the information exists on the error and nothing
// looks at it, which is the failure the section aligner once had:
// telemetry written and never read is worse than none, because it looks like the question is covered.

/**
 Threshold below which a first byte never arrived at all.

 The guard reports a negative time rather than zero when nothing came, so a
 call cut during its very first millisecond still reads as having started.
 */
const NO_FIRST_BYTE_MS = 0;

/**
 Decimal places the distinct ratio is reported to, enough to tell the
 degenerate range near 0.001 from the threshold at 0.1.
 */
const RATIO_DIGITS = 4;

/**
 Names why a voice was lost, in a phrase a log can be grouped by.

 FALLS BACK TO THE ERROR'S OWN TEXT rather than to a catch-all name, because
 an unrecognised failure that all read `other` would be invisible in exactly
 the way this exists to prevent.

 @param error - whatever the call threw

 @returns Short cause, safe to put in a log line

 @example
 ```ts
 const cause = describeAbandon({ error, },);
 ```
 */
export function describeAbandon({ error, }: { readonly error: unknown; },): string {
  if (error instanceof StreamOverrunError)
    return `overran ${error.channel} at ${String(error.charsSeen,)} ${
      wordForCount({
        count: error.charsSeen,
        one: 'char',
        many: 'chars',
      },)
    } `
      + `against a bound of ${String(error.cap,)}`;

  if (error instanceof StreamDegenerateError) {
    /**
     Share of recent windows that were distinct when the call was ended.
     */
    const { distinctRatio, } = error;

    /**
     That share, rendered.
     */
    const ratio = distinctRatio.toFixed(RATIO_DIGITS,);
    return `degenerate in ${error.channel} at ${ratio} distinct over `
      + `${String(error.charsSeen,)} ${
        wordForCount({
          count: error.charsSeen,
          one: 'char',
          many: 'chars',
        },)
      }`;
  }

  // Any class this does not recognise is named rather than rendered: a
  // provider error's message may carry a body excerpt, and this line lands in a
  // warn line beside the voice it lost.
  if (!(error instanceof StreamCutShortError))
    return refusalText({ error, },);

  /**
   When the first byte arrived, negative when none ever did.
   */
  const { firstByteMs, } = error.progress;
  if (firstByteMs < NO_FIRST_BYTE_MS)
    return 'no-first-byte, nothing was ever delivered';

  /**
   The stream as it arrived, framing included, which is not what the model wrote: the cuts of Qwen on the run of TianqiChen66625
   delivered 1,308,758 characters on the wire for 49,293 of reasoning.
   */
  const { partialText, } = error;
  /**
   How many characters the stream delivered, framing included.
   */
  const wire = partialText.length;

  /**
   The wire length, as a separate named field of the line.
   */
  const onTheWire = `${String(wire,)} ${
    wordForCount({
      count: wire,
      one: 'char',
      many: 'chars',
    },)
  } on the wire`;

  /**
   The characters of the answer and of the reasoning the stream counted, as a list of zero or one.
   */
  const [generated,] = (error.generatedChars === undefined) ? [] : [error.generatedChars,];
  if (generated === undefined)
    return `cut-mid-reply after ${onTheWire}, first byte at ${String(firstByteMs,)}ms`;
  return `cut-mid-reply after ${String(generated.content,)} answer ${
    wordForCount({
      count: generated.content,
      one: 'char',
      many: 'chars',
    },)
  } and ${String(generated.reasoning,)} reasoning ${
    wordForCount({
      count: generated.reasoning,
      one: 'char',
      many: 'chars',
    },)
  } (${onTheWire}), first byte at ${String(firstByteMs,)}ms`;
}

//endregion Abandon kind
