import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { SliceDeliveryRecord, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type SliceReplacement,
  spliceSlices,
} from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Delivery invariants
// The two claims a delivery ledger makes about a document it does not contain.
//
// `buildSliceDelivery` joins three reports from one lane, and every check it
// makes is INSIDE that join: a row cannot say shipped and undecided at once,
// and a shipped row's text is its accepted text by construction. What no row
// can check is whether the join describes the document the lane returned, since
// the document is not one of its inputs.
//
// TWO CLAIMS, then, both of which need the document in hand: the rows marked
// shipped are exactly the slices the result names, and writing those rows over
// the archive reproduces the returned text.
//
// THE SECOND IS THE ONE THAT EARNS ITS KEEP. It crosses from what the lane
// DECIDED, which is where a row's text comes from, to what the document
// CARRIES, which the assembly guard decided; those are two derivations, they
// agree today by construction, and nothing until now made them say so.
//
// A ROW'S TEXT IS NOT A SUBSTRING OF THE DOCUMENT, and the reassembly here
// works because it re-splices through the same assembly rather than because it
// could search for one. Where several slices anchor at one boundary, the blank
// lines between their renderings are composed by assembly and belong to no
// slice, so a reader locating a row's shipped text by searching the document
// would find it for content slices and fail for anchored ones.

/**
 What the ledger and the returned document disagree about.

 @example
 ```ts
 const fault: DeliveryInvariantFault = { kind: 'unclaimed', indices: [2, 5,], };
 ```
 */
export type DeliveryInvariantFault = {
  /**
   Ledger ships slices the result does not name as changed.
   */
  readonly kind: 'unclaimed';

  /**
   Slices shipped and not claimed.
   */
  readonly indices: readonly number[];
} | {
  /**
   Result names slices as changed that the ledger does not ship.
   */
  readonly kind: 'unledgered';

  /**
   Slices claimed and not shipped.
   */
  readonly indices: readonly number[];
} | {
  /**
   Writing the shipped rows over the archive gives another document.
   */
  readonly kind: 'reassembly-differs';

  /**
   Rows the ledger ships.
   */
  readonly shippedRows: number;
};

/**
 Words a delivery invariant fault.

 @param fault - what the ledger and the document disagree about

 @returns Sentence composed from slice indices and counts alone

 @throws {@link UnhandledMemberInvariantError} when a member of `DeliveryInvariantFault` has no branch here, which the compiler rules out

 @example
 ```ts
 const sentence = deliveryInvariantSentence({ fault: { kind: 'reassembly-differs', shippedRows: 3, }, },);
 ```
 */
export function deliveryInvariantSentence(
  { fault, }: { readonly fault: DeliveryInvariantFault; },
): string {
  if (fault.kind === 'unclaimed') {
    /**
     Slices shipped and not claimed.
     */
    const { indices, } = fault;
    return `ledger ships slices ${
      indices.join(', ',)
    } that the result does not name as changed, so one of the two describes another run`;
  }
  if (fault.kind === 'unledgered') {
    /**
     Slices claimed and not shipped.
     */
    const { indices, } = fault;
    return `result names slices ${
      indices.join(', ',)
    } as changed and the ledger ships none of them, so one of the two describes another run`;
  }
  if (fault.kind === 'reassembly-differs') {
    return `writing the ledger's ${
      String(fault.shippedRows,)
    } shipped ${
      wordForCount({
        count: fault.shippedRows,
        one: 'row',
        many: 'rows',
      },)
    } over the archive produces a different document than the lane returned, so the ${
      wordForCount({
        count: fault.shippedRows,
        one: 'row does',
        many: 'rows do',
      },)
    } not say what the document carries`;
  }
  throw new UnhandledMemberInvariantError({
    what: 'delivery invariant fault',
    member: fault,
  },);
}

/**
 Failure of the delivery invariants: the ledger and the returned document
 disagree.

 MARKED: its message is the sentence `deliveryInvariantSentence` writes from
 slice indices and counts.

 @example
 ```ts
 throw new DeliveryInvariantError({ fault: { kind: 'unclaimed', indices: [4,], }, },);
 ```
 */
export class DeliveryInvariantError extends Error {
  /**
   Declares this message safe to forward: slice indices and counts in a
   sentence written here.
   */
  readonly messageNamesOnly: true = true;

  /**
   What the ledger and the document disagree about.
   */
  readonly fault: DeliveryInvariantFault;

