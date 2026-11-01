//region Tense candidate labels
// THE LABELS A JUDGING SHEET GIVES THE TEXTS ON OFFER, in one place so the
// sheet that prints a tense block, the writer that records its finding and the
// reader that checks the record read the same spellings. A label the writer
// spelled one way and the reader another would refuse every record of an entry
// that records a death.

/**
 Label every sheet gives the archive's own wording.
 */
export const ARCHIVE_TENSE_LABEL = 'ARCHIVE RENDERING';

/**
 Label the lane contest sheet gives the repair lane's wording.
 */
export const REPAIR_TENSE_LABEL = 'CANDIDATE "repair"';

/**
 Label the lane contest sheet gives the translate lane's wording.
 */
export const TRANSLATE_TENSE_LABEL = 'CANDIDATE "translate"';

/**
 Label the consolidation gate gives the consolidated wording.
 */
export const CONSOLIDATED_TENSE_LABEL = 'CANDIDATE "consolidated"';

/**
 Label the consolidation gate gives the standing wording.
 */
export const STANDING_TENSE_LABEL = 'CANDIDATE "standing"';

/**
 Labels a contest slice can carry in its finding, in the order its sheet prints the texts.
 */
export const CONTEST_TENSE_LABELS: readonly string[] = [
  ARCHIVE_TENSE_LABEL,
  REPAIR_TENSE_LABEL,
  TRANSLATE_TENSE_LABEL,
];

/**
 Label the sheets gave the wording a consolidation slice ships: the
 consolidation's own under the label the gate gave it, and an incumbent that
 stood in under the archive's.

 @param kind - which wording the slice ships

 @returns The label its finding is recorded under

 @example
 ```ts
 const label = shippedTenseLabelOf({ kind: 'incumbent', },);
 ```
 */
export function shippedTenseLabelOf(
  { kind, }: { readonly kind: 'consolidated' | 'incumbent'; },
): string {
  return (kind === 'consolidated') ? CONSOLIDATED_TENSE_LABEL : ARCHIVE_TENSE_LABEL;
}

//endregion Tense candidate labels
