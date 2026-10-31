import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  alternateModelsFor,
  describeLeftOut,
  type LeftOutModel,
  type ModelCaller,
  readsImages,
  type ReaskClaims,
  type ReaskMode,
  type ReaskSeat,
  takenFor,
  takeFor,
} from '@monochromatic-dev/translation-repair-provider/ts';
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
// 2125 characters in 77 s the day before. ONCE, not up to the refusal limit: a
// second cut is a model that cannot finish this picture, and a third ask would
// spend another cap's worth of tokens to be told so.
//
// THE SECOND ASK GOES TO ANOTHER MODEL, since 2026-10-30 (ledger B1074, the
// owner's answer "Everywhere (Recommended)"). A reply cut with no content is a
// reasoning runaway, which is the model and not the serving stack: asking the
// same weights on another route repeated it six times in seven in the runs of
// 2026-10-22 to 2026-10-24 (`reask-alternate-models.ts` in the provider
// package). The models asked are the ones that read pictures, in the order of
// the cap census's runaway shares, that the reader role does not hold out
// (`reader-unmeasured`, `owner-culled`: a picture reader joins only after a
// keyed measurement, `design-commitments-providers-and-seats.md`, the rule of
// 2026-09-08, so a model whose card holds it out of the role cannot stand in),
// and that are not readers of the same picture (the bench), since their reading
// recorded as this reader's would count one model twice. The picture is asked
// as the first ask asked it ("Plain question (Recommended)").
//
// ONLY WHERE NO MODEL CAN TAKE IT is the reader itself asked, as before: on
// another provider that serves the model when one is wet and on the same
// provider when none is (the router refuses a hinted ask with nowhere else to
// go, `NoProviderForModelError`, and the ask is then made without the hint). That
// ask takes the next reading responsibility, because the uniqueness wrapper
// hashes the prompt and would otherwise replay the empty reply it stored.
//
// A CALLER THAT NAMES NO BENCH ASKS THE READER ITSELF AND NO OTHER MODEL (ledger B1100). The measurement of one reader
// (`reader-measure-run.ts` in the bench package) calls this with no bench and no record, and a rival's reading scored as
// the candidate's would measure the rival. Only the readers of a picture (`readImagePair`) name a bench.
//
// A REFUSAL AFTER A CUT IS THE REFUSAL OF THE MODEL THAT GAVE IT (ledger B1100). The reading the cut re-ask brought
// back may itself decline; the loop then asks that model again, as it asks any model that declines, and logs it by name.

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
 What a re-ask of a cut reading brought back, and the model that gave it.
 */
type CutReask = {
  /**
   Reading the re-ask produced.
   */
  readonly reading: ImageReading;

  /**
   Model that gave it, which a refusal in it is then the refusal of.
   */
  readonly askedModelId: RosterModelId;
};

