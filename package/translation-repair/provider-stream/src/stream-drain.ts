import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  maskCredentials,
  maskCredentialsInCutText,
} from './credential-mask.ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { IdleGuard, } from './stream-idle-guard.ts';
import type { StreamWireFormat, } from './stream-wire-format.ts';
import {
  type RunawayVerdict,
  type RunawayWatch,
  StreamDegenerateError,
  watchRunaway,
} from './stream-runaway-watch.ts';
import {
  isSelfEndedStream,
  StreamOverrunError,
} from './stream-overrun.ts';
import {
  reportStreamProgress,
  StreamCutShortError,
  type StreamOutcome,
} from './stream-cut.ts';

//region Stream drain
// Reads a response body chunk by chunk instead of in one `response.text()`
// call, which is what makes silence observable at all: a whole-body read cannot
// tell a stream that is still emitting from one that died, so the only signal
// left was total elapsed time. Parsers above the transport seam are unaffected,
// because the chunks are concatenated and handed back whole.

/**
 Logger root for the stream drain.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Stops pulling from a stream that will not stop on its own.

 ITS OWN FUNCTION, and it swallows nothing: a cancel that fails is logged
 rather than allowed to replace the reason the stream is being abandoned,
 which is the more useful of the two errors.

 THE ADDRESS IS MASKED LIKE ANY OTHER PROVIDER TEXT. After a redirect,
 `response.url` is the address the redirect named, its path and query as the
 provider wrote them (measured with node's `fetch` against a local server:
 a `302` to `/landing?key=<key>` leaves that address whole), so it can repeat
 the credential the request sent, and the line is masked as the opening line
 is.

 @param reader - reader to release

 @param url - stream being abandoned, for the log line

 @param credentials - secrets the request carried, which the logged address
 must not repeat

 @mutates reader - cancels it, so the body cannot be read further

 @example
 ```ts
 await stopReading({ reader, url: response.url, credentials, },);
 ```
 */
async function stopReading(
  {
    reader,
    url,
    credentials,
  }: {
    readonly reader: ForeignBorrowed<ReadableStreamDefaultReader<Uint8Array>>;
    readonly url: string;
    readonly credentials: readonly string[];
  },
): Promise<void> {
  try {
    await reader.cancel();
  }
  catch (error) {
    /**
     Logger tagged with this drain.
     */
    const cl = tagged({
      tag: drainBody.name,
      l,
    },);
    cl.warn(`could not cancel ${
      maskCredentials({
        text: url,
        credentials,
      },)
    }: ${refusalText({ error, },)}`,);
  }
}

/**
 Turns a runaway verdict into the error that ends the call.

 ONE PLACE THE MAPPING LIVES, so the drain's read loop stays a single
 decision: stop reading, then report. A verdict carries the evidence its own
 kind gathered, and each error class takes exactly that evidence, which is
 why neither can be built from the other.

 @param label - what was being called, for the message

 @param verdict - runaway verdict to report

 @param rawChars - raw wire characters the stream delivered, which the
 abandoned-spend reckoning prices (ledger P7)

 @returns Error naming why this call was ended

 @example
 ```ts
 throw runawayError({ label, verdict, rawChars, },);
 ```
 */
function runawayError(
  {
    label,
    verdict,
    rawChars,
  }: {
    readonly label: string;
    readonly verdict: Extract<RunawayVerdict, { readonly kind: 'overrun' | 'runaway'; }>;
    readonly rawChars: number;
  },
): StreamDegenerateError | StreamOverrunError {
  if (verdict.kind === 'overrun')
    return new StreamOverrunError({
      label,
      channel: verdict.channel,
      charsSeen: verdict.charsSeen,
      cap: verdict.cap,
      rawChars,
    },);

  return new StreamDegenerateError({
    label,
    channel: verdict.channel,
    distinctRatio: verdict.distinctRatio,
    charsSeen: verdict.charsSeen,
    rawChars,
  },);
}

/**
 Names how a stream ended, for the progress line.

 SEPARATE FROM {@link isSelfEndedStream}, which answers whether to retry.
 This answers what to call it, and the two questions have different shapes:
 a reader counting `cut` lines to measure stalls needs the chosen endings
 kept apart from stalls AND from each other, while the retry ladder only
 needs to know that we chose it.

 @param error - whatever the drain caught

 @returns Outcome name for this ending

 @example
 ```ts
 const outcome = endedOutcome({ error, },);
 ```
 */
function endedOutcome({ error, }: { readonly error: unknown; },): StreamOutcome {
  if (error instanceof StreamOverrunError)
    return 'overrun';

  if (error instanceof StreamDegenerateError)
    return 'degenerate';

  return 'cut';
}

