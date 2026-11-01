import {
  archiveContributorNameForms,
  type Candidate,
  findDroppedDeclaredNames,
  mergeProducers,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { ArchiveBlockReviewWire, } from './archive-block-review-wire.ts';
import { revisionFootnoteFindings, } from './archive-revision-footnotes.ts';
import { revisionShapeFindings, } from './archive-revision-shape.ts';
import { restoreTypography, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Archive replacement candidates
// THE SLATE THE ARCHIVE BLOCK REVIEW JUDGES SEE: every revision a reviewer
// wrote, collapsed by bytes, in the archive's quote style, holding the
// contributor identities the block declares and the block's own shape.
// Split out of `archive-block-review-stage.ts` when the shape floor (class
// seventy-seven) took the stage over its line budget.

/**
 Collapses byte-identical replacement proposals while preserving authorship,
 withholding every revision whose shape is not the block's own (class
 seventy-seven) or that leaves the page a footnote defect (class eighty-four).

 @param voices - review replies eligible to revise

 @param blockText - archive block under review, verbatim, whose declared
 contributors and shape a revision must keep

 @param blockOffset - where the block starts in that page, which its footnote
 check revises it at

 @param targetText - page the block stands in, as the revisions already
 applied leave it: the quote style a revision is restored to, and the page
 its footnote check reads (ledger B80)

 @returns Distinct replacement candidates, with a finding per revision withheld

 @throws BlockOutsideArchiveError when the page does not carry the block

 @example
 ```ts
 const { candidates, withheld, } = replacementCandidates({ voices, blockText, blockOffset, targetText, },);
 ```
 */
export function replacementCandidates(
  {
    voices,
    blockText,
    blockOffset,
    targetText,
  }: {
    readonly voices: readonly {
      readonly modelId: RosterModelId;
      readonly value: ArchiveBlockReviewWire
    }[];
    readonly blockText: string;
    readonly blockOffset: number;
    readonly targetText: string;
  },
): {
  readonly candidates: readonly Candidate<string>[];
  readonly withheld: readonly string[];
} {
  /**
   Contributor identities current block makes authoritative.
   */
  const contributorNames = archiveContributorNameForms({ text: blockText, });
  /**
   Distinct candidates accumulated in roster order.
   */
  const candidates: Candidate<string>[] = [];
  /**
   Findings for the revisions withheld on shape, in roster order.
   */
  const withheld: string[] = [];
  for (const voice of voices) {
    if (voice.value
      .disposition
      !== 'revise')
      continue;
    if (findDroppedDeclaredNames({
      forms: contributorNames,
      baseText: blockText,
      candidateText: voice.value
        .replacementText,
    },)
      .length
      > 0)
      continue;
    /**
     Replacement as it will ship, the archive's quote style restored (class
     thirty-eight, 2026-09-16: a revised chat block shipped fourteen straight
     apostrophes into a page whose archive has none), so the judges and the
     gate read the shipped bytes rather than text a later pass alters.
     */
    const replacement = restoreTypography({
      replacement: voice.value
        .replacementText,
      replaced: blockText,
      convention: targetText,
    },);
    /**
     Why the revision cannot stand for the block, when its shape differs
     (class seventy-seven: a four-block letter offered for a one-line label).
     */
    const shapeFindings = revisionShapeFindings({
      modelId: voice.modelId,
      blockText,
      replacementText: replacement,
    },);
    if (shapeFindings.length > 0) {
      withheld.push(...shapeFindings,);
      continue;
    }
    /**
     Why the revision cannot stand for the block, when the page it would
     leave carries a footnote defect the page as it stands does not (class
     eighty-four: a translator's note removed while its marker stood in the
     body; ledger B80: the page is the one the revisions already applied
     leave).
     */
    const footnoteFindings = revisionFootnoteFindings({
      modelId: voice.modelId,
      blockText,
      blockOffset,
      replacementText: replacement,
      targetText,
    },);
    if (footnoteFindings.length > 0) {
      withheld.push(...footnoteFindings,);
      continue;
    }
    /**
     Earlier byte-identical correction.
     */
    const existing = candidates.find(function sameReplacement(candidate,): boolean {
      return candidate.value === replacement;
    },);
    if (existing === undefined) {
      candidates.push({
        producer: {
          kind: 'model',
          modelId: voice.modelId,
        },
        value: replacement,
        rendered: (replacement === '')
          ? '[REMOVE BLOCK]'
          : replacement,
      },);
      continue;
    }
    candidates.splice(
      candidates.indexOf(existing,),
      1,
      {
        ...existing,
        producer: mergeProducers({
          left: existing.producer,
          right: {
            kind: 'model',
            modelId: voice.modelId,
          },
        },),
      },
    );
  }
  return {
    candidates,
    withheld,
  };
}

//endregion Archive replacement candidates
