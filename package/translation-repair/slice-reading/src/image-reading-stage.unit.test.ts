/**
 Tests for reading one picture with one model, and screening what comes back.

 WHAT THESE PIN is that every way a reading can fail to arrive is a NAMED
 outcome rather than a thrown error or a caveat attached to usable text.
 Whether a reading is of the RIGHT picture is settled one level up, in
 `image-reading-pair.ts`, against a second reader rather than against the
 archive. The
 stage sits between two things nobody controls, a picture on disk and a model's
 willingness to read it, and a caller that has to distinguish "nobody could
 send this" from "nobody could read it" cannot do so from an empty string.

 THE CAPTURED REQUEST IS THE POINT of the first case. The translate lane's judging window
 existed for weeks while production never passed it, and nothing failed,
 because no test asserted on what the call actually carried. A picture that
 never reaches the wire looks exactly like one the model ignored, so the parts
 array is asserted rather than assumed.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatTextRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  isTransientReadingReason,
  readImageAsset,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Logger the stage writes its progress to.
 */
const l = tagged({ tag: 'image-reading-stage-test', },);

/**
 Logger that tells the warning level from every other, for the cases that count warnings.

 @param warned - array the warning lines land in

 @param other - array every other line lands in

 @returns Logger writing into them

 @example
 ```ts
 const logger = levelledLogger({ warned: [], other: [], },);
 ```
 */
function levelledLogger(
  {
    warned,
    other,
  }: {
    readonly warned: string[];
    readonly other: string[];
  },
): Logger {
  /**
   Keeps a warning line.

   @param message - line as the code under test wrote it
   */
  function keepWarning(message: string,): void {
    warned.push(message,);
  }

  /**
   Keeps a line of any other level.

   @param message - line as the code under test wrote it
   */
  function keepOther(message: string,): void {
    other.push(message,);
  }

  return {
    debug: keepOther,
    error: keepOther,
    fatal: keepOther,
    flush: function flushNothing(): Promise<void> {
      return Promise.resolve();
    },
    info: keepOther,
    trace: keepOther,
    warn: keepWarning,
  };
}

/**
 Model that reads images, per the provider's own `input_modalities`.
 */
const READER: RosterModelId = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 Model that does not, so the catalog refuses before a call is spent.
 */
const TEXT_ONLY: RosterModelId = SEAT_SYNTHETIC_TEXT_EVERYWHERE;

/**
 Bytes past the reading stage's own ceiling of 8388608.

 THE CEILING IS NO LONGER PER MODEL and no longer derived from a context. It
 used to be half a model's context converted to base64 characters, which
 measured the wrong thing: sent as they are, both readers accept every asset
 in the corpus, including one of 1274028 bytes that the old derivation refused
 to the smaller of them at 294912.

 WHAT REPLACED IT MEASURES THE GATEWAY, since 2026-08-22. A body over the
 gateway's cap returns `400` naming a parse failure, so the ceiling exists to
 keep a picture from turning into an error that describes the wrong thing.
 This number stays above it, and nothing in the corpus approaches either.
 */
const OVERSIZED_BYTES = 9_000_000;

/**
 Size sitting between the ceiling this stage configures and the one it
 configured until 2026-08-22.

 BETWEEN THEM ON PURPOSE. 7340032 refuses this and 8388608 admits it, so a
 case built on this size is the only thing in the suite that can tell the two
 numbers apart. `OVERSIZED_BYTES` clears both and would pass under either.
 */
const PAST_THE_NEW_CEILING_ONLY = 8_000_000;

/**
 Transcription a reader returns for the picture under test.
 */
const A_READING = '虎斑猫 Mittens，2019 年领养，联系方式 @mittenspaw。';

/**
 Bytes standing in for a picture, whose content no rule here reads.

 @param length - how many bytes the picture occupies

 @returns Buffer of that size

 @example
 ```ts
 const bytes = bytesOf({ length: 64, },);
 ```
 */
function bytesOf({ length, }: { readonly length: number; },): Uint8Array {
  return new Uint8Array(length,).fill(7,);
}

/**
 Client answering every reading with one fixed reply, recording what it was
 asked.

 @param text - reply the model returns

 @returns Client and the requests it received

 @example
 ```ts
 const { client, requests, } = replyingClient({ text: A_READING, },);
 ```
 */