/**
 Logs how a stream ended, with the sent credentials taken out of the two
 pieces of provider text the line carries.

 MASKED BEFORE THE EXCERPT IS CUT. The line shows the opening of what the
 model generated, and the credential a request carried may be in it, whole,
 or cut by the end of a stream that was torn down inside it. The generated
 text the watch kept is masked whole, a cut-off credential included, and the
 line's excerpt is cut from that, so no head of a credential reaches the
 cut. Masking the excerpt after the cut showed the head of a credential the
 cut fell inside. The text is the watch's own opening, which it caps, so the
 mask reads a short text except where one frame carried far more.

 @param label - model this call went to

 @param guard - silence guard, whose totals the line reports

 @param watch - runaway watch whose opening text, counts and upstream the line reports

 @param outcome - how the stream ended

 @param credentials - secrets the request carried, none where the caller has none to name

 @example
 ```ts
 reportMaskedProgress({ label, guard, watch, outcome: 'cut', credentials, },);
 ```
 */
function reportMaskedProgress(
  {
    label,
    guard,
    watch,
    outcome,
    credentials,
  }: {
    readonly label: string;
    readonly guard: ForeignBorrowed<IdleGuard>;
    readonly watch: RunawayWatch;
    readonly outcome: StreamOutcome;
    readonly credentials: readonly string[];
  },
): void {
  reportStreamProgress({
    label,
    progress: guard.progress(),
    unreadableFrames: watch.unreadableFrames(),
    outcome,
    openingText: maskCredentialsInCutText({
      text: watch.openingText(),
      credentials,
    },),
    generatedChars: watch.generatedChars(),
    servedBy: maskCredentials({
      text: watch.servedBy(),
      credentials,
    },),
  },);
}

/**
 Drains a response body chunk by chunk, telling the guard about each arrival
 so silence is measured rather than inferred from total elapsed time. The
 decoded text is concatenated and handed back whole, so every parser above
 the transport seam sees exactly what it saw when the body was read with
 `response.text()`.

 @param response - response whose body is drained

 @param guard - silence guard notified per chunk

 @param callerSignal - caller's own signal, to tell steering from a stall

 @param label - model this call went to, so a latency figure can be read per
 model rather than per endpoint. Reasoning from abandon counts instead is what
 produced the retracted conclusion that one vendor's models were slow

 @mutates guard - each chunk is counted on the wire via guard.notify,
 each growth of generated text resets the guard's silence window via
 guard.notifyGenerated, and guard.progress reads the totals it accumulated

 @mutates response - body.getReader locks the body stream to this reader and
 every read consumes from it, so the response's body is drained and cannot be
 read again by anyone else

 @returns Whole decoded body

 @throws `StreamCutShortError` when the stream was cut off, carrying whatever
 it had already delivered and wrapping the original failure as its cause

 @throws `StreamDegenerateError` when the model stopped saying anything new,
 which no silence window can detect because such a stream is never silent

 @throws `StreamOverrunError` when the answer channel passed its content
 bound, which catches a runaway whose period is too long to repeat inside
 what the repetition detectors can hold

 @param maxAnswerChars - bound for this one call, when the caller knows its
 own input size; the module default polices every call that names none

 @param credentials - secrets the request carried, which the progress line's
 excerpt of generated text and its upstream name, and the address a failed
 cancel names, must not repeat; an empty list where the request carried none,
 which every caller states so that none forgets the mask

 @example
 ```ts
 const bodyText = await drainBody({ response, guard, callerSignal, label, credentials, },);
 ```
 */
