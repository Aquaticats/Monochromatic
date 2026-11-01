import {
  ADDITION_CATEGORY,
  type AdjudicatedIssue,
  type EditableEnvelope,
  type IssueClaim,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Licensed quotes
// What each envelope's issues quoted as the defect, which is what the
// preservation gate treats as licensed to disappear. Everything else in the
// replaced span has to survive.

/**
 Admits every claim, since any accepted issue's quote licenses rewording.

 @returns Always true

 @example
 ```ts
 admitsEveryClaim();
 ```
 */
function admitsEveryClaim(): boolean {
  return true;
}

/**
 Admits addition claims only, the one category whose fix is a removal.

 @param claim - claim under test

 @returns Whether it is an addition claim

 @example
 ```ts
 isAdditionClaim(claim,);
 ```
 */
function isAdditionClaim(claim: IssueClaim,): boolean {
  return claim.category === ADDITION_CATEGORY;
}

/**
 Collects, per envelope, the target-side text its issues' admitted claims
 quoted.

 TARGET-SIDE SPANS ONLY. A source-side quote is Chinese prose that never
 appears in the English being edited, so licensing it would license nothing
 and only slow the lookup.

 @param envelopes - envelopes about to be edited

 @param issues - adjudicated issues for the chunk

 @param admits - which claims' quotes count

 @returns Quotes keyed by envelope id

 @example
 ```ts
 const quotes = quotesByEnvelope({ envelopes, issues, admits: () => true, },);
 ```
 */
function quotesByEnvelope(
  {
    envelopes,
    issues,
    admits,
  }: {
    readonly envelopes: readonly EditableEnvelope[];
    readonly issues: readonly AdjudicatedIssue[];
    readonly admits: (claim: IssueClaim,) => boolean;
  },
): ReadonlyMap<string, readonly string[]> {
  /**
   Quoted defect text of each issue, by issue id.
   */
  const quotesByIssue = new Map(issues.map(function toEntry(issue,) {
    return [
      issue.issueId,
      issue.claims
        .filter(function isAdmitted(member,): boolean {
          return admits(member.claim,);
        },)
        .flatMap(function toQuotes(member,): readonly string[] {
          return member.claim
            .spans
            .filter(function isTargetSide(span,): boolean {
              return span.side === 'target';
            },)
            .map(function toText(span,): string {
              return span.quotedText;
            },);
        },),
    ] as const;
  },),);

  return new Map(envelopes.map(function toEntry(envelope,) {
    return [
      envelope.envelopeId,
      [...new Set(envelope.issueIds
        .flatMap(function toQuotes(issueId,): readonly string[] {
          return quotesByIssue.get(issueId,) ?? [];
        },),),].filter(function isUsable(quote,): boolean {
          return quote !== '';
        },),
    ] as const;
  },),);
}

/**
 Collects the defect text each envelope's accepted issues quoted.

 @param envelopes - envelopes about to be edited

 @param issues - adjudicated issues for the chunk

 @returns Quotes keyed by envelope id

 @example
 ```ts
 const licensedQuotes = buildLicensedQuotes({ envelopes, issues, },);
 ```

 @internal
 */
export function buildLicensedQuotes(
  {
    envelopes,
    issues,
  }: {
    readonly envelopes: readonly EditableEnvelope[];
    readonly issues: readonly AdjudicatedIssue[];
  },
): ReadonlyMap<string, readonly string[]> {
  return quotesByEnvelope({
    envelopes,
    issues,
    admits: admitsEveryClaim,
  },);
}

/**
 Collects the text each envelope's addition claims quoted, whose markup atoms
 an edit may remove (ledger L4): removing the detail an addition quotes is the
 fix, and a footnote or a piece of inline code the translator added goes with
 it.

 CLAIM BY CLAIM, NOT ISSUE BY ISSUE. An issue groups claims, and a quote an
 addition claim did not make licenses no removal because a sibling claim of
 another category shares its issue.

 @param envelopes - envelopes about to be edited

 @param issues - adjudicated issues for the chunk

 @returns Addition quotes keyed by envelope id

 @example
 ```ts
 const removableQuotes = buildRemovableQuotes({ envelopes, issues, },);
 ```

 @internal
 */
export function buildRemovableQuotes(
  {
    envelopes,
    issues,
  }: {
    readonly envelopes: readonly EditableEnvelope[];
    readonly issues: readonly AdjudicatedIssue[];
  },
): ReadonlyMap<string, readonly string[]> {
  return quotesByEnvelope({
    envelopes,
    issues,
    admits: isAdditionClaim,
  },);
}

//endregion Licensed quotes
