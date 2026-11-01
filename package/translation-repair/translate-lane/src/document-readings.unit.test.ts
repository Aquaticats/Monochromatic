/**
 Tests for gathering one document's picture readings before its slices run.

 WHAT THESE PIN is the arithmetic that makes the stage affordable and the
 store that makes a resume honest.

 ONCE PER PICTURE, NOT ONCE PER SLICE. A picture named by one slice is shown to
 that slice and to both its neighbours, so a naive gather would send the same
 asset three times. Over the pinned corpus that difference is most of the work.

 AND ONCE PER RUN, NOT ONCE PER ATTEMPT. A reading is not deterministic: ask
 one model the same question about the same picture twice and the wording
 differs. Those words are in the translate slice key, because a judge shown
 different words can reach a different answer. Without the store, a resumed
 entry would re-read every picture into slightly different words, every key
 naming a picture would change, and every settled slice on a picture-bearing
 document would be re-bought. The store is what makes a resumed key equal to
 the key it resumes.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { PictureReaderSeating, } from '@monochromatic-dev/translation-repair-glossary';
import type {
  ChunkPair,
  PairedReading,
} from '@monochromatic-dev/translation-repair-slice-reading';
import { readDocumentPictures, } from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { sliceOf, } from '@monochromatic-dev/translation-repair-slice-reading/ts/content-slice-of.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Logger the gather writes its progress to.
 */
const l = tagged({ tag: 'document-readings-test', },);

/**
 Deterministic reader that always finds text, so every picture reaches the
 models and these cases keep asking what they were written to ask.

 SUPPLIED RATHER THAN LEFT TO THE REAL ONE, which shells out to `dwebp` and
 `tesseract`. A case that reached those would be asking what tools this
 machine carries, and would answer differently on one that carries none.

 @returns Reading with enough characters to clear the gate

 @example
 ```ts
 await readDocumentPictures({ readOcr: sawText, ... },);
 ```
 */
async function sawText(): Promise<{
  readonly kind: 'read';
  readonly text: string;
}> {
  return {
    kind: 'read',
    text: '走失猫咪 Mittens，虎斑，请电 555 0134。',
  };
}

/**
 Vision sub-roster, which is exactly these two models.
 */
const READERS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
];

/**
 Placeholder the corpus writes for an entry's own directory.

 WRITTEN AS AN ESCAPED TEMPLATE LITERAL, the same way `photo-reference`'s own
 fixtures write it: the corpus text carries a literal dollar-brace, which a
 template literal escapes without an expression and a plain string cannot
 carry without looking like an accident.
 */
const ENTRY_PLACEHOLDER = `\${path}`;

/**
 What one reader transcribed from the picture under test.
 */
const READING = '走失猫咪 Mittens，虎斑，2019 年出生，联系 @mittenspaw。';

/**
 What the other transcribed from it, worded differently where a transcription
 can differ and identically where it cannot.
 */
const AGREEING_READING = '走失猫咪 Mittens，虎斑，2019 年出生，请联系 @mittenspaw。';

/**
 Source text naming one picture, in the corpus's only image construct.

 @param assetName - file name within entry's photos directory

 @returns Passage showing that picture

 @example
 ```ts
 const text = showing({ assetName: 'noticeboard.webp', },);
 ```
 */
function showing({ assetName, }: { readonly assetName: string; },): string {
  return `小猫在窗台上睡觉。\n\n<PhotoScroll photos={[ '${ENTRY_PLACEHOLDER}/photos/${assetName}' ]} />\n`;
}

/**
 Bytes standing in for a picture, whose content no rule here reads.

 @param seed - byte every position carries, so two calls differ by content

 @returns Small buffer of that byte

 @example
 ```ts
 const bytes = bytesOf({ seed: 7, },);
 ```
 */
function bytesOf({ seed, }: { readonly seed: number; },): Uint8Array {
  return new Uint8Array(64,).fill(seed,);
}

