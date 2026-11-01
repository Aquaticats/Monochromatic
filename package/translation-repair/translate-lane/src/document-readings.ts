import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type ChunkPair,
  imageReadingKey,
  isResumableReading,
  type OcrReader,
  type PairedReading,
  readImagePair,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  photoReferences,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { PictureReaderSeating, } from '@monochromatic-dev/translation-repair-glossary/ts';

import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Document readings
// EVERY PICTURE IN ONE DOCUMENT, READ ONCE, before any of its slices is
// translated.
//
// ONCE PER PICTURE, NOT ONCE PER SLICE. A picture named by a slice is shown to
// that slice and to both its neighbours, so reading per slice would send the
// same asset three times. Measured over the pinned corpus, 79 slices name 192
// references resolving to 191 distinct assets, so the difference is most of the
// work.
//
// BEFORE THE SLICES RATHER THAN DURING THEM, because a reading is EVIDENCE that
// has to be in the cache key of every slice that sees it. Gathering them first
// means a slice's key is a function of readings that already exist, rather than
// of readings a later slice might still produce.
//
// BYTES ARE HANDED IN, never read from disk here. `readDocumentPictures` is a function
// of its inputs and its injected client, which is what makes it testable without
// a corpus and what keeps a corpus layout out of the lane. The corpus-run layer
// knows the entry id and the photos directory; this knows neither.

/**
 Reads every picture one document's slices name, through the resumable store.

 @param client - injected model client

 @param slices - prepared slice pairs of one entry

 @param assets - picture bytes per asset name, gathered by the caller

 @param readerModelIds - vision sub-roster

 @param cache - cross-run store, so a resumed run re-reads no picture

 @param priorReadings - completed evidence from this same pinned entry before re-preparation

 @param signal - entry abort honoured by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - driver logger

 @param beforePicture - awaited before each picture the readers are asked about,
 handing it the readers it runs on; none keeps the given ones

 @returns What reading produced per asset name, including refusals

 @throws {@link DOMException} on the caller's abort, and whatever the reading
 cache raises, which is a disk failure rather than an unreadable picture. A
 READER THAT FAILS IS NOT AMONG THESE: `readImagePair` contains it as an
 unavailable reading, because nothing downstream requires a reading and an
 entry must not be lost to one

 @example
 ```ts
 const readings = await readDocumentPictures({ client, readOcr, slices, assets, readerModelIds, cache, signal, perCallTimeoutMs, l, },);
 ```
 */
export async function readDocumentPictures(
  {
    client,
    readOcr,
    slices,
    assets,
    readerModelIds,
    cache,
    priorReadings = new Map(),
    signal,
    perCallTimeoutMs,
    l,
    beforePicture,
  }: {
    readonly client: ModelClient;
    readonly readOcr: OcrReader;
    readonly slices: readonly ChunkPair[];
    readonly assets: ReadonlyMap<string, Uint8Array>;
    readonly readerModelIds: readonly RosterModelId[];
    readonly cache: SliceCache<PairedReading>;
    readonly priorReadings?: ReadonlyMap<string, PairedReading>;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly beforePicture?: () => Promise<PictureReaderSeating>;
  },
): Promise<ReadonlyMap<string, PairedReading>> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readDocumentPictures.name,
    l,
  },);

  /**
   Every picture any slice names on its source side, each once, in document
   order.
   */
  const named = new Set<string>();
  for (const slice of slices) {
    for (const reference of photoReferences({
      text: slice.source
        .text,
    },))
      named.add(reference.assetName,);
  }

  /**
   What each picture produced.
   */
  const readings = new Map<string, PairedReading>();
  if (named.size === 0)
    return readings;

  rl.info(`reading ${String(named.size,)} ${
    wordForCount({
      count: named.size,
      one: 'picture',
      many: 'pictures',
    },)
  } for this document`,);

  for (const assetName of named) {
    /**
     Evidence already completed in this entry, independent of changed slice boundaries.
     */
    const prior = priorReadings.get(assetName,);
    if ((prior !== undefined) && (prior.kind !== 'unavailable')) {
      rl.info(`${assetName}: retained entry evidence, ${prior.kind}`,);
      readings.set(
        assetName,
        prior,
      );
      continue;
    }
    /**
     Picture itself, absent when the caller gathered no bytes for it.
     */
    const bytes = assets.get(assetName,);
    if (bytes === undefined) {
      // NOT AN ERROR HERE. The caller enumerates the same references this does,
      // so a gap means the file was unreadable where the corpus was read, and
      // that is the caller's finding to make. A slice shown no reading for a
      // picture reports it as unread, which is exactly what happened.
      rl.warn(`${assetName}: no bytes were gathered, so it was not read`,);
      continue;
    }

    /* oxlint-disable no-await-in-loop -- current cache protocol settles each picture before advancing so restart has one ordered frontier; replacement DAG must preserve restart while exposing independent assets */
    // A PICTURE IS READ BY THE READERS ITS HOOK HANDS OVER (ledger X12,
    // 2026-09-28), as every phase that buys item by item seats its items: the
    // readings read their bench once, so a dry-out inside them left every
    // later picture on the readers read before it.
    /**
     Seating the hook hands this picture: readers read under a hold, or none,
     which keeps those the readings started on.
     */
    const seating: PictureReaderSeating = (beforePicture === undefined) ? {} : await beforePicture();
    /**
     Readers this picture is read by.
     */
    const pictureReaders = seating.readerModelIds ?? readerModelIds;
    /**
     Where this reading is stored, derived from what it was asked rather than
     from what came back: readings are not deterministic, and a key over their
     text would miss on every run. The readers are in it, so a re-seated
     picture is keyed by those that read it.
     */
    const key = imageReadingKey({
      bytes,
      readerModelIds: pictureReaders,
    },);

    /**
     Reading settled on an earlier run, absent on a first pass.
     */
    const resumed = cache.resumed
      .get(key,);
    if (resumed !== undefined) {
      rl.info(`${assetName}: resumed, ${resumed.kind}`,);
      readings.set(
        assetName,
        resumed,
      );
      continue;
    }

    /**
     What the roster made of it now.
     */
    const paired = await readImagePair({
      readOcr,
      client,
      readerModelIds: pictureReaders,
      bytes,
      assetName,
      signal,
      perCallTimeoutMs,
      l: rl,
    },);
    readings.set(
      assetName,
      paired,
    );

    // A TRANSIENT VERDICT IS NOT REMEMBERED. A reader that timed out once used
    // to leave this picture unread on every later run, served from the cache
    // with only `resumed, unavailable` in the log, until a rebuild retired the
    // cache; the provider recovering never did. That is provider trouble
    // degrading the pipeline durably, which the standing rule forbids.
    if (!isResumableReading({ reading: paired, },)) {
      rl.warn(`${assetName}: unavailable for a transient reason, so it is not cached and is read again next run`,);
      continue;
    }
    await cache.persist({
      key,
      serialized: JSON.stringify(paired,),
    },);
    /* oxlint-enable no-await-in-loop */
  }

  return readings;
}

//endregion Document readings
