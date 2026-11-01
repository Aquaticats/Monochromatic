import { mkdir, } from 'node:fs/promises';
import { dirname, } from 'node:path';

import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { errorClassOf, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  fixedPagePath,
  type InterimCounts,
  type InterimStop,
  publishInterimPage,
  UnansweredContestSliceError,
} from '@monochromatic-dev/translation-repair-page-check/ts';
import type { EntryProgress, } from './pass-entry-interim.ts';

//region Pass entry interim page
// THE PAGE AN ENTRY SHIPS WHEN ITS WORK ENDS, WHATEVER ENDED IT (B1220 to B1223). The owner's rules are "no matter what,
// we must ship" and "No valid wording: keep the archive at that slice and ship", and XingZ6023 spent its whole budget and
// shipped no page because one guard threw. This is the ladder under every stop; each rung writes a page, and a rung is
// taken only when the one above it threw, which the log names with the class of the error it threw:
//
// 1. THE PAGE COMPOSED FROM WHAT THE ENTRY SETTLED, the interim page of B1006. If the reading of what ships refuses one
//    slice (`UnansweredContestSliceError`, a slice the contest was bound to decide and no row names), the archive is kept
//    at that slice and the page is composed again, once per slice it names.
// 2. THE ARCHIVE'S OWN PAGE, composed with nothing settled, when the composition itself throws (`SliceSpliceError`: a
//    fault of the data or the code, not of the content). It cannot throw those, since it reads no lane row.
// 3. THE ARCHIVE'S TEXT AS IT WAS READ, written as it stands, when even the deterministic slicing of the two texts
//    cannot be made. It is the page the corpus already had, and no check reads it.
//
// BEFORE THE SLICING EXISTS the entry holds no slices, so they are cut here by the deterministic preparation of the two
// texts the entry was read from (the roster pairs blocks and reviews the archive in the pass's own preparation; this
// takes neither, and its slices only address the archive). After the preparation returned, the archive the entry
// reads from there on is the reviewed one, which the progress holds.

/**
 What the ladder shipped.
 */
export type ShippedStop = InterimCounts & {
  /**
   Page as written, which the record's digest is taken over.
   */
  readonly pageText: string;
};

/**
 Slicing and archive the ladder composes over: the entry's own where it held them, otherwise cut from the texts.

 @param progress - what the entry had finished

 @param sourceText - whole original page

 @param targetText - whole archive English as the entry read it

 @param entryId - entry stopped

 @param l - entry logger

 @returns Preparation and archive the page addresses, none where even the deterministic preparation threw

 @example
 ```ts
 const sliced = slicingOf({ progress, sourceText, targetText, entryId, l, },);
 ```
 */
function slicingOf(
  {
    progress,
    sourceText,
    targetText,
    entryId,
    l,
  }: {
    readonly progress: EntryProgress;
    readonly sourceText: string;
    readonly targetText: string;
    readonly entryId: string;
    readonly l: Logger;
  },
): { readonly prepared: PreparedDocumentPair; readonly archiveText: string; } | undefined {
  /**
   What the entry held.
   */
  const { prepared, } = progress;
  if (prepared !== undefined) {
    return {
      prepared,
      archiveText: progress.archiveText ?? prepared.targetText,
    };
  }
  try {
    /**
     The slicing cut from the two texts, which asks no roster.
     */
    const cut = prepareDocumentPair({
      sourceText,
      targetText,
      sealArchiveOriginal: true,
    },);
    return {
      prepared: cut,
      archiveText: cut.targetText,
    };
  }
  catch (error) {
    l.error(
      `entry ${entryId}: the two texts could not be sliced for the archive's page (class=${errorClassOf({ error, },)}; `
        + `${refusalText({ error, },)}), so the archive's text is written as it stands`,
    );
    return undefined;
  }
}

/**
 What an interim page is composed from, in the shape the publisher takes.

 @param progress - what the entry had finished

 @param keepArchiveAt - slices kept at the archive whatever their records say

 @returns Stop for `publishInterimPage`

 @example
 ```ts
 const stop = stopOf({ progress, keepArchiveAt: [], },);
 ```
 */
function stopOf(
  {
    progress,
    keepArchiveAt,
  }: {
    readonly progress: EntryProgress;
    readonly keepArchiveAt: readonly number[];
  },
): InterimStop {
  /**
   What the entry settled before the stop.
   */
  const {
    projected,
    contestSlices,
    consolidated,
  } = progress;
  return {
    consolidateSlices: [...consolidated,],
    ...((projected === undefined) ? {} : { comparison: projected.comparison, }),
    ...((contestSlices === undefined)
      ? {}
      : {
        laneSelection: {
          kind: 'contested',
          slices: contestSlices,
        },
      }),
    ...((keepArchiveAt.length === 0) ? {} : { keepArchiveAt, }),
  };
}