export async function drainBody(
  {
    response,
    guard,
    callerSignal,
    label,
    maxAnswerChars,
    wireFormat,
    credentials,
  }: {
    readonly response: ForeignBorrowed<Response>;
    readonly guard: ForeignBorrowed<IdleGuard>;
    readonly callerSignal: AbortSignal;
    readonly label: string;
    readonly maxAnswerChars?: number;
    readonly wireFormat?: StreamWireFormat;
    readonly credentials: readonly string[];
  },
): Promise<string> {
  /**
   Body stream; absent on a reply the platform gave no body at all, which
   still has to read as the empty string rather than fail.
   */
  const { body, } = response;
  if (body === null)
    return await response.text();

  /**
   Reader pulling one chunk at a time.
   */
  const reader = body.getReader();

  /**
   Incremental decoder, so a multi-byte character split across chunks is
   still decoded correctly.
   */
  const decoder = new TextDecoder();

  /**
   Decoded chunks, joined once at the end: repeated string concatenation
   would rebuild the whole accumulated body on every chunk.
   */
  const parts: string[] = [];

  /**
   Watches for a model that has stopped saying anything new, on either the
   answer channel or the thinking one.

   SEPARATE FROM THE IDLE GUARD because they detect opposite things. The idle
   guard asks whether bytes are arriving; a degenerating model answers yes
   forever. Neither can stand in for the other.

   GIVEN THIS CALL'S OWN BOUND WHEN THE CALLER KNOWS ONE, and policed at the
   module default otherwise. The default is absolute, and an absolute number
   cannot separate a long legitimate passage from a short line that ran away:
   the emission that forced this through was 10381 characters against a
   56-character source, well under the default and 185 times its input. A
   caller that knows how large its input was can say so, and
   `produced-volume-bound.ts` records what that bound rests on.
   */
  const watch = watchRunaway({
    // Conditional spreads keep each knob absent rather than explicitly
    // undefined, so the watch applies its own default for whichever the
    // caller left unsaid.
    ...((maxAnswerChars === undefined) ? {} : { contentCap: maxAnswerChars, }),
    ...((wireFormat === undefined) ? {} : { wireFormat, }),
  },);

  /**
   Loop cursor, a named record so the body-root binding stays immutable, with
   the generated characters the guard was last told of.
   */
  const cursor = {
    done: false,
    generated: 0,
  };

  try {
    while (!cursor.done) {
      /* oxlint-disable no-await-in-loop -- chunks arrive in order; each read depends on the previous one completing */
      /**
       Next chunk, or the end-of-stream marker.
       */
      const chunk = await reader.read();
      /* oxlint-enable no-await-in-loop */
      cursor.done = chunk.done;
      if (chunk.value === undefined)
        continue;

      /**
       This chunk decoded, held so its length feeds the guard.
       */
      const text = decoder.decode(
        chunk.value,
        { stream: true, },
      );
      guard.notify(text.length,);
      parts.push(text,);

      /**
       Whether this call has stopped producing anything new.
       */
      const runaway = watch.notifyChunk({ chunk: text, },);

      /**
       Generated text so far, content and reasoning, which the scanner reads
       out of the frames and a comment line, a blank keep-alive line or an
       event with an empty delta does not grow.
       */
      const generated = watch.generatedChars();
      // ONLY GENERATED TEXT RESETS THE GUARD'S WINDOW (ledger B804): the chunk's bytes
      // were counted for the wire's own bookkeeping, and a stream that
      // sends bytes and no text is as silent as one that sends nothing.
      if ((generated.content + generated.reasoning) > cursor.generated) {
        cursor.generated = generated.content + generated.reasoning;
        guard.notifyGenerated(generated,);
      }
      if (runaway.kind !== 'continuing') {
        // Released before reporting, so the socket does not stay open feeding a
        // model that will not stop. The provider ends neither, and no token cap
        // bounds it, so this is the only place the call can end.
        // oxlint-disable-next-line no-await-in-loop -- the loop ends on this branch
        await stopReading({
          reader,
          url: response.url,
          credentials,
        },);
        // The MODEL'S label, not `response.url`: every chat-completions call
        // shares one endpoint across the whole roster, so attributing a
        // runaway to the endpoint makes a per-model latency figure unreadable.
        // `stopReading` still logs `response.url`, masked, because releasing
        // the right socket is a URL question and naming the runaway is a
        // model question.
        throw runawayError({
          label,
          verdict: runaway,
          rawChars: guard.progress()
            .chars,
        },);
      }
    }
  }
  catch (error) {
    /**
     Guard's own signal, whose abort means silence rather than steering.
     */
    const guardSignal = guard.signal;

    /**
     What the stream had already delivered, which used to be discarded here.
     */
    const partialText = parts.join('',);

    /**
     Whether this catch is a termination THIS SYSTEM CHOSE rather than a
     stall or steering, decided once so the logged outcome and the rethrow
     agree with each other by construction rather than by staying in
     sync across two separate checks.
     */
    const isSelfEnded = isSelfEndedStream({ error, },);

    // Reported BEFORE the throw, and on this path as well as the other, because
    // a figure computed only over streams that finished describes only streams
    // that finished. A degenerate ending is reported as its own outcome rather
    // than as `cut`, so a stall figure read off this line counts stalls and not
    // every deliberate termination alongside them.
    reportMaskedProgress({
      label,
      guard,
      watch,
      outcome: endedOutcome({ error, },),
      credentials,
    },);

    // OUR OWN DELIBERATE TERMINATION PASSES THROUGH UNCHANGED. Both guard
    // errors are thrown from inside this try, and each already carries the
    // channel and the cost, a repetition ratio on one and the bound it passed
    // on the other. Wrapping either would bury a finished diagnosis inside a
    // description of a cut, and every reader would have to unwrap it to learn
    // what the drain already knew.
    if (isSelfEnded)
      throw error;

    // A stall aborts the guard's own controller and never the caller's, so this
    // ordering keeps steering identifiable as steering and silence as silence.
    // The chosen failure becomes the cause rather than being replaced.
    throw new StreamCutShortError({
      label,
      partialText,
      progress: guard.progress(),
      // The characters of the answer and of the reasoning the watch counted, which the abandonment line reports in place of
      // the wire length (the lead's ruling of 2026-10-15).
      generatedChars: watch.generatedChars(),
      cause: (guardSignal.aborted && (!callerSignal.aborted)) ? guardSignal.reason : error,
    },);
  }

  // Flush any bytes the incremental decoder was still holding.
  parts.push(decoder.decode(),);

  /**
   Whole decoded body.
   */
  const bodyText = parts.join('',);

  reportMaskedProgress({
    label,
    guard,
    watch,
    outcome: 'completed',
    credentials,
  },);

  return bodyText;
}

//endregion Stream drain
