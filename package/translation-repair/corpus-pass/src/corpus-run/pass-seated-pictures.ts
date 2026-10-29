import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type {
  ChunkPair,
  PairedReading,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  type PassPictureSources,
  type PassVisualEvidenceReader,
  readPassVisualEvidence,
} from './pass-visual-evidence.ts';
import {
  readJudgeSeats,
  RUN_PER_CALL_TIMEOUT_MS,
  type RunClient,
} from '@monochromatic-dev/translation-repair-run/ts';
import { picturesHooksFor, } from './pass-pictures-reseat.ts';
import type { VisualEvidenceInterruptedError, } from '@monochromatic-dev/translation-repair-slice-cache/ts';

//region Seated picture reading
// WHO READS THE PICTURES IS DECIDED THE WAY THE BENCHES ARE. The reader
// roster used to be the catalog's answer alone (`RUN_READER_MODELS`), which
// is the right roster while every provider is wet and the wrong one the
// moment a model is withheld on the provider that would serve it: the first
// OpenRouter-only pass (keyword233, 2026-09-03 18:15 UTC) kept Kimi-K3 off
// every judge bench and still bought six of its translations, and a picture
// is the dearest call there is. Split out of `pass-entry.ts` at the line
// cap, on the seam between seating the readers and reading.

/**
 Seats the picture readers off the meters, then reads every picture with
 the seated ones.

 ONE READING OF THE METERS FOR THE STAGE, logged as `JUDGE SEATS
 phase=pictures`, taken here rather than borrowed from the lanes' reading
 that follows: the lanes read theirs after this stage has spent its calls,
 and XIEPT2 on 2026-09-03 showed a provider going dry inside one entry.

 @param client - run client, whose dryness view seats the readers and whose
 chat surface reads the pictures

 @param slices - prepared entry slices naming assets

 @param entryId - corpus entry whose asset directory is read

 @param pin - corpus clone and commit the entry's pictures are read at, the
 pass's own

 @param cache - durable paired reading cache

 @param signal - entry cancellation

 @param l - entry logger, which records the seating line and the readings

 @param visualEvidenceReader - optional integration-test evidence seam

 @param priorReadings - completed evidence retained within this pinned entry,
 empty before the entry's first reading; required, since the one caller
 ({@link createPassPictureReader}) always holds the entry's map (ledger T8)

 @param pictureSources - where bytes and OCR text come from:
 `RUN_PICTURE_SOURCES` in a run, stand-ins in a test, which also drive the
 readers' re-seat hook (ledger X14). Required, so that leaving it out is a type
 error rather than a reading from the corpus through the real programs
 (ledger M70, M113)

 @returns Corroborated or reviewed no-text evidence by asset

 @throws {@link VisualEvidenceInterruptedError}
 when any referenced asset lacks usable evidence

 @example
 ```ts
 const readings = await readSeatedPictures({ client, slices, entryId, pin, cache, signal, l, priorReadings, pictureSources, },);
 ```
 */
export async function readSeatedPictures(
  {
    client,
    slices,
    entryId,
    pin,
    cache,
    signal,
    l,
    visualEvidenceReader,
    priorReadings,
    pictureSources,
  }: {
    readonly client: RunClient;
    readonly slices: readonly ChunkPair[];
    readonly entryId: string;
    readonly pin: CorpusPin;
    readonly cache: SliceCache<PairedReading>;
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly visualEvidenceReader?: PassVisualEvidenceReader;
    readonly priorReadings: ReadonlyMap<string, PairedReading>;
    readonly pictureSources: PassPictureSources;
  },
): Promise<ReadonlyMap<string, PairedReading>> {
  /**
   The readers this reading of the meters seats.
   */
  const seats = await readJudgeSeats({
    client,
    phase: 'pictures',
    signal,
    l,
  },);
  return await readPassVisualEvidence({
    client,
    slices,
    pin,
    entryId,
    readerModelIds: seats.readers,
    cache,
    signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    ...((visualEvidenceReader === undefined) ? {} : { visualEvidenceReader, }),
    priorReadings,
    pictureSources,
    // EVERY PICTURE RE-SEATS UNDER A HOLD (ledger X12,
    // `pass-pictures-reseat.ts`), with a memo per call, since each call
    // already reads the seats afresh.
    beforePicture: picturesHooksFor({
      client,
      signal,
      l,
    },)
      .beforePicture,
  },);
}

/**
 Binds one entry's reader so preparation and lanes share completed evidence.

 Disk-cache persistence does not refresh its open snapshot. Retaining readings
 here also preserves the evidence already used by archive review if the reader
 roster changes before lanes start. Newly exposed references still get read.

 @param input - pinned entry, provider, cache and cancellation boundary

 @returns Reader whose evidence belongs to this entry alone

 @example
 ```ts
 const readPictures = createPassPictureReader({ client, entryId, pin, cache, signal, l, pictureSources, });
 ```
 */
export function createPassPictureReader(
  input: Omit<Parameters<typeof readSeatedPictures>[0], 'slices' | 'priorReadings'>,
): PassVisualEvidenceReader {
  /**
   Completed readings retained across this entry's preparation boundaries.
   */
  const priorReadings = new Map<string, PairedReading>();
  /**
   Reads missing entry pictures and retains the completed result.

   @param slices - current prepared source references

   @returns Evidence shared by archive review and later lanes
   */
  async function readPictures(
    { slices, }: Parameters<PassVisualEvidenceReader>[0],
  ): Promise<ReadonlyMap<string, PairedReading>> {
    /**
     Complete evidence, checked before anything enters the retained map.
     */
    const readings = await readSeatedPictures({
      ...input,
      slices,
      priorReadings,
    },);
    for (const [name, reading,] of readings)
      priorReadings.set(
        name,
        reading,
      );
    return priorReadings;
  }
  return readPictures;
}

//endregion Seated picture reading