/**
 Asks again after a reply the completion limit cut, which is a reasoning runaway (ledger B1074): of another model that
 reads pictures, in the order of the cap census's runaway shares, with the picture asked as the first ask asked it;
 where none can take it, or the caller names no bench, of the reader itself, on another provider that serves it when
 one is wet and on the same provider otherwise.

 @param client - provider client

 @param modelId - reader whose reply was cut

 @param bytes - picture as it sits in the corpus

 @param assetName - file name, for the log line and the content part

 @param perspective - responsibility of the ask of the reader itself, which changes the prompt so the uniqueness
 wrapper buys a new reply; the asks of other models carry the first responsibility, since a new model name is a new
 prompt already

 @param cutOn - provider whose reply was cut, when the client said; absent
 for a client that routes nowhere, which then gets no hint

 @param benchModelIds - every reader of this picture, whose reading recorded as the cut reader's would count one
 model twice; absent where the caller names none, which asks the reader itself and no other model, as a measurement of
 one model does (ledger B1100)

 @param claims - record of the models asked for the readers of this picture, which they share so that no model is
 asked for two of them (ledger B1080); absent for a reader that is alone on its picture

 @param signal - abort honoured inside the ask

 @param perCallTimeoutMs - ceiling on this exchange

 @param l - logger

 @returns Reading of the re-ask and the model that gave it

 @example
 ```ts
 const { reading, } = await reaskCut({ client, modelId, bytes, assetName, perspective, cutOn: 'neuralwatt', benchModelIds, signal, perCallTimeoutMs, l, },);
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
    benchModelIds,
    claims,
    reask,
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
    readonly benchModelIds?: readonly RosterModelId[];
    readonly claims?: ReaskClaims;
    readonly reask?: ReaskMode;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  },
): Promise<CutReask> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: reaskCut.name,
    l,
  },);

  /**
   How the cut is told in every line: the picture, and the provider when the client said.
   */
  const cutText = `${modelId} was cut by the completion limit on ${assetName} (finish reason length${
    (cutOn === undefined) ? '' : ` on ${cutOn}`
  })`;
  if (reask === 'measured-model') {
    // A MEASUREMENT SCORES THE READER IT ASKS (ledger B1141), bench and record notwithstanding: a rival's reading
    // recorded as this reader's would be scored as its own.
    rl.info(`${cutText}; this is a measurement, so no other model is asked`,);
  }
  else if (benchModelIds === undefined) {
    // NO BENCH, NO OTHER MODEL: a caller that names the readers of the picture can be sure an answer recorded as this
    // reader's is no second voice of one of them; a caller that names none (the measurement of one model, which must
    // score that model's own reading) asks the reader itself (ledger B1100).
    rl.info(`${cutText}; the caller names no readers of the picture, so no other model is asked`,);
  }
  else {
    /**
     The other models this picture may be asked of, the seat the asks are claimed on and the ones left out with why.
     */
    const {
      readers,
      seat,
      leftOut,
    } = otherReadersOf({
      modelId,
      benchModelIds,
      ...((claims === undefined) ? {} : { claims, }),
    },);

    /**
     The models to ask in turn, in the words of the log.
     */
    const toAsk = (readers.length === 0) ? 'none' : readers.join(', ',);

    // EACH MODEL LEFT OUT AND WHY IS WRITTEN ONCE, BEFORE THE FIRST ASK, and also where the list is empty, so the line
    // of an ask that a model without a provider repeats never carries it again (ledger B1100).
    rl.info(`${cutText}; left out: ${describeLeftOut({ leftOut, },)}; to ask in turn: ${toAsk}`,);
    for (const other of readers) {
      // TAKEN BEFORE THE ASK, in the same step as the check (`reask-claims.ts` in the provider): another cut reader of
      // this picture may have taken this model since the list was made (ledger B1080).
      if (!takeFor({
        seat,
        modelId: other,
      },)) {
        rl.info(`${modelId}: ${other} was asked for another reader of ${assetName} since the list was made`,);
        continue;
      }
      rl.info(`${modelId}: re-asking ${other} instead, plain question`,);
      try {
        return {
          askedModelId: other,
          /* oxlint-disable-next-line no-await-in-loop -- one model at a time: the next is asked only when this one had no provider with budget */
          reading: await readImageAsset({
            client,
            modelId: other,
            bytes,
            assetName,
            perspective: IMAGE_READING_PERSPECTIVES[0],
            signal,
            perCallTimeoutMs,
            l,
          },),
        };
      }
      catch (error) {
        // Only the router's "nowhere to ask" moves on to the next model; a stop, or a transport failure, keeps its own handling.
        if (!(error instanceof NoProviderForModelError))
          throw error;
        rl.debug(`${modelId}: ${other} cannot take the re-ask of ${assetName}, no provider serves it with budget`,);
      }
    }
    rl.warn(
      `${modelId}: no other model could take the re-ask of ${assetName} (${String(readers.length,)} tried); `
        + `asking ${modelId} again${(cutOn === undefined) ? '' : ' on another provider that serves it'}`,
    );
  }
  if (cutOn !== undefined) {
    try {
      return {
        askedModelId: modelId,
        reading: await readImageAsset({
          client,
          modelId,
          bytes,
          assetName,
          perspective,
          otherThan: cutOn,
          signal,
          perCallTimeoutMs,
          l,
        },),
      };
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
  return {
    askedModelId: modelId,
    reading: await readImageAsset({
      client,
      modelId,
      bytes,
      assetName,
      perspective,
      signal,
      perCallTimeoutMs,
      l,
    },),
  };
}

/**
 The models a cut picture reading may be re-asked of, in order, with the seat they are drawn for and every model left
 out with why.

 @param modelId - reader whose reply was cut

 @param benchModelIds - every reader of this picture

 @param claims - record of the models asked for the readers of this picture; absent for a reader alone on its picture

 @returns Models that read pictures in the order of their runaway shares, the seat the asks are claimed on, and the
 models left out: those the reader role or the bench keeps out, those that read no picture and those asked already

 @example
 ```ts
 const others = otherReadersOf({ modelId, benchModelIds, },);
 ```
 */
function otherReadersOf(
  {
    modelId,
    benchModelIds,
    claims,
  }: {
    readonly modelId: RosterModelId;
    readonly benchModelIds: readonly RosterModelId[];
    readonly claims?: ReaskClaims;
  },
): {
  readonly readers: readonly RosterModelId[];
  readonly seat: ReaskSeat;
  readonly leftOut: readonly LeftOutModel[];
} {
  /**
   The seat of this reader: the reader role, the readers of the picture and the record they share.
   */
  const seat: ReaskSeat = {
    role: 'reader',
    bench: benchModelIds,
    // Conditional spread keeps the record absent for a reader that is alone on its picture.
    ...((claims === undefined) ? {} : { claims, }),
  };

  /**
   Models to ask in the order of their runaway shares, and the ones the reader role, the bench or an ask for another
   reader leaves out.
   */
  const {
    alternates,
    leftOut,
  } = alternateModelsFor({
    modelId,
    role: seat.role,
    bench: seat.bench,
    askedForOtherSeats: takenFor({ seat, },),
  },);

  /**
   The models that read pictures among them, in the same order.
   */
  const readers = alternates.filter(function readsPictures(alternate,): boolean {
    return readsImages({ modelId: alternate, },);
  },);
  return {
    readers,
    seat,
    leftOut: [
      ...leftOut,
      ...alternates
        .filter(function readsNoPicture(alternate,): boolean {
          return !readers.includes(alternate,);
        },)
        .map(function withReason(alternate,): LeftOutModel {
          return {
            modelId: alternate,
            why: 'reads no picture',
          };
        },),
    ],
  };
}

/**
 Asks one model to read one picture, past a refusal, up to a bounded limit.

 @param client - provider client

 @param modelId - reader asked, the same one every time, since the refusal is
 this model's roll rather than a fact about the picture

 @param bytes - picture as it sits in the corpus

 @param assetName - file name, for the log line and the content part

 @param benchModelIds - every reader of this picture, left out of the models a reply the length limit cut is re-asked
 of, since a reading by one of them recorded as this reader's would count one model twice (ledger B1076); absent where
 the caller names none, and then a reply the length limit cut is asked of the reader itself and of no other model, which
 is what a measurement of one model needs (ledger B1100)

 @param claims - record of the models asked for the readers of this picture, which the readers share so that no model
 is asked for two of them (ledger B1080); absent for a reader that is alone on its picture

 @param reask - whose answer a re-ask of a reply the length limit cut may give: `measured-model` asks the reader itself,
 bench and record notwithstanding (ledger B1141); `readImagePair` hands in the launch's mode, and a caller that names
 neither mode nor bench is a measurement of one model and asks the reader itself as well

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
    benchModelIds,
    claims,
    reask,
    signal,
    perCallTimeoutMs,
    l,
  }: {
    readonly client: ModelCaller;
    readonly modelId: RosterModelId;
    readonly bytes: Uint8Array;
    readonly assetName: string;
    readonly benchModelIds?: readonly RosterModelId[];
    readonly claims?: ReaskClaims;
    readonly reask?: ReaskMode;
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

  /**
   Model that gave the latest reading, which a refusal in it is the refusal of: the reader, until a cut reading is
   asked of another model.
   */
  let askedModelId: RosterModelId = modelId;

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
      /* oxlint-disable no-await-in-loop -- re-asks are sequential by design: the second ask exists only when the first was cut */
      /**
       The re-ask's reading and the model that gave it.
       */
      const reasked = await reaskCut({
        client,
        modelId,
        bytes,
        assetName,
        perspective,
        ...((reading.servedBy === undefined) ? {} : { cutOn: reading.servedBy, }),
        // Conditional spreads keep the bench and the record absent where the caller names none.
        ...((benchModelIds === undefined) ? {} : { benchModelIds, }),
        ...((claims === undefined) ? {} : { claims, }),
        ...((reask === undefined) ? {} : { reask, }),
        signal,
        perCallTimeoutMs,
        l: rl,
      },);
      /* oxlint-enable no-await-in-loop */
      ({
        reading,
        askedModelId,
      } = reasked);
      continue;
    }
    // A REFUSAL IS THE ROLL OF THE MODEL THAT GAVE IT (ledger B1100): after a cut reading was asked of another model,
    // that model's refusal is asked of that model again, and the log names it.
    rl.info(
      `${askedModelId} declined ${assetName}, asking again (${String(ask,)} of ${String(REFUSAL_ASK_LIMIT,)})`,
    );
    /* oxlint-disable-next-line no-await-in-loop -- re-asks are sequential by design: each later perspective exists only when the prior answer was refusal-shaped */
    reading = await readImageAsset({
      client,
      modelId: askedModelId,
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
