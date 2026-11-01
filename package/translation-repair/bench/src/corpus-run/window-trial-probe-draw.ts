import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type CorpusPin,
  isMissingCorpusObject,
  type PreparedSheetFacts,
  preparedSheetFactsOf,
  sliceSizesOf,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { classifyDisplacement, } from '@monochromatic-dev/translation-repair-artifact/ts';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import { readEntryInfoRecord, } from '@monochromatic-dev/translation-repair-insertion/ts';
import {
  controlSlices,
  flaggedSlices,
  type TrialSlice,
} from './window-trial-draw.ts';

//region Window trial probe draw
// What one entry contributes to the trial: its two pages read from the pinned
// corpus, prepared as the lanes see them, screened, and the flagged slices
// drawn with their controls. Moved out of `window-trial-probe.ts`, which keeps
// the wiring. The corpus is reached through a reader the caller passes, so a
// case scripts the pages and the command passes `readCorpusFile`.
//
// THE ENTRY'S `info.yml` IS READ TOO, after both pages, because the date of death
// lives only there and the sheets the pass asks tell the judges of it: a trial
// whose preparation never saw it would measure sheets without the tense block.
// A file that is absent or not YAML records no death, as in the pass (`entry-info-record.ts`).

/**
 Controls drawn per entry that contributes any flagged slice.

 Small on purpose. Controls exist to detect a general context-induced
 conservatism, which would show across many entries rather than within one, so
 breadth is worth more here than depth.
 */
const CONTROLS_PER_ENTRY = 1;

/**
 Reader of one page of the pinned corpus, as `readCorpusFile` is.

 @example
 ```ts
 const readPage: CorpusPageReader = readCorpusFile;
 ```
 */
export type CorpusPageReader = (
  input: {
    /**
     Corpus checkout and commit the read resolves against.
     */
    readonly pin: CorpusPin;

    /**
     Path of the page inside the corpus.
     */
    readonly relPath: string;
  },
) => Promise<string>;

/**
 Both sides of one entry, or the fact that it carries only one.

 @example
 ```ts
 const texts: PairTexts = { kind: 'missing', };
 ```
 */
type PairTexts = {
  /**
   Entry carries both sides.
   */
  readonly kind: 'read';

  /**
   Original page.
   */
  readonly source: string;

  /**
   Translated page.
   */
  readonly target: string;
} | {
  /**
   Entry carries one side, which is an ordinary state of this corpus.
   */
  readonly kind: 'missing';
};

/**
 What one entry contributes, with the preparation the picks index into.

 @example
 ```ts
 const drawn: DrawnEntry = { picks: [], sheetFacts: {}, slices: [], };
 ```
 */
export type DrawnEntry = {
  /**
   Slices to buy, flagged then controls.
   */
  readonly picks: readonly TrialSlice[];

  /**
   What the preparation tells every judging sheet of the page as a whole: the
   declared names, whether the entry records a death and the names the
   translation declares for its subject. Empty for an entry that was not read.
   The preparation reads no work titles, corpus names or cited pages, so its
   identity block holds the page's own declarations and the death alone.
   */
  readonly sheetFacts: PreparedSheetFacts;

  /**
   Every prepared slice of the entry, which a pick's index points into.
   */
  readonly slices: readonly ChunkPair[];
};

/**
 Reads both sides of one entry.

 @param entryId - entry to read

 @param pin - corpus checkout and commit to read at

 @param readPage - reader of one corpus page

 @param l - logger the skip of an entry with one side is told to

 @returns Both texts, absent when either side is missing

 @throws Whatever the read threw, when it was not a corpus read failure

 @example
 ```ts
 const texts = await readPairTexts({ entryId, pin, readPage, l, },);
 ```
 */
async function readPairTexts(
  {
    entryId,
    pin,
    readPage,
    l,
  }: {
    readonly entryId: string;
    readonly pin: CorpusPin;
    readonly readPage: CorpusPageReader;
    readonly l: Logger;
  },
): Promise<PairTexts> {
  try {
    return {
      kind: 'read',
      source: await readPage({
        pin,
        relPath: `people/${entryId}/page.md`,
      },),
      target: await readPage({
        pin,
        relPath: `people/${entryId}/page.en.md`,
      },),
    };
  }
  catch (error) {
    if (!isMissingCorpusObject(error,))
      throw error;
    l.info(`${entryId}: skipped, ${String(error,)}`,);
    return { kind: 'missing', };
  }
}

/**
 Slices one entry contributes, flagged plus its controls.

 @param entryId - entry to read

 @param pin - corpus checkout and commit to read at

 @param readPage - reader of one corpus page

 @param l - logger the skip of an entry with one side is told to, and the
 absence or damage of its `info.yml`

 @returns Slices to buy, the preparation they index into and its page facts,
 no slices when the entry cannot be read and no picks when the screen flagged nothing

 @throws Whatever the read threw, when it was not a corpus read failure

 @example
 ```ts
 const drawn = await drawEntry({ entryId, pin, readPage, l, },);
 ```
 */
export async function drawEntry(
  {
    entryId,
    pin,
    readPage,
    l,
  }: {
    readonly entryId: string;
    readonly pin: CorpusPin;
    readonly readPage: CorpusPageReader;
    readonly l: Logger;
  },
): Promise<DrawnEntry> {
  /**
   Both sides, absent when this entry carries only one.
   */
  const texts = await readPairTexts({
    entryId,
    pin,
    readPage,
    l,
  },);
  if (texts.kind === 'missing') {
    return {
      picks: [],
      sheetFacts: {},
      slices: [],
    };
  }

  /**
   The entry's `info.yml` at the pin, which records the date of death.
   */
  const entryInfo = await readEntryInfoRecord({
    pin,
    entryId,
    readFile: readPage,
    l,
  },);

  /**
   Slices exactly as the lanes would see them.
   */
  const prepared = prepareDocumentPair({
    sourceText: texts.source,
    targetText: texts.target,
    ...((entryInfo.kind === 'read') ? { sourceEntryInfo: entryInfo.info, } : {}),
  },);

  /**
   What that preparation tells every judging sheet of the page.
   */
  const sheetFacts = preparedSheetFactsOf({ prepared, },);

  /**
   What the screen makes of their sizes.
   */
  const displacement = classifyDisplacement({
    slices: sliceSizesOf({ slices: prepared.slices, },),
  },);

  /**
   Flagged slices, deduplicated across overlapping candidates.
   */
  const flagged = flaggedSlices({
    entryId,
    displacement,
  },);
  if (flagged.length === 0) {
    return {
      picks: [],
      sheetFacts,
      slices: prepared.slices,
    };
  }

  return {
    // CONTROLS ONLY FROM ENTRIES THAT CONTRIBUTE FLAGGED SLICES, so the two
    // populations share their documents. A control drawn from an entry the
    // screen never flagged would differ in whatever made that entry clean.
    picks: [
      ...flagged,
      ...controlSlices({
        entryId,
        displacement,
        wanted: CONTROLS_PER_ENTRY,
      },),
    ],
    sheetFacts,
    slices: prepared.slices,
  };
}

//endregion Window trial probe draw