/**
 Ships the page of a stopped entry down the ladder, naming each rung that threw.

 @param progress - what the entry had finished

 @param sourceText - whole original page, read for the destinations the page links to

 @param targetText - whole archive English as the entry read it

 @param entryId - entry stopped

 @param publishDir - root of the mirrored tree

 @param l - entry logger

 @returns Page as written and what each slice of it carries

 @throws What writing the archive's text raises, where no rung above it wrote a page

 @example
 ```ts
 const shipped = await shipPageOfStop({ progress, sourceText, targetText, entryId, publishDir, l, },);
 ```
 */
export async function shipPageOfStop(
  {
    progress,
    sourceText,
    targetText,
    entryId,
    publishDir,
    l,
  }: {
    readonly progress: EntryProgress;
    readonly sourceText: string;
    readonly targetText: string;
    readonly entryId: string;
    readonly publishDir: string;
    readonly l: Logger;
  },
): Promise<ShippedStop> {
  /**
   Slicing and archive the page addresses, absent where none could be cut.
   */
  const sliced = slicingOf({
    progress,
    sourceText,
    targetText,
    entryId,
    l,
  },);
  if (sliced === undefined)
    return await archiveTextAsPage({
      text: progress.archiveText ?? targetText,
      entryId,
      publishDir,
    },);
  const {
    prepared,
    archiveText,
  } = sliced;
  /**
   Everything a rung is handed but its stop.
   */
  const publishing = {
    slices: prepared.slices,
    archiveText,
    sourceText,
    entryId,
    publishDir,
    l,
    ...((prepared.archiveOriginalSpans === undefined)
      ? {}
      : { archiveOriginalSpans: prepared.archiveOriginalSpans, }),
  };
  /**
   Slices kept at the archive because the reading of what ships refused them.
   */
  const keepArchiveAt: number[] = [];
  /* oxlint-disable no-await-in-loop -- each page is composed after the slice the last refused is kept at the archive */
  for (const _round of Array.from({ length: prepared.slices.length + 1, },)) {
    try {
      return await publishInterimPage({
        ...publishing,
        stop: stopOf({
          progress,
          keepArchiveAt,
        },),
      },);
    }
    catch (error) {
      if ((error instanceof UnansweredContestSliceError) && (!keepArchiveAt.includes(error.sliceIndex,))) {
        l.error(
          `entry ${entryId}: class=${errorClassOf({ error, },)}: ${refusalText({ error, },)}; the archive is kept at `
            + `slice ${String(error.sliceIndex,)} and the page is composed again. This is a contradiction in what the `
            + 'entry settled, which a run cannot reach by design, so it is a defect of the code',
        );
        keepArchiveAt.push(error.sliceIndex,);
        continue;
      }
      l.error(
        `entry ${entryId}: the page could not be composed from what the entry settled (class=${
          errorClassOf({ error, },)
        }; ${refusalText({ error, },)}), so the archive's own page ships. This is a fault of the data or of the `
          + 'code, not of the content',
      );
      break;
    }
  }
  /* oxlint-enable no-await-in-loop */
  try {
    return await publishInterimPage({
      ...publishing,
      stop: { consolidateSlices: [], },
    },);
  }
  catch (error) {
    l.error(
      `entry ${entryId}: the archive's own page could not be composed (class=${errorClassOf({ error, },)}; ${
        refusalText({ error, },)
      }), so the archive's text is written as it stands`,
    );
    return await archiveTextAsPage({
      text: archiveText,
      entryId,
      publishDir,
    },);
  }
}

/**
 Writes the archive's text as the entry's page, as it stands.

 @param text - archive English

 @param entryId - entry stopped

 @param publishDir - root of the mirrored tree

 @returns The page, counted as carrying no slice, since no slicing addressed it

 @throws What making the directory or writing the page raises

 @example
 ```ts
 const shipped = await archiveTextAsPage({ text, entryId, publishDir, },);
 ```
 */
async function archiveTextAsPage(
  {
    text,
    entryId,
    publishDir,
  }: {
    readonly text: string;
    readonly entryId: string;
    readonly publishDir: string;
  },
): Promise<ShippedStop> {
  /**
   Where the page goes.
   */
  const path = fixedPagePath({
    publishDir,
    entryId,
  },);
  await mkdir(
    dirname(path,),
    { recursive: true, },
  );
  await writeFileAtomic({
    path,
    text,
  },);
  return {
    pageText: text,
    settledSlices: 0,
    archiveSlices: 0,
    sourceSlices: 0,
  };
}

//endregion Pass entry interim page
