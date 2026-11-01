import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { CarriedInsertion, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  assertCarriedInsertionsRemain,
  CarriedEvidenceLostError,
} from './carried-insertion-completeness.ts';
import {
  assertPageFootnotesIntact,
  PageFootnoteIntegrityError,
  type ShippedOriginal,
} from './page-footnote-integrity.ts';
import type { PublishCheckStep, } from './publish-defects.ts';
import { textOnPage, } from './source-ships.ts';
import {
  type WouldShipSource,
  wouldShipTextPerSlice,
} from './would-ship-text.ts';

//region Publish page guard steps
// THE TWO PAGE-LEVEL GUARDS AS CHECKS OF THE PAGE THE PUBLISHER WRITES (B1181). Both used to be called by the pass before
// the page was composed, and both threw `TranslationRepairInterruptedError`, which the pass files as a stop with no page
// and no artifact: XingZ6023 settled every slice and shipped nothing for an orphan definition. The owner's rules are
// "no matter what, we must ship" and that a page-agreement check ships with the defect reported, so each is a step of
// `publishFixedPage` whose refusal `publishDefects` collects as a reported defect.
//
// APART FROM `publish-fixed.ts` at its line budget, and so that both guards sit where the other checks of the page are
// listed: the footnote check reads the composed page (B1182), the carried-evidence check the page with no original.

/**
 Builds the two page-guard steps for one entry's page.

 @param entryId - entry whose page is weighed

 @param pageText - page as it ships, the original of an untranslated passage under its note included

 @param archiveText - archive the page is weighed against

 @param artifact - settled entry, read for the page with no original in it

 @param slices - preparation defining replacement spans

 @param carried - source-only passages proven rendered in the archive before the lanes

 @param l - logger the lost carried regions are written to, which a defect's message does not hold

 @returns Footnote integrity step, then the carried-evidence step

 @example
 ```ts
 const steps = pageGuardSteps({ entryId, pageText, archiveText, artifact, slices, carried, l, },);
 ```
 */
export function pageGuardSteps(
  {
    entryId,
    pageText,
    archiveText,
    artifact,
    slices,
    carried,
    l,
  }: {
    readonly entryId: string;
    readonly pageText: string;
    readonly archiveText: string;
    readonly artifact: WouldShipSource;
    readonly slices: readonly ChunkPair[];
    readonly carried: readonly CarriedInsertion[];
    readonly l: Logger;
  },
): readonly PublishCheckStep[] {
  return [
    // THE SAME PAGE THE `page-footnotes` STEP READS, against the archive: the defects it introduces over it.
    {
      check: 'page-footnote-integrity',
      refusal: PageFootnoteIntegrityError,
      run: function pageFootnotesIntact(): void {
        assertPageFootnotesIntact({
          entryId,
          pageText,
          targetText: archiveText,
          // THE ORIGINALS THE PAGE SHIPS, so a footnote that sits in one is named by its slice (B1210): the check reads
          // them with the rest of the page, and without this a note of an original reads as one the lanes introduced.
          shippedOriginals: wouldShipTextPerSlice({ artifact, },)
            .flatMap(function originalOf({
              sliceIndex,
              reading,
            },): readonly ShippedOriginal[] {
              return (reading.kind === 'source-ships')
                ? [{
                  sliceIndex,
                  text: textOnPage({ reading, },),
                },]
                : [];
            },),
        },);
      },
    },
    // A CARRIED PASSAGE WHOSE EVIDENCE THE ENGLISH LOST: read off the page with no original in it, and its regions
    // written to the run log, since the defect's message names slices and counts only.
    {
      check: 'carried-evidence',
      refusal: CarriedEvidenceLostError,
      run: function carriedEvidenceRemains(): void {
        try {
          assertCarriedInsertionsRemain({
            entryId,
            artifact,
            slices,
            targetText: archiveText,
            carried,
          },);
        }
        catch (error) {
          if (error instanceof CarriedEvidenceLostError) {
            for (const finding of error.findings)
              l.warn(`entry ${entryId}: ${finding}`,);
          }
          throw error;
        }
      },
    },
  ];
}

//endregion Publish page guard steps