function replyingClient({ text, }: { readonly text: string; },): {
  readonly client: ModelClient;
  readonly requests: ChatTextRequest[];
} {
  /**
   Requests the stage sent, for the assertion that a picture reached the wire.
   */
  const requests: ChatTextRequest[] = [];

  return {
    requests,
    client: {
      chatText: async (request,) => {
        requests.push(request as ChatTextRequest,);
        return { text, };
      },
      chatJson: async () => {
        throw new Error('chatJson unused by the reading stage',);
      },
      quotas: async () => {
        throw new Error('quotas unused by the reading stage',);
      },
    },
  };
}

await describe({
  name: readImageAsset.name,
  children: [
    it({
      name: 'SENDS THE PICTURE AS A CONTENT PART BESIDE THE INSTRUCTION, and returns what came '
        + 'back. A reading the wire never carried is indistinguishable from one the model ignored, '
        + 'so this asserts the parts array rather than only the returned text',
      fn: async () => {
        const { client, requests, } = replyingClient({ text: A_READING, },);

        /**
         One reading of a picture the archive already transcribes.
         */
        const reading = await readImageAsset({
          client,
          modelId: READER,
          bytes: bytesOf({ length: 64, },),
          assetName: 'mittens.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('read',);
        if (reading.kind !== 'read')
          throw new Error('read by construction',);
        expect(reading.text,).toBe(A_READING,);

        expect(requests.length,).toBe(1,);

        /**
         Sole message the reader was sent.
         */
        const message = requests[0]
          ?.messages[0];
        if (message === undefined)
          throw new Error('one message by construction',);

        /**
         Its parts, which must be an array rather than a bare string.
         */
        const { content, } = message;
        if ((typeof content) === 'string')
          throw new Error('a picture cannot travel as a string',);

        expect(content.length,).toBe(2,);
        expect(content[0]
          ?.type,).toBe('text',);
        expect(content[1]
          ?.type,).toBe('image_url',);

        /**
         Picture part, whose data URI declares the media type it was encoded
         under.
         */
        const [
          ,
          picture,
        ] = content;
        if (picture?.type !== 'image_url')
          throw new Error('image part by construction',);
        expect(picture.image_url
          .url
          .startsWith('data:image/webp;base64,',),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES A MODEL THE CATALOG SAYS DOES NOT READ IMAGES, without spending a call. A '
        + 'text-only model sent a picture answers about nothing or errors, and either way the quota '
        + 'is gone',
      fn: async () => {
        const { client, requests, } = replyingClient({ text: A_READING, },);

        /**
         Attempt against a model whose `input_modalities` carry no image.
         */
        const reading = await readImageAsset({
          client,
          modelId: TEXT_ONLY,
          bytes: bytesOf({ length: 64, },),
          assetName: 'mittens.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('unavailable',);
        if (reading.kind !== 'unavailable')
          throw new Error('unavailable by construction',);
        expect(reading.reason,).toBe('model-does-not-read-images',);
        expect(requests.length,).toBe(0,);
      },
    },),

    it({
      name: 'REFUSES AN EXTENSION THE CORPUS DOES NOT USE, rather than guessing a media type. A '
        + 'picture sent under the wrong one asks a model to decode something it was not given',
      fn: async () => {
        const { client, requests, } = replyingClient({ text: A_READING, },);

        /**
         Attempt against a file whose name declares nothing sendable.
         */
        const reading = await readImageAsset({
          client,
          modelId: READER,
          bytes: bytesOf({ length: 64, },),
          assetName: 'mittens.heic',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('unavailable',);
        if (reading.kind !== 'unavailable')
          throw new Error('unavailable by construction',);
        expect(reading.reason,).toBe('unknown-media-type',);
        expect(requests.length,).toBe(0,);
      },
    },),

    it({
      name: 'NAMES A PICTURE WITH NO BYTES AS UNAVAILABLE WITHOUT A CALL: the encoder refuses a zero-byte file by '
        + 'throwing, and the stage turns that into the reason a later reader can tell from a model that failed, so the '
        + 'picture is not retried tomorrow and no seat spends a request on it',
      fn: async () => {
        const { client, requests, } = replyingClient({ text: A_READING, },);

        /**
         Attempt with a zero-byte file.
         */
        const reading = await readImageAsset({
          client,
          modelId: READER,
          bytes: new Uint8Array(0,),
          assetName: 'mittens.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('unavailable',);
        if (reading.kind !== 'unavailable')
          throw new Error('unavailable by construction',);
        expect(reading.reason,).toBe('empty-asset',);
        expect(isTransientReadingReason({ reason: reading.reason, },),).toBe(false,);
        expect(requests.length,).toBe(0,);
      },
    },),

    it({
      name: 'WARNS ONCE FOR A PICTURE WITH NO BYTES: the stage says it cannot send '
        + 'the picture, and the error the encoder threw is logged under the tag of the function that caught it, '
        + 'at the debug level, so a seat read leaves one warning and not two',
      fn: async () => {
        /**
         Lines written at the warning level.
         */
        const warned: string[] = [];

        /**
         Lines written at every other level.
         */
        const other: string[] = [];
        const { client, requests, } = replyingClient({ text: A_READING, },);
        await readImageAsset({
          client,
          modelId: READER,
          bytes: new Uint8Array(0,),
          assetName: 'mittens.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: levelledLogger({
            warned,
            other,
          },),
        },);

        expect(requests.length,).toBe(0,);
        expect(warned.length,).toBe(1,);
        expect(warned[0],).toContain('mittens.webp cannot be sent',);
        expect(warned[0],).toContain('empty-asset',);
        expect(other.some(function namesTheCaughtError(line,): boolean {
          return line.includes('[readImageAsset] [encodedForReading] ',) && line.includes('has no bytes',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES A PICTURE PAST THE STAGE CEILING rather than downscaling it. A shrunk '
        + 'photograph of handwriting is the exact input that produces a confident wrong reading, '
        + 'and the ceiling now measures what the gateway will carry rather than predicting what '
        + 'a model will take, which it did badly. THE REQUEST COUNT IS THE OTHER HALF: refusing '
        + 'before sending is what keeps an oversize body from returning as a parse error',
      fn: async () => {
        const { client, requests, } = replyingClient({ text: A_READING, },);

        /**
         Attempt with a picture past the stage's ceiling.
         */
        const reading = await readImageAsset({
          client,
          modelId: READER,
          bytes: bytesOf({ length: OVERSIZED_BYTES, },),
          assetName: 'mittens.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('unavailable',);
        if (reading.kind !== 'unavailable')
          throw new Error('unavailable by construction',);
        expect(reading.reason,).toBe('too-large-for-transport',);
        expect(requests.length,).toBe(0,);
      },
    },),

    it({
      name: 'REFUSES AT THE CEILING THE MEASUREMENT DECIDED, not at the one it replaced. This '
        + 'size clears 7340032 and does not clear 8388608, so it is the only input in the suite '
        + 'that can tell those two numbers apart. The old ceiling mapped onto a request body of '
        + '11185335 bytes against the 10485760 measured to pass, which means the guard against '
        + 'oversize requests was itself admitting requests the gateway rejects, and rejecting '
        + 'them as a parse error naming our JSON',
      fn: async () => {
        const { client, requests, } = replyingClient({ text: A_READING, },);

        /**
         Attempt with a picture between the two ceilings.
         */
        const reading = await readImageAsset({
          client,
          modelId: READER,
          bytes: bytesOf({ length: PAST_THE_NEW_CEILING_ONLY, },),
          assetName: 'mittens.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('unavailable',);
        if (reading.kind !== 'unavailable')
          throw new Error('unavailable by construction',);
        expect(reading.reason,).toBe('too-large-for-transport',);
        expect(requests.length,).toBe(0,);
      },
    },),

    it({
      name: 'NAMES AN EMPTY REPLY SEPARATELY FROM A SHORT ONE. Nothing at all means the exchange '
        + 'produced no content, which is a transport outcome; a few characters means the model '
        + 'answered and said too little, which is a reading outcome',
      fn: async () => {
        const { client, } = replyingClient({ text: '', },);

        /**
         Attempt whose reply carries no content at all.
         */
        const reading = await readImageAsset({
          client,
          modelId: READER,
          bytes: bytesOf({ length: 64, },),
          assetName: 'mittens.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('unavailable',);
        if (reading.kind !== 'unavailable')
          throw new Error('unavailable by construction',);
        expect(reading.reason,).toBe('empty-reply',);
      },
    },),

    it({
      name: 'LOGS A READING\'S LENGTH IN THE SOLID CHARACTERS THE VERDICT COUNTED, not in UTF-16 units: fifteen '
        + 'astral letters are fifteen characters and a short reading, twenty spaced ones are twenty and a read',
      fn: async () => {
        /**
         Lines the stage logged for each reading, in the order the readings are listed.
         */
        const logged = await Promise.all(
          ['\u{1D4B6}'.repeat(15,), Array.from({ length: 20, }, () => '\u{1D4B6}',).join(' ',),]
            .map(async function readOne(text,): Promise<readonly string[]> {
              /**
               Lines this reading wrote.
               */
              const lines: string[] = [];
              await readImageAsset({
                client: replyingClient({ text, },).client,
                modelId: READER,
                bytes: bytesOf({ length: 64, },),
                assetName: 'mittens.webp',
                signal: AbortSignal.timeout(HANG_STOP_MS,),
                perCallTimeoutMs: HANG_STOP_MS,
                l: capturingLogger({ messages: lines, },),
              },);
              return lines;
            },),
        );
        /**
         Every line, the first reading's first.
         */
        const messages = logged.flat();
        expect(messages.filter(function isReading(line,): boolean {
          return line.includes(' read mittens.webp: ',);
        },),).toStrictEqual([
          `[readImageAsset] ${READER} read mittens.webp: 15 characters, fewer than a transcript`,
          `[readImageAsset] ${READER} read mittens.webp: 20 characters`,
        ],);
      },
    },),

    it({
      name: 'RETURNS A SHORT READING for a reply under the transcript line that refuses nothing, which is '
        + 'what a picture carrying a few characters produces; never usable alone, and the pair stage '
        + 'counts two of them as a textless picture',
      fn: async () => {
        const { client, } = replyingClient({ text: '喵。', },);

        /**
         Attempt whose reply is content but not a transcription.
         */
        const reading = await readImageAsset({
          client,
          modelId: READER,
          bytes: bytesOf({ length: 64, },),
          assetName: 'mittens.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('short',);
        if (reading.kind !== 'short')
          throw new Error('short by construction',);
        expect(reading.text,).toBe('喵。',);
      },
    },),

    it({
      name: 'NAMES AN ABSENCE REPORT SEPARATELY FROM A DECLINE, since a reader saying the picture carries '
        + 'no text is describing the picture and is not asked again (Uekawakuyuurei, 2026-09-04: twelve '
        + 'asks about a painting of ships)',
      fn: async () => {
        const { client, } = replyingClient({ text: 'There is no text visible in this image.', },);

        /**
         Attempt whose reply reports that the picture carries nothing.
         */
        const reading = await readImageAsset({
          client,
          modelId: READER,
          bytes: bytesOf({ length: 64, },),
          assetName: 'mittens.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('unavailable',);
        if (reading.kind !== 'unavailable')
          throw new Error('unavailable by construction',);
        expect(reading.reason,).toBe('reports-no-text',);
      },
    },),

    it({
      name: 'REFUSES A REPLY THAT ANNOUNCES ITSELF AS A REFUSAL, which would otherwise reach a '
        + 'translator as a source passage saying the model could not read the picture',
      fn: async () => {
        const { client, } = replyingClient({
          text: 'I cannot read the text in this image, as it is too blurry to make out.',
        },);

        /**
         Attempt whose reply is an apology of transcription length.
         */
        const reading = await readImageAsset({
          client,
          modelId: READER,
          bytes: bytesOf({ length: 64, },),
          assetName: 'mittens.webp',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('unavailable',);
        if (reading.kind !== 'unavailable')
          throw new Error('unavailable by construction',);
        expect(reading.reason,).toBe('reads-as-refusal',);
      },
    },),

    it({
      name: 'ACCEPTS A JPEG AS READILY AS A WEBP, since four of the corpus assets are jpg and a '
        + 'reader that only understood one extension would report those four as unsendable',
      fn: async () => {
        const { client, requests, } = replyingClient({ text: A_READING, },);

        /**
         Attempt against the other extension the corpus uses.
         */
        const reading = await readImageAsset({
          client,
          modelId: READER,
          bytes: bytesOf({ length: 64, },),
          assetName: 'mittens.jpg',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);

        expect(reading.kind,).toBe('read',);

        /**
         Picture part of the sole message, whose media type follows the name.
         */
        const content = requests[0]
          ?.messages[0]
          ?.content;
        if (((typeof content) === 'string') || (content === undefined))
          throw new Error('a picture cannot travel as a string',);

        /**
         That part, narrowed to the picture it is.
         */
        const [
          ,
          picture,
        ] = content;
        if (picture?.type !== 'image_url')
          throw new Error('image part by construction',);
        expect(picture.image_url
          .url
          .startsWith('data:image/jpeg;base64,',),).toBe(true,);
      },
    },),
  ],
},);
