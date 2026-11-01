import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Lane comparison faults
// Why two lane ledgers cannot be compared, as a union the class words itself.
// Its own file because `lane-comparison.ts` holds the comparison and its row
// types and has no room left under the file-length limit.

/**
 Lane a ledger belongs to.

 @example
 ```ts
 const lane: ComparedLane = 'repair';
 ```
 */
export type ComparedLane = 'repair' | 'translate';

/**
 Why two ledgers cannot be compared slice by slice.

 @example
 ```ts
 const fault: LaneComparisonFault = { kind: 'missing-from-translate', sliceIndex: 4, };
 ```
 */
export type LaneComparisonFault = {
  /**
   Ledgers name different slicings.
   */
  readonly kind: 'different-slicings';
} | {
  /**
   Ledgers report different slice counts.
   */
  readonly kind: 'slice-counts-differ';

  /**
   Slices the repair ledger reports.
   */
  readonly repair: number;

  /**
   Slices the translate ledger reports.
   */
  readonly translate: number;
} | {
  /**
   One ledger reports more rows than distinct slices.
   */
  readonly kind: 'rows-repeat';

  /**
   Ledger that repeats.
   */
  readonly lane: ComparedLane;

  /**
   Rows it reports.
   */
  readonly rows: number;

  /**
   Distinct slices among them.
   */
  readonly distinct: number;
} | {
  /**
   Repair ledger names a slice the translate ledger lacks.
   */
  readonly kind: 'missing-from-translate';

  /**
   Slice missing.
   */
  readonly sliceIndex: number;
} | {
  /**
   Slice sits at different positions in the two ledgers.
   */
  readonly kind: 'position-differs';

  /**
   Slice out of place.
   */
  readonly sliceIndex: number;

  /**
   Where the repair ledger holds it.
   */
  readonly position: number;
} | {
  /**
   Slice covers a different original in each lane.
   */
  readonly kind: 'source-differs';

  /**
   Slice affected.
   */
  readonly sliceIndex: number;
} | {
  /**
   Slice carries a different incumbent in each lane.
   */
  readonly kind: 'incumbent-differs';

  /**
   Slice affected.
   */
  readonly sliceIndex: number;
} | {
  /**
   Lanes disagree about whether the archive translates the slice.
   */
  readonly kind: 'incumbent-kind-differs';

  /**
   Slice affected.
   */
  readonly sliceIndex: number;

  /**
   What the repair ledger says.
   */
  readonly repair: 'present' | 'absent';

  /**
   What the translate ledger says.
   */
  readonly translate: 'present' | 'absent';
};

/**
 Words a comparison fault from its kinds and numbers.

 @param fault - why the ledgers cannot be compared

 @returns Sentence written here

 @throws {@link UnhandledMemberInvariantError} when a member of `LaneComparisonFault` has no branch here, which the compiler rules out

 @example
 ```ts
 const sentence = comparisonSentence({ fault: { kind: 'different-slicings', }, },);
 ```
 */
export function comparisonSentence({ fault, }: { readonly fault: LaneComparisonFault; },): string {
  if (fault.kind === 'different-slicings')
    return 'the two ledgers name different slicings, so their slice indices number different passages';
  if (fault.kind === 'slice-counts-differ')
    return `lanes report ${String(fault.repair,)} and ${
      String(fault.translate,)
    } ${
      wordForCount({
        count: fault.translate,
        one: 'slice',
        many: 'slices',
      },)
    }, so they ran over different preparations`;
  if (fault.kind === 'rows-repeat')
    return `${fault.lane} lane reports ${String(fault.rows,)} ${
      wordForCount({
        count: fault.rows,
        one: 'row',
        many: 'rows',
      },)
    } over ${String(fault.distinct,)} distinct ${
      wordForCount({
        count: fault.distinct,
        one: 'slice',
        many: 'slices',
      },)
    }`;
  if (fault.kind === 'missing-from-translate')
    return `slice ${String(fault.sliceIndex,)} is missing from the translate lane`;
  if (fault.kind === 'position-differs')
    return `slice ${String(fault.sliceIndex,)} sits at position ${
      String(fault.position,)
    } in one ledger and elsewhere in the other, so the two are not in one document order`;
  if (fault.kind === 'source-differs')
    return `slice ${
      String(fault.sliceIndex,)
    } covers a different original in each lane, so the two results describe different preparations`;
  if (fault.kind === 'incumbent-differs')
    return `slice ${
      String(fault.sliceIndex,)
    } carries a different incumbent in each lane, so the two results describe different preparations`;
  if (fault.kind === 'incumbent-kind-differs') {
    return `the archive's wording at slice ${String(fault.sliceIndex,)} is ${fault.repair} in the repair lane and ${
      fault.translate
    } in the translate lane, so the two disagree about whether the archive translates it`;
  }
  throw new UnhandledMemberInvariantError({
    what: 'lane comparison fault',
    member: fault,
  },);
}

//endregion Lane comparison faults
