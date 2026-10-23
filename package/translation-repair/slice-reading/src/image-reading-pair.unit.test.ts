/**
 Tests for reading one picture with the whole vision sub-roster and letting
 the two readers decide whether either reading may be used.

 WHAT THESE PIN is that corroboration is the gate. A reading that arrives is
 not a reading that may be used: it has to be matched by a second reader shown
 the same picture and nothing else, and where no second reading exists the
 first is refused rather than passed along with a caveat.

 BOTH READINGS TRAVEL when they agree, which one of these asserts directly.
 Agreement establishes that two readers describe the same picture, not the
 same amount of it, so a stage handed only the longer would lose the shorter's
 vouching and a stage handed only the shorter would lose content.

 A FAILING READER IS CONTAINED, which three of these pin from both sides. A
 reading is the one output nothing downstream requires, so a reader that
 throws must cost its own reading and nothing else. An ABORT is not such a
 failure and must still travel, because a run told to stop must not settle a
 document on the readings that beat the stop.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { setImmediate as turnOfTheLoop, } from 'node:timers/promises';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatTextRequest,
  EveryProviderDryError,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  isResumableReading,
  type PairedReading,
  readImagePair,
} from '../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { modelDarkRefusal, } from '@monochromatic-dev/translation-repair-provider/ts/model-dark-refusal.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import {
  refusedClient,
  statusFailureLogText,
} from '@monochromatic-dev/translation-repair-provider/ts/provider-status-failure.test-fixture.ts';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';
import {
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { untilAborted, } from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';

/**
 Logger the stage writes its progress to.
 */
const l = tagged({ tag: 'image-reading-pair-test', },);

/**
 Vision sub-roster, which is exactly these two models.
 */
const READERS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
];

/**
 Model whose context is the larger of the two, so a picture can be sized to
 fit it alone.
 */
const LARGER_READER: RosterModelId = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 What one reader transcribed from a picture of a noticeboard.
 */
const READING = '走失猫咪 Mittens，虎斑，2019 年出生，联系 @mittenspaw，请电 555 0134。';

/**
 What the other reader transcribed from the same picture, worded differently
 where a transcription can differ and identical where it cannot.
 */
const AGREEING_READING = '走失猫咪 Mittens，虎斑，2019 年出生，联系 @mittenspaw，电话 555 0134。';

/**
 What a reader transcribed from some other picture entirely.
 */
const OTHER_PICTURE = '兽医诊所营业时间：周一至周五上午九点到下午六点，周六休息。';

/**
 Deterministic reader that finds text, which is what lets the models be asked.

 THE REAL ONE SHELLS OUT to `dwebp` and `tesseract`, so every case here supplies
 a stub instead. That is the point of the seam: a test must not depend on which
 command-line tools a machine happens to carry.
 */
async function found(): Promise<{
  readonly kind: 'read';
  readonly text: string;
}> {
  return {
    kind: 'read',
    text: '走失猫咪 Mittens',
  };
}

/**
 Deterministic reader that finds nothing, which is two thirds of the corpus.
 */
async function empty(): Promise<{
  readonly kind: 'no-text';
  readonly characters: number;
}> {
  return {
    kind: 'no-text',
    characters: 3,
  };
}

/**
 Deterministic reader that could not run, standing in for a machine without
 the tools installed.
 */
async function missing(): Promise<{
  readonly kind: 'unavailable';
  readonly reason: 'ocr-tool-missing';
}> {
  return {
    kind: 'unavailable',
    reason: 'ocr-tool-missing',
  };
}

/**
 Deterministic reader that rejects, standing in for a machine fault such as a
 scratch device with no space left.
 */
async function rejecting(): Promise<never> {
  throw new Error('no space left on the scratch device',);
}

/**
 Bytes standing in for a picture, whose content no rule here reads.

 @param length - how many bytes picture occupies

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
 Client answering each model with whatever that model is scripted to say.

 @param byModel - reply per model; a model absent from this map answers with
 nothing, which the reading stage reports as an empty reply

 @returns Client and models it was asked, in the order asks arrived

 @example
 ```ts
 const { client, asked, } = scriptedClient({ byModel: { [LARGER_READER]: READING, }, },);
 ```
 */
