import type { ComparisonRowField, } from './artifact-two-lane-row-equality.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Artifact version 2 comparison refusals
// Why two version 2 ledgers, or two derivations of one comparison, cannot be
// read as one, stated as a closed set of faults so the sentence is written in
// one place.
//
// THE CLASS WRITES ITS SENTENCE from lane names, counts, positions, slice
// indexes, archive-wording kinds and comparison-row field names. Its throw
// sites used to hand it finished text, which kept it from carrying the
// `messageNamesOnly` marker, so the artifact reader forwarded an unmarked
// message into a marked one (ledger B34).

/**
 Why a comparison refused, with the numbers and names its sentence states.

 @example
 ```ts
 const fault: ArtifactComparisonFault = { kind: 'different-originals', sliceIndex: 3, };
 ```
 */
export type ArtifactComparisonFault =
  | {
    /**
     The two ledgers hold different numbers of rows.
     */
    readonly kind: 'ledger-lengths';

    /**
     Rows the repair ledger holds.
     */
    readonly repairRows: number;

    /**
     Rows the translate ledger holds.
     */
    readonly translateRows: number;
  }
    | {
    /**
     One position names a different slice in each ledger.
     */
      readonly kind: 'slice-positions';

    /**
     Position in both ledgers.
     */
      readonly position: number;

    /**
     Slice the repair ledger names there.
     */
      readonly repairSliceIndex: number;

    /**
     Slice the translate ledger names there.
     */
      readonly translateSliceIndex: number;
    }
    | {
    /**
     One slice carries a different original in each ledger.
     */
      readonly kind: 'different-originals';

    /**
     Slice they disagree on.
     */
      readonly sliceIndex: number;
    }
    | {
    /**
     One slice carries a different archive wording in each ledger.
     */
      readonly kind: 'different-archive-wordings';

    /**
     Slice they disagree on.
     */
      readonly sliceIndex: number;
    }
    | {
    /**
     The archive holds wording at one slice to one ledger and none to the
     other.
     */
      readonly kind: 'different-archive-kinds';

    /**
     Slice they disagree on.
     */
      readonly sliceIndex: number;

    /**
     Whether the archive holds wording there, to the repair ledger.
     */
      readonly repairKind: 'present' | 'absent';

    /**
     Same, to the translate ledger.
     */
      readonly translateKind: 'present' | 'absent';
    }
    | {
    /**
     Version 2's rules and the pipeline's derive different numbers of rows.
     */
      readonly kind: 'derivation-lengths';

    /**
     Rows version 2's rules derive.
     */
      readonly frozenRows: number;

    /**
     Rows the pipeline derives.
     */
      readonly liveRows: number;
    }
    | {
    /**
     Version 2's rules and the pipeline's disagree about one row.
     */
      readonly kind: 'derivations-differ';

    /**
     Slice the row describes.
     */
      readonly sliceIndex: number;

    /**
     Fields they disagree on, named and never quoted: the rows carry slice
     text.
     */
      readonly fields: readonly ComparisonRowField[];
    };

/**
 Writes the sentence a comparison refusal carries.

 @param fault - why the comparison refused

 @returns Sentence naming the slice or position and what disagreed there,
 composed from numbers, lane names, kinds and field names alone

 @throws {@link UnhandledMemberInvariantError} when a member of the version 2 comparison fault union has no branch here, which the compiler rules out

 @example
 ```ts
 const sentence = comparisonFaultSentence({ fault: { kind: 'different-originals', sliceIndex: 3, }, },);
 ```
 */
function comparisonFaultSentence(
  { fault, }: { readonly fault: ArtifactComparisonFault; },
): string {
  if (fault.kind === 'ledger-lengths')
    return `the repair ledger covers ${String(fault.repairRows,)} ${
      wordForCount({
        count: fault.repairRows,
        one: 'slice',
        many: 'slices',
      },)
    } and the translate ledger ${
      String(fault.translateRows,)
    }, so they describe different preparations`;
  if (fault.kind === 'slice-positions')
    return `position ${String(fault.position,)} names slice ${
      String(fault.repairSliceIndex,)
    } in the repair ledger and slice ${String(fault.translateSliceIndex,)} in the translate ledger`;
  if (fault.kind === 'different-originals')
    return `slice ${String(fault.sliceIndex,)} carries a different original in each ledger, `
      + 'so the two ledgers were built over different slicings';
  if (fault.kind === 'different-archive-wordings')
    return `slice ${String(fault.sliceIndex,)} carries a different archive wording in each ledger, `
      + 'so the two ledgers were built over different preparations';
  if (fault.kind === 'different-archive-kinds')
    return `the archive's wording at slice ${String(fault.sliceIndex,)} is ${
      fault.repairKind
    } in the repair ledger and ${fault.translateKind} in the translate ledger`;
  if (fault.kind === 'derivation-lengths')
    return `version 2 derives ${String(fault.frozenRows,)} comparison ${
      wordForCount({
        count: fault.frozenRows,
        one: 'row',
        many: 'rows',
      },)
    } where the pipeline derives ${
      String(fault.liveRows,)
    }, so the two no longer describe one comparison`;
  if (fault.kind === 'derivations-differ') {
    return `version 2 and the pipeline disagree about slice ${String(fault.sliceIndex,)} on ${
      fault.fields
        .join(', ',)
    }; one of them changed, and which artifacts mean what depends on which`;
  }
  throw new UnhandledMemberInvariantError({
    what: 'version 2 comparison fault',
    member: fault,
  },);
}

/**
 Reports a pair of version 2 ledgers that cannot be compared row for row, or
 two derivations of one comparison that disagree.

 @example
 ```ts
 throw new ArtifactComparisonError({ fault: { kind: 'different-originals', sliceIndex: 3, }, },);
 ```
 */
export class ArtifactComparisonError extends Error {
  /**
   Declares this message safe to forward: lane names, counts, positions,
   slice indexes, archive-wording kinds and field names in a sentence written
   here.
   */
  readonly messageNamesOnly: true = true;

  /**
   Why the comparison refused.
   */
  readonly fault: ArtifactComparisonFault;

  /**
   @param fault - why the comparison refused

   @example
   ```ts
   new ArtifactComparisonError({ fault: { kind: 'different-originals', sliceIndex: 3, }, },);
   ```
   */
  constructor({ fault, }: { readonly fault: ArtifactComparisonFault; },) {
    super(comparisonFaultSentence({ fault, },),);
    this.name = 'ArtifactComparisonError';
    this.fault = fault;
  }
}

//endregion Artifact version 2 comparison refusals
