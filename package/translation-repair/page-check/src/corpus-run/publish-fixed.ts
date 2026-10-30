//region Publishing the settled entries as a corpus tree

import {
  mkdir,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import type {
  Logger,
} from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { codePointLength, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  assertContributorNamesComplete,
  assertHeadingsStayDistinct,
  CollapsedHeadingError,
  ContributorCompletenessError,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { composeSourcePage, } from './source-ships-compose.ts';
import type { WouldShipSource, } from './would-ship-text.ts';
import {
  assertFrontMatterComplete,
  FrontMatterCompletenessError,
} from './front-matter-completeness.ts';
import {
  ArchiveOriginalCompletenessError,
  assertArchiveOriginalComplete,
} from './archive-original-completeness.ts';
import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { refusePageThatDisagrees, } from './published-page-check.ts';
import { PublishedPageDisagreesError, } from './published-page-disagreement.ts';
import {
  assertDestinationsComplete,
  DroppedDestinationError,
  traceDroppedDestinations,
} from './destination-completeness.ts';
import {
  type DestinationCheck,
  droppedDestinations,
} from './dropped-destinations.ts';
import {
  assertPageFootnotesResolve,
  PageFootnoteResolutionError,
} from './page-footnote-resolution.ts';
import { assertPageParses, } from './page-grammar.ts';
import { placeholderHeadingDefects, } from './publish-placeholder-heading.ts';
import {
  archiveKeptDefects,
  pageWithoutBodyDefects,
  type PublishDefect,
  publishDefects,
  tenseSwapDefects,
} from './publish-defects.ts';

/**
 Directory under a runs dir holding the published corpus tree.

 BESIDE `artifacts/` RATHER THAN ANYWHERE ELSE, which is the property that
 matters rather than the name. A runs dir lives outside this repository, so a
 tree written inside one cannot be committed by accident, and the corpus is
 unlicensed: its wording must never reach git. Rooting the tree here also
 means a throwaway run produces a throwaway tree, so verification never writes
 over anything real.
 */
export const FIXED_TREE_DIR = 'fixed';

/**
 Corpus-relative directory every person entry lives under.

 EXPORTED SO `verify-published.ts` CAN LIST WHAT A RUN PUBLISHED rather than
 only compose one path at a time. A verifier that spelled this name itself
 would report an empty tree as a clean one.

 @internal
 */
export const PEOPLE_DIR = 'people';

/**
 Page file name the corpus gives an entry's English rendering.

 EXPORTED BESIDE {@link PEOPLE_DIR} and for the same reason.

 @internal
 */
export const ENGLISH_PAGE_FILE = 'page.en.md';

/**
 Names where one entry's fixed English page is written.

 MIRRORS THE CORPUS LAYOUT EXACTLY, `people/<id>/page.en.md`, because the
 owner asked for the corpus's directory structure replicated rather than for
 a flat pile named by id. A tree shaped this way is a drop-in set: it can be
 diffed against the corpus directory it mirrors, entry by entry, with nothing
 to translate between the two layouts first.

 @param publishDir - root of the tree being written, holding no corpus text itself

 @param entryId - person entry this page belongs to

 @returns Absolute path this entry's page is written to

 @example
 ```ts
 const path = fixedPagePath({ publishDir, entryId: 'lintong', },);
 ```
 */
export function fixedPagePath(
  {
    publishDir,
    entryId,
  }: {
    readonly publishDir: string;
    readonly entryId: string;
  },
): string {
  return join(
    publishDir,
    PEOPLE_DIR,
    entryId,
    ENGLISH_PAGE_FILE,
  );
}

/**
 Writes one settled entry's fixed English page into the mirrored tree.

 CALLED BEFORE THE ARTIFACT IS WRITTEN, and that ordering is the whole
 correctness argument rather than a preference. A pass builds its skip set
 from the artifacts already on disk, so "done" means exactly "an artifact
 exists". Publishing after the artifact would let a crash between the two
 leave an entry marked done forever with no page ever written; publishing
 before it makes "done implies published" true by construction, and a resumed
 pass skips an entry whose page is already there.

 BYTE-FAITHFUL, WITH NOTHING APPLIED HERE. The readings arrive as
 `wouldShipTextPerSlice` gives them: wrapped as the stage that settled them
 left them, in the archive's quote style, and with the page-assembly rows in
 place. Re-wrapping at this site would change
 text both deciders already approved, which is the defect already closed at the
 consolidation. Nothing normalizes the trailing newline either: the archive
 text is preserved byte for byte outside the slices that were replaced.

 @param artifact - settled entry, read for what each slice would carry

 @param slices - pairs this entry was prepared into, carrying the spans to write into

 @param archiveText - whole archive English this entry started from

 @param sourceText - whole source page, read for the destinations it links to

 @param entryId - person entry being published

 @param publishDir - root of the mirrored tree

 @param l - logger, tagged by the caller with this entry

 @param archiveOriginalSpans - spans of the archive the preparation sealed as
 the English original, which the page must carry byte for byte; empty when
 none

 @returns Path written, and what the page carries of the source's destinations

 @throws {@link UnansweredContestSliceError} when a slice the contest was
 obliged to decide is named nowhere in it, so no reading can say what ships

 @example
 ```ts
 const { path, } = await publishFixedPage({ artifact, slices, archiveText, sourceText, entryId, publishDir, l, },);
 ```
 */
export async function publishFixedPage(
  {
    artifact,
    slices,
    archiveText,
    sourceText,
    entryId,
    publishDir,
    l,
    archiveOriginalSpans = [],
  }: {
    readonly artifact: WouldShipSource;
    readonly slices: readonly ChunkPair[];
    readonly archiveText: string;
    readonly sourceText: string;
    readonly entryId: string;
    readonly publishDir: string;
    readonly l: Logger;
    readonly archiveOriginalSpans?: readonly ArchiveOriginalSpan[];
  },
): Promise<{
  readonly path: string;
  readonly pageText: string;
  readonly destinations: DestinationCheck;
  readonly defects: readonly PublishDefect[];
}> {
  /**
   This entry's page as the deciders settled it, with the original of every untranslated passage under its note and the
   steps that make the page parse where the original as recorded does not.
   */
  const composed = composeSourcePage({
    artifact,
    slices,
    archiveText,
    entryId,
    l,
  },);
  /**
   The page.
   */
  const { pageText, } = composed;
  /**
   What the page was spliced from.
   */
  const { replacements, } = composed;

  /**
   What would-ship page carries of source destinations, the archive's
   rendering of one accepted in its place.
   */
  const destinations = droppedDestinations({
    sourceText,
    pageText,
    archiveText,
  },);
  /**
   Where each dropped destination sits among the slices (ledger E1).
   */
  const traces = traceDroppedDestinations({
    dropped: destinations.dropped,
    slices,
    replacements,
  },);
  // THE ADDRESSES GO TO THE RUN LOG, where `destinations-line.ts` says they
  // are; the defect's message carries slice indices only.
  for (const [
    at,
    address,
  ] of destinations
    .dropped
    .entries()) {
    /**
     This address's trace, present since the traces walk the same list.
     */
    const trace = nonNullishOrThrow(traces[at],);
    l.warn(
      `entry ${entryId}: page drops source destination ${address}; original carries it in slices `
        + `[${(trace.sourceSlices).join(', ',)}], archive in [${(trace.archiveSlices).join(', ',)}], `
        + `shipped text in [${(trace.shippedSlices).join(', ',)}]`,
    );
  }
  // THE CONTENT CHECKS REPORT, THEY DO NOT REFUSE (the owner, 2026-09-27): a
  // settled page ships with every failed check named in the run log and on
  // its tally (`publish-defects.ts`). Two different source headings rendered
  // as one is among them, a defect no slice floor can see since a slice holds
  // one heading (yulianNyanner, 2026-09-06).
  /**
   The artifact the page is weighed against: itself, or without its record of the originals the page was shipped
   without, whose missing originals `page-source-unparseable` already names (the lanes' record stays the artifact's
   own).
   */
  const { weighed, } = composed;
  /**
   Failed content checks the page ships with.
   */
  const failedChecks = publishDefects({
    steps: [
      // A PAGE THAT DISAGREES WITH ITS ARTIFACT ships too (owner, 2026-09-27,
      // "Ship with defect reported"): the check is one-sided, so a failure is
      // a defect in assembly rather than in the text. The archive handed in is
      // the text actually spliced rather than the copy the artifact stores, so
      // the weighing is an equality on every entry instead of reporting an
      // older artifact as unweighable.
      {
        check: 'page-agreement',
        refusal: PublishedPageDisagreesError,
        run: function pageAgreement(): void {
          refusePageThatDisagrees({
            artifact: weighed,
            archive: {
              kind: 'stored',
              text: archiveText,
            },
            pageText,
            entryId,
          },);
        },
      },
      {
        check: 'front-matter',
        refusal: FrontMatterCompletenessError,
        run: function frontMatter(): void {
          assertFrontMatterComplete({
            entryId,
            sourceText,
            archiveText,
            pageText,
            slices,
          },);
        },
      },
      {
        check: 'archive-original',
        refusal: ArchiveOriginalCompletenessError,
        run: function archiveOriginal(): void {
          assertArchiveOriginalComplete({
            entryId,
            archiveText,
            pageText,
            spans: archiveOriginalSpans,
          },);
        },
      },
      {
        check: 'contributor-names',
        refusal: ContributorCompletenessError,
        run: function contributorNames(): void {
          assertContributorNamesComplete({
            entryId,
            archiveText,
            pageText,
          },);
        },
      },
      {
        check: 'destinations',
        refusal: DroppedDestinationError,
        run: function destinationsKept(): void {
          assertDestinationsComplete({
            entryId,
            destinations,
            traces,
          },);
        },
      },
      // THE PAGE'S OWN FOOTNOTES, NOT THE ARCHIVE'S (D7, XingZ6020): a reference
      // with no definition, a definition nothing references, a doubled identifier.
      {
        check: 'page-footnotes',
        refusal: PageFootnoteResolutionError,
        run: function pageFootnotes(): void {
          assertPageFootnotesResolve({
            entryId,
            pageText,
          },);
        },
      },
      {
        check: 'headings',
        refusal: CollapsedHeadingError,
        run: function headingsDistinct(): void {
          assertHeadingsStayDistinct({
            entryId,
            sourceText,
            pageText,
          },);
        },
      },
    ],
  },);
  /**
   Every defect the page ships with: the failed content checks, then the
   slices the archive kept because no wording passed the rule (owner,
   2026-09-27, "Keep archive, ship").
   */
  const defects: readonly PublishDefect[] = [
    ...failedChecks,
    ...archiveKeptDefects({
      entryId,
      consolidation: artifact.consolidation,
    },),
    // WHERE THE TENSE DETECTOR DECIDED WHAT SHIPS (owner, 2026-10-29, "Swap only where flagged"): named, so a run
    // shows each swap.
    ...tenseSwapDefects({
      entryId,
      consolidation: artifact.consolidation,
    },),
    // A PAGE WITH NO BODY, whatever the archive held: the one outcome the commitment to ship the original under
    // a note rules out, and a 0-byte page said nothing before this (B1046).
    ...pageWithoutBodyDefects({
      entryId,
      sourceText,
      pageText,
    },),
    // A HEADING OF PLACEHOLDER MARKS ONLY where the original has words, reported and left as it stands (owner,
    // 2026-10-29, "Report it as a defect"; B1052).
    ...placeholderHeadingDefects({
      entryId,
      sourceText,
      pageText,
    },),
    ...composed.defects,
  ];
  for (const {
    check,
    message,
  } of defects)
    l.warn(`publish: shipping with defect ${check}: ${message}`,);

  // A PAGE THE GRAMMAR REFUSES is a defect every slice floor passed, since the
  // would-ship reading runs after them; the yulianNyanner page of 2026-09-06
  // carried a curled JSX string literal that way (page-grammar.ts).
  assertPageParses({
    entryId,
    pageText,
  },);

  /**
   Where the page goes, once every check has reported and the grammar passed.
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
    text: pageText,
  },);

  /**
   Characters the page holds, counted as code points as the published page
   check counts the same page; until 2026-10-06 this was its UTF-16 length,
   which counts a character beyond the first plane twice.
   */
  const pageCharacters = codePointLength({ text: pageText, },);
  l.info(
    `publish: wrote ${String(replacements.length,)} ${
      wordForCount({
        count: replacements.length,
        one: 'slice',
        many: 'slices',
      },)
    } into a page of `
      + `${String(pageCharacters,)} ${
        wordForCount({
          count: pageCharacters,
          one: 'character',
          many: 'characters',
        },)
      }`,
  );
  for (const finding of destinations.findings)
    l.warn(`publish: ${finding}`,);
  return {
    path,
    pageText,
    destinations,
    defects,
  };
}

//endregion Publishing the settled entries as a corpus tree