  /**
   @param fault - what the two say that cannot both be true
   */
  public constructor({ fault, }: { readonly fault: DeliveryInvariantFault; },) {
    super(deliveryInvariantSentence({ fault, },),);
    this.name = 'DeliveryInvariantError';
    this.fault = fault;
  }
}

/**
 Names the indices one list holds and another does not.

 @param held - indices to check

 @param against - indices to check them against

 @returns Those of `held` that `against` does not name, in the order given

 @example
 ```ts
 const extra = indicesMissingFrom({ held: fromLedger, against: claimed, },);
 ```
 */
function indicesMissingFrom(
  {
    held,
    against,
  }: {
    readonly held: readonly number[];
    readonly against: readonly number[];
  },
): readonly number[] {
  /**
   Membership test for the list being checked against.
   */
  const known = new Set(against,);
  return held.filter(function isUnknown(index,): boolean {
    return !known.has(index,);
  },);
}

/**
 Checks a ledger against the document its lane returned.

 PASS THE RESULT'S OWN REPORTS, not the values a ledger was built from. Where
 the same index set is handed to both, the first claim holds by construction
 and costs a comparison; it is worth making anyway, because the case it exists
 for is a ledger read back from an artifact, or joined to the wrong result,
 and neither of those can be distinguished from a correct one by reading the
 rows alone. The second claim is not by construction in either case.

 @param ledger - delivery rows, one per prepared slice, in document order

 @param slices - preparation both the ledger and the document were built over

 @param incumbentText - archive's own translation, which the lane wrote into

 @param documentText - text that lane returned

 @param changedSliceIndices - slices that result names as carrying a change

 @throws {@link DeliveryInvariantError} when the rows marked shipped are not
 the slices the result names, or when writing those rows over the archive
 produces some other document

 @example
 ```ts
 assertDeliveryAgreesWithDocument({
   ledger,
   slices: prepared.slices,
   incumbentText: prepared.targetText,
   documentText: repair.repairedText,
   changedSliceIndices: repair.changedSliceIndices,
 },);
 ```
 */
export function assertDeliveryAgreesWithDocument(
  {
    ledger,
    slices,
    incumbentText,
    documentText,
    changedSliceIndices,
  }: {
    readonly ledger: readonly SliceDeliveryRecord[];
    readonly slices: readonly ChunkPair[];
    readonly incumbentText: string;
    readonly documentText: string;
    readonly changedSliceIndices: readonly number[];
  },
): void {
  /**
   Rows saying the document carries this slice's change, in document order.
   */
  const shipped = ledger.filter(function carriesAChange(record,): boolean {
    return record.delivery
      .kind
      === 'replacement-shipped';
  },);

  /**
   Slices those rows name.
   */
  const fromLedger = shipped.map(function toIndex(record,): number {
    return record.sliceIndex;
  },);

  /**
   Slices the result names, deduplicated so a repeated index is not read as
   a disagreement about membership.
   */
  const claimed = [...new Set(changedSliceIndices,),];

  /**
   Slices the ledger ships that the result does not name.
   */
  const unclaimed = indicesMissingFrom({
    held: fromLedger,
    against: claimed,
  },);

  /**
   Slices the result names that the ledger does not ship.
   */
  const unledgered = indicesMissingFrom({
    held: claimed,
    against: fromLedger,
  },);
  if (unclaimed.length > 0) {
    throw new DeliveryInvariantError({
      fault: {
        kind: 'unclaimed',
        indices: unclaimed,
      },
    },);
  }
  if (unledgered.length > 0) {
    throw new DeliveryInvariantError({
      fault: {
        kind: 'unledgered',
        indices: unledgered,
      },
    },);
  }

  /**
   Writes those rows describe.
   */
  const replacements = shipped.map(function toReplacement(record,): SliceReplacement {
    return {
      sliceIndex: record.sliceIndex,
      replacementText: record.shippedText,
    };
  },);

  /**
   Document the rows assemble to, computed here rather than trusted.

   Through the same assembly the lane used, which is what makes the comparison
   byte-exact: the separators around an anchored insertion are composed rather
   than carried by any row, so a concatenation of row texts would differ from
   the document while nothing was wrong.
   */
  const reassembled = spliceSlices({
    targetText: incumbentText,
    slices,
    replacements,
  },);
  if (reassembled !== documentText) {
    throw new DeliveryInvariantError({
      fault: {
        kind: 'reassembly-differs',
        shippedRows: replacements.length,
      },
    },);
  }
}

//endregion Delivery invariants
