import { withoutWhitespace, } from './translate-untranslated.ts';

//region Disputed wording
// OWNER ANSWER 2026-09-27, "No eligible standing" (sixteenth addendum of
// `doc/decision/translation-repair-ineligible-standing.md`). On a disputed
// slice whose disputed reading the repair did not fix, the archive's own
// wording and the repair lane's text carry a reading the adjudicators found
// wrong. Refusing them here, as the deterministic rule refuses any text it
// cannot ship, lets every existing refusal path apply unchanged: the translate
// lane withholds them as incumbent and as candidate, the repair turn tells an
// author who copied one why, and the consolidation refuses them as standing,
// as incumbent stand-in and as lane offer.
//
// COMPARED IN ALL BUT WHITESPACE, as the untranslated check is (fifth
// addendum): a copy respaced or rewrapped is the same wording.

/**
 One wording the deterministic rule refuses on a disputed slice, and why.

 @example
 ```ts
 const wording: DisputedWording = { text: archiveText, reason: 'the archive rendering the adjudicators disputed', };
 ```
 */
export type DisputedWording = {
  /**
   Wording refused.
   */
  readonly text: string;

  /**
   Why it is refused, as the finding says it.
   */
  readonly reason: string;
};

/**
 Findings for a candidate that is a disputed wording, none otherwise.

 @param candidateText - text the rule is reading

 @param disputedWordings - wordings refused on this slice

 @returns One finding per disputed wording the candidate is, in all but
 whitespace

 @example
 ```ts
 const findings = disputedWordingFindings({ candidateText, disputedWordings, },);
 ```
 */
export function disputedWordingFindings(
  {
    candidateText,
    disputedWordings,
  }: {
    readonly candidateText: string;
    readonly disputedWordings: readonly DisputedWording[];
  },
): readonly string[] {
  if (disputedWordings.length === 0)
    return [];
  /**
   Candidate in all but whitespace.
   */
  const bare = withoutWhitespace({ text: candidateText, },);
  if (bare === '')
    return [];
  return disputedWordings
    .filter(function isThisWording(wording,): boolean {
      return withoutWhitespace({ text: wording.text, },) === bare;
    },)
    .map(function toFinding(wording,): string {
      return `Your translation is ${wording.reason}, which cannot ship on this slice; render the ORIGINAL afresh.`;
    },);
}

//endregion Disputed wording
