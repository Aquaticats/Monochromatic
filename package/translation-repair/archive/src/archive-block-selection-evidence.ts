import type { ArchiveBlockReviewWire, } from './archive-block-review-wire.ts';
import type { CompositeContribution, } from './archive-composite-revision.ts';
import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { citedReferenceEvidence, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  declaredNamesEvidence,
  type SelectEvidence,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

/**
 Review reply whose role remains available when preparing selection evidence.
 */
export type ArchiveReviewVoice = {
  readonly modelId: RosterModelId;
  readonly value: ArchiveBlockReviewWire;
};

/**
 Anonymous assessment preserving what its reason was arguing for.
 Candidate number links that opinion to text without exposing authorship.
 */
type ArchiveSelectionAssessment = {
  readonly disposition: ArchiveBlockReviewWire['disposition'];
  readonly proposedAction: 'retain' | 'revise';
  readonly candidate: number;
  readonly finding: string;
};

/**
 An assessment carried onto the composite candidate from the revision it
 reviewed, with the part of the composite it speaks for.
 */
type ArchiveCompositeAssessment = ArchiveSelectionAssessment & {
  /**
   Lines of the composite, counted from one, that the reviewer's revision put
   there and so the only lines the assessment covers: the lines it changed or
   added, and for lines it only removed, the line that now follows the cut
   (ledger B803).
   */
  readonly coversLines: readonly number[];
  /**
   Candidate the reviewer proposed, whose own entry keeps the whole opinion.
   */
  readonly fromCandidate: number;
};

/**
 Label of the prior assessments, saying what a composite's entries are.
 */
const ASSESSMENTS_LABEL = 'PRIOR REVIEW ASSESSMENTS, opinions to check against the documents, not authority or votes '
  + 'deciding your answer';

/**
 What the label adds when a composite candidate stands on the slate. No
 reviewer read the composite as a whole, so its entries are said to be the
 contributors' own, each limited to the lines that contributor changed.
 */
const COMPOSITE_ASSESSMENTS_NOTE = '. A composite candidate was proposed by no reviewer: its entries are its '
  + 'contributors\' own assessments, copied from the candidate named in fromCandidate, and each covers only the '
  + 'lines listed in coversLines (counted from one in the composite), the lines that reviewer put there, or, for '
  + 'lines that reviewer only removed, the line that now follows the cut; no reviewer assessed the '
  + 'composite as a whole';

/**
 Refusal when the composite a caller names is not on the slate it passes, so
 its assessments cannot be filed under a number the judges see.

 @example
 ```ts
 throw new CompositeNotOnSlateError({ slateSize: 4, },);
 ```
 */
export class CompositeNotOnSlateError extends Error {
  /**
   Message names a count only, never archive wording.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds refusal.

   @param slateSize - how many candidates the slate holds
   */
  public constructor({ slateSize, }: { readonly slateSize: number; },) {
    super(
      `the composite candidate is not one of the ${String(slateSize,)} ${
        wordForCount({
          count: slateSize,
          one: 'candidate',
          many: 'candidates',
        },)
      } the judges are shown, so its assessments cannot be numbered`,
    );
    this.name = 'CompositeNotOnSlateError';
  }
}

/**
 Number the judges see a candidate under.

 @param candidates - actual anonymous order shown to selectors

 @param candidate - candidate looked for, by identity

 @returns One-based position in the slate

 @throws CompositeNotOnSlateError when the slate does not hold the candidate
 */
function slateNumberOf(
  {
    candidates,
    candidate,
  }: {
    readonly candidates: readonly Candidate<string>[];
    readonly candidate: Candidate<string>;
  },
): number {
  /**
   Zero-based position, negative one where absent.
   */
  const index = candidates.indexOf(candidate,);
  if (index === (-1))
    throw new CompositeNotOnSlateError({ slateSize: candidates.length, },);
  return index + 1;
}

/**
 Carries each contributor's assessments onto the composite they shaped.
 Nothing is composed: an assessment is copied as its reviewer gave it, the
 disposition and finding untouched, and only marked with the lines it covers.

 @param assessments - assessments of the slate's other candidates, in the order the reviewers were heard

 @param composite - candidate number the composite stands under

 @param contributions - what each revision it carries contributed, in slate order

 @returns Marked copies, grouped by contributor in slate order

 @example
 ```ts
 const carried = onCompositeAssessments({ assessments, composite: 3, contributions, },);
 ```
 */
function onCompositeAssessments(
  {
    assessments,
    composite,
    contributions,
  }: {
    readonly assessments: readonly ArchiveSelectionAssessment[];
    readonly composite: number;
    readonly contributions: readonly CompositeContribution[];
  },
): readonly ArchiveCompositeAssessment[] {
  return contributions.flatMap(function ofContributor({
    candidate,
    changed,
  },): readonly ArchiveCompositeAssessment[] {
    return assessments
      .filter(function onRevision(assessment,): boolean {
        return assessment.candidate === candidate;
      },)
      .map(function marked(assessment,): ArchiveCompositeAssessment {
        return {
          ...assessment,
          candidate: composite,
          coversLines: changed.map(function oneBased(index,): number {
            return index + 1;
          },),
          fromCandidate: candidate,
        };
      },);
  },);
}

/**
 Adds the exact unchanged block as a choice only when revisions are available.
 An actual writer echoing the original already supplies that choice and keeps
 its authorship; reviewers merely favoring retention are not treated as authors.

 @param revisions - admissible distinct revision candidates

 @param blockText - original replacement scope, never surrounding context

 @returns Comparison slate, or the existing empty-slate path

 @example
 ```ts
 const candidates = withArchiveOriginal({ revisions, blockText });
 ```
 */
export function withArchiveOriginal(
  {
    revisions,
    blockText,
  }: {
    readonly revisions: readonly Candidate<string>[];
    readonly blockText: string;
  },
): readonly Candidate<string>[] {
  if (revisions.length === 0)
    return revisions;
  if (revisions.some(function alreadyOriginal(candidate,): boolean {
    return candidate.value === blockText;
  },))
    return revisions;
  return [
    ...revisions,
    {
      producer: {
        kind: 'incumbent',
        matched: [],
      },
      value: blockText,
      rendered: blockText,
    },
  ];
}

/**
 Preserves review action/category and the archive context a selector needs.
 Only eligible retention assessments enter; inadmissible revision proposals
 remain in the stage's audit findings rather than becoming candidate evidence.

 @param sourceText - aligned factual source and corroborated picture support

 @param targetText - archive context, not factual source authority

 @param blockText - exact block under replacement consideration

 @param voices - reviews surviving source-anchor or apparatus verification

 @param candidates - actual anonymous order shown to selectors

 @param composite - the composite candidate on the slate and what each revision
 it carries contributed, in slate order; absent where the slate holds none.
 The composite earns its entries from its contributors', since no reviewer
 proposed it (ledger B716)

 @param priorFindings - earlier opinions, kept separate from current assessments

 @param identityContext - declared names the reviewers read, absent when the
 page declares none (ledger B28)

 @param referenceContext - what the pages the original links say, which the
 first criterion counts as support, absent when it links nowhere

 @returns Evidence whose reasons retain their proposed meaning

 @example
 ```ts
 const evidence = archiveBlockSelectionEvidence({ sourceText, targetText, blockText, voices, candidates, priorFindings: [] });
 ```
 */
export function archiveBlockSelectionEvidence(
  {
    sourceText,
    targetText,
    blockText,
    voices,
    candidates,
    composite,
    priorFindings,
    identityContext,
    referenceContext,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly blockText: string;
    readonly voices: readonly ArchiveReviewVoice[];
    readonly candidates: readonly Candidate<string>[];
    readonly composite?: {
      readonly candidate: Candidate<string>;
      readonly contributions: readonly CompositeContribution[];
    };
    readonly priorFindings: readonly string[];
    readonly identityContext?: string;
    readonly referenceContext?: string;
  },
): readonly SelectEvidence[] {
  /**
   Retain/revise intent is data, never a vote deciding the next stage's answer.
   */
  const assessments = voices.flatMap(function toAssessment(
    voice,
  ): readonly ArchiveSelectionAssessment[] {
    /**
     Actual disposition beside the wording its reviewer proposes.
     */
    const {
      disposition,
      replacementText,
      assessment: finding,
    } = voice.value;
    /**
     Retention points at original text, not an unused replacement field.
     */
    const proposedText = disposition === 'revise' ? replacementText : blockText;
    /**
     A proposal excluded by contributor protection is not a candidate.
     */
    const index = candidates.findIndex(function sameValue(candidate,): boolean {
      return candidate.value === proposedText;
    },);
    if (index === (-1))
      return [];
    return [{
      disposition,
      proposedAction: disposition === 'revise' ? 'revise' : 'retain',
      candidate: index + 1,
      finding,
    },];
  },);
  /**
   The composite's entries: its contributors' assessments, marked with the
   lines each covers, because no reviewer proposed the composite and a
   judge would otherwise weigh the one candidate it cannot check against
   reasons beside the others.
   */
  const compositeAssessments = (composite === undefined)
    ? []
    : onCompositeAssessments({
      assessments,
      composite: slateNumberOf({
        candidates,
        candidate: composite.candidate,
      },),
      contributions: composite.contributions,
    },);
  return [
    {
      label: 'EXPECTED ORIGINAL SECTION',
      text: sourceText,
    },
    {
      label: 'ENGLISH ARCHIVE, context for the current block’s role and placement only; not source authority or text to rewrite',
      text: targetText,
    },
    {
      label: 'CURRENT ARCHIVE BLOCK',
      text: blockText,
    },
    // The names and pages the reviewers read (ledger B28), after the block so
    // a selector judging its claims has what the reviewers judged them by.
    ...declaredNamesEvidence((identityContext === undefined) ? {} : { identityContext, },),
    ...citedReferenceEvidence({ referenceContext: referenceContext ?? '', },),
    {
      label: compositeAssessments.length === 0 ? ASSESSMENTS_LABEL : `${ASSESSMENTS_LABEL}${COMPOSITE_ASSESSMENTS_NOTE}`,
      text: JSON.stringify([
        ...assessments,
        ...compositeAssessments,
      ],),
    },
    ...(priorFindings.length === 0 ? [] : [{
      label: 'EARLIER REVIEW FINDINGS, prior opinions only',
      text: JSON.stringify(priorFindings,),
    },]),
  ];
}