/**
 Client answering each reader with its scripted transcription, recording every
 model it was asked.

 @returns Client and models a reading was requested from, in order

 @example
 ```ts
 const { client, asked, } = agreeingClient();
 ```
 */
function agreeingClient(): {
  readonly client: ModelClient;
  readonly asked: RosterModelId[];
} {
  /**
   Models a reading was requested from.
   */
  const asked: RosterModelId[] = [];

  /**
   Transcription each reader returns.
   */
  const scripted: Readonly<Record<string, string>> = {
    [SEAT_SYNTHETIC_VISION_WITHHELD]: READING,
    [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: AGREEING_READING,
  };

  return {
    asked,
    client: {
      chatText: async (request,) => {
        /**
         Model this exchange names.
         */
        const { modelId, } = request;
        asked.push(modelId,);
        return { text: scripted[modelId] ?? '', };
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
 Store recording what it was asked to persist.

 @param resumed - readings an earlier run settled, keyed by reading key

 @returns Cache and what it was told to write

 @example
 ```ts
 const { cache, persisted, } = recordingCache({ resumed: new Map(), },);
 ```
 */
function recordingCache(
  { resumed, }: { readonly resumed: ReadonlyMap<string, PairedReading>; },
): {
  readonly cache: SliceCache<PairedReading>;
  readonly persisted: string[];
} {
  /**
   Keys written during this gather.
   */
  const persisted: string[] = [];

  return {
    persisted,
    cache: {
      resumed,
      persist: async ({ key, },) => {
        persisted.push(key,);
      },
    },
  };
}

/**
 A client whose every reader throws, which is what provider trouble looks
 like from here.

 @returns Client and the models it was asked about

 @example
 ```ts
 const { client, } = failingClient();
 ```
 */
function failingClient(): {
  readonly client: ModelClient;
  readonly asked: RosterModelId[];
} {
  /**
   Models asked, in order.
   */
  const asked: RosterModelId[] = [];
  return {
    asked,
    client: {
      chatText: async (request,) => {
        asked.push(request.modelId,);
        throw new Error('HTTP 503 after every retry',);
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
 A client whose two readers describe different pictures, which is a stable
 verdict about the roster rather than about the evening.

 @returns Client

 @example
 ```ts
 const { client, } = disagreeingClient();
 ```
 */
function disagreeingClient(): { readonly client: ModelClient; } {
  /**
   Readings that share no words.
   */
  const scripted: Readonly<Record<string, string>> = {
    [SEAT_SYNTHETIC_VISION_WITHHELD]: READING,
    [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 'A handwritten note about kibble prices at the market stall.',
  };
  return {
    client: {
      chatText: async (request,) => ({ text: scripted[request.modelId] ?? '', }),
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
 Readers a hook hands back after a dry-out, neither of them the fixture's own.
 */
const RESEATED_READERS: readonly RosterModelId[] = [
  SEAT_HYPER_VISION,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
];

/**
 Reads one picture on the fixture readers, recording the models asked and the
 keys looked up.

 @param beforePicture - hook handing the picture its readers, none for a
 caller with no hook

 @returns Models asked and keys looked up, in order

 @example
 ```ts
 const { asked, looked, } = await readOnePicture({},);
 ```
 */
async function readOnePicture(
  { beforePicture, }: { readonly beforePicture?: () => Promise<PictureReaderSeating>; },
): Promise<{
  readonly asked: readonly RosterModelId[];
  readonly looked: readonly string[];
}> {
  /**
   Client recording every reader it was asked for.
   */
  const agreeing = agreeingClient();
  /**
   Every key the reading looked up.
   */
  const looked: string[] = [];
  /**
   Store holding nothing, recording each lookup.
   */
  const recordedMiss = {
    get: function recordKey(key: string,): undefined {
      looked.push(key,);
      return undefined;
    },
  };
  await readDocumentPictures({
    readOcr: sawText,
    client: agreeing.client,
    slices: [sliceOf({
      text: showing({ assetName: 'noticeboard.webp', },),
      sliceIndex: 0,
    },),],
    assets: new Map([['noticeboard.webp', bytesOf({ seed: 7, },),],],),
    readerModelIds: READERS,
    cache: {
      resumed: recordedMiss as unknown as ReadonlyMap<string, PairedReading>,
      persist: async function keepNothing(): Promise<void> {},
    },
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    ...((beforePicture === undefined) ? {} : { beforePicture, }),
  },);
  return {
    asked: agreeing.asked,
    looked,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readDocumentPictures.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'DOES NOT PERSIST A VERDICT THAT RESTS ON A READER FAILING FOR NOW, so the picture is read '
            + 'again next run rather than served unread until a rebuild retires the cache; the same walk '
            + 'persists a disagreement, which is a stable fact about the roster, as the positive control',
          fn: async () => {
            /**
             Both readers throwing, which used to be cached as permanent.
             */
            const failing = failingClient();

            /**
             Cache under the failing client.
             */
            const transientCache = recordingCache({ resumed: new Map(), },);

            /**
             Readings under the failing client.
             */
            const unread = await readDocumentPictures({
              readOcr: sawText,
              client: failing.client,
              slices: [sliceOf({
                text: showing({ assetName: 'noticeboard.webp', },),
                sliceIndex: 0,
              },),],
              assets: new Map([['noticeboard.webp', bytesOf({ seed: 7, },),],],),
              readerModelIds: READERS,
              cache: transientCache.cache,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);
            expect(failing.asked.length,).toBe(2,);
            expect(unread.get('noticeboard.webp',)?.kind,).toBe('unavailable',);
            expect(transientCache.persisted.length,).toBe(0,);

            /**
             Cache under the disagreeing client.
             */
            const stableCache = recordingCache({ resumed: new Map(), },);
            const disagreed = await readDocumentPictures({
              readOcr: sawText,
              client: disagreeingClient().client,
              slices: [sliceOf({
                text: showing({ assetName: 'noticeboard.webp', },),
                sliceIndex: 0,
              },),],
              assets: new Map([['noticeboard.webp', bytesOf({ seed: 7, },),],],),
              readerModelIds: READERS,
              cache: stableCache.cache,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);
            expect(disagreed.get('noticeboard.webp',)?.kind,).toBe('unavailable',);
            expect(stableCache.persisted.length,).toBe(1,);
          },
        },),
        it({
          name: 'KEEPS READING A DOCUMENT WHOSE DETERMINISTIC READER REJECTS, so the models read the picture '
            + 'without its gate and the reading is remembered, rather than the rejection ending the entry',
          fn: async () => {
            const { client, asked, } = agreeingClient();
            const { cache, persisted, } = recordingCache({ resumed: new Map(), },);

            /**
             Deterministic reader that rejects, as a scratch device with no space does.
             */
            async function rejecting(): Promise<never> {
              throw new Error('no space left on the scratch device',);
            }

            /**
             Readings under a deterministic reader that rejects.
             */
            const readings = await readDocumentPictures({
              readOcr: rejecting,
              client,
              slices: [sliceOf({
                text: showing({ assetName: 'noticeboard.webp', },),
                sliceIndex: 0,
              },),],
              assets: new Map([['noticeboard.webp', bytesOf({ seed: 7, },),],],),
              readerModelIds: READERS,
              cache,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(asked.length,).toBe(2,);
            expect(readings.get('noticeboard.webp',)?.kind,).toBe('corroborated',);
            expect(persisted.length,).toBe(1,);
          },
        },),
        it({
          name: 'READS ONE PICTURE ONCE HOWEVER MANY SLICES NAME IT, which is the arithmetic that '
            + 'makes this affordable: a picture travels to its slice and to both neighbours, so '
            + 'gathering per slice would send the same asset three times',
          fn: async () => {
            const { client, asked, } = agreeingClient();
            const { cache, persisted, } = recordingCache({ resumed: new Map(), },);

            /**
             Three consecutive slices all showing one picture.
             */
            const slices: readonly ChunkPair[] = [
              0,
              1,
              2,
            ].map(function toSlice(sliceIndex,): ChunkPair {
              return sliceOf({
                text: showing({ assetName: 'noticeboard.webp', },),
                sliceIndex,
              },);
            },);

            /**
             What the gather produced.
             */
            const readings = await readDocumentPictures({
              readOcr: sawText,
              client,
              slices,
              assets: new Map([['noticeboard.webp', bytesOf({ seed: 7, },),],],),
              readerModelIds: READERS,
              cache,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(readings.size,).toBe(1,);
            expect(asked.length,).toBe(2,);
            expect(persisted.length,).toBe(1,);
            expect(readings.get('noticeboard.webp',)
              ?.kind,).toBe('corroborated',);
          },
        },),

        it({
          name: 'REUSES entry readings after preparation and reads only newly referenced pictures',
          fn: async () => {
            /** Reader calls expose every additional picture purchase. */
            const { client, asked, } = agreeingClient();
            /** Open cache deliberately does not update its resumed snapshot on persist. */
            const { cache, persisted, } = recordingCache({ resumed: new Map(), },);
            /** Assets available to both preparations. */
            const assets = new Map([
              ['chat.webp', bytesOf({ seed: 1, },),],
              ['letter.webp', bytesOf({ seed: 2, },),],
            ],);
            /** First preparation references only the chat. */
            const firstSlices = [sliceOf({ text: showing({ assetName: 'chat.webp', },), sliceIndex: 0, },),];
            /** Shared reader inputs independent of slice boundaries. */
            const input = {
              client, readOcr: sawText, assets, readerModelIds: READERS, cache,
              signal: new AbortController().signal, perCallTimeoutMs: HANG_STOP_MS, l,
            };
            /** Completed evidence from the archive-review preparation. */
            const priorReadings = await readDocumentPictures({ ...input, slices: firstSlices, },);
            /** Calls and writes bought by the first reading, not a roster-size constant. */
            const before = { calls: asked.length, writes: persisted.length, };
            /** Same assets split differently after archive correction. */
            const reused = await readDocumentPictures({ ...input, slices: firstSlices, priorReadings, },);
            expect(asked.length,).toBe(before.calls,);
            expect(persisted.length,).toBe(before.writes,);
            expect(reused.get('chat.webp',),).toBe(priorReadings.get('chat.webp',),);
            /** A newly exposed source reference shares a slice with one already read. */
            const expanded = await readDocumentPictures({
              ...input,
              slices: [sliceOf({
                text: `${showing({ assetName: 'chat.webp', },)}\n${showing({ assetName: 'letter.webp', },)}`,
                sliceIndex: 0,
              },),],
              priorReadings,
            },);
            expect(asked.slice(before.calls,),).toEqual(asked.slice(0, before.calls,),);
            expect(expanded.get('chat.webp',),).toBe(priorReadings.get('chat.webp',),);
            expect(expanded.get('letter.webp',)?.kind,).toBe('corroborated',);
          },
        },),
        ...(['no-text', 'unavailable',] as const).map(function priorVerdict(kind,) {
          return it({
            name: `REUSES only completed entry evidence when the prior reading is ${kind}`,
            fn: async () => {
              /** Calls distinguish reuse from a fresh reading. */
              const { client, asked, } = agreeingClient();
              /** Empty disk snapshot cannot hide a repeated purchase. */
              const { cache, } = recordingCache({ resumed: new Map(), },);
              /** Textless evidence is complete; unavailable evidence is not. */
              const prior: PairedReading = kind === 'no-text'
                ? { kind, characters: 0, }
                : { kind, reason: 'one-reader-only', transient: true, perReader: [], };
              /** One asset presented again within the same entry. */
              const result = await readDocumentPictures({
                client, readOcr: sawText,
                slices: [sliceOf({ text: showing({ assetName: 'chat.webp', },), sliceIndex: 0, },),],
                assets: new Map([['chat.webp', bytesOf({ seed: 1, },),],]),
                readerModelIds: READERS,
                cache,
                priorReadings: new Map([['chat.webp', prior,],]),
                signal: new AbortController().signal,
                perCallTimeoutMs: HANG_STOP_MS,
                l,
              },);
              expect(asked,).toEqual(kind === 'no-text' ? [] : READERS,);
              expect(result.get('chat.webp',)?.kind,).toBe(kind === 'no-text' ? 'no-text' : 'corroborated',);
            },
          },);
        },),
        it({
          name: 'RESUMES A STORED READING AND SPENDS NO CALL, which is what keeps a resumed slice key '
            + 'equal to the key it resumes. A reading is not deterministic, so re-reading would change '
            + 'the words in the key and re-buy every settled slice on this document',
          fn: async () => {
            const { client, asked, } = agreeingClient();

            /**
             Reading an earlier run settled for this picture.
             */
            const stored: PairedReading = {
              kind: 'corroborated',
              readings: [{
                modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
                text: READING,
              },],
              overlap: 1,
            };

            /**
             First gather, whose only purpose is to learn the key this picture is
             stored under, so the second gather stores it under the same one.
             */
            const learning = recordingCache({ resumed: new Map(), },);
            await readDocumentPictures({
              readOcr: sawText,
              client,
              slices: [sliceOf({
                text: showing({ assetName: 'noticeboard.webp', },),
                sliceIndex: 0,
              },),],
              assets: new Map([['noticeboard.webp', bytesOf({ seed: 7, },),],],),
              readerModelIds: READERS,
              cache: learning.cache,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            /**
             Key that gather wrote under.
             */
            const [key,] = learning.persisted;
            if (key === undefined)
              throw new Error('one key by construction',);

            /**
             Calls spent before the resuming gather, so the `toBe(spentBefore)`
             assertion reads the difference rather than a total.
             */
            const spentBefore = asked.length;

            /**
             Resuming gather, over a store already holding that key.
             */
            const { cache, persisted, } = recordingCache({
              resumed: new Map([[key, stored,],],),
            },);
            const readings = await readDocumentPictures({
              readOcr: sawText,
              client,
              slices: [sliceOf({
                text: showing({ assetName: 'noticeboard.webp', },),
                sliceIndex: 0,
              },),],
              assets: new Map([['noticeboard.webp', bytesOf({ seed: 7, },),],],),
              readerModelIds: READERS,
              cache,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(asked.length,).toBe(spentBefore,);
            expect(persisted.length,).toBe(0,);
            expect(readings.get('noticeboard.webp',),).toEqual(stored,);
          },
        },),

        it({
          name: 'KEYS BY THE PICTURE RATHER THAN BY ITS NAME, so two assets named alike in different '
            + 'entries never share a reading. The bytes are what was asked about',
          fn: async () => {
            const { client, } = agreeingClient();

            /**
             Two gathers of one asset name over different bytes.
             */
            const keys = await Promise.all([
              7,
              9,
            ].map(async function keyFor(seed,): Promise<string> {
              const { cache, persisted, } = recordingCache({ resumed: new Map(), },);
              await readDocumentPictures({
                readOcr: sawText,
                client,
                slices: [sliceOf({
                  text: showing({ assetName: 'noticeboard.webp', },),
                  sliceIndex: 0,
                },),],
                assets: new Map([['noticeboard.webp', bytesOf({ seed, },),],],),
                readerModelIds: READERS,
                cache,
                signal: AbortSignal.timeout(HANG_STOP_MS,),
                perCallTimeoutMs: HANG_STOP_MS,
                l,
              },);
              return persisted[0] ?? '';
            },),);

            expect(keys[0] === keys[1],).toBe(false,);
          },
        },),

        it({
          name: 'SKIPS A PICTURE NOBODY GATHERED BYTES FOR, spending nothing and recording nothing. '
            + 'The slices showing it report it as unread, which is what actually happened, and one '
            + 'unreadable asset must not cost an entry the other fifty slices it has to settle',
          fn: async () => {
            const { client, asked, } = agreeingClient();
            const { cache, persisted, } = recordingCache({ resumed: new Map(), },);

            /**
             Gather over a slice naming a picture the caller could not read.
             */
            const readings = await readDocumentPictures({
              readOcr: sawText,
              client,
              slices: [sliceOf({
                text: showing({ assetName: 'missing.webp', },),
                sliceIndex: 0,
              },),],
              assets: new Map(),
              readerModelIds: READERS,
              cache,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(readings.size,).toBe(0,);
            expect(asked.length,).toBe(0,);
            expect(persisted.length,).toBe(0,);
          },
        },),

        it({
          name: 'ASKS NOTHING OF A DOCUMENT THAT SHOWS NO PICTURE, which is 1181 of the pinned '
            + 'corpus\'s 1260 slices and must cost exactly nothing',
          fn: async () => {
            const { client, asked, } = agreeingClient();
            const { cache, } = recordingCache({ resumed: new Map(), },);

            /**
             Gather over ordinary prose.
             */
            const readings = await readDocumentPictures({
              readOcr: sawText,
              client,
              slices: [sliceOf({
                text: '小猫在窗台上睡觉。\n',
                sliceIndex: 0,
              },),],
              assets: new Map(),
              readerModelIds: READERS,
              cache,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l,
            },);

            expect(readings.size,).toBe(0,);
            expect(asked.length,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: `${readDocumentPictures.name} re-seated under a hold (ledger X12)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEATS A PICTURE ON THE READERS ITS HOOK RETURNS, as every phase that buys item by item seats its '
            + 'items, so readers re-read after a provider dry-out are the ones asked rather than those read before it',
          fn: async () => {
            /**
             Models a caller with no hook asked.
             */
            const control = await readOnePicture({},);
            /**
             Models asked when the hook re-seats the picture elsewhere.
             */
            const moved = await readOnePicture({
              beforePicture: async (): Promise<PictureReaderSeating> => ({ readerModelIds: RESEATED_READERS, }),
            },);
            expect({
              controlOnReaders: (control.asked.length > 0) && control.asked.every(function onReaders(seat,): boolean {
                return READERS.includes(seat,);
              },),
              movedAskedAny: moved.asked.length > 0,
              outsideReseated: moved.asked.filter(function outside(seat,): boolean {
                return !RESEATED_READERS.includes(seat,);
              },),
            },).toEqual({
              controlOnReaders: true,
              movedAskedAny: true,
              outsideReseated: [],
            },);
          },
        },),
        it({
          name: 'KEYS A RE-SEATED PICTURE BY THE READERS IT RUNS ON, so a reading the readers before the dry-out '
            + 'made is never resumed for it, while a hook handing back the starting readers keys it as a caller '
            + 'with no hook does',
          fn: async () => {
            /**
             Keys a caller with no hook looked up.
             */
            const starting = await readOnePicture({},);
            /**
             Keys looked up when the hook re-seats the picture elsewhere.
             */
            const moved = await readOnePicture({
              beforePicture: async (): Promise<PictureReaderSeating> => ({ readerModelIds: RESEATED_READERS, }),
            },);
            /**
             Keys looked up when the hook hands back the starting readers.
             */
            const kept = await readOnePicture({
              beforePicture: async (): Promise<PictureReaderSeating> => ({ readerModelIds: READERS, }),
            },);
            expect({
              lookups: [starting.looked.length, moved.looked.length, kept.looked.length,],
              movedDiffers: moved.looked[0] !== starting.looked[0],
              keptMatches: kept.looked[0] === starting.looked[0],
            },).toEqual({
              lookups: [1, 1, 1,],
              movedDiffers: true,
              keptMatches: true,
            },);
          },
        },),
      ],
    },),
  ],
},);