function scriptedClient(
  { byModel, }: { readonly byModel: Readonly<Record<string, string>>; },
): {
  readonly client: ModelClient;
  readonly asked: RosterModelId[];
} {
  /**
   Models a reading was requested from.
   */
  const asked: RosterModelId[] = [];

  return {
    asked,
    client: {
      chatText: async (request,) => {
        /**
         Request as this package's own contract describes it.
         */
        const { modelId, } = request as ChatTextRequest;
        asked.push(modelId,);
        return { text: byModel[modelId] ?? '', };
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

/**
 Client that throws for named models and answers the rest from a script.

 THROWS RATHER THAN RETURNING AN ERROR SHAPE, because that is what the
 production client does: the runaway guard, the retry ceiling and a transport
 failure all leave `chatText` by rejecting.

 @param failing - models whose exchange throws, and the message it throws with

 @param byModel - reply per model that does not throw

 @returns Client and models it was asked, in the order asks arrived

 @example
 ```ts
 const { client, } = failingClient({ failing: { [LARGER_READER]: 'runaway', }, byModel: {}, },);
 ```
 */
function failingClient(
  {
    failing,
    byModel,
  }: {
    readonly failing: Readonly<Record<string, string>>;
    readonly byModel: Readonly<Record<string, string>>;
  },
): {
  readonly client: ModelClient;
  readonly asked: RosterModelId[];
} {
  /**
   Models a reading was requested from.
   */
  const asked: RosterModelId[] = [];

  return {
    asked,
    client: {
      chatText: async (request,) => {
        /**
         Request as this package's own contract describes it.
         */
        const { modelId, } = request as ChatTextRequest;
        asked.push(modelId,);

        /**
         Message this model is scripted to fail with, absent when it succeeds.
         */
        const failure = failing[modelId];
        if (failure !== undefined)
          throw new Error(failure,);
        return { text: byModel[modelId] ?? '', };
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
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readImagePair.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CORROBORATES TWO READINGS OF ONE PICTURE AND FORWARDS BOTH, labelled by model. '
            + 'Agreement says the two readers describe the same picture, not the same amount of it, so '
            + 'a stage handed only one of them loses either the vouching or the content',
          fn: async () => {
            const { client, asked, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: READING,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: AGREEING_READING,
              },
            },);

            /**
             What the roster made of the picture.
             */
            const paired = await readImagePair({
              client,
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(paired.kind,).toBe('corroborated',);
            if (paired.kind !== 'corroborated')
              throw new Error('corroborated by construction',);

            expect(asked.length,).toBe(2,);
            expect(paired.readings
              .length,).toBe(2,);
            expect(paired.readings
              .map(function toModel(reading,): RosterModelId {
                return reading.modelId;
              },),).toEqual([...READERS,],);
            expect(paired.readings
              .map(function toText(reading,): string {
                return reading.text;
              },),).toEqual([
                READING,
                AGREEING_READING,
              ],);
            expect(paired.overlap > 0,).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES TWO READINGS THAT DESCRIBE DIFFERENT PICTURES, and records how far apart they '
            + 'were. Used, a reading of the wrong picture would licence replacing a careful '
            + 'transcription with something derived from a misreading, and no judge downstream could '
            + 'tell, because the reading is the only evidence any of them has about the picture',
          fn: async () => {
            const { client, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: READING,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: OTHER_PICTURE,
              },
            },);

            /**
             What the roster made of two irreconcilable readings.
             */
            const paired = await readImagePair({
              client,
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(paired.kind,).toBe('unavailable',);
            if (paired.kind !== 'unavailable')
              throw new Error('unavailable by construction',);
            expect(paired.reason,).toBe('readers-disagree',);
            // Both readers answered, so the disagreement is about the picture and
            // the roster, not the call, and it is remembered like a corroboration.
            expect(paired.transient,).toBe(false,);
            expect(isResumableReading({ reading: paired, },),).toBe(true,);
            expect(paired.overlap === undefined,).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A PICTURE ONLY ONE READER CAN BE SENT, which is the cost this design accepts '
            + 'rather than caveats. A caveat travelling downstream is a caveat somebody has to '
            + 'remember, and the passage it would qualify is already protected by the guards that hold '
            + 'every transcript in the corpus today',
          fn: async () => {
            // ONE READER SAYING NOTHING, rather than a picture only one could be
            // sent. That used to be a size case, and size no longer separates the
            // two readers: the per-model ceiling was measuring base64 length
            // against a context and predicting nothing, so both now share one
            // ceiling far above anything in the corpus.
            const { client, } = scriptedClient({
              byModel: { [SEAT_SYNTHETIC_VISION_WITHHELD]: READING, },
            },);

            /**
             What the roster made of a picture only one reader answered about.
             */
            const paired = await readImagePair({
              client,
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(paired.kind,).toBe('unavailable',);
            if (paired.kind !== 'unavailable')
              throw new Error('unavailable by construction',);
            expect(paired.reason,).toBe('one-reader-only',);
            expect(paired.overlap,).toBe(undefined,);
          },
        },),

        it({
          name: 'NAMES WHY EACH READER PRODUCED NOTHING, so a finding can tell a picture nobody could '
            + 'send apart from one nobody could read. Both are silence in the artifact otherwise',
          fn: async () => {
            const { client, } = scriptedClient({ byModel: {}, },);

            /**
             What the roster made of a picture neither reader answered about.
             */
            const paired = await readImagePair({
              client,
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(paired.kind,).toBe('unavailable',);
            if (paired.kind !== 'unavailable')
              throw new Error('unavailable by construction',);
            expect(paired.reason,).toBe('no-reader-available',);
            expect(paired.perReader
              .length,).toBe(2,);
            expect(paired.perReader
              .every(function names(entry,): boolean {
                return entry.includes('empty-reply',);
              },),).toBe(true,);
          },
        },),

        it({
          name: 'CONFIRMS A TEXTLESS PICTURE FROM TWO ABSENCE REPORTS where the deterministic reader found '
            + 'noise (Uekawakuyuurei IMG_1308, 2026-09-04: a painting of ships, 24 characters of canvas from '
            + 'tesseract, every reader on both rosters saying it carries no text, and the entry stopped at the '
            + 'completeness gate on every run), and RESUMES it',
          fn: async () => {
            const { client, asked, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'There is no visible text in this image. It is a painting of ships at sea.',
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'I cannot read any text in this image. There are no visible words.',
              },
            },);

            /**
             What the roster made of a picture the deterministic reader called text.
             */
            const paired = await readImagePair({
              client,
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'ships.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(asked.length,).toBe(2,);
            expect(paired,).toStrictEqual({
              kind: 'no-text',
              characters: 11,
              confirmedBy: READERS,
            },);
            expect(isResumableReading({ reading: paired, },),).toBe(true,);
          },
        },),

        it({
          name: 'CONFIRMS A TEXTLESS PICTURE FROM TWO SHORT READINGS, which is what a drawing with a hull '
            + 'number on it produces (Uekawakuyuurei img370: DE581, DE581 and D650 from the three readers)',
          fn: async () => {
            const { client, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'DE581',
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'D650',
              },
            },);

            /**
             What the roster made of a picture carrying a few characters.
             */
            const paired = await readImagePair({
              client,
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'destroyer.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(paired.kind,).toBe('no-text',);
            if (paired.kind !== 'no-text')
              throw new Error('no-text by construction',);
            expect(paired.confirmedBy,).toStrictEqual(READERS,);
          },
        },),

        it({
          name: 'STAYS UNAVAILABLE AND TRANSIENT with one absence report beside a decline, since one reader '
            + 'saying nothing is there and one declining to look is not two witnesses to anything',
          fn: async () => {
            const { client, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'There is no visible text in this image.',
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'I cannot read the image.',
              },
            },);

            /**
             What the roster made of a picture one reader would not look at.
             */
            const paired = await readImagePair({
              client,
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'ships.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(paired.kind,).toBe('unavailable',);
            if (paired.kind !== 'unavailable')
              throw new Error('unavailable by construction',);
            expect(paired.reason,).toBe('no-reader-available',);
            expect(paired.transient,).toBe(true,);
            expect(paired.perReader,).toStrictEqual([
              'hf:moonshotai/Kimi-K3: reports-no-text',
              'hf:Qwen/Qwen3.8-27B: reads-as-refusal',
            ],);
          },
        },),

        it({
          name: 'KEEPS ONE READING BESIDE ONE SHORT ONE UNCORROBORATED, naming the short one, since a reader '
            + 'that read a passage and one that read five characters did not read the same thing',
          fn: async () => {
            const { client, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: READING,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'DE581',
              },
            },);

            /**
             What the roster made of a picture the readers saw differently.
             */
            const paired = await readImagePair({
              client,
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(paired.kind,).toBe('unavailable',);
            if (paired.kind !== 'unavailable')
              throw new Error('unavailable by construction',);
            expect(paired.reason,).toBe('one-reader-only',);
            expect(paired.perReader,).toStrictEqual([
              '',
              'hf:Qwen/Qwen3.8-27B: short reading of 5 characters',
            ],);
          },
        },),

        it({
          name: 'REFUSES A ROSTER OF ONE, since a sole reader has nothing to be corroborated by '
            + 'however well it reads',
          fn: async () => {
            const { client, asked, } = scriptedClient({
              byModel: { [SEAT_SYNTHETIC_VISION_WITHHELD]: READING, },
            },);

            /**
             What one reader alone produced.
             */
            const paired = await readImagePair({
              client,
              readOcr: found,
              readerModelIds: [LARGER_READER,],
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(asked.length,).toBe(1,);
            expect(paired.kind,).toBe('unavailable',);
            if (paired.kind !== 'unavailable')
              throw new Error('unavailable by construction',);
            expect(paired.reason,).toBe('one-reader-only',);
          },
        },),

        it({
          name: 'CONTAINS A READER THAT THROWS and keeps the other reader\'s reading, rather than '
            + 'failing the picture. Measured on a real run before this held: one reader looped, the '
            + 'client\'s runaway guard ended the call, the rejection reached the entry driver, and the '
            + 'entry finished with nothing settled, so both lanes were lost to a transcription nobody '
            + 'downstream requires',
          fn: async () => {
            const { client, asked, } = failingClient({
              failing: { [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'ended a runaway call, reasoning channel repeated itself', },
              byModel: { [SEAT_SYNTHETIC_VISION_WITHHELD]: READING, },
            },);

            /**
             Lines the pair logged while one reader failed.
             */
            const lines: string[] = [];

            /**
             What the roster made of a picture one reader could not finish reading.
             */
            const paired = await readImagePair({
              client,
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l: levelCapturingLogger({ lines, },),
            },);

            expect(asked.length,).toBe(2,);
            // THE FAILED READER IS NAMED BY CLASS: an exchange failure's message can quote a header value
            // or a provider body.
            expect(lines.filter(function failedOutright(line,): boolean {
              return line.includes('failed outright',);
            },),).toStrictEqual([
              `warn [readImagePair] noticeboard.webp: ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} failed outright, `
                + 'so it contributes no reading (refused by Error)',
            ],);
            expect(paired.kind,).toBe('unavailable',);
            if (paired.kind !== 'unavailable')
              throw new Error('unavailable by construction',);

            // The surviving reader is short of corroboration, which is a
            // one-reader shortfall rather than a picture nobody could send.
            expect(paired.reason,).toBe('one-reader-only',);
            expect(paired.perReader
              .some(function named(entry,): boolean {
                return entry.includes('reader-failed',);
              },),).toBe(true,);
            // A reader that threw may read tomorrow, so this verdict describes
            // the evening and must not be remembered.
            expect(paired.transient,).toBe(true,);
            expect(isResumableReading({ reading: paired, },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES WITH THE DRY ERROR when every provider is out of budget, rather than recording each reader '
            + 'as failed and going on with no reader available, even for a client built without the pass guard '
            + 'and a signal nobody aborted',
          fn: async () => {
            /**
             What the router raises when it finds every provider out of budget.
             */
            const dry = new EveryProviderDryError({ measured: 'the cats ate every meter', },);
            /**
             Lines the pair logged.
             */
            const lines: string[] = [];
            /**
             What the pair did on a dry router.
             */
            const refusal = await rejectionOf({
              promise: readImagePair({
                client: {
                  chatText: async () => {
                    throw dry;
                  },
                  chatJson: async () => {
                    throw new Error('chatJson unused by the reading stage',);
                  },
                  quotas: async () => {
                    throw new Error('quotas unused by the reading stage',);
                  },
                },
                readOcr: found,
                readerModelIds: READERS,
                bytes: bytesOf({ length: 64, },),
                assetName: 'noticeboard.webp',
                signal: AbortSignal.timeout(HANG_STOP_MS,),
                perCallTimeoutMs: HANG_STOP_MS,
                l: levelCapturingLogger({ lines, },),
              },),
            },);
            expect(refusal,).toBe(dry,);
            expect(lines.filter(function failedOutright(line,): boolean {
              return line.includes('failed outright',);
            },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'CONTAINS BOTH READERS MEETING A MODEL-DARK REFUSAL, for a client built without the pass guard, and '
            + 'reports a picture nobody could read, since a reader whose own providers are dry is a lost voice and '
            + 'the dry refusal that ends the run is a different class',
          fn: async () => {
            /**
             What the pair did when every reader's own providers were dry.
             */
            const paired = await readImagePair({
              client: {
                chatText: async () => {
                  throw modelDarkRefusal({ modelId: SEAT_SYNTHETIC_VISION_WITHHELD, },);
                },
                chatJson: async () => {
                  throw new Error('chatJson unused by the reading stage',);
                },
                quotas: async () => {
                  throw new Error('quotas unused by the reading stage',);
                },
              },
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);
            expect(paired.kind,).toBe('unavailable',);
          },
        },),

        it({
          name: 'CONTAINS BOTH READERS THROWING and reports a picture nobody could read, rather than '
            + 'propagating either failure. The pipeline reads pictures to gain evidence, so the worst '
            + 'a reading stage may cost is the evidence it failed to gather',
          fn: async () => {
            const { client, } = failingClient({
              failing: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'HTTP 500 after every retry',
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'ended a runaway call',
              },
              byModel: {},
            },);

            /**
             What the roster made of a picture neither reader could finish.
             */
            const paired = await readImagePair({
              client,
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(paired.kind,).toBe('unavailable',);
            if (paired.kind !== 'unavailable')
              throw new Error('unavailable by construction',);
            expect(paired.reason,).toBe('no-reader-available',);
            expect(paired.perReader
              .filter(function named(entry,): boolean {
                return entry.includes('reader-failed',);
              },).length,).toBe(2,);
            expect(paired.transient,).toBe(true,);
          },
        },),

        it({
          name: 'NAMES EACH READER THE PROVIDER REFUSED by the picture, the model, the status and the provider\'s '
            + 'words with the key its refusal echoed masked, for readers asked over the real client and transport',
          fn: async ctx => {
            /**
             Lines the pair logged while both readers were refused.
             */
            const lines: string[] = [];

            /**
             What the roster made of a picture whose readers the provider refused.
             */
            const paired = await readImagePair({
              client: refusedClient({
                sinon: ctx.sinon,
                status: 401,
              },),
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l: levelCapturingLogger({ lines, },),
            },);

            expect(paired.kind,).toBe('unavailable',);
            expect(lines.filter(function failedOutright(line,): boolean {
              return line.includes('failed outright',);
            },),).toEqual(READERS.map(function lineOf(modelId,): string {
              return `warn [readImagePair] noticeboard.webp: ${modelId} failed outright, so it contributes no reading `
                + `(${statusFailureLogText({ provider: 'synthetic', status: 401, },)})`;
            },),);
          },
        },),

        it({
          name: 'CONTAINS A DETERMINISTIC READER THAT REJECTS, asks both models without its gate, and warns '
            + 'naming the picture and the failure, rather than ending the entry for a reading nothing '
            + 'downstream requires',
          fn: async () => {
            const { client, asked, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: READING,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: AGREEING_READING,
              },
            },);

            /**
             Lines the pair logged while the deterministic reader failed.
             */
            const lines: string[] = [];

            /**
             What the roster made of a picture whose deterministic reader rejected.
             */
            const paired = await readImagePair({
              client,
              readOcr: rejecting,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l: levelCapturingLogger({ lines, },),
            },);

            expect(asked.length,).toBe(2,);
            expect(paired.kind,).toBe('corroborated',);
            expect(lines.filter(function fromPair(line,): boolean {
              return line.startsWith('warn [readImagePair]',);
            },),).toStrictEqual([
              'warn [readImagePair] noticeboard.webp: the deterministic reader failed outright, so the models '
                + 'are asked without its gate (refused by Error)',
            ],);
          },
        },),

        it({
          name: 'NAMES A DETERMINISTIC READER THAT REJECTED UNAVAILABLE AS ocr-failed in a textless '
            + 'confirmation, recording no count of characters it never read',
          fn: async () => {
            const { client, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'There is no visible text in this image.',
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'I cannot read any text in this image.',
              },
            },);

            /**
             What the roster made of a picture whose deterministic reader rejected.
             */
            const paired = await readImagePair({
              client,
              readOcr: rejecting,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(paired,).toStrictEqual({
              kind: 'no-text',
              deterministicUnavailable: 'ocr-failed',
              confirmedBy: READERS,
            },);
          },
        },),

        it({
          name: 'FORWARDS AN ABORT THAT ARRIVES WHILE THE DETERMINISTIC READER RUNS, rather than absorbing it '
            + 'as that reader\'s failure and asking the models on a run told to stop',
          fn: async () => {
            const { client, asked, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: READING,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: AGREEING_READING,
              },
            },);

            /**
             Stop that lands while the deterministic reader is running.
             */
            const stopping = new AbortController();

            /**
             Deterministic reader the stop interrupts: it rejects as a decoder
             killed by the stop would.
             */
            async function interrupted(): Promise<never> {
              stopping.abort();
              throw new Error('decoder killed by the stop',);
            }

            /**
             Name of whatever escaped, or what the call returned instead.
             */
            let escaped = 'nothing thrown';
            try {
              /**
               What the roster made of a picture a run stopped while reading.
               */
              const paired = await readImagePair({
                client,
                readOcr: interrupted,
                readerModelIds: READERS,
                bytes: bytesOf({ length: 64, },),
                assetName: 'noticeboard.webp',
                signal: stopping.signal,
                perCallTimeoutMs: HANG_STOP_MS,
                l,
              },);
              escaped = `returned ${paired.kind}`;
            } catch (error) {
              escaped = Error.isError(error,) ? error.name : String(error,);
            }

            expect(escaped,).toBe('AbortError',);
            expect(asked.length,).toBe(0,);
          },
        },),

        it({
          name: 'FORWARDS AN ABORT rather than absorbing it, even where both readings beat the stop. '
            + 'Absorbed, a run told to halt would return two usable readings, the document would '
            + 'settle on them, and the driver would cache a decision the run was told not to make',
          fn: async () => {
            const { client, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: READING,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: AGREEING_READING,
              },
            },);

            /**
             Stop that has already arrived, standing in for one that lands while
             the readings are in flight.
             */
            const stopped = new AbortController();
            stopped.abort();

            /**
             Name of whatever escaped the call, or what it returned instead.
             Recorded rather than asserted inline so a verdict slipping past the
             stop reads as its own value rather than as a missing throw.
             */
            let escaped = 'nothing thrown';
            try {
              /**
               What the roster made of a picture a stopped run asked about.
               */
              const paired = await readImagePair({
                client,
                readOcr: found,
                readerModelIds: READERS,
                bytes: bytesOf({ length: 64, },),
                assetName: 'noticeboard.webp',
                signal: stopped.signal,
                perCallTimeoutMs: HANG_STOP_MS,
                l,
              },);
              escaped = `returned ${paired.kind}`;
            } catch (error) {
              escaped = Error.isError(error,) ? error.name : String(error,);
            }

            expect(escaped,).toBe('AbortError',);
          },
        },),

        it({
          name: 'WRITES EACH READER\'S OWN FAILURE AS THE CALLER STOPS THE READING, on a line naming the picture, the '
            + 'reader and the provider\'s words, in roster order though the later-listed reader failed first, writes '
            + 'none for a reader ending in the caller\'s own reason, and refuses with that reason',
          fn: async () => {
            /**
             Why the caller stopped the reading.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             The caller's steering.
             */
            const steering = new AbortController();
            /**
             What the first-listed reader ends in, once the later-listed reader has ended.
             */
            const firstRefusal = new ProviderHttpError({
              provider: 'synthetic',
              status: 429,
              bodyText: 'the first cat is out of treats',
            },);
            /**
             What the later-listed reader ends in, first.
             */
            const laterRefusal = new ProviderHttpError({
              provider: 'synthetic',
              status: 503,
              bodyText: 'the later cat is napping in the sun',
            },);
            /**
             The later-listed reader's failure at once, the first-listed reader's after it.
             */
            const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
            /**
             Readers in roster order: the first-listed, the one ending in the caller's own reason, and the
             later-listed.
             */
            const readers: readonly RosterModelId[] = [
              SEAT_SYNTHETIC_VISION_WITHHELD,
              SEAT_HYPER_VISION,
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            ];
            /**
             Readers whose exchange has begun, in the order they began.
             */
            const begun: RosterModelId[] = [];
            /**
             Lines the pair logged.
             */
            const lines: string[] = [];
            /**
             The reading, refused once every reader has ended.
             */
            const refusal = await rejectionOf({
              promise: readImagePair({
                client: {
                  chatText: async (request,) => {
                    /**
                     Request as this package's own contract describes it.
                     */
                    const { modelId, signal: readerSignal, } = request as ChatTextRequest;
                    begun.push(modelId,);
                    // THE STOP LANDS ONCE EVERY READER IS ASKED, so it reaches each
                    // exchange in flight and never the gate before them.
                    if (begun.length === readers.length)
                      steering.abort(reason,);
                    await untilAborted({ signal: readerSignal, },);
                    if (modelId === SEAT_SYNTHETIC_VISION_WITHHELD)
                      return await refuseAfterThat(firstRefusal,);
                    if (modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER)
                      return await refuseAtOnce(laterRefusal,);
                    readerSignal.throwIfAborted();
                    throw new Error('unreachable: a reader that waited for its abort was handed a signal that had not '
                      + 'aborted',);
                  },
                  chatJson: async () => {
                    throw new Error('chatJson unused by the reading stage',);
                  },
                  quotas: async () => {
                    throw new Error('quotas unused by the reading stage',);
                  },
                },
                readOcr: found,
                readerModelIds: readers,
                bytes: bytesOf({ length: 64, },),
                assetName: 'noticeboard.webp',
                signal: AbortSignal.any([
                  steering.signal,
                  AbortSignal.timeout(HANG_STOP_MS,),
                ],),
                perCallTimeoutMs: HANG_STOP_MS,
                l: levelCapturingLogger({ lines, },),
              },),
            },);

            expect(refusal,).toBe(reason,);
            expect(begun,).toEqual(readers,);
            expect(lines,).toEqual([
              `warn [readImagePair] noticeboard.webp: ${SEAT_SYNTHETIC_VISION_WITHHELD} failed outright (refused by `
                + 'ProviderHttpError from synthetic with HTTP 429 (the provider said: "the first cat is out of treats")), and the '
                + 'caller stopped the reading',
              `warn [readImagePair] noticeboard.webp: ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} failed outright (refused `
                + 'by ProviderHttpError from synthetic with HTTP 503 (the provider said: "the later cat is napping in the sun")), and '
                + 'the caller stopped the reading',
            ],);
          },
        },),

        it({
          name: 'WRITES A READER\'S OWN FAILURE THAT ENDED BEFORE THE CALLER STOPPED THE READING, which a line written '
            + 'inside the reader as it ended would miss, since no stop stood then, and refuses with the caller\'s '
            + 'reason',
          fn: async () => {
            /**
             Why the caller stopped the reading.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             The caller's steering.
             */
            const steering = new AbortController();
            /**
             Lines the pair logged.
             */
            const lines: string[] = [];
            /**
             The reading, refused once both readers have ended.
             */
            const refusal = await rejectionOf({
              promise: readImagePair({
                client: {
                  chatText: async (request,) => {
                    /**
                     Request as this package's own contract describes it.
                     */
                    const { modelId, signal: readerSignal, } = request as ChatTextRequest;
                    if (modelId === SEAT_SYNTHETIC_VISION_WITHHELD) {
                      throw new ProviderHttpError({
                        provider: 'synthetic',
                        status: 401,
                        bodyText: 'this cat has no collar tag',
                      },);
                    }
                    // A TURN OF THE EVENT LOOP LATER the first reader's failure has
                    // travelled through every await of its reader, since only
                    // microtasks stand between them, and the stop lands after it.
                    await turnOfTheLoop();
                    steering.abort(reason,);
                    readerSignal.throwIfAborted();
                    throw new Error('unreachable: a reader whose caller has just aborted was handed a signal that '
                      + 'had not aborted',);
                  },
                  chatJson: async () => {
                    throw new Error('chatJson unused by the reading stage',);
                  },
                  quotas: async () => {
                    throw new Error('quotas unused by the reading stage',);
                  },
                },
                readOcr: found,
                readerModelIds: READERS,
                bytes: bytesOf({ length: 64, },),
                assetName: 'noticeboard.webp',
                signal: AbortSignal.any([
                  steering.signal,
                  AbortSignal.timeout(HANG_STOP_MS,),
                ],),
                perCallTimeoutMs: HANG_STOP_MS,
                l: levelCapturingLogger({ lines, },),
              },),
            },);

            expect(refusal,).toBe(reason,);
            expect(lines,).toEqual([
              `warn [readImagePair] noticeboard.webp: ${SEAT_SYNTHETIC_VISION_WITHHELD} failed outright (refused by `
                + 'ProviderHttpError from synthetic with HTTP 401 (the provider said: "this cat has no collar tag")), and the caller '
                + 'stopped the reading',
            ],);
          },
        },),

        it({
          name: 'ASKS THE DETERMINISTIC READER NOTHING WHEN THE RUN HAS ALREADY STOPPED, which the '
            + '"FORWARDS AN ABORT rather than absorbing it" case cannot show because it throws either way. The gate shells out to a '
            + 'decoder and to tesseract per picture and consults no signal of its own, so a stopped '
            + 'run that reached it would spend that on every remaining asset and then persist verdicts '
            + 'that beat the stop',
          fn: async () => {
            /**
             Roster that must never be reached, since the gate sits before it.
             */
            const { client, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: READING,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: AGREEING_READING,
              },
            },);

            /**
             How many times the deterministic reader was asked. Counted rather
             than asserted inside the stub so a call that does happen reads as a
             number in the failure rather than as a thrown assertion from a place
             the test does not name.
             */
            let asked = 0;

            /**
             Deterministic reader that records being asked, and would otherwise
             return the verdict two thirds of this corpus reaches.
             */
            async function counting(): Promise<{
              readonly kind: 'no-text';
              readonly characters: number;
            }> {
              asked += 1;
              return {
                kind: 'no-text',
                characters: 0,
              };
            }

            /**
             Stop that arrived before the picture did.
             */
            const stopped = new AbortController();
            stopped.abort();

            /**
             Name of whatever escaped, kept so a verdict slipping past the stop
             reads as its own value.
             */
            let escaped = 'nothing thrown';
            try {
              /**
               What the roster made of a picture a stopped run asked about.
               */
              const paired = await readImagePair({
                client,
                readOcr: counting,
                readerModelIds: READERS,
                bytes: bytesOf({ length: 64, },),
                assetName: 'noticeboard.webp',
                signal: stopped.signal,
                perCallTimeoutMs: HANG_STOP_MS,
                l,
              },);
              escaped = `returned ${paired.kind}`;
            } catch (error) {
              escaped = Error.isError(error,) ? error.name : String(error,);
            }

            expect(escaped,).toBe('AbortError',);
            expect(asked,).toBe(0,);
          },
        },),

        it({
          name: 'ASKS NO MODEL ABOUT A PICTURE WITH NO TEXT, and reports that as its own verdict rather '
            + 'than as a failure. Measured over the corpus, 119 of 191 pictures carry no text at all, '
            + 'so this is the common case and every model call it would have spent is spent on being '
            + 'told there is nothing there',
          fn: async () => {
            const { client, asked, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: READING,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: AGREEING_READING,
              },
            },);

            /**
             What the roster made of a picture the deterministic reader found bare.
             */
            const paired = await readImagePair({
              client,
              readOcr: empty,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            // The gate, which is the whole point: nothing was asked.
            expect(asked.length,).toBe(0,);
            expect(paired,).toStrictEqual({
              kind: 'no-text',
              characters: 3,
            },);
          },
        },),

        it({
          name: 'ASKS THE MODELS ANYWAY WHEN THE DETERMINISTIC READER CANNOT RUN, since a machine '
            + 'without the tools installed must degrade to the behaviour it had before them rather '
            + 'than report every picture as bare. A missing tool and an empty picture are opposite '
            + 'facts and must not produce the same verdict',
          fn: async () => {
            const { client, asked, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: READING,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: AGREEING_READING,
              },
            },);

            /**
             What the roster made of a picture nothing could pre-screen.
             */
            const paired = await readImagePair({
              client,
              readOcr: missing,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(asked.length,).toBe(2,);
            expect(paired.kind,).toBe('corroborated',);
          },
        },),

        it({
          name: 'STATES THE DETERMINISTIC READER\'S COUNT in a textless confirmation where it read text, and NAMES '
            + 'IT UNAVAILABLE WITH ITS REASON where it could not run, recording no count of characters it never read',
          fn: async () => {
            /**
             Client whose two readers both report little text.
             */
            const { client, } = scriptedClient({
              byModel: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 'There is no visible text in this image.',
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'I cannot read any text in this image.',
              },
            },);

            /**
             Lines logged where the deterministic reader read words off the picture.
             */
            const counted: string[] = [];
            await readImagePair({
              client,
              readOcr: found,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l: levelCapturingLogger({ lines: counted, },),
            },);

            /**
             Lines logged where the deterministic reader could not run.
             */
            const unread: string[] = [];

            /**
             What the roster made of the picture the deterministic reader never read.
             */
            const confirmed = await readImagePair({
              client,
              readOcr: missing,
              readerModelIds: READERS,
              bytes: bytesOf({ length: 64, },),
              assetName: 'noticeboard.webp',
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l: levelCapturingLogger({ lines: unread, },),
            },);

            expect(confirmed,).toStrictEqual({
              kind: 'no-text',
              deterministicUnavailable: 'ocr-tool-missing',
              confirmedBy: READERS,
            },);
            expect([
              counted,
              unread,
            ].map(function pairLines(lines,): readonly string[] {
              return lines.filter(function fromPair(line,): boolean {
                return line.includes('[readImagePair]',);
              },);
            },),).toStrictEqual([
              [
                `info [readImagePair] noticeboard.webp: 2 of 2 readers report little or no text (${
                  READERS.join(', ',)
                }), so the picture is confirmed textless past the deterministic reader's 11 characters`,
              ],
              [
                `info [readImagePair] noticeboard.webp: 2 of 2 readers report little or no text (${
                  READERS.join(', ',)
                }), so the picture is confirmed textless where the deterministic reader was unavailable `
                  + '(ocr-tool-missing)',
              ],
            ],);
          },
        },),
      ],
    },),

    describe({
      name: isResumableReading.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RESUMES every verdict that describes the picture: a corroboration, a textless picture, and a '
            + 'stable unavailability, and REFUSES only the unavailability that rests on a reader failing for now',
          fn: async () => {
            /**
             Verdicts and whether each is worth remembering.
             */
            const cases: readonly (readonly [PairedReading, boolean,])[] = [
              [
                {
                  kind: 'corroborated',
                  readings: [],
                  overlap: 1,
                },
                true,
              ],
              [
                {
                  kind: 'no-text',
                  characters: 0,
                },
                true,
              ],
              [
                {
                  kind: 'unavailable',
                  reason: 'no-reader-available',
                  perReader: ['hf:cat/Tabby: model-does-not-read-images',],
                  transient: false,
                },
                true,
              ],
              [
                {
                  kind: 'unavailable',
                  reason: 'one-reader-only',
                  perReader: ['hf:cat/Tabby: reader-failed',],
                  transient: true,
                },
                false,
              ],
            ];
            for (const [reading, resumable,] of cases)
              expect(`${reading.kind}: ${String(isResumableReading({ reading, },),)}`,).toBe(`${reading.kind}: ${String(resumable,)}`,);
          },
        },),
      ],
    },),
  ],
},);
