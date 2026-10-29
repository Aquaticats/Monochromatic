import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  askDeterministicReader,
  type OcrReading,
} from './image-ocr.ts';
import {
  containFailedReaders,
  type LabelledReading,
} from './image-reading-contain.ts';
import { readPastRefusal, } from './image-reading-past-refusal.ts';
import { solidCharacters, } from './image-reading-sense.ts';
import {
  type ImageReading,
  isTransientReadingReason,
} from './image-reading-stage.ts';
import { clusterReadings, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Image reading pair
// READING ONE PICTURE TWICE, and letting the two readers decide whether either
// reading may be used.
//
// WHY A SECOND READER IS THE CHECK. What a reading has to earn is not
// plausibility but IDENTITY: is this a reading of the picture we sent. Nothing
// in one reading answers that, and the archive's own transcript cannot answer
// it either, for the two reasons `reading-corroboration.ts` records. Two models
// shown the same picture, neither shown the other's answer, agree about what it
// says when they can read it and do not when they cannot.
//
// THEY ARE NOT JUDGING EACH OTHER. Neither reader sees the other's reading and
// neither is asked whether it is sensible. Both are asked the same question
// about the same picture, blind, and the two answers are compared mechanically.
// A model judging a model is another failure surface; two witnesses are not.
//
// AN UNCORROBORATED READING IS UNAVAILABLE, not usable-with-a-caveat. This
// follows the rule the whole reading stage is built on: falling back costs
// nothing that exists today, since the passage is protected and left alone,
// while trusting a wrong reading licenses replacing a human's transcription
// with something derived from a misreading. A caveat travelling downstream is a
// caveat somebody has to remember.
//
// A FAILING READER COSTS ITS OWN READING AND NOTHING ELSE. Reading is the only
// stage in the pipeline whose output nothing requires, so it is also the only
// one that must never be able to fail an entry. It could, until the first real
// CLI run said so: `wangzihao980`, six pictures gathered, one reader looping on
// `picture1.webp` until the client's runaway guard ended the call, and the
// entry finishing `status=ERROR` with `processed=0`. Both lanes were lost to a
// transcription nobody needed. `Promise.allSettled` plus an explicit abort
// check is the containment; the abort still travels, because a stop is not a
// failure to absorb.
//
// WHAT THAT COSTS. It used to be 31 pictures: the ones that fit the larger
// reader's allowance and not the smaller one's, which could therefore never be
// corroborated. THAT FIGURE IS VOID, and it is worth saying why rather than
// quietly deleting it, because it was measured carefully from a premise that
// was wrong.
//
// The allowance it measured against was derived here, from half a model's
// context converted to base64 characters. A vision model does not tokenize an
// image that way. Sent as they are, both readers accept every asset in the
// corpus, up to the largest at 1344454 bytes. So the 31 were refused by this
// package and by nothing else, and the number described an estimate rather than
// the world. Every picture can now be offered to both readers, and the only
// ones that go uncorroborated are the ones the readers themselves cannot agree
// on.
//
// A PICTURE THE READERS AGREE CARRIES NOTHING IS A VERDICT, not a shortfall,
// since 2026-09-04. The deterministic reader gates on presence at 16
// characters, and a painting's canvas texture can clear that line as noise:
// `Uekawakuyuurei/IMG_1308.webp` returned 24 characters from tesseract and
// every model asked about it said, truthfully, that it carries no text. Those
// replies were screened as refusals, the pair ended `no-reader-available` and
// transient, the verdict was read again on every run, and the entry stopped at
// the completeness gate each time, on both rosters. Two readers reporting
// absence, or answering with fewer characters than a transcript (the hull
// number on `img370.webp`), now confirm what the deterministic reader could
// not: the picture is textless past its noise. Nothing travels to the sheets
// from such a verdict, exactly as from the deterministic one.

/**
 Decimal places an agreement figure is logged to.

 THREE, because the measured gap runs from 0.129 to 0.643 and two places would
 round several distinct readings to the same line.
 */
const LOGGED_OVERLAP_PLACES = 3;

/**
 Reads a picture without a model, which is what gates the model calls.

 PASSED IN RATHER THAN REACHED FOR, because the real one shells out to
 `dwebp` and `tesseract`. Injecting it keeps this function testable on a
 machine that has neither, and keeps a test that forgets to supply one a TYPE
 ERROR rather than a slow, machine-dependent pass.

 @example
 ```ts
 const readOcr: OcrReader = async () => ({ kind: 'no-text', characters: 0, });
 ```
 */
export type OcrReader = (
  input: {
    readonly bytes: Uint8Array;
    readonly assetName: string;
    readonly l: Logger;
  },
) => Promise<OcrReading>;

/**
 One model's reading of one picture.

 @example
 ```ts
 const reading: ModelReading = { modelId: 'hf:moonshotai/Kimi-K3', text: '喵。', };
 ```
 */
export type ModelReading = {
  /**
   Model that produced it.
   */
  readonly modelId: RosterModelId;

  /**
   What it transcribed.
   */
  readonly text: string;
};

/**
 What reading one picture with the whole sub-roster produced.

 @example
 ```ts
 const paired: PairedReading = { kind: 'corroborated', readings, overlap: 0.97, };
 ```
 */
export type PairedReading = {
  /**
   Both readers read the picture and agree about what it says.
   */
  readonly kind: 'corroborated';

  /**
   Every reading, labelled by its model, in roster order.

   BOTH RATHER THAN THE LONGER ONE. Corroboration establishes that the two
   describe the same picture, not that they describe the same AMOUNT of it: on
   `Mio/photo7.webp` one reader returned 178 characters and the other 590. The
   shorter vouches for what it carries and the longer carries more, so a stage
   shown both learns more than one shown either.
   */
  readonly readings: readonly ModelReading[];

  /**
   How far above the threshold they agreed, recorded so a run can be read for
   how close its corroborations ran.
   */
  readonly overlap: number;
} | {
  /**
   The picture carries no text, so there was nothing to read and no model was
   asked.

   A VERDICT RATHER THAN A FAILURE, and the distinction is not cosmetic. Two
   thirds of this corpus's pictures are photographs of people: 119 of 191
   assets return nothing from the deterministic reader. Recording those as
   `unavailable` would report a correct answer as a shortfall, and would leave
   anyone reading a run unable to tell a picture nobody could read from one
   with nothing on it.
   */
  readonly kind: 'no-text';

  /**
   How much the deterministic reader did return, so a clean nothing is
   distinguishable from a few characters below the line, and a confirmed
   textless picture from its noise above it.
   */
  readonly characters: number;

  /**
   Readers that confirmed it, when the deterministic reader found enough to
   ask them; absent when no model was asked.
   */
  readonly confirmedBy?: readonly RosterModelId[];
} | {
  /**
   The readers confirmed the picture carries no text where the deterministic
   reader could not read it at all.

   NO COUNT, BECAUSE NONE WAS TAKEN. A count of zero here would state a reading
   the deterministic reader never made, and would read exactly like a clean
   nothing, which a missing tool is not.
   */
  readonly kind: 'no-text';

  /**
   Why the deterministic reader could not read the picture.
   */
  readonly deterministicUnavailable: Extract<OcrReading, { readonly kind: 'unavailable'; }>['reason'];

  /**
   Readers that confirmed it, the only witnesses this verdict has.
   */
  readonly confirmedBy: readonly RosterModelId[];
} | {
  /**
   No reading may be used, for a reason a finding can name.
   */
  readonly kind: 'unavailable';

  /**
   Which of the ways it failed, so a reader can tell a picture nobody could
   send from one the readers disagreed about.
   */
  readonly reason:
    | 'no-reader-available'
      | 'one-reader-only'
      | 'readers-disagree';

  /**
   Whether this verdict rests on a reader that produced nothing for a reason
   that may not hold tomorrow: one that threw, answered nothing, answered too
   little, or refused.

   WHAT DECIDES WHETHER THE VERDICT IS REMEMBERED. A transient verdict
   describes the provider's evening and is read again on the next run; a
   stable one describes the picture and the roster and is resumed. Until
   this field every `unavailable` verdict was persisted, so one reader
   timing out once left a picture unread on every later run until a rebuild
   retired the cache.
   */
  readonly transient: boolean;

  /**
   Why each reader produced nothing usable, in roster order, empty for a reader
   that did produce one.
   */
  readonly perReader: readonly string[];

  /**
   Agreement measured between the readings, when there were two to compare.
   Absent when there were not, so a zero never reads as disagreement that was
   actually a missing reading.
   */
  readonly overlap?: number;

  /**
   What the readers said, kept even though none of it may be used.

   SO A DISAGREEMENT CAN BE DIAGNOSED RATHER THAN ONLY COUNTED. Discarding
   these left a run reporting `readers-disagree` at some number and nothing
   else, and the first time that happened on a picture already known to read
   well, no evidence survived to say whether the models had genuinely differed
   or something had gone wrong upstream of them. The number alone cannot tell
   those apart.

   NOTHING DOWNSTREAM READS THIS. `slicePictures` builds its context from
   corroborated readings only, so these travel into the stored record and no
   further. Absent when no reading arrived at all.
   */
  readonly readings?: readonly ModelReading[];
};

/**
 Whether a paired reading is a fact worth resuming on a later run.

 @param reading - what reading one picture produced

 @returns False only for an unavailable verdict that rests on a transient
 reader failure, which a later run should read again

 @example
 ```ts
 if (isResumableReading({ reading: paired, },)) await cache.persist({ key, serialized, },);
 ```
 */
export function isResumableReading({ reading, }: { readonly reading: PairedReading; },): boolean {
  if (reading.kind !== 'unavailable')
    return true;
  return !reading.transient;
}

/**
 Reads one picture with every reader and settles whether either reading is
 usable.

 @param client - transport to the provider

 @param readerModelIds - vision sub-roster, asked in this order

 @param readOcr - deterministic reader consulted before any model, which
 decides whether the picture is worth asking about at all

 @param bytes - picture as read from disk

 @param assetName - its file name, which carries the media type

 @param signal - abort honoured for every exchange

 @param perCallTimeoutMs - deadline bounding each exchange

 @param l - lane logger

 @returns Corroborated readings, or why none may be used

 @throws {@link DOMException} when `signal` aborts, since a run told to stop
 must not settle a document on the readings that beat the stop, once every
 reader has ended and each one's own failure, other than the caller's reason
 itself, is written on a line naming it, in roster order. Every other
 failure a reader raises, the deterministic reader and each model alike, is
 contained: a model reader as an unavailable reading for that reader alone, the
 deterministic reader as an unavailable gate, after which the models are asked

 @example
 ```ts
 const paired = await readImagePair({ client, readerModelIds, readOcr, bytes, assetName, signal, perCallTimeoutMs, l, },);
 ```
 */
export async function readImagePair(
  {
    client,
    readerModelIds,
    readOcr,
    bytes,
    assetName,
    signal,
    perCallTimeoutMs,
    l,
  }: {
    readonly client: ModelClient;
    readonly readerModelIds: readonly RosterModelId[];
    readonly readOcr: OcrReader;
    readonly bytes: Uint8Array;
    readonly assetName: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  },
): Promise<PairedReading> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readImagePair.name,
    l,
  },);

  // BEFORE THE GATE, not only before the models. The deterministic reader
  // spawns a decoder and tesseract per picture and consults no signal of its
  // own, so a run already told to stop would spend that on every remaining
  // asset and then persist verdicts that outlive the stop. Nothing downstream
  // distinguishes a verdict reached after the stop from one reached before it.
  signal.throwIfAborted();

  /**
   What the deterministic reader made of the picture, asked before any model.

   A GATE RATHER THAN A THIRD READER, which is the opposite of what it looks
   like it should be and was settled by measurement. Its readings do not
   corroborate the models': on `Word1.webp` it returns 405 characters against
   their 390 and 394, so it reads the same text, and still scores 0.019 and
   0.023 against them while they score 0.643 against each other. Tesseract on
   handwritten Chinese recovers the layout and substitutes lookalike glyphs,
   which leaves length intact and destroys trigram overlap. Letting it vote
   would refuse readings that are fine.

   WHAT IT IS RELIABLE AT IS PRESENCE, six of six against the models in both
   directions, which is the question worth asking first.
   */
  const ocr = await askDeterministicReader({
    readOcr,
    bytes,
    assetName,
    signal,
    l: rl,
  },);
  if (ocr.kind === 'no-text') {
    rl.info(
      `${assetName}: no text to read, so no model was asked (${
        String(ocr.characters,)
      } ${
        wordForCount({
          count: ocr.characters,
          one: 'character',
          many: 'characters',
        },)
      } from the deterministic reader)`,
    );
    return {
      kind: 'no-text',
      characters: ocr.characters,
    };
  }

  /**
   How each reader's exchange ended, asked concurrently.

   CONCURRENTLY BECAUSE THE LIMITER IS PER MODEL. Different models hold
   different slots, so asking them in sequence would double a reading's
   latency for nothing.
   */
  const settled = await Promise.allSettled(readerModelIds.map(async function ask(modelId,): Promise<{
    readonly modelId: RosterModelId;
    readonly reading: ImageReading;
  }> {
    return {
      modelId,
      reading: await readPastRefusal({
        client,
        modelId,
        bytes,
        assetName,
        signal,
        perCallTimeoutMs,
        l,
      },),
    };
  },),);

  /**
   What every reader made of the picture, a reader that failed outright carrying its failure rather than taking the
   others down with it (`containFailedReaders`).
   */
  const outcomes: readonly LabelledReading[] = containFailedReaders({
    settled,
    readerModelIds,
    assetName,
    signal,
    l: rl,
  },);

  // The entry's own abort has to stay a FAILURE rather than become two
  // unreadable pictures. `allSettled` swallows every ask a teardown rejected,
  // so a stop arriving mid-reading would otherwise return `no-reader-available`
  // and let the document settle without the readings the run was told to stop
  // gathering. This is the guard `runGatherRound` places after its own
  // `allSettled`, for the same reason. It stands after `contained`, which
  // only renders each settlement and writes a failed reader's line, so every
  // reader's own failure is written before the stop is raised.
  signal.throwIfAborted();

  /**
   Readings that arrived, labelled by model.
   */
  const readings: readonly ModelReading[] = outcomes
    .flatMap(function labelled(
      {
        modelId,
        reading,
      },
    ): readonly ModelReading[] {
      // ONE PASS RATHER THAN FILTER AND THEN MAP, so the narrowing the mapper
      // needs is the narrowing that selects. Split across two calls the
      // compiler cannot carry a predicate`s result into the mapper, and the
      // empty string that stood here to satisfy it was a reading nobody
      // produced. It was unreachable only for as long as the filter above it
      // stayed exactly right, and it is known what that arrangement costs: a
      // named absence turned back into a sentinel one call from where the
      // shape existed to forbid it.
      if (reading.kind !== 'read')
        return [];

      return [{
        modelId,
        text: reading.text,
      },];
    },);

  /**
   Readers that answered there is little or nothing to read: an absence
   report, or a reply shorter than a transcript that refused nothing.
   */
  const littleText: readonly RosterModelId[] = outcomes.flatMap(function reportsLittle(
    {
      modelId,
      reading,
    },
  ): readonly RosterModelId[] {
    if (reading.kind === 'short')
      return [modelId,];
    if ((reading.kind === 'unavailable') && (reading.reason === 'reports-no-text'))
      return [modelId,];
    return [];
  },);
  if ((readings.length < 2) && (littleText.length >= 2)) {
    /**
     The readers' confirmation, as the log states it whatever the
     deterministic reader made of the picture.
     */
    const confirmation = `${assetName}: ${String(littleText.length,)} of ${String(readerModelIds.length,)} ${
      wordForCount({
        count: readerModelIds.length,
        one: 'reader',
        many: 'readers',
      },)
    } ${
      wordForCount({
        count: littleText.length,
        one: 'reports',
        many: 'report',
      },)
    } little or no text (${littleText.join(', ',)}), so the picture is confirmed textless`;

    // A DETERMINISTIC READER THAT COULD NOT RUN TOOK NO COUNT, so the verdict
    // and the log name it unavailable with its reason: a zero here would read
    // exactly like a picture it had found bare, which a missing tool is not.
    if (ocr.kind === 'unavailable') {
      rl.info(`${confirmation} where the deterministic reader was unavailable (${ocr.reason})`,);
      return {
        kind: 'no-text',
        deterministicUnavailable: ocr.reason,
        confirmedBy: littleText,
      };
    }

    /**
     What the deterministic reader had found, which the readers now overrule.
     */
    const characters = solidCharacters({ text: ocr.text, },);
    rl.info(
      `${confirmation} past the deterministic reader's ${String(characters,)} ${
        wordForCount({
          count: characters,
          one: 'character',
          many: 'characters',
        },)
      }`,
    );
    return {
      kind: 'no-text',
      characters,
      confirmedBy: littleText,
    };
  }

  /**
   Why each reader produced nothing usable, empty for one that did.
   */
  const perReader: readonly string[] = outcomes.map(function reason(
    {
      modelId,
      reading,
    },
  ): string {
    if (reading.kind === 'unavailable')
      return `${modelId}: ${reading.reason}`;
    if (reading.kind === 'short') {
      /**
       The few characters it read.
       */
      const { text, } = reading;
      return `${modelId}: short reading of ${String(text.length,)} ${
        wordForCount({
          count: text.length,
          one: 'character',
          many: 'characters',
        },)
      }`;
    }
    return '';
  },);

  if (readings.length < 2) {
    /**
     Which shortfall this is, since one reading and none are different
     situations for anyone reading the finding.
     */
    const reason = (readings.length === 1) ? 'one-reader-only' : 'no-reader-available';
    rl.warn(
      `${assetName}: ${String(readings.length,)} of ${
        String(readerModelIds.length,)
      } ${
        wordForCount({
          count: readerModelIds.length,
          one: 'reader',
          many: 'readers',
        },)
      } produced a reading, so nothing is corroborated (${
        perReader.filter(function stated(entry,): boolean {
          return entry !== '';
        },)
          .join('; ',)
      })`,
    );
    return {
      kind: 'unavailable',
      reason,
      perReader,
      transient: outcomes.some(function failedForNow({ reading, },): boolean {
        return (reading.kind === 'unavailable') && isTransientReadingReason({ reason: reading.reason, },);
      },),
      ...(readings.length > 0) ? { readings, } : {},
    };
  }

  /**
   Which readings vouch for each other, over EVERY PAIR rather than the first
   two (class fifty-two, XingZ601, 2026-09-17: the first two stood at 0.297
   while the third agreed with both, and the entry stopped on a picture three
   readers had read).
   */
  const {
    vouched,
    closest,
  } = clusterReadings({ readings, },);

  /**
   Readers of the closest pair, named in the log either way.
   */
  const {
    left: closestLeft,
    right: closestRight,
  } = closest;
  if (vouched.length < 2) {
    rl.warn(
      `${assetName}: no two of ${String(readings.length,)} ${
        wordForCount({
          count: readings.length,
          one: 'reader',
          many: 'readers',
        },)
      } agree about what it says, `
        + `closest ${closestLeft.modelId} and ${closestRight.modelId} at overlap ${closest
          .overlap
          .toFixed(LOGGED_OVERLAP_PLACES,)}`,
    );
    // EVERY READER ANSWERED, so the disagreement is about the picture and the
    // roster, not about the call, and it is remembered like a corroboration.
    return {
      kind: 'unavailable',
      reason: 'readers-disagree',
      perReader,
      transient: false,
      overlap: closest.overlap,
      readings,
    };
  }

  rl.info(
    `${assetName}: corroborated by ${String(vouched.length,)} of ${
      String(readings.length,)
    } ${
      wordForCount({
        count: readings.length,
        one: 'reader',
        many: 'readers',
      },)
    } at overlap ${closest
      .overlap
      .toFixed(LOGGED_OVERLAP_PLACES,)}`,
  );
  return {
    kind: 'corroborated',
    readings: vouched,
    overlap: closest.overlap,
  };
}

//endregion Image reading pair
