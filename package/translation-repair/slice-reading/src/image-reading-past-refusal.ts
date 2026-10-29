import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { ModelCaller, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type ImageReading,
  IMAGE_READING_PERSPECTIVES,
  type ImageReadingPerspective,
  readImageAsset,
} from './image-reading-stage.ts';
import type {
  ProviderName,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Reading past a refusal
// ASKING THE SAME MODEL AGAIN WHEN IT DECLINES TO READ A PICTURE, because a
// refusal is a property of the ROLL and not of the picture.
//
// MEASURED, on `wangzihao980/Word1.webp`, a picture that plainly carries text:
// the deterministic reader gets 205 characters out of it and the other reader
// gets 389 and 393 on consecutive asks. Asked six times with identical input,
// `hf:moonshotai/Kimi-K3` refused four times and read it twice, at 377 and 403
// characters. Nothing about the picture changed between those asks.
//
// WHY IT MATTERED MORE THAN IT LOOKED. Corroboration needed both readers, and
// the vision sub-roster was exactly two when this was measured. One reader
// refusing two asks in three therefore cost two thirds of the readings, and the
// picture ended `one-reader-only` with a perfectly good transcription thrown
// away. That is what happened to `Word1.webp` on the run that found this.
//
// AND THEN MEASURED AGAIN, OVER THE WHOLE CORPUS, WHICH CORRECTED THAT.
// `Word1.webp` is an outlier, not a sample. Across all 191 assets, 119 reader
// and picture pairs reached a model, 109 read on the FIRST ask, and 1 more read
// only after a re-ask. Seven pairs ever returned `reads-as-refusal` and six of
// them refused through all four asks, so the first-ask refusal rate is 7 in 119,
// not 2 in 3. Most refusals are about the PICTURE and only some are about the
// roll, which is the reverse of what one picture suggested.
//
// COUNT THE CLAUSE, NOT THE SENTENCE. Three more pairs produced no reading
// without ever being re-asked: two exited on `too-short` and one on an empty
// reply. An earlier tally keyed on the log sentence, which reads "the reading
// was refused: <clause>" for every screen, and so reported those three as
// stubborn refusals and this loop as more exhausted than it is.
//
// WHAT THE LIMIT ACTUALLY BUYS, therefore, is small and worth having: 20 extra
// calls over a whole corpus pass, against 119 first asks, to recover the roll
// case whenever it occurs. An earlier version of this comment projected "four
// asks retain four readings in five" from six asks on one picture. That
// projection is refuted here and must not be quoted.
//
// THE LIMIT STAYS AT FOUR rather than dropping to three, even though the single
// recovery arrived on the third ask. Fitting the bound to one observation is the
// error that produced the wrong projection in the first place, and the cost of
// the fourth ask is a handful of calls per corpus.
//
// AN ABSENCE REPORT IS NOT A REFUSAL, since 2026-09-04. The Uekawakuyuurei
// run of that day asked three readers four times each about a painting of
// ships, and every answer said the painting carries no text: twelve calls to
// be told the same true thing, then an entry stopped for want of a reading. A
// reply that reports absence (`reports-no-text`) is about the picture and is
// returned at once; only a reply that declines to read (`reads-as-refusal`)
// is asked again.
//
// SCOPED TO REFUSAL AND ONE KIND OF EMPTY REPLY, deliberately. A model that
// does not read images, a picture too large to send, an empty reply with no
// cut reason, and a reading that fails the screen for some other clause are
// all properties of the input or the roster: asking again spends a call to be
// told the same thing. `reads-as-refusal` has been measured to vary between
// identical asks, so it is re-asked.
//
// AN EMPTY REPLY THE LENGTH LIMIT CUT IS ASKED ONCE MORE, since 2026-10-21.
// XingZ6016 stopped for want of a second reading because `glm-5.3-flash` on
// NeuralWatt reasoned until its completion cap (18316 tokens, `finishReason`
// `length`, 0 content characters) on a dense screenshot that Hyper read for
// 2125 characters in 77 s the day before. The cut is a property of one roll on
// one serving stack, so the second ask goes to another provider that serves
// the model when one is wet and to the same provider when none is: the router
// refuses a hinted ask with nowhere else to go (`NoProviderForModelError`),
// and the ask is then made without the hint. ONCE, not up to the refusal
// limit: a second cut is a model that cannot finish this picture, and a third
// ask would spend another cap's worth of tokens to be told so. The second ask
// takes the next reading responsibility, because the uniqueness wrapper
// hashes the prompt and would otherwise replay the empty reply it stored.

/**
 Clause the screen reports for a reading that declined to transcribe.

 NAMED HERE RATHER THAN IMPORTED because it is the one clause this file acts
 on, and a rename that silently stopped the re-asking would otherwise leave no
 trace. A mismatch shows up as readings that stop being retried.
 */
const REFUSAL_CLAUSE = 'reads-as-refusal';

/**
 How many times one model may be asked about one picture.

 FOUR, kept on the corpus-wide measurement recorded in the module
 note: over one pass the limit bought 20 extra calls against 119 first asks
 and recovered the one roll case, on the third ask. The bound is not fitted
 to that single observation, which is why it is not three; and the projection
 an earlier version quoted here ("four readings in five") is the refuted one
 the module note names, never to be reinstated as a justification.
 */
export const REFUSAL_ASK_LIMIT = 4;

/**
 Whether a reading is a refusal that asking again might get past.

 @param reading - outcome of one ask

 @returns True when the model declined to transcribe rather than failing for a
 reason another ask cannot change

 @example
 ```ts
 if (isRefusal({ reading, },)) { ask again }
 ```
 */
function isRefusal({ reading, }: { readonly reading: ImageReading; },): boolean {
  if (reading.kind !== 'unavailable')
    return false;
  return reading.reason === REFUSAL_CLAUSE;
}

/**
 Whether a reading is an empty reply the completion limit ended.

 @param reading - outcome of one ask

 @returns True when the reply carried no content because it was cut

 @example
 ```ts
 if (isCutByLength({ reading, },)) { mark it }
 ```
 */
function isCutByLength({ reading, }: { readonly reading: ImageReading; },): boolean {
  if (reading.kind !== 'unavailable')
    return false;
  return reading.cutByLength === true;
}

/**
 Whether a reading is an empty reply the length limit cut that has not yet had
 its one re-ask.

 @param reading - outcome of the latest ask

 @param cutReasked - whether a length-cut reply was already asked again

 @returns True when one more ask is owed

 @example
 ```ts
 if (canReaskCut({ reading, cutReasked: false, },)) { ask again }
 ```
 */
function canReaskCut(
  {
    reading,
    cutReasked,
  }: {
    readonly reading: ImageReading;
    readonly cutReasked: boolean;
  },
): boolean {
  if (cutReasked)
    return false;
  return isCutByLength({ reading, },);
}

/**
 Whether another ask is owed: the reading is a refusal, or a length-cut empty
 reply that has not had its one re-ask.

 @param reading - outcome of the latest ask

 @param cutReasked - whether a length-cut reply was already asked again

 @returns True when the loop asks again

 @example
 ```ts
 const owed = isOwedAsk({ reading, cutReasked: false, },);
 ```
 */
function isOwedAsk(
  {
    reading,
    cutReasked,
  }: {
    readonly reading: ImageReading;
    readonly cutReasked: boolean;
  },
): boolean {
  return isRefusal({ reading, },) || canReaskCut({
    reading,
    cutReasked,
  },);
}

/**
 Asks again after a reply the completion limit cut: on another provider that
 serves the model when one is wet, on the same provider otherwise.

 @param client - provider client

 @param modelId - reader asked again

 @param bytes - picture as it sits in the corpus

 @param assetName - file name, for the log line and the content part

 @param perspective - responsibility of this ask, which changes the prompt so
 the uniqueness wrapper buys a new reply

 @param cutOn - provider whose reply was cut, when the client said; absent
 for a client that routes nowhere, which then gets no hint

 @param signal - abort honoured inside the ask

 @param perCallTimeoutMs - ceiling on this exchange

 @param l - logger

 @returns Reading of the re-ask

 @example
 ```ts
 const reading = await reaskCut({ client, modelId, bytes, assetName, perspective, cutOn: 'neuralwatt', signal, perCallTimeoutMs, l, },);
 ```
 */
async function reaskCut(
  {
    client,
    modelId,
    bytes,
    assetName,
    perspective,
    cutOn,
    signal,
    perCallTimeoutMs,
    l,
  }: {
    readonly client: ModelCaller;
    readonly modelId: RosterModelId;
    readonly bytes: Uint8Array;
    readonly assetName: string;
    readonly perspective: ImageReadingPerspective;
    readonly cutOn?: ProviderName;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  },
): Promise<ImageReading> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: reaskCut.name,
    l,
  },);
  rl.info(
    `${modelId} was cut by the completion limit on ${assetName} (finish reason length${
      (cutOn === undefined) ? '' : ` on ${cutOn}`
    }), asking again${(cutOn === undefined) ? '' : ` on another provider that serves it`}`,
  );
  if (cutOn !== undefined) {
    try {
      return await readImageAsset({
        client,
        modelId,
        bytes,
        assetName,
        perspective,
        otherThan: cutOn,
        signal,
        perCallTimeoutMs,
        l,
      },);
    }
    catch (error) {
      // Only the router's "nowhere else" is answered by the same provider; a
      // stop, a refusal on the other provider's budget that threw something
      // else, or a transport failure keeps its own handling.
      if (!(error instanceof NoProviderForModelError))
        throw error;
      rl.info(`${modelId}: no other provider serves ${assetName}, asking ${cutOn} again`,);
    }
  }
  return await readImageAsset({
    client,
    modelId,
    bytes,
    assetName,
    perspective,
    signal,
    perCallTimeoutMs,
    l,
  },);
}

