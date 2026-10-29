import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { readDocumentPictures, } from '@monochromatic-dev/translation-repair-translate-lane/ts';
import {
  type ChunkPair,
  type OcrReader,
  ocrReaderOver,
  type PairedReading,
  runInstalledProgram,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  photoReferences,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { PictureReaderSeating, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  assertVisualEvidenceComplete,
  gatherEntryPictures,
  type VisualEvidenceInterruptedError,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';

//region Pass visual evidence

/**
 Test seam supplying reviewed visual evidence without corpus asset I/O.
 */
export type PassVisualEvidenceReader = (args: {
  readonly slices: readonly ChunkPair[];
},) => Promise<ReadonlyMap<string, PairedReading>>;

/**
 Where a picture's bytes and its OCR text come from: the pinned corpus and
 the local OCR tools in a run, stand-ins in a test that drives the readers
 and their re-seat hook (ledger X14). `PassVisualEvidenceReader` replaces the
 whole reading, hook and all; this replaces only what lies outside it.

 @example
 ```ts
 const sources: PassPictureSources = { gather: gatherEntryPictures, readOcr: ocrReaderOver({ runProgram: runInstalledProgram, },), };
 ```
 */
export type PassPictureSources = {
  /**
   Reads the bytes of every picture the slices name.
   */
  readonly gather: typeof gatherEntryPictures;
  /**
   Reads a picture's text before any model is asked about it.
   */
  readonly readOcr: OcrReader;
};

/**
 The run's picture sources: the pinned corpus, and the OCR reader over the
 programs installed on this machine (`dwebp`, `magick` and `tesseract`).

 THE ONE PLACE THE REAL PROGRAM RUNNER IS NAMED. `readImageWithOcr` requires
 its runner of every caller (ledger M70), so nothing beneath this constant
 starts a program a test did not hand it.

 NAMED ONCE, WHERE THE PASS IS ASSEMBLED (`corpus-pass.ts`), and REQUIRED of
 every function between there and the reading (ledger M70, M113): a default
 here was the production value, and a test that left the seam out read the
 pictures from the corpus and ran the real programs.
 */
export const RUN_PICTURE_SOURCES: PassPictureSources = {
  gather: gatherEntryPictures,
  readOcr: ocrReaderOver({ runProgram: runInstalledProgram, },),
};

/**
 Reads and requires complete visual evidence before any lane work.

 @param client - provider client for image readers

 @param slices - prepared entry slices naming assets

 @param pin - corpus commit assets belong to

 @param entryId - corpus entry whose asset directory is read

 @param readerModelIds - vision roster

 @param cache - durable paired reading cache

 @param signal - entry cancellation

 @param perCallTimeoutMs - image exchange deadline

 @param l - entry logger

 @param visualEvidenceReader - optional integration-test evidence seam

 @param priorReadings - completed evidence retained within this pinned entry,
 empty before the entry's first reading

 @param beforePicture - per-picture hook handing each picture the readers it
 runs on (ledger X12); required, as the one caller (`readSeatedPictures`)
 always passes it and its entry map (ledger T8)

 @param pictureSources - where bytes and OCR text come from: `RUN_PICTURE_SOURCES`
 in a run, stand-ins in a test. Required, so that leaving it out is a type
 error rather than a reading from the corpus through the real programs

 @returns Corroborated or reviewed no-text evidence by asset

 @throws {@link VisualEvidenceInterruptedError}
 when any referenced asset lacks usable evidence

 @example
 ```ts
 const readings = await readPassVisualEvidence({ client, slices, pin, entryId, readerModelIds, cache, signal, perCallTimeoutMs, l, priorReadings, beforePicture, pictureSources: RUN_PICTURE_SOURCES, });
 ```
 */
export async function readPassVisualEvidence(
  {
    client,
    slices,
    pin,
    entryId,
    readerModelIds,
    cache,
    signal,
    perCallTimeoutMs,
    l,
    visualEvidenceReader,
    priorReadings,
    beforePicture,
    pictureSources,
  }: {
    readonly client: ModelClient;
    readonly slices: readonly ChunkPair[];
    readonly pin: CorpusPin;
    readonly entryId: string;
    readonly readerModelIds: readonly RosterModelId[];
    readonly cache: SliceCache<PairedReading>;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly visualEvidenceReader?: PassVisualEvidenceReader;
    readonly priorReadings: ReadonlyMap<string, PairedReading>;
    readonly beforePicture: () => Promise<PictureReaderSeating>;
    readonly pictureSources: PassPictureSources;
  },
): Promise<ReadonlyMap<string, PairedReading>> {
  /**
   Whether earlier preparation already completed every reference now in scope.
   */
  const alreadyRead = slices.every(function covered(slice,): boolean {
    return photoReferences({ text: slice.source
      .text, },)
      .every(function complete(reference,): boolean {
      /**
       Evidence from this entry, never a failed reading carried forward as support.
       */
        const prior = priorReadings.get(reference.assetName,);
        return (prior !== undefined) && (prior.kind !== 'unavailable');
      },);
  },);
  if (alreadyRead) {
    l.debug(`${readPassVisualEvidence.name}: every picture already has entry evidence`,);
    return priorReadings;
  }
  /**
   Assets from the picture sources, none when the evidence seam replaces the reading.
   */
  const assets = (visualEvidenceReader === undefined)
    ? await pictureSources.gather({
      pin,
      entryId,
      slices,
      l,
    },)
    : new Map<string, Uint8Array>();
  /**
   Paired visual evidence from production or integration seam.
   */
  const readings = (visualEvidenceReader === undefined)
    ? await readDocumentPictures({
      readOcr: pictureSources.readOcr,
      client,
      slices,
      assets,
      readerModelIds,
      cache,
      priorReadings,
      signal,
      perCallTimeoutMs,
      l,
      beforePicture,
    },)
    : await visualEvidenceReader({ slices, },);
  assertVisualEvidenceComplete({
    slices,
    readings,
  },);
  return readings;
}

//endregion Pass visual evidence
