import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  wordForCount,
  wrapReplacementText,
} from '@monochromatic-dev/translation-repair-typography/ts';
import type { TranslateSliceRecord, } from './translate-document-contract.ts';
import { sameWording, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Translate lane wrap
// APPLIES THE SEMANTIC WRAP TO WHAT THE TRANSLATE LANE PRODUCED, at the one
// point both consumers read from.
//
// `assembleTranslation` builds the replacements AND the lane wordings out of
// the same settled list, and the delivery invariant requires those two to agree
// byte for byte. Wrapping the list once, here, is what keeps them agreeing.
//
// ONLY CHANGED RECORDS ARE TOUCHED. A record that stands on the archive, either
// because the judges preferred it or because no translator answered, carries
// the archive's own wording; wrapping it would report a change nobody decided
// on and would contradict `sliceRecordAgrees`.

/**
 Wraps every changed translate record, re-deriving whether it still changes.

 RE-DERIVED RATHER THAN CARRIED FORWARD, for the reason `wrapRepairOutcomes`
 gives: a passage differing from the archive only in its wrapping becomes the
 archive once wrapped, and a record still claiming a change there fails the
 assembly assertion. Since 2026-09-29 the question is the page's (ledger B26,
 `sameWording`), asked of every changed record including one the wrap leaves
 as it is, and a record that is the archive in all but layout keeps the
 archive's own bytes.

 NEVER APPLIED TO A LINE-STRUCTURED SLICE. The pipeline hands a governed
 producer `TRANSLATE_LINE_STRUCTURE_RULE`, one output line per original line,
 and then broke that work afterwards: over the 211 line-structured slices of
 the pinned corpus the wrap changed 189 and broke 470 of 1091 lines, after
 every decider had approved them. Flattening is caught by the structural
 guard and sent back to its author instead of papered over here, because
 `wrapReplacementText` splits and never joins, so it cannot put back a break
 a producer merged away.

 @param slices - prepared slice pairs, for the archive wording per index

 @param settled - settled per-slice records in document order

 @param lineStructuredSlices - global indices the line-structure rule
 governs, whose lines are the producer's to set

 @param l - lane logger

 @returns Same records with produced wording wrapped

 @example
 ```ts
 const wrapped = wrapTranslateRecords({ slices, settled, lineStructuredSlices, l, },);
 ```
 */
export function wrapTranslateRecords(
  {
    slices,
    settled,
    lineStructuredSlices,
    l,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly settled: readonly TranslateSliceRecord[];
    readonly lineStructuredSlices: ReadonlySet<number>;
    readonly l: Logger;
  },
): readonly TranslateSliceRecord[] {
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
   How many records the wrap altered, and how many it demoted.
   */
  const counted = {
    rewrapped: 0,
    demoted: 0,
    governed: 0,
  };

  /**
   Records with produced wording wrapped.
   */
  const wrapped = settled.map(function perRecord(record,): TranslateSliceRecord {
    if (!record.changed)
      return record;

    /**
     Archive wording here, absent when the slice is not in the pair list.
     */
    const incumbentText = incumbentByIndex.get(record.sliceIndex,);

    /**
     Whether the line-structure rule governs this slice, which leaves its
     lines as the producer wrote them.
     */
    const lineStructured = lineStructuredSlices.has(record.sliceIndex,);

    /**
     Wording as it would ship: as the rule would write it, or as produced on
     a governed slice, whose line breaks the producer was told to set.
     */
    const outputText = lineStructured
      ? record.outputText
      : wrapReplacementText({ text: record.outputText, },);

    // THE ARCHIVE'S WORDING IN ALL BUT LAYOUT IS THE ARCHIVE'S (ledger B26),
    // read before either early return of this function: a proposal the wrap leaves as it
    // is can still be the archive with its soft breaks elsewhere, and a
    // governed one the archive with a trailing newline. Either would ship a
    // change the page does not show, so the record keeps the archive's own
    // bytes.
    if (
      (incumbentText !== undefined)
        && sameWording({
          proposal: outputText,
          standing: incumbentText,
          lineStructured,
        },)
    ) {
      counted.demoted += 1;
      return {
        ...record,
        outputText: incumbentText,
        changed: false,
      };
    }

    if (lineStructured) {
      counted.governed += 1;
      return record;
    }
    if (outputText === record.outputText)
      return record;
    counted.rewrapped += 1;

    return {
      ...record,
      outputText,
    };
  },);

  if (counted.governed > 0) {
    l.info(
      `semantic wrap: skipped ${String(counted.governed,)} line-structured translated ${
        wordForCount({
          count: counted.governed,
          one: 'slice',
          many: 'slices',
        },)
      }, whose line breaks the producer was told to set`,
    );
  }

  if (counted.demoted > 0) {
    l.info(
      `wording: ${String(counted.demoted,)} of ${String(settled.length,)} translated ${
        wordForCount({
          count: settled.length,
          one: 'slice differs',
          many: 'slices differ',
        },)
      } from the archive only in layout the page does not show, and ${
        wordForCount({
          count: settled.length,
          one: 'keeps',
          many: 'keep',
        },)
      } the archive's own wording (ledger B26)`,
    );
  }

  if (counted.rewrapped > 0) {
    l.info(
      `semantic wrap: rewrapped ${String(counted.rewrapped,)} of ${String(settled.length,)} translated ${
        wordForCount({
          count: settled.length,
          one: 'slice',
          many: 'slices',
        },)
      }`,
    );
  }

  return wrapped;
}

//endregion Translate lane wrap
