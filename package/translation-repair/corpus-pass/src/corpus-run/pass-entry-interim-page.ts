import { mkdir, } from 'node:fs/promises';
import { dirname, } from 'node:path';

import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { errorClassOf, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  fixedPagePath,
  frontMatterAuthorityOf,
  type InterimCounts,
  type InterimStop,
  type PageStanding,
  publishInterimPage,
  standingStays,
  UnansweredContestSliceError,
} from '@monochromatic-dev/translation-repair-page-check/ts';
import { passArchiveText, } from './pass-archive.ts';
import type { EntryProgress, } from './pass-entry-interim.ts';

//region Pass entry interim page
// THE PAGE AN ENTRY SHIPS WHEN ITS WORK ENDS, WHATEVER ENDED IT (B1220 to B1223). The owner's rules are "no matter what,
// we must ship" and the recorded answer "Keep archive, ship" (2026-09-27, which keeps the archive at a slice with no valid
// wording). XingZ6023 spent its whole budget and shipped no page because one guard threw. This is the ladder under every
// stop; each rung writes a page, and a rung is taken only when the one above it threw, which the log names with the
// class of the error it threw:
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
   Page as written, which the record's digest is taken over; the page that was composed where none was written.
   */
  readonly pageText: string;

  /**
   Whether the page was written. False where a better page stands (B1242, B1243), when the counts describe the page that
   was composed and not the one on disk.
   */
  readonly written: boolean;
};

/**
 The slicing the ladder composes over, or that none could be cut.
 */
type Slicing =
  | {
    /**
     A slicing exists.
     */
    readonly kind: 'cut';

    /**
     Preparation the page addresses.
     */
    readonly prepared: PreparedDocumentPair;

    /**
     Archive the preparation addresses.
     */
    readonly archiveText: string;

    /**
     Whether the passages the archive lacks ship nothing, because no admission has been read for them (B1244).
     */
    readonly silenceInsertions: boolean;
  }
    | {
    /**
     Not even the deterministic preparation of the two texts could be made.
     */
      readonly kind: 'uncut';
    };

/**
 Slicing and archive the ladder composes over: the entry's own where it held them, otherwise cut from the texts.

 @param progress - what the entry had finished

 @param sourceText - whole original page

 @param targetText - whole archive English as the entry read it

 @param entryId - entry stopped

 @param l - entry logger

 @returns Preparation and archive the page addresses, or that even the deterministic preparation threw

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
): Slicing {
  /**
   What the entry held.
   */
  const { prepared, } = progress;
  if (prepared !== undefined) {
    return {
      kind: 'cut',
      prepared,
      archiveText: progress.archiveText ?? prepared.targetText,
      // The fold of carried passages is what the admission of the source-only passages decides (B1244).
      silenceInsertions: progress.folded !== true,
    };
  }
  try {
    // THE ARCHIVE AS THE PASS READS IT (B1245): the pass normalises the archive before it prepares anything
    // (`passArchiveText`, `pass-prepare.ts`) and decides whose front matter the page carries on that text, so this
    // slicing addresses the same bytes and the same slices as the preparation it stands in for.
    /**
     The archive after the pass's own normalisation.
     */
    const archive = passArchiveText({
      text: targetText,
      l,
    },);
    /**
     The slicing cut from the two texts, which asks no roster.
     */
    const cut = prepareDocumentPair({
      sourceText,
      targetText: archive,
      frontMatterAuthority: frontMatterAuthorityOf({
        entryId,
        sourceText,
        archiveText: archive,
      },),
      attestedDetails: [],
      referenceContext: '',
      sealArchiveOriginal: true,
    },);
    return {
      kind: 'cut',
      prepared: cut,
      archiveText: cut.targetText,
      silenceInsertions: true,
    };
  }
  catch (error) {
    l.error(
      `entry ${entryId}: the two texts could not be sliced for the archive's page (class=${errorClassOf({ error, },)}; `
        + `${refusalText({ error, },)}), so the archive's text is written as it stands`,
    );
    return { kind: 'uncut', };
  }
}