/**
 Asks one model to read one picture, past a refusal, up to a bounded limit.

 @param client - provider client

 @param modelId - reader asked, the same one every time, since the refusal is
 this model's roll rather than a fact about the picture

 @param bytes - picture as it sits in the corpus

 @param assetName - file name, for the log line and the content part

 @param signal - abort honoured between asks as well as inside them, so a
 stopped run does not keep re-asking

 @param perCallTimeoutMs - ceiling on one exchange, applied per ask rather than
 across all of them

 @param l - logger

 @returns Reading, or the last refusal when every ask was declined

 @throws {@link DOMException} when `signal` aborts between asks

 @example
 ```ts
 const reading = await readPastRefusal({ client, modelId, bytes, assetName, signal, perCallTimeoutMs, l, },);
 ```
 */
export async function readPastRefusal(
  {
    client,
    modelId,
    bytes,
    assetName,
    signal,
    perCallTimeoutMs,
    l,
  }: {
    readonly client: ModelCaller;
    readonly modelId: RosterModelId;
    readonly bytes: Uint8Array;
    readonly assetName: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  },
): Promise<ImageReading> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readPastRefusal.name,
    l,
  },);

  /**
   Outcome of the latest ask, replaced by each re-ask.
   */
  let reading = await readImageAsset({
    client,
    modelId,
    bytes,
    assetName,
    perspective: IMAGE_READING_PERSPECTIVES[0],
    signal,
    perCallTimeoutMs,
    l,
  },);

  /**
   Whether the one re-ask a length-cut empty reply gets has been spent.
   */
  let cutReasked = false;

  for (
    let ask = 2;
    (ask <= REFUSAL_ASK_LIMIT) && isOwedAsk({
      reading,
      cutReasked,
    },);
    ask += 1
  ) {
    // Between asks as well as inside them. A stopped run that only checked
    // inside the exchange would spend the whole limit on every remaining
    // picture during teardown.
    signal.throwIfAborted();

    // SEQUENTIAL ON PURPOSE. Each later perspective exists only when prior
    // response was refusal-shaped. Concurrent asks would ignore dependency
    // and spend every perspective even after one succeeds.
    /**
     Distinct visual responsibility for this retry.
     */
    const perspective = nonNullishOrThrow(
      IMAGE_READING_PERSPECTIVES[ask - 1],
    );
    if ((reading.kind === 'unavailable') && (reading.cutByLength === true)) {
      cutReasked = true;
      /* oxlint-disable-next-line no-await-in-loop -- re-asks are sequential by design: the second ask exists only when the first was cut */
      reading = await reaskCut({
        client,
        modelId,
        bytes,
        assetName,
        perspective,
        ...((reading.servedBy === undefined) ? {} : { cutOn: reading.servedBy, }),
        signal,
        perCallTimeoutMs,
        l: rl,
      },);
      continue;
    }
    rl.info(
      `${modelId} declined ${assetName}, asking again (${String(ask,)} of ${String(REFUSAL_ASK_LIMIT,)})`,
    );
    /* oxlint-disable-next-line no-await-in-loop -- re-asks are sequential by design: each later perspective exists only when the prior answer was refusal-shaped */
    reading = await readImageAsset({
      client,
      modelId,
      bytes,
      assetName,
      perspective,
      signal,
      perCallTimeoutMs,
      l,
    },);
  }

  return reading;
}

//endregion Reading past a refusal
