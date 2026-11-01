//region Publishing the settled entries as a corpus tree

import {
  mkdir,
} from 'node:fs/promises';
import { dirname, } from 'node:path';

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
import type {
  ArchiveOriginalSpan,
  CarriedInsertion,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
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
import { pageGuardSteps, } from './publish-page-guard-steps.ts';
import {
  type FallbackPolicy,
  pageThatParses,
  refuseFallbackPage,
} from './page-grammar-fallback.ts';
import { fixedPagePath, } from './fixed-page-path.ts';
import { placeholderHeadingDefects, } from './publish-placeholder-heading.ts';
import {
  archiveKeptDefects,
  pageWithoutBodyDefects,
  type PublishDefect,
  publishDefects,
  tenseSwapDefects,
} from './publish-defects.ts';

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

 @param carried - source-only passages the lanes proved already rendered in
 the archive, whose evidence the English of the page must still carry; empty
 when none. A region the page lost is reported as the `carried-evidence`
 defect and the page ships (B1181).

 @param readGrammar - the grammar's reading of a page, the package's by default; a case plants another to make a page
 the strict grammar refuses

 @param mayWrite - asked once the page is composed and its checks have run, with the slices the grammar put back to the
 archive, and before anything is written or logged as shipped; false leaves the page standing at its path untouched
 (an interim page that would replace a better page, B1242). Absent means write.

 @param fallback - what a page that fell back to the archive does: `ship`
 writes it with its defect named (a first publish, since a run always ships),
 `refuse` writes nothing, for a republish over a page that stands (B1260)

 @returns Path written, and what the page carries of the source's destinations

 @throws {@link UnansweredContestSliceError} when a slice the contest was
 obliged to decide is named nowhere in it, so no reading can say what ships

 @throws {@link FallbackPageRefusedError} when `fallback` is `refuse` and the
 composed page is refused by the strict grammar, before anything is written

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
    carried = [],
    readGrammar,
    mayWrite,
    fallback = 'ship',
  }: {
    readonly artifact: WouldShipSource;
    readonly slices: readonly ChunkPair[];
    readonly archiveText: string;
    readonly sourceText: string;
    readonly entryId: string;
    readonly publishDir: string;
    readonly l: Logger;
    readonly archiveOriginalSpans?: readonly ArchiveOriginalSpan[];
    readonly carried?: readonly CarriedInsertion[];
    readonly readGrammar?: Parameters<typeof pageThatParses>[0]['readGrammar'];
    readonly mayWrite?: (composed: { readonly keptArchiveAt: readonly number[]; },) => boolean;
    readonly fallback?: FallbackPolicy;
  },
): Promise<{
  readonly path: string;
  readonly pageText: string;
  readonly destinations: DestinationCheck;
  readonly defects: readonly PublishDefect[];
  readonly keptArchiveAt: readonly number[];
  readonly written: boolean;
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
  // A PAGE THE GRAMMAR REFUSES STILL SHIPS (B1207): the archive is kept at the slices that break it and they are named
  // as a defect, where `assertPageParses` used to throw here after the defects were logged and before anything was
  // written (`page-grammar-fallback.ts`).
  /**
   The page as composed, or with the archive kept at the slices that broke it.
   */
  const parsing = pageThatParses({
    entryId,
    pageText: composed.pageText,
    replacements: composed.replacements,
    slices,
    archiveText,
    l,
    ...((readGrammar === undefined) ? {} : { readGrammar, }),
  },);
  // A REPUBLISH OVER A PAGE THAT STANDS KEEPS IT when this page fell back (B1260): nothing has been written yet.
  refuseFallbackPage({
    parsing,
    policy: fallback,
  },);
  /**
   The page.
   */
  const { pageText, } = parsing;
  /**
   What the page was spliced from.
   */
  const { replacements, } = parsing;

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
      // THE TWO PAGE GUARDS THAT STOPPED THE ENTRY BEFORE THE PAGE WAS WRITTEN (B1181): the footnote defects the page
      // introduces over its archive, and a carried passage whose evidence the English lost. They ship as reported
      // defects (`publish-page-guard-steps.ts`).
      ...pageGuardSteps({
        entryId,
        pageText,
        archiveText,
        artifact,
        slices,
        keptArchiveAt: parsing.keptArchiveAt,
        carried,
        l,
      },),
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
    // A PAGE THE GRAMMAR REFUSED is a defect every slice floor passed, since the would-ship reading runs after them; the
    // yulianNyanner page of 2026-09-06 carried a curled JSX string literal that way (page-grammar.ts).
    ...parsing.defects,
  ];
  /**
   Where the page goes, once every check has reported.
   */
  const path = fixedPagePath({
    publishDir,
    entryId,
  },);
  if ((mayWrite !== undefined) && (!mayWrite({ keptArchiveAt: parsing.keptArchiveAt, },))) {
    return {
      path,
      pageText,
      destinations,
      defects,
      keptArchiveAt: parsing.keptArchiveAt,
      written: false,
    };
  }
  for (const {
    check,
    message,
  } of defects)
    l.warn(`publish: shipping with defect ${check}: ${message}`,);
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
    keptArchiveAt: parsing.keptArchiveAt,
    written: true,
  };
}

// The names of the fixed page path moved to `fixed-page-path.ts` at the line budget; every importer keeps its import.
export {
  ENGLISH_PAGE_FILE,
  FIXED_TREE_DIR,
  fixedPagePath,
  PEOPLE_DIR,
} from './fixed-page-path.ts';

//endregion Publishing the settled entries as a corpus tree
