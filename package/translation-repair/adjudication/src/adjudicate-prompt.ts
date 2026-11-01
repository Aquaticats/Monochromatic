import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import {
  APPARATUS_KINDS,
  type ClaimCluster,
  ISSUE_SEVERITIES,
  NARRATIVE_DETAIL_IS_NOT_APPARATUS,
  type SpanAnchor,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  ACCURACY_CATEGORY_SCOPE,
  ADJUDICATION_CATEGORY_CHECK,
  OBLIGATORY_DIFFERENCES,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  PANEL_NEUTRAL_REGRADE_RULE,
  PANEL_NOTHING_WRONG_RULE,
  SEVERITY_SCALE,
} from './severity-scale.ts';
import {
  flattenSpace,
  HOUSE_POLICY_BLOCK,
  indentContinuation,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { withoutTranslatorsNotes, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';
import { citedReferenceBlockText, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  DECLARED_IDENTITY_RULES,
  declaredNamesBlock,
} from './declared-identity-rule.ts';
import { REPAIR_EVIDENCE_ROLE, } from './repair-evidence-role.ts';

//region Adjudication prompt
// One prompt per panelist per chunk, covering every cluster in it. Claims are
// numbered so ballots reference small integers instead of hex ids, which
// unreliable models mistype. The prompt never names proposers: adjudication
// is provenance-blind by design.


/**
 What the neighbouring blocks are for, stated inside the panel sheet.

 THE PANEL NEEDS THIS MORE THAN THE CRITIC DOES, because it decides claims
 rather than raises them. A critic that can see next door may raise a claim
 saying a passage belongs to a neighbouring section; a panel that CANNOT see
 next door has no way to check that and must reject it as unfounded. Widening
 the critic without widening the panel would therefore produce exactly the
 claims the panel is guaranteed to throw away.

 DEMANDING A QUOTE WAS TRIED AND MEASURED WORSE, 2026-08-20, and reverted in
 `2c93f49bb`. The stronger wording required a panelist voting unsupported on
 relocation grounds to quote the nearby wording holding the content, on the
 reasoning that the rule otherwise invites the hypothesis without demanding
 evidence. That reasoning is sound and the result contradicted it:

 ```text
 supported votes            lintong 72% -> 65%   saurikissa 52% -> 51%
 distinctive words kept     saurikissa repair lane 89.4% -> 83.8%
 ```

 It lowered the panel support it was added to raise, and lowered retention of
 the archive's specifics, which is the damage the window exists to prevent.
 `doc/audit/the-damage-no-instrument-was-catching.md` carries the measurement.
 Do not re-add it without evidence that beats the plain wording.
 */
const NEARBY_RULE = REPAIR_EVIDENCE_ROLE;

/**
 System instructions shared by every panelist call.
 */
const ADJUDICATION_SYSTEM_PROMPT = `You are an impartial bilingual adjudicator.
Reviewers reported the numbered claims below against the TRANSLATION of the ORIGINAL document.
You did not write these claims. Judge each claim strictly on the document evidence.

For EVERY claim, write its reason first, then cast exactly one vote. The reason is one or two sentences naming the document evidence that decides the claim; the vote follows from it:
- supported: the documents confirm the defect exists as claimed
- unsupported: the documents contradict the claim, or the quoted evidence does not show the claimed defect
- ambiguous: the documents genuinely permit both readings; a human must decide
- source-defect: the ORIGINAL itself is wrong at the claimed spot (typo, corruption), so the translation must not be "corrected" toward it
- abstain: you cannot judge this claim at all

${ACCURACY_CATEGORY_SCOPE}

${ADJUDICATION_CATEGORY_CHECK}

Before voting supported, check the claim against its OWN quoted evidence, which is the cheapest way a claim fails. A claim that something is missing is contradicted when the quoted TRANSLATION text already carries it, even in different words; a claim that something was added is contradicted when the quoted ORIGINAL text already carries it. Vote unsupported whenever the claim's own quotes refute it, however confidently it is worded.

${HOUSE_POLICY_BLOCK}

Translation policy, which governs what may count as a defect at all. A claim that survives its own quotes can still be unsupported because it asks for the wrong thing:
- Non-literalness is not a defect. A rendering whose wording, sentence boundaries, or clause order differ from the ORIGINAL is correct when it reads naturally and carries the same feeling. Vote unsupported on any claim whose whole case is that a more literal rendering exists.
- A merely possible alternative gloss is not a defect. When the claim argues that a word "could be" or "should be" some other rendering, and the shipped rendering is defensible in context, vote unsupported. The question is whether the translation is wrong, not whether another choice was available.
- Do not apply prose standards to verse. When the span is poetry, lyrics, or deliberately stylized lines, compression, inversion, unusual punctuation, and non-literal imagery are the form working as intended, not defects.
- In-group vocabulary rendered by its conventional meaning is correct even when a literal reading of the characters says otherwise; never vote supported on the strength of a literal reading alone.
- ${PANEL_NOTHING_WRONG_RULE}
- The ORIGINAL is not golden. A TRANSLATION that is clearer, better punctuated, or more explicit than the ORIGINAL is doing its job, and that alone is never a defect.
- Accurate page apparatus a translator ADDED is not an addition: ${APPARATUS_KINDS}. It is correct information a reader benefits from. Vote unsupported on a claim whose whole case is that the ORIGINAL does not carry it; vote supported only when the added detail is WRONG. ${NARRATIVE_DETAIL_IS_NOT_APPARATUS}

${OBLIGATORY_DIFFERENCES}
- Vote unsupported on a claim whose whole case is one of these differences; a claim that the supplied or rendered choice is the WRONG one is judged on the reading the ORIGINAL supports.

${DECLARED_IDENTITY_RULES}
- Vote unsupported on a claim whose whole case is a rendering the block makes correct.

${SEVERITY_SCALE}

Optionally re-grade a supported claim's severity on that scale: one of ${ISSUE_SEVERITIES.join(', ',)}. ${PANEL_NEUTRAL_REGRADE_RULE}
For every GROUP holding more than one claim, also state whether its claims describe one single defect (sameDefect true) or genuinely distinct defects (sameDefect false).

Reply with ONLY a JSON object of shape
{"groups": [{"group": 1, "sameDefect": true}], "verdicts": [{"claim": 1, "reason": "The quoted TRANSLATION names the wrong city.", "severity": "major", "vote": "supported"}]}.
The severity field is optional; the reason is not. No prose outside the JSON, no code fences.
Every claim number must appear exactly once in verdicts.`;

/**
 One line of human-readable evidence for one span.

 @param span - anchored evidence to present

 @returns Line naming side and quoted text, or the insertion-point wording

 @example
 ```ts
 evidenceLine({ span, },);
 ```
 */
function evidenceLine(
  { span, }: { readonly span: SpanAnchor; },
): string {
  /**
   Side label in prompt vocabulary.
   */
  const sideLabel = span.side === 'source' ? 'ORIGINAL' : 'TRANSLATION';
  if (span.startOffset === span.endOffset)
    return `- evidence (${sideLabel}): insertion point, content claimed missing here`;

  return `- evidence (${sideLabel}): ${indentContinuation({
    text: span.quotedText,
    indent: '  ',
  },)}`;
}

/**
 Messages plus the index maps ballots resolve through:
 claim number N on the wire means `claimIds[N - 1]`,
 group number M means `clusterIds[M - 1]`.

 @example
 ```ts
 const plan: AdjudicationPromptPlan = buildAdjudicationMessages({
   sourceText,
   targetText,
   clusters,
 },);
 ```
 */
export type AdjudicationPromptPlan = {
  /**
   Messages ready for `chatJson`.
   */
  readonly messages: readonly ChatMessage[];

  /**
   Claim ids in prompt numbering order.
   */
  readonly claimIds: readonly string[];

  /**
   Cluster ids in prompt numbering order.
   */
  readonly clusterIds: readonly string[];
};

/**
 Builds the panel sheet for one chunk:
 documents fenced, clusters as numbered groups, claims numbered globally.

 @param sourceText - original chunk text

 @param targetText - translation chunk text

 @param clusters - aggregation output for this chunk, in document order

 @param neighbouringSourceText - nearby source evidence without extra coverage obligations

 @param neighbouringIncumbentText - existing placement context, not factual authority

 @param documentSourceText - optional same-entry original evidence for checking current claims

 @param referenceContext - what the pages the original links say (class
 thirty-five), evidence for judging addition claims

 @param identityContext - declared names and handles both documents carry,
 so a claim against a declared name is judged against the declaration
 (ledger S4); absent when the page declares none

 @returns Messages plus index maps for ballot resolution

 @example
 ```ts
 const plan = buildAdjudicationMessages({ sourceText, targetText, clusters, },);
 ```
 */
export function buildAdjudicationMessages(
  {
    sourceText,
    targetText,
    clusters,
    neighbouringIncumbentText,
    neighbouringSourceText,
    documentSourceText,
    referenceContext,
    identityContext,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly clusters: readonly ClaimCluster[];
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly documentSourceText?: string;
    readonly referenceContext?: string;
    readonly identityContext?: string;
  },
): AdjudicationPromptPlan {
  /**
   Claim ids in numbering order, filled while rendering.
   */
  const claimIds: string[] = [];

  /**
   Rendered group blocks in document order.
   */
  const groupBlocks = clusters.map(function toGroupBlock(
    cluster,
    clusterIndex,
  ) {
    /**
     Rendered member claims of this group.
     */
    const memberBlocks = cluster.members
      .map(function toClaimBlock(member,) {
        claimIds.push(member.claimId,);

      /**
       Evidence lines for every span of this claim.
       */
        const evidence = member
          .claim
          .spans
          .map(function toLine(span,) {
            return evidenceLine({ span, },);
          },)
          .join('\n',);

        return `CLAIM ${claimIds.length}
- category: ${member.claim
  .category}
- claimed severity: ${member.claim
  .severity}
- summary: ${flattenSpace({ text: member.claim
  .summary, },)}
${evidence}`;
      },);

    /**
     Group header; single-claim groups need no same-defect question.
     */
    const header = cluster.members
      .length
      > 1
      ? `GROUP ${clusterIndex + 1} (claims below may describe one defect; answer sameDefect)`
      : `GROUP ${clusterIndex + 1}`;

    return [
      header,
      ...memberBlocks,
    ].join('\n',);
  },);

  // THE PANEL IS SHOWN THE TRANSLATION WITHOUT THE NOTES THE ARCHIVE'S
  // TRANSLATORS LEFT EACH OTHER, as the critic whose claims it weighs is
  // (`critic-prompt.ts`): no reader sees one, so no claim may rest on one and no
  // panelist may count one for or against a claim. Only what is SHOWN is cut;
  // the claims' evidence lines quote the stored text, and the original's own
  // comments stay in view.
  /**
   Texts whose comments are the original's own.
   */
  const originals = [
    sourceText,
    neighbouringSourceText ?? '',
    documentSourceText ?? '',
  ].join('\n',);
  /**
   The translation as a panelist reads it.
   */
  const targetShown = withoutTranslatorsNotes({
    text: targetText,
    original: originals,
  },);
  /**
   The existing translation next door as a panelist reads it.
   */
  const incumbentShown = withoutTranslatorsNotes({
    text: neighbouringIncumbentText ?? '',
    original: originals,
  },);

  /**
   Fence no enclosed text can reproduce, chosen against every text the sheet encloses,
   the rendered claims included, each text as written and as shown.
   */
  const fence = selectFence({
    texts: [
      sourceText,
      targetText,
      targetShown,
      neighbouringSourceText ?? '',
      neighbouringIncumbentText ?? '',
      incumbentShown,
      documentSourceText ?? '',
      referenceContext ?? '',
      identityContext ?? '',
      ...groupBlocks,
    ],
  },);

  /**
   What the original's cited pages say, after the full document and before
   the end, so an addition claim is judged against them (class thirty-five).
   */
  const referenceBlock = citedReferenceBlockText({
    fence,
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
  },);

  /**
   The passages either side, or nothing when this slice stands alone.

   PLACED BEFORE THE CLAIMS AND AFTER THE PAIR, so a panelist reads the
   evidence in the order the question needs it: what is under review, then
   what sits beside it, then what is alleged about the first.
   */
  const nearbyBlock = ((neighbouringSourceText === undefined)
    || (neighbouringSourceText === ''))
    && (incumbentShown === '')
    ? ''
    : `${fence} NEARBY ORIGINAL, CONTEXT ONLY ${fence}
${neighbouringSourceText ?? ''}
${fence} NEARBY EXISTING TRANSLATION, CONTEXT ONLY ${fence}
${incumbentShown}
${fence} ${NEARBY_RULE} ${fence}
`;

  /**
   Complete same-entry evidence checks absence claims without expanding coverage.
   */
  const documentBlock = (documentSourceText === undefined) || (documentSourceText === '')
    ? ''
    : `${fence} FULL ORIGINAL DOCUMENT, FACTUAL EVIDENCE ONLY ${fence}
${documentSourceText}
${fence} This original document is additional evidence for checking claims about the CURRENT TRANSLATION. It does not expand that editable scope or require translating the rest of the document here. ${fence}
`;

  /**
   Declared identity before the documents, read as given facts as the critic
   reads it (ledger S4), or nothing when the page declares nothing.
   */
  const identityBlock = declaredNamesBlock({
    fence,
    ...((identityContext === undefined) ? {} : { identityContext, }),
  },);

  return {
    messages: [
      {
        role: 'system',
        content: ADJUDICATION_SYSTEM_PROMPT,
      },
      {
        role: 'user',
        content: `${identityBlock}${fence} ORIGINAL ${fence}
${sourceText}
${fence} TRANSLATION ${fence}
${targetShown}
${nearbyBlock}${fence} CLAIMS ${fence}
${groupBlocks.join('\n\n',)}
${documentBlock}${referenceBlock}${fence} END ${fence}`,
      },
    ],
    claimIds,
    clusterIds: clusters.map(function toClusterId(cluster,) {
      return cluster.clusterId;
    },),
  };
}

//endregion Adjudication prompt
