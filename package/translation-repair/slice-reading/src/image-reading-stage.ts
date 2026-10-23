import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { wholeOpening, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  EmptyImageAssetError,
  type EncodedAsset,
  encodeImageAsset,
} from './image-asset.ts';
import {
  readingMakesSense,
  solidCharacters,
} from './image-reading-sense.ts';
import {
  readsImages,
  type ModelCaller,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type {
  ProviderName,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Image reading stage
// ONE CALL THAT READS A PICTURE, screened before anybody is allowed to use what
// it says.
//
// READING IS ITS OWN STAGE, which is what makes sending pictures workable at all. The
// cross-provider vision sub-roster is four models, selection needs a minimum weight of
// two, and a producer's ballot for its own work counts half. Combining image
// transcription with translation would entangle reading provenance with candidate
// authorship and its weights. Asking them only to READ turns the picture into text, and the
// ordinary six-model roster then translates and judges from that text with its
// weights and its disinterest untouched.
//
// THE READING IS EVIDENCE, NOT AN ANSWER. It goes beside the source and the
// archive as something a later stage may consult. Nothing here decides what
// ships.
//
// SCREENED BEFORE IT LEAVES. `readingMakesSense` implements the rule written in
// `doc/planning/when-an-image-reading-makes-no-sense.md`, and a reading that
// fails it is reported as unavailable rather than returned with a caveat: a
// caveat downstream is a caveat somebody has to remember.

/**
 What the reader is asked to do.

 EXPORTED SO THE CACHE KEY CAN FOLD IT IN. A stored reading was produced by
 ASKING something, and an edit to this sentence changes what was asked. A key
 that ignored it would serve a reading of the old question as an answer to the
 new one.

 TRANSCRIBE RATHER THAN DESCRIBE, and in the picture's own language. A
 description ("a screenshot of a profile card") shares no anchors with a
 transcript and would be refused by the screen anyway, having cost a call. The
 house policy is deliberately absent: those rules govern how this corpus is
 WRITTEN, and reading what a picture already says is not writing.

 THE SIDE OF EACH BUBBLE IS PART OF WHAT A CHAT SAYS (class thirty-three,
 2026-09-16). Only the reader sees the picture; every stage after it sees
 lines. Mio16's transcript of `photo6.webp` was bare lines, so the five judges
 reviewing the archive's chat translation guessed that the friend opened with
 在嘛 when the grey left bubble was Mio's, and revised a human translation that
 had the speakers right into one that has them swapped. A sticker went the
 same way: not a word, so not transcribed, so a message nobody was told about.
 The markers are a small fixed vocabulary so readers agree on them and the
 corroboration trigrams still overlap. MEASURED 2026-09-16 on `Mio/photo6.webp`
 and `photo7.webp`: under a first wording ("the side its bubble sits on") all
 six readers sided photo6 right, but on photo7 two of six sided the wide green
 bubbles [left] because their text starts at the left edge, so the wording
 names colour, tail and the hugged edge and says a wide bubble still has a side.
 */
export const READING_INSTRUCTION: string = 'Transcribe every word visible in this image, in the language it is '
  + 'written in, preserving line breaks and the order things appear. Include names, handles, dates, '
  + 'numbers and addresses exactly as written. Where the image is a chat or message thread, begin '
  + 'every message with [left] or [right] for the side of the screen its bubble is attached to, '
  + 'judged by its colour, its tail and the edge it hugs rather than where its text starts, since a '
  + 'wide bubble fills the width and still belongs to one side; write [sticker] or [image] on its '
  + 'own line where a message is a picture rather than words. Do not '
  + 'translate, summarize, describe the image, or add any commentary. If you cannot read the image, '
  + 'say so plainly and say nothing else.';

/**
 Most UTF-16 units of a refused reply the log keeps: enough for the sentence
 that refused, which the screen measured at 27 and 41 characters.
 */
const REFUSED_OPENING_CHARS = 160;

/**
 Substantively distinct visual reading responsibilities in retry order.
 */
export const IMAGE_READING_PERSPECTIVES = [
  'Complete transcription pass: read the full image in normal visual order.',
  'Small-text pass: prioritize low-contrast, small, edge, header, and footer text missed by a broad scan.',
  'Layout pass: inspect each visual region in order and preserve line grouping across panels or columns.',
  'Identifier pass: verify names, handles, dates, numbers, and addresses character by character, then include surrounding text.',
] as const;

/**
 One visual reading responsibility.
 */
export type ImageReadingPerspective = typeof IMAGE_READING_PERSPECTIVES[number];

/**
 The text a reader is sent beside the picture: the instruction, then the
 responsibility this reading takes. Named so the rendered-sheets fixture reads
 what the readers read (ledger X17).

 @param perspective - reading responsibility this call takes

 @returns Text part of the reading request

 @example
 ```ts
 const text = imageReadingText({ perspective: IMAGE_READING_PERSPECTIVES[0], },);
 ```
 */
export function imageReadingText({ perspective, }: { readonly perspective: ImageReadingPerspective; },): string {
  return `${READING_INSTRUCTION}\n\nREADING RESPONSIBILITY:\n${perspective}`;
}

/**
 Most bytes a picture may occupy in a reading request.

 WHAT THE GATEWAY WILL CARRY, not what the model will read. The model is the
 authority on the second and says so plainly when asked: every asset in the
 pinned corpus is accepted at its natural size, including the largest at
 1344454 bytes, and `gqt/photo1.webp` at 1274028 was read for 2631
 characters. The derivation this replaced allowed 294912 for the same model,
 so it refused 45 of 191 pictures that nobody upstream had any trouble with.

 THE GATEWAY IS THE AUTHORITY ON THE FIRST, and unlike the model it does not
 answer plainly. A body over its cap comes back as `400` naming a parse
 failure at a byte offset, which describes our JSON rather than its size, so
 a request refused for being too big reads as a request that was malformed.
 `doc/troubleshooting/synthetic-request-body-size-cap.md` is the measurement.

 SEVEN MEBIBYTES, which is more than five times the corpus's largest asset.
 The overhead around the picture is a constant 501 bytes and base64 costs a
 third on top, so this ceiling maps onto a body of 9787235 bytes, leaving
 698525 under the only size measured to pass. The eight mebibytes here until
 2026-08-22 mapped onto 11185335, which is 699575 ABOVE it: the number
 guarding the request permitted requests the gateway rejects.

 NOT THE EXACT FIT OF 7863927. Only the passing size is exact, the failing
 one is reported as approximate, and the boundary between them has never been
 bisected, so a ceiling with one byte of headroom would rest on an assumption.
 Seven mebibytes also absorbs growth in the instruction text, which is part of
 that constant.
 */
const READING_MAX_BYTES = 7_340_032;

/**
 What one reading attempt produced.

 @example
 ```ts
 const reading: ImageReading = { kind: 'read', text: 'Name: Mittens', };
 ```
 */
export type ImageReading = {
  readonly kind: 'read';

  /**
   What the model transcribed, having passed the screen.
   */
  readonly text: string;
} | {
  /**
   The model answered with fewer characters than a transcript and refused
   nothing. Never usable on its own; the pair stage counts two of them as
   confirmation that the picture carries too little to read.
   */
  readonly kind: 'short';

  /**
   The few characters it read, kept for the record.
   */
  readonly text: string;
} | {
  readonly kind: 'unavailable';

  /**
   Why no reading is available, so a finding names the reason and a later
   reader can tell a picture nobody could send from one nobody could read.
   */
  readonly reason:
    | 'model-does-not-read-images'
      | 'unknown-media-type'
      | 'too-large-for-transport'
      | 'empty-asset'
      | 'too-short'
      | 'reads-as-refusal'
      | 'reports-no-text'
      | 'empty-reply'
      | 'reader-failed';

  /**
   Set on an `empty-reply` whose finish reason was `length`: the completion
   cap ended the model while it was still reasoning, before any answer.
   That is the one empty reply a second ask has been seen to change
   (`image-reading-past-refusal.ts`), and the only one marked.
   */
  readonly cutByLength?: true;

  /**
   Provider that served the cut reply, so the second ask can go elsewhere.
   Set only together with `cutByLength`, and only when the client said.
   */
  readonly servedBy?: ProviderName;
};

/**
 Reasons that describe the provider's evening rather than the picture.

 A reader that threw, answered nothing, answered too little or declined may
 read the same picture tomorrow; a model that does not read images, a media
 type nobody names, a file too large to send, a file with no bytes, and a reader
 reporting that the picture carries no text will not change. The split decides what a pair
 verdict built on the reason is allowed to remember.
 */
const TRANSIENT_READING_REASONS: ReadonlySet<string> = new Set([
  'too-short',
  'reads-as-refusal',
  'empty-reply',
  'reader-failed',
],);

/**
 Whether a reader's reason for producing nothing may not hold tomorrow.

 @param reason - why the reader produced nothing

 @returns Whether the reason describes the call rather than the picture

 @example
 ```ts
 const transient = isTransientReadingReason({ reason: 'reader-failed', },);
 ```
 */
export function isTransientReadingReason(
  { reason, }: { readonly reason: Extract<ImageReading, { readonly kind: 'unavailable'; }>['reason']; },
): boolean {
  return TRANSIENT_READING_REASONS.has(reason,);
}

/**
 Refusal of a picture that has no bytes, shaped like the refusals `encodeImageAsset` returns.

 @example
 ```ts
 const refusal: EmptyAssetRefusal = { kind: 'refused', reason: 'empty-asset', };
 ```
 */
type EmptyAssetRefusal = {
  readonly kind: 'refused';
  readonly reason: 'empty-asset';
};

/**
 Encodes a picture for the reader, naming a picture with no bytes as refused rather than letting it throw.

 @param bytes - picture as read from disk

 @param assetName - its file name, which carries the media type

 @param l - logger the caught empty-picture error is written to, at the debug level, since the caller
 warns of the refusal once

 @returns Encoding, or the refusal, which for a picture with no bytes carries the reason `empty-asset`

 @example
 ```ts
 const encoded = encodedForReading({ bytes, assetName, l, },);
 ```
 */
function encodedForReading(
  {
    bytes,
    assetName,
    l,
  }: {
    readonly bytes: Uint8Array;
    readonly assetName: string;
    readonly l: Logger;
  },
): EncodedAsset | EmptyAssetRefusal {
  /**
   Logger pre-tagged with this function's name.
   */
  const el = tagged({
    tag: encodedForReading.name,
    l,
  },);
  try {
    return encodeImageAsset({
      bytes,
      assetName,
      maxBytes: READING_MAX_BYTES,
    },);
  }
  catch (error) {
    // ONLY THE EMPTY PICTURE IS NAMED. Any other throw is a defect of this code and keeps propagating.
    if (!(error instanceof EmptyImageAssetError))
      throw error;
    // LOGGED, NOT WARNED: `readImageAsset` warns of the refusal once for the seat read, so this line only records
    // which error was caught and which picture it named.
    el.debug(`${error.name}: ${assetName} has no bytes, so nothing of it is sent`,);
    return {
      kind: 'refused',
      reason: 'empty-asset',
    };
  }
}

/**
 Reads one picture with one model, screening what comes back.

 @param client - transport to the provider

 @param modelId - model doing the reading, which must read images

 @param bytes - picture as read from disk

 @param assetName - its file name, which carries the media type

 @param perspective - distinct visual scan responsibility

 @param otherThan - provider the exchange must not be served by, set on the
 second ask of a reply the completion cap cut, which the router serves
 elsewhere or refuses with `NoProviderForModelError`

 @param signal - abort honoured for the whole exchange

 @param perCallTimeoutMs - deadline bounding the exchange

 @param l - lane logger

 @returns Reading that passed the screen, or why none is available

 @throws {@link ProviderHttpError} on a
 non-success status, which is a transport failure rather than an unreadable
 picture and is not this function's to interpret

 @example
 ```ts
 const reading = await readImageAsset({ client, modelId, bytes, assetName, signal, perCallTimeoutMs, l, },);
 ```
 */
export async function readImageAsset(
  {
    client,
    modelId,
    bytes,
    assetName,
    perspective = IMAGE_READING_PERSPECTIVES[0],
    otherThan,
    signal,
    perCallTimeoutMs,
    l,
  }: {
    readonly client: ModelCaller;
    readonly modelId: RosterModelId;
    readonly bytes: Uint8Array;
    readonly assetName: string;
    readonly perspective?: ImageReadingPerspective;
    readonly otherThan?: ProviderName;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  },
): Promise<ImageReading> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readImageAsset.name,
    l,
  },);

  // ASKED OF BOTH CATALOGS RATHER THAN OF THE PROVIDER. A text-only model sent
  // an image part answers about nothing, or errors, and either way the call is
  // spent. Each catalog's answer comes from that provider's own reported
  // modalities, so this is a lookup rather than a guess.
  //
  // BOTH, because one provider can add or remove image support independently.
  // Asking only one catalog could refuse a reading another serving path can buy.
  if (!readsImages({ modelId, },)) {
    rl.warn(`${modelId} does not read images, so ${assetName} was not sent`,);
    return {
      kind: 'unavailable',
      reason: 'model-does-not-read-images',
    };
  }

  /**
   Picture as a content part carries it, or why it cannot be sent.
   */
  const encoded = encodedForReading({
    bytes,
    assetName,
    l: rl,
  },);
  if (encoded.kind === 'refused') {
    rl.warn(`${assetName} cannot be sent to ${modelId}: ${encoded.reason}`,);
    return {
      kind: 'unavailable',
      reason: encoded.reason,
    };
  }

  /**
   Reply from the reader.
   */
  const reply = await client.chatText({
    modelId,
    messages: [{
      role: 'user',
      content: [
        {
          type: 'text',
          text: imageReadingText({ perspective, },),
        },
        {
          type: 'image_url',
          image_url: { url: encoded.dataUri, },
        },
      ],
    },],
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((otherThan === undefined) ? {} : { otherThan, }),
  },);

  if (reply.text === '') {
    // CUT BY THE LENGTH LIMIT IS MARKED, because it is the one empty reply
    // that varies between asks: a reasoning model spent its whole completion
    // cap thinking (XingZ6016, 2026-10-21: 18316 completion tokens, 0 content
    // characters, finish reason length) and wrote no answer.
    if (reply.finishReason === 'length') {
      rl.warn(
        `${modelId} returned nothing for ${assetName}: the completion limit cut it before an answer (finish reason length`
          + `${(reply.servedBy === undefined) ? '' : ` on ${reply.servedBy}`})`,
      );
      return {
        kind: 'unavailable',
        reason: 'empty-reply',
        cutByLength: true,
        ...((reply.servedBy === undefined) ? {} : { servedBy: reply.servedBy, }),
      };
    }
    rl.warn(`${modelId} returned nothing for ${assetName}`,);
    return {
      kind: 'unavailable',
      reason: 'empty-reply',
    };
  }

  /**
   Reply text, taken off the exchange once.
   */
  const { text, } = reply;

  /**
   Reply without its surrounding whitespace.
   */
  const trimmed = text.trim();

  /**
   Whether the reading may be used at all.
   */
  const verdict = readingMakesSense({ reading: text, },);
  if (verdict.kind === 'refused') {
    // THE OPENING IS LOGGED because a refused reply carries the model's words
    // about the picture, never the picture's own text, and the Uekawakuyuurei
    // run of 2026-09-04 (15:03 UTC) could not tell from "reads-as-refusal"
    // alone whether three readers had declined to read or had reported that
    // a painting carries no text.
    /**
     Opening of the reply, ending on a whole character.
     */
    const sliced = wholeOpening({
      text: trimmed,
      units: REFUSED_OPENING_CHARS,
    },);

    /**
     Opening quoted for the log.
     */
    const opening = JSON.stringify(sliced,);
    rl.warn(`${modelId} read ${assetName} but the reading was refused: ${verdict.clause}; it opened ${opening}`,);
    return {
      kind: 'unavailable',
      reason: verdict.clause,
    };
  }
  /**
   Length the log reports, in the solid characters the verdict counted
   (non-whitespace code points), so the number and the verdict never read
   one reading in two counts.
   */
  const solid = solidCharacters({ text: trimmed, },);
  if (verdict.kind === 'short') {
    rl.info(`${modelId} read ${assetName}: ${String(solid,)} ${
      wordForCount({
        count: solid,
        one: 'character',
        many: 'characters',
      },)
    }, fewer than a transcript`,);
    return {
      kind: 'short',
      text: trimmed,
    };
  }

  // SOLID, AS THE SHORT BRANCH IS. Xu_Yushu1 (2026-09-26): a reader padded
  // 27 characters with about 2,700 ideographic spaces, and the untrimmed
  // length logged "2753 characters" for what the verdict had judged as 28.
  // Trimming fixed that, and counting the verdict's own way fixes the rest:
  // inner whitespace and astral characters (two UTF-16 units) still moved the
  // trimmed length away from the count the verdict used.
  rl.info(`${modelId} read ${assetName}: ${String(solid,)} ${
    wordForCount({
      count: solid,
      one: 'character',
      many: 'characters',
    },)
  }`,);
  return {
    kind: 'read',
    text: trimmed,
  };
}

//endregion Image reading stage
