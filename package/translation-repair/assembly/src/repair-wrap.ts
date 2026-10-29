import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { ChunkRepairOutcome, } from './repair-contract.ts';
import { wrapReplacementText, } from '@monochromatic-dev/translation-repair-typography/ts';
import { sameWording, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Repair lane wrap
// APPLIES THE SEMANTIC WRAP TO WHAT THE REPAIR LANE PRODUCED, at the one point
// both consumers read from.
//
// `assembleRepair` builds the replacements AND the lane wordings out of the
// same outcome list, and the delivery invariant requires those two to agree
// byte for byte: the ledger's rows are spliced over the archive and compared
// against the document the lane returned. Wrapping the list once, here, is what
// keeps them agreeing. Wrapping either consumer alone would break the other.
//
// ONLY CHANGED OUTCOMES ARE TOUCHED. An unchanged outcome carries the archive's
// own wording, and wrapping it would report a change nobody decided on. Two
// checks refuse exactly that, `assertReplacementsChange` and the delivery
// coherence rule that a replacement's wording may not be the archive's, so this
// is a correctness constraint rather than a preference.

/**
 Wraps every changed repair outcome, re-deriving whether it still changes.

 RE-DERIVED RATHER THAN CARRIED FORWARD. A passage whose only difference from
 the archive was its wrapping becomes identical to the archive once wrapped,
 and an outcome still claiming a change at that point fails the assembly
 assertion. It is a retention, so it is recorded as one, with the archive's
 own bytes. SINCE 2026-09-29 THE QUESTION IS THE PAGE'S (ledger B26,
 `sameWording`): a proposal that is the archive with its soft line breaks
 elsewhere, which the site renders as spaces, is a retention too, and so is a
 governed one that is the archive with a trailing newline. It is asked of
 every changed outcome, including one the wrap leaves as it is, which the
 earlier byte check after the wrap never reached; over the stored artifacts
 186 lane texts were such a proposal.

 NEVER APPLIED TO A LINE-STRUCTURED SLICE. The pipeline hands a governed
 producer `TRANSLATE_LINE_STRUCTURE_RULE`, one output line per original line,
 and then broke that work afterwards: over the 211 line-structured slices of
 the pinned corpus the wrap changed 189 and broke 470 of 1091 lines, after
 every decider had approved them. Flattening is caught by the structural
 guard and sent back to its author instead of papered over here, because
 `wrapReplacementText` splits and never joins, so it cannot put back a break
 a producer merged away.

 @param slices - prepared slice pairs, for the archive wording per index

 @param outcomes - settled per-slice outcomes, refinement included

 @param lineStructuredSlices - global indices the line-structure rule
 governs, whose lines are the producer's to set

 @param l - lane logger

 @returns Same outcomes with produced wording wrapped

 @example
 ```ts
 const wrapped = wrapRepairOutcomes({ slices, outcomes, lineStructuredSlices, l, },);
 ```
 */
export function wrapRepairOutcomes(
  {
    slices,
    outcomes,
    lineStructuredSlices,
    l,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly outcomes: readonly ChunkRepairOutcome[];
    readonly lineStructuredSlices: ReadonlySet<number>;
    readonly l: Logger;
  },
): readonly ChunkRepairOutcome[] {
  /**
   Archive wording per slice index, which decides whether a wrap left anything
   to change.
   */
  const incumbentByIndex = new Map(slices.map(function toEntry(slice,): readonly [
    number,
    string,
  ] {
    return [
      slice.target
        .sliceIndex,
      slice.target
        .text,
    ];
  },),);

  /**
   How many outcomes the wrap altered, and how many it demoted.
   */
  const counted = {
    rewrapped: 0,
    demoted: 0,
    governed: 0,
  };

  /**
   Outcomes with produced wording wrapped.
   */
  const wrapped = outcomes.map(function perOutcome(outcome,): ChunkRepairOutcome {
    if (!outcome.changed)
      return outcome;

    /**
     Archive wording here, absent when the slice is not in the pair list.
     */
    const incumbentText = incumbentByIndex.get(outcome.sliceIndex,);

    /**
     Whether the line-structure rule governs this slice, which leaves its
     lines as the producer wrote them.
     */
    const lineStructured = lineStructuredSlices.has(outcome.sliceIndex,);

    /**
     Wording as it would ship: as the rule would write it, or as produced on
     a governed slice, whose line breaks the producer was told to set.
     */
    const repairedText = lineStructured
      ? outcome.repairedText
      : wrapReplacementText({ text: outcome.repairedText, },);

    // THE ARCHIVE'S WORDING IN ALL BUT LAYOUT IS THE ARCHIVE'S (ledger B26),
    // read before either early return of this function: a proposal the wrap leaves as it
    // is can still be the archive with its soft breaks elsewhere, and a
    // governed one the archive with a trailing newline. Either would ship a
    // change the page does not show, so the outcome keeps the archive's own
    // bytes.
    if (
      (incumbentText !== undefined)
        && sameWording({
          proposal: repairedText,
          standing: incumbentText,
          lineStructured,
        },)
    ) {
      counted.demoted += 1;
      return {
        ...outcome,
        repairedText: incumbentText,
        changed: false,
      };
    }

    if (lineStructured) {
      counted.governed += 1;
      return outcome;
    }
    if (repairedText === outcome.repairedText)
      return outcome;
    counted.rewrapped += 1;

    return {
      ...outcome,
      repairedText,
    };
  },);

  if (counted.governed > 0)
    l.info(
      `semantic wrap: skipped ${String(counted.governed,)} line-structured repair outcomes, whose `
        + 'line breaks the producer was told to set',
    );

  if (counted.demoted > 0)
    l.info(
      `wording: ${String(counted.demoted,)} of ${String(outcomes.length,)} repair outcomes differ from the `
        + 'archive only in layout the page does not show, and keep the archive\'s own wording (ledger B26)',
    );

  if (counted.rewrapped > 0)
    l.info(
      `semantic wrap: rewrapped ${String(counted.rewrapped,)} of ${String(outcomes.length,)} repair outcomes`,
    );

  return wrapped;
}

//endregion Repair lane wrap
