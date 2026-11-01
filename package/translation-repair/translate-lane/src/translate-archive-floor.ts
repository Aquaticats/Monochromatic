import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { describeStandingVerdict, } from '@monochromatic-dev/translation-repair-consolidate/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type DisputedWording,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Translate archive floor
// LEDGER X6 (the whole-package audit, 2026-09-28): the translate lane's
// alignment, quote-loss and declared-name refusals keep the archive wherever
// one exists, and the consolidation judges what a lane keeps by the
// deterministic publication rule. An archive that rule refuses was therefore
// kept only to be refused as a standing, and the replacement the judges chose
// never reached the slate: up to 47 slices in the run logs (a refusal kept the
// archive and the consolidation refused that slice's standing, whichever lane
// the standing came from), 46 of them for a link the original carries.
//
// THE SAME QUESTION THE CONSOLIDATION ASKS, from the same inputs: the slice's
// original, the archive wording it would keep as both candidate and page (the
// eligible stand-in on a disputed slice, as the consolidation's incumbent is),
// the front-matter role, the line-structure rule, the declared name pairs and
// the disputed wordings (`readStandingVerdict`). THE SAME ANSWER TOO: an
// original the grammar cannot read gives no verdict (`unknown`), and the
// consolidation refuses a standing on that as it does on a finding, so no
// refusal here keeps the archive on it either.
//
// REACH NOT MEASURED ON THE AGREEMENT PATH. Where the repair lane also left
// the archive, a kept archive made the lanes agree and the slice shipped
// with no floor at all, so no consolidation line records it; those slices
// now go through the contest and the consolidation whenever the archive fails
// the rule. The TianqiChen666 runs refused nothing in the translate lane.

/**
 Finding a slice carries when the publication rule refused its archive, so
 no translate refusal kept it and the judges' replacement went on. Names no
 slice, since a translate record is stored and re-stamped by position.

 @example
 ```ts
 const findings = [ARCHIVE_INELIGIBLE_FINDING,];
 ```
 */
export const ARCHIVE_INELIGIBLE_FINDING: string = 'translate-archive-ineligible (the archive fails the publication '
  + 'rule, so no refusal kept it and the judges\' replacement went on)';

/**
 The publication rule's answer on the archive a translate refusal would keep.

 @example
 ```ts
 const verdict: ArchiveFloorVerdict = { admitted: false, reason: 'the page carries a link the text drops', };
 ```
 */
export type ArchiveFloorVerdict =
  | {
    /**
     The rule admits the archive, so a refusal may keep it.
     */
    readonly admitted: true;
  }
    | {
    /**
     The rule refuses the archive, so no refusal may keep it.
     */
      readonly admitted: false;

    /**
     The rule's findings as the consolidation words them.
     */
      readonly reason: string;
    };

/**
 Asks the publication rule whether the archive wording a translate refusal
 would keep may stand.

 @param slice - slice being settled

 @param prepared - document preparation, which carries the line-structure
 rule and the declared name pairs

 @param archiveText - wording a refusal would keep: the archive's own, or the
 eligible stand-in on a disputed slice

 @param disputedWordings - wordings a disputed slice refuses, none elsewhere

 @returns Admitted, or refused with the rule's reason

 @example
 ```ts
 const verdict = archiveFloorVerdict({ slice, prepared, archiveText, disputedWordings: [], },);
 ```
 */
export function archiveFloorVerdict(
  {
    slice,
    prepared,
    archiveText,
    disputedWordings,
  }: {
    readonly slice: ChunkPair;
    readonly prepared: PreparedDocumentPair;
    readonly archiveText: string;
    readonly disputedWordings: readonly DisputedWording[];
  },
): ArchiveFloorVerdict {
  /**
   The rule's verdict on the archive standing where it is.
   */
  const validation = validateTranslatedSlice({
    sourceText: slice.source
      .text,
    candidateText: archiveText,
    pageText: archiveText,
    ...((slice.syntax === undefined) ? {} : { syntax: slice.syntax, }),
    lineStructured: prepared.lineStructuredSliceIndices
      .has(slice.target
        .sliceIndex,),
    declared: prepared.declaredNamePairs ?? [],
    disputedWordings,
  },);
  if (validation.kind === 'valid')
    return { admitted: true, };
  return {
    admitted: false,
    reason: describeStandingVerdict({ validation, },),
  };
}

//endregion Translate archive floor
