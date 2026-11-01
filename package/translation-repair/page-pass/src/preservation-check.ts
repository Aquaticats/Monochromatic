import {
  isAsciiDigits,
  textsInCodePointOrder,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  contentTokens,
  properNouns,
} from './preservation-tokens.ts';

//region Preservation check
// A deterministic, model-free gate against the failure mode that actually
// damaged shipped repairs: the editor is asked to change one quoted fragment
// and replaces a much larger span, silently dropping content nobody complained
// about.
//
// The rule is one sentence: everything in the replaced text that no accepted
// issue quoted as defective must still be present afterwards. What an issue
// DID quote is licensed to disappear, because changing it is the entire point
// of the edit.
//
// CALIBRATED ON REAL GRADED REPAIRS rather than chosen by taste. Over the 50
// drawn repairs of round three, 34 of them graded: this gate rejects the two
// regions a human read as damage (the Acheron edit, drawn four times as items
// 2, 7, 11 and 15, and the contributor-line edit at item 21), rejects one
// wholesale deletion at item 48, and rejects NONE of the 29 repairs graded
// sound. The separation is not marginal: the worst sound repair loses 57% of
// its unlicensed tokens, and the rejected one loses 92%.
//
// WHAT IT CAN SEE TODAY (ledger L4, measured 2026-09-28). Envelopes are cut
// from the accepted issues' own quotes, so the replaced text and the licensed
// text are the same text: `residualTokens` is 0 on 536 of 540 recorded
// regions, and this token rule rejects almost nothing. What guards inside a
// licensed quote is the markup-atom check in `apply-patch.ts`.
//
// WHAT IT DOES NOT CATCH, stated so nobody expects it to. Rewording is not
// deletion. Item 37 turned "reminiscing" into "pleading", which is real damage
// and passes this gate, because the sentence is still represented. Catching
// that needs a meaning comparison, which is not deterministic.

/**
 Share of unlicensed content tokens that may vanish before an edit reads as a
 deletion rather than a rewrite.

 0.8 sits in a measured gap rather than at a round number: across the graded
 repairs the highest loss among sound edits is 0.57, the highest among edits
 that merely reworded is 0.67, and the deletion this gate exists to stop loses
 0.92.
 */
const LOSS_FRACTION_LIMIT = 0.8;

/**
 Unlicensed content tokens required before the bulk rule applies at all.

 Under this, a single substituted word reads as total loss and the fraction
 says nothing. Distinctive-token loss still applies at any size.
 */
const MIN_RESIDUAL_TOKENS = 5;

/**
 What the preservation gate concluded about one edit.

 @example
 ```ts
 const verdict: PreservationVerdict = { preserved: true, lostDistinctive: [], lossFraction: 0, residualTokens: 4, };
 ```
 */
export type PreservationVerdict = {
  /**
   Whether the edit may proceed.
   */
  readonly preserved: boolean;

  /**
   Names and numbers present before the edit and absent after it, which are
   the losses no rewrite explains.
   */
  readonly lostDistinctive: readonly string[];

  /**
   Share of unlicensed content tokens that did not survive.
   */
  readonly lossFraction: number;

  /**
   Unlicensed content tokens the edit was measured against, so a caller can
   tell a real pass from a vacuous one.
   */
  readonly residualTokens: number;
};

/**
 Removes the quoted defects from the replaced text, leaving what the edit had
 no licence to change.

 @param before - text the edit replaced

 @param licensedQuotes - fragments accepted issues quoted as defective

 @returns Text with every licensed fragment blanked out

 @example
 ```ts
 const residual = unlicensedText({ before, licensedQuotes, },);
 ```
 */
function unlicensedText(
  {
    before,
    licensedQuotes,
  }: {
    readonly before: string;
    readonly licensedQuotes: readonly string[];
  },
): string {
  // Longest first, so a short quote nested inside a longer one cannot blank a
  // fragment of it and leave the remainder looking unlicensed.
  return [...licensedQuotes,]
    .toSorted(function byLengthDescending(
      left,
      right,
    ): number {
      return right.length - left.length;
    },)
    .reduce(
    function blank(
      text,
      quote,
    ): string {
      return (quote === '') ? text : text.split(quote,)
        .join(' ',);
    },
    before,
    );
}

/**
 Decides whether an edit preserved everything it was not asked to change.

 @param before - exact text the edit replaced

 @param after - exact text the edit wrote, empty for a deletion

 @param licensedQuotes - fragments accepted issues quoted as the defect

 @returns Verdict, with the evidence behind it

 @example
 ```ts
 const verdict = checkPreservation({ before, after, licensedQuotes: [quote,], },);
 ```
 */
export function checkPreservation(
  {
    before,
    after,
    licensedQuotes,
  }: {
    readonly before: string;
    readonly after: string;
    readonly licensedQuotes: readonly string[];
  },
): PreservationVerdict {
  /**
   Content tokens the edit had no licence to remove.
   */
  const residual = contentTokens({
    text: unlicensedText({
      before,
      licensedQuotes,
    },),
  },);

  /**
   Everything the edit wrote, as a lookup.
   */
  const survivors = new Set(contentTokens({ text: after, },),);

  /**
   Unlicensed tokens with no survivor.
   */
  const missing = residual.filter(function isMissing(token,): boolean {
    return !survivors.has(token,);
  },);

  /**
   Names appearing mid-sentence in the ORIGINAL text, where its sentence
   structure is still intact.
   */
  const names = properNouns({ text: before, },);

  /**
   Missing tokens that are names or numbers, whose loss no rewrite explains,
   ordered by code point, since these are words of the document and the
   default order compares UTF-16 units, which puts a name past U+FFFF before
   one between U+E000 and U+FFFF (ledger B95).
   */
  const lostDistinctive = textsInCodePointOrder({ texts: [...new Set(missing.filter(function isDistinctive(token,): boolean {
    // EVERY character must be a digit, not merely the first. A token like
    // "10th" begins with a digit while being a word, and an edit rewriting
    // "July 10th" as "July 10" loses it without losing anything: measured, that
    // exact case rejected a repair a human graded sound.
    return names.has(token,) || isAsciiDigits({ text: token, },);
  },),),], },);

  /**
   Share of unlicensed tokens that vanished.
   */
  const lossFraction = (residual.length === 0)
    ? 0
    : missing.length / residual.length;

  return {
    preserved: (lostDistinctive.length === 0)
      && ((residual.length < MIN_RESIDUAL_TOKENS) || (lossFraction <= LOSS_FRACTION_LIMIT)),
    lostDistinctive,
    lossFraction,
    residualTokens: residual.length,
  };
}

//endregion Preservation check
