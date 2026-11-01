import {
  hashContent,
  withoutTranslatorsNotes,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  ADDITION_CATEGORY,
  type AdjudicatedIssue,
  computeIssueClaimId,
  type IssueClaim,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
// The one category the screen reads: extra information the critic says has
// no counterpart in the original.
import {
  type AttestedDetail,
  attestedDetailsOverlapping,
} from './reference-attest-match.ts';

//region Reference attestation claim screen
// Addition claims on attested archive details never reach the panel: the
// claim is recorded rejected with the attestation as its reason, and the
// editor never hears of it (class thirty-seven, 2026-09-16). The panel is
// where Mio21 lost the clause, three ballots to two, with the references on
// the sheet.

/**
 Claims after the screen: those the panel hears, and the issues recorded for
 those it never hears.

 @example
 ```ts
 const { claims, issues, findings, } = screenAttestedAdditions({ claims, attested, targetText, },);
 ```
 */
export type ScreenedClaims = {
  /**
   Claims left for the panel.
   */
  readonly claims: readonly IssueClaim[];
  /**
   Rejected issues for the claims the attestation answered.
   */
  readonly issues: readonly AdjudicatedIssue[];
  /**
   One finding per rejected claim, naming the detail and its reference.
   */
  readonly findings: readonly string[];
};

/**
 Rejected issue for one claim the attestation answered: no ballots, the
 attestation is the decision.

 @param claim - claim screened out

 @param detail - attested detail it overlapped

 @returns Issue recorded rejected

 @example
 ```ts
 const issue = attestedRejection({ claim, detail, },);
 ```
 */
function attestedRejection(
  {
    claim,
    detail,
  }: {
    readonly claim: IssueClaim;
    readonly detail: AttestedDetail;
  },
): AdjudicatedIssue {
  /**
   Stable identity of the claim.
   */
  const claimId = computeIssueClaimId({ claim, },);
  return {
    issueId: `adjudicated/${hashContent({ content: JSON.stringify([
      claimId,
      'reference-attested',
      detail.reference,
    ],), },)}`,
    status: 'rejected',
    severity: claim.severity,
    claims: [{
      claimId,
      claim,
    },],
    tallies: {
      [claimId]: {
        supported: 0,
        unsupported: 0,
        ambiguous: 0,
        sourceDefect: 0,
        abstain: 0,
      },
    },
  };
}

/**
 Screens addition claims against the attested details: a claim whose
 archive-side quote overlaps an attested archive quote is rejected before
 the panel.

 @param claims - validated claims of one chunk

 @param attested - details the attestation produced for the entry

 @param targetText - the chunk's archive text the quotes are placed in

 @param original - the texts whose comments are the original's own on the
 sheets the attestation read: the attested quotes were placed in the archive as
 those sheets showed it, without the notes between translators, so the claims'
 words are placed in the same text; absent when the archive was not cut

 @returns Claims for the panel, rejected issues, findings

 @example
 ```ts
 const screened = screenAttestedAdditions({ claims, attested, targetText, },);
 ```
 */
export function screenAttestedAdditions(
  {
    claims,
    attested,
    targetText,
    original,
  }: {
    readonly claims: readonly IssueClaim[];
    readonly attested: readonly AttestedDetail[];
    readonly targetText: string;
    readonly original?: string;
  },
): ScreenedClaims {
  if (attested.length === 0) {
    return {
      claims,
      issues: [],
      findings: [],
    };
  }
  /**
   The archive text as the attestation placed its quotes in it.
   */
  const placedIn = (original === undefined)
    ? targetText
    : withoutTranslatorsNotes({
      text: targetText,
      original,
    },);
  /**
   Claims kept for the panel.
   */
  const kept: IssueClaim[] = [];
  /**
   Issues recorded for screened claims.
   */
  const issues: AdjudicatedIssue[] = [];
  /**
   Findings, one per screened claim.
   */
  const findings: string[] = [];
  for (const claim of claims) {
    /**
     Details an addition claim's archive-side quotes overlap, none for any
     other category.
     */
    const hits = (claim.category === ADDITION_CATEGORY)
      ? claim.spans
        .filter(function onArchiveSide(span,): boolean {
          return span.side === 'target';
        },)
        .flatMap(function detailsFor(span,): readonly AttestedDetail[] {
          return attestedDetailsOverlapping({
            quote: span.quotedText,
            text: placedIn,
            details: attested,
          },);
        },)
      : [];
    /**
     First detail hit, when any.
     */
    const [detail,] = hits;
    if (detail === undefined) {
      kept.push(claim,);
      continue;
    }
    issues.push(attestedRejection({
      claim,
      detail,
    },),);
    findings.push(
      `addition claim rejected before the panel, reference-attested: "${claim.summary}" is on "${detail.archiveQuote}", which reference ${
        String(detail.reference,)
      } states (${String(detail.voices,)} of ${String(detail.heard,)} ${
        wordForCount({
          count: detail.heard,
          one: 'voice',
          many: 'voices',
        },)
      })`,
    );
  }
  return {
    claims: kept,
    issues,
    findings,
  };
}

//endregion Reference attestation claim screen
