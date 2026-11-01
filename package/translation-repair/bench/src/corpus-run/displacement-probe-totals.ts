import type { EntryDisplacement, } from './displacement-probe-row.ts';

//region Displacement probe totals
// Counts across the entries of the displacement probe, and whether one entry
// carries anything worth printing.

/**
 Counts across every complete pair.

 @example
 ```ts
 const totals: CorpusTotals = NO_TOTALS;
 ```
 */
export type CorpusTotals = {
  /**
   Slices read.
   */
  readonly slices: number;

  /**
   Entries whose own expansion was not believable.
   */
  readonly fellBack: number;

  /**
   ADJACENCIES, not slices: one high slice beside two qualifying neighbours
   counts twice, and one donor can account for two separate highs. Anything
   reported as a share of slices has to be counted as unique slices instead,
   which is what the handover does.
   */
  readonly relocationCandidates: number;

  /**
   Slices whose original was left essentially unrendered.
   */
  readonly untranslated: number;

  /**
   Slices carrying translation the original does not account for.
   */
  readonly targetOnly: number;

  /**
   Relocation candidates a transcription would also explain.
   */
  readonly transcriptionSuspects: number;

  /**
   Surpluses with only one end.
   */
  readonly otherImbalances: number;
};

/**
 Counts of nothing, where every total starts.
 */
const NO_TOTALS: CorpusTotals = {
  slices: 0,
  fellBack: 0,
  relocationCandidates: 0,
  untranslated: 0,
  targetOnly: 0,
  transcriptionSuspects: 0,
  otherImbalances: 0,
};

/**
 Adds one entry's counts to a running total.

 @param totals - counts so far

 @param row - entry to add

 @returns Counts including that entry

 @example
 ```ts
 const totals = addEntry({ totals: corpusTotals({ rows: [], },), row, },);
 ```
 */
function addEntry(
  {
    totals,
    row,
  }: {
    readonly totals: CorpusTotals;
    readonly row: EntryDisplacement;
  },
): CorpusTotals {
  /**
   Whether this entry could not speak for itself.
   */
  const fellBack = (row.baselineFrom === 'corpus-reference') ? 1 : 0;

  /**
   Relocation candidates this entry carries.
   */
  const relocations = row.relocationCandidates
    .length;

  /**
   Untranslated slices this entry carries.
   */
  const untranslated = row.untranslated
    .length;

  /**
   Target-only slices this entry carries.
   */
  const targetOnly = row.targetOnly
    .length;

  /**
   Relocation candidates a transcription would also explain.
   */
  const suspects = row.transcriptionSuspects
    .length;

  /**
   One-ended surpluses this entry carries.
   */
  const imbalances = row.otherImbalances
    .length;
  return {
    slices: totals.slices + row.sliceCount,
    fellBack: totals.fellBack + fellBack,
    relocationCandidates: totals.relocationCandidates + relocations,
    untranslated: totals.untranslated + untranslated,
    targetOnly: totals.targetOnly + targetOnly,
    transcriptionSuspects: totals.transcriptionSuspects + suspects,
    otherImbalances: totals.otherImbalances + imbalances,
  };
}

/**
 Whether one entry carries anything worth printing.

 @param row - entry to check

 @returns Whether any class named it

 @example
 ```ts
 if (isNotable({ row, },)) log.info(row.entryId,);
 ```
 */
export function isNotable({ row, }: { readonly row: EntryDisplacement; },): boolean {
  /**
   Everything this entry was flagged for, however classified.
   */
  const flagged = addEntry({
    totals: NO_TOTALS,
    row,
  },);
  if (flagged.relocationCandidates > 0)
    return true;
  if (flagged.untranslated > 0)
    return true;
  if (flagged.targetOnly > 0)
    return true;
  return flagged.otherImbalances > 0;
}

/**
 Corpus-wide totals, one pass over every entry.

 @param rows - every entry's reading

 @returns Counts across the corpus

 @example
 ```ts
 const totals = corpusTotals({ rows, },);
 ```
 */
export function corpusTotals(
  { rows, }: { readonly rows: readonly EntryDisplacement[]; },
): CorpusTotals {
  return rows.reduce(
    function add(
      totals: CorpusTotals,
      row: EntryDisplacement,
    ): CorpusTotals {
      return addEntry({
        totals,
        row,
      },);
    },
    NO_TOTALS,
  );
}

//endregion Displacement probe totals