/**
 What an interim page is composed from, in the shape the publisher takes.

 @param projected - comparison both lanes derived, once they returned

 @param contestSlices - what the contest decided per slice, once it returned

 @param consolidated - consolidation records, in the order the slices finished

 @param keepArchiveAt - slices kept at the archive whatever their records say

 @param silenceInsertions - whether the passages the archive lacks ship nothing, for want of an admission

 @returns Stop for `publishInterimPage`

 @example
 ```ts
 const stop = stopOf({ projected, contestSlices, consolidated, keepArchiveAt: [], silenceInsertions: false, },);
 ```
 */
function stopOf(
  {
    projected,
    contestSlices,
    consolidated,
    keepArchiveAt,
    silenceInsertions,
  }: ForeignBorrowed<{
    readonly projected: EntryProgress['projected'];
    readonly contestSlices: EntryProgress['contestSlices'];
    readonly consolidated: EntryProgress['consolidated'];
    readonly keepArchiveAt: readonly number[];
    readonly silenceInsertions: boolean;
  }>,
): InterimStop {
  return {
    ...(silenceInsertions ? { silenceInsertions, } : {}),
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

 @param standing - page already standing where this one goes, which a settled page or an interim page with more settled
 slices keeps (B1242, B1243); none by default

 @param l - entry logger

 @returns Page as written and what each slice of it carries, or the page that was composed and not written where a
 better page stands

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
    standing = { kind: 'none', },
    l,
  }: {
    readonly progress: EntryProgress;
    readonly sourceText: string;
    readonly targetText: string;
    readonly entryId: string;
    readonly publishDir: string;
    readonly standing?: PageStanding;
    readonly l: Logger;
  },
): Promise<ShippedStop> {
  // A SETTLED PAGE STANDS ALREADY, so nothing is composed: no interim page can be better (B1243).
  if (standing.kind === 'settled') {
    return {
      pageText: '',
      settledSlices: 0,
      archiveSlices: 0,
      sourceSlices: 0,
      written: false,
    };
  }
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
  if (sliced.kind === 'uncut')
    return await archiveTextAsPage({
      text: progress.archiveText ?? targetText,
      entryId,
      publishDir,
      standing,
    },);
  /**
   The slicing and the archive it addresses.
   */
  const {
    prepared,
    archiveText,
    silenceInsertions,
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
    standing,
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
  for (const _round of Array.from({ length: prepared.slices
    .length
    + 1, },)) {
    try {
      return await publishInterimPage({
        ...publishing,
        stop: stopOf({
          projected: progress.projected,
          contestSlices: progress.contestSlices,
          consolidated: progress.consolidated,
          keepArchiveAt,
          silenceInsertions,
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
      standing,
    },);
  }
}

/**
 Writes the archive's text as the entry's page, as it stands.

 @param text - archive English

 @param entryId - entry stopped

 @param publishDir - root of the mirrored tree

 @param standing - page already standing where this one goes, which an interim page with settled slices keeps, since
 this page settled none

 @returns Page, counted as carrying no slice, since no slicing addressed it

 @throws What making the directory or writing the page raises

 @example
 ```ts
 const shipped = await archiveTextAsPage({ text, entryId, publishDir, standing, },);
 ```
 */
async function archiveTextAsPage(
  {
    text,
    entryId,
    publishDir,
    standing,
  }: {
    readonly text: string;
    readonly entryId: string;
    readonly publishDir: string;
    readonly standing: PageStanding;
  },
): Promise<ShippedStop> {
  if (standingStays({
    standing,
    settledSlices: 0,
  },)) {
    return {
      pageText: text,
      settledSlices: 0,
      archiveSlices: 0,
      sourceSlices: 0,
      written: false,
    };
  }
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
    written: true,
  };
}

//endregion Pass entry interim page
