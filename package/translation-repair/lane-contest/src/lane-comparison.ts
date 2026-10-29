import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { PreparationIdentity, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { assertDeliveryCoherent, } from './delivery-coherence.ts';
import {
  comparisonSentence,
  type LaneComparisonFault,
} from './lane-comparison-fault.ts';
import {
  assertWordingCoherent,
  type LaneSliceOutcome,
  type SliceDelivery,
  type SliceDeliveryRecord,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Lane comparison
// What the two lanes did to the SAME slice, which is the question running both
// of them over one preparation exists to answer.
//
// Computed here rather than in the combined driver on purpose. The driver
// returns both documents and arbitrates nothing, and that property is what its
// tests pin; a comparison is not arbitration, but a driver that computed one
// would be the obvious place to later put a winner. Anyone who wants the
// comparison asks for it.
//
// BUILT FROM THE DELIVERY LEDGER, not from wordings and an index set. The two
// disagree exactly where this comparison is most worth reading, and reading an
// index set here meant a shipped index left out of it was indistinguishable
// from a lane that kept the archive: an omission compared as a deliberate keep,
// which is the defect class this whole area sits downstream of. The ledger has
// already refused a decided slice that is neither shipped, withdrawn, nor
// blocked, so what each document carries arrives as a stated fact.

/**
 What the two lanes did to one slice.

 @example
 ```ts
 const verdict: SliceLaneVerdict = 'both-differ';
 ```
 */
export type SliceLaneVerdict =
  /**
   Neither document carries a change: the archive's own English stands in
   both, whether because both lanes left it alone or because the assembly
   guard took both replacements back.
   */
  | 'archive-stands'
  /**
   Repair's document changed this slice and translate's did not.
   */
    | 'repair-only'
  /**
   Translate's document changed this slice and repair's did not.
   */
    | 'translate-only'
  /**
   Both documents changed this slice to the SAME wording, character for
   character.
   */
    | 'both-agree'
  /**
   Both documents changed this slice and the two wordings differ, which is the
   case a human has to read.
   */
    | 'both-differ'
  /**
   Neither document carries anything here, and the archive never did either:
   the passage is still MISSING.

   Split from `archive-stands` because that word asserts a translation is
   being kept, and at a slice the archive never translated there is none to
   keep. Every anchor neither lane filled read as the archive standing until
   2026-08-16.
   */
    | 'gap-remains';

/**
 Whether the two lanes' own decisions can be compared on a slice, and how they
 came out.

 NOT DERIVABLE FROM THE VERDICT, which describes the two documents. A slice
 neither lane could decide is not a slice where they agreed.

 @example
 ```ts
 const comparison: DecisionComparison = { kind: 'comparable', verdict: 'same', };
 ```
 */
export type DecisionComparison = {
  /**
   Both lanes decided a wording, so the two are comparable.
   */
  readonly kind: 'comparable';

  /**
   Whether those wordings are the same, character for character.
   */
  readonly verdict: 'same' | 'different';
} | {
  /**
   At least one lane decided nothing here.
   */
  readonly kind: 'not-comparable';

  /**
   Lanes that decided nothing, in lane order, and BOTH of them when neither
   did.

   A single free-text reason named only the first lane checked, so a slice
   neither lane decided read as the repair lane's fault alone. What each lane
   did instead is stated on the row itself.
   */
  readonly undecidedLanes: readonly ('repair' | 'translate')[];
};

/**
 One slice as both lanes left it.

 @example
 ```ts
 const row: SliceLaneComparison = { sliceIndex: 3, verdict: 'both-differ', ... };
 ```
 */
export type SliceLaneComparison = {
  /**
   Global slice index both lanes name it by.
   */
  readonly sliceIndex: number;

  /**
   Whether the archive holds any wording at this slice at all.
   */
  readonly incumbentKind: 'present' | 'absent';

  /**
   Archive's own English for this slice.
   */
  readonly incumbentText: string;

  /**
   Wording the repair document CARRIES, which is the incumbent wherever that
   lane changed nothing or had its change withdrawn.
   */
  readonly repairText: string;

  /**
   Wording the translate document CARRIES, on the same rule.
   */
  readonly translateText: string;

  /**
   How the two documents relate on this slice.
   */
  readonly verdict: SliceLaneVerdict;

  /**
   What the repair lane did about this slice.

   THE OUTCOME RATHER THAN A REACHED FLAG. Both documents carrying the archive
   wording says nothing about whether anyone looked, and a boolean says only
   that somebody did: it cannot separate a lane that examined the slice and
   kept it from one that reached it and could not fill it, or from one that
   heard no voice at all. A pair of booleans recorded the second of those as
   nobody having looked.
   */
  readonly repairOutcome: LaneSliceOutcome;

  /**
   What the translate lane did about this slice.
   */
  readonly translateOutcome: LaneSliceOutcome;

  /**
   Whether the two lanes' own decisions were comparable here, and how they
   came out, which the delivery verdict cannot say.
   */
  readonly decisionComparison: DecisionComparison;

  /**
   How the repair document came to carry what it carries.
   */
  readonly repairDelivery: SliceDelivery;

  /**
   How the translate document came to carry what it carries.
   */
  readonly translateDelivery: SliceDelivery;
};

/**
 One lane's delivery ledger, carrying the slicing it was built over.

 THE IDENTITY TRAVELS WITH THE LEDGER rather than being supplied once beside
 both. Two ledgers loaded from different artifacts are exactly the pair a
 comparison must refuse, and a single identity passed in by the caller is the
 caller's claim about them rather than either ledger's own.

 @example
 ```ts
 const ledger: IdentifiedDeliveryLedger = { preparationIdentity, records, };
 ```
 */
export type IdentifiedDeliveryLedger = {
  /**
   Slicing this ledger's rows were built over.
   */
  readonly preparationIdentity: PreparationIdentity;

  /**
   One row per prepared slice, in document order.
   */
  readonly records: readonly SliceDeliveryRecord[];
};

/**
 Every slice as both lanes left it, bound to the slicing it describes.

 THE BINDING IS PART OF THE VALUE rather than something a writer remembers to
 record beside it. Rows joined on a slice index mean nothing without the
 slicing that numbered them, and a comparison persisted without it can be read
 against a later preparation of the same entry with every row still looking
 well formed.

 @example
 ```ts
 const comparison: LaneComparison = compareDocumentLanes({ preparationIdentity, repair, translate, },);
 ```
 */
export type LaneComparison = {
  /**
   Slicing both lanes ran over, which every row is joined on.
   */
  readonly preparationIdentity: PreparationIdentity;

  /**
   One row per slice, in the order the repair ledger reports them.
   */
  readonly slices: readonly SliceLaneComparison[];
};

/**
 Refusal to compare two ledgers that were not built over one preparation.

 MARKED: its message is the sentence `comparisonSentence` writes from lane
 names, kinds and numbers.

 @example
 ```ts
 throw new LaneComparisonError({ fault: { kind: 'different-slicings', }, },);
 ```
 */
export class LaneComparisonError extends Error {
  /**
   Declares this message safe to forward: lane names, kinds and numbers in
   a sentence written here.
   */
  readonly messageNamesOnly: true = true;

  /**
   Why the ledgers cannot be compared.
   */
  readonly fault: LaneComparisonFault;

  /**
   @param fault - why the ledgers cannot be compared
   */
  constructor({ fault, }: { readonly fault: LaneComparisonFault; },) {
    super(comparisonSentence({ fault, },),);
    this.name = 'LaneComparisonError';
    this.fault = fault;
  }
}

/**
 Names how the two lanes' DECISIONS relate, where both made one.

 SEPARATE FROM THE DELIVERY VERDICT because they answer different questions
 and this file used to answer only the second while claiming the first. Two
 lanes accepting different replacements that are both withdrawn deliver the
 same document and disagree completely; two lanes accepting the same wording
 where only one ships deliver differently and agree exactly.

 @param repair - what the repair lane did with this slice

 @param translate - what the translate lane did

 @returns Whether the two decisions can be compared, and how they came out

 @example
 ```ts
 const comparison = compareDecisions({ repair, translate, },);
 ```
 */
function compareDecisions(
  {
    repair,
    translate,
  }: {
    readonly repair: LaneSliceOutcome;
    readonly translate: LaneSliceOutcome;
  },
): DecisionComparison {
  /**
   Lanes with no wording to compare, collected rather than returned at the
   first one found: a slice neither lane decided is a fact about both.
   */
  const undecidedLanes = [
    ...(repair.kind === 'decided') ? [] : ['repair',] as const,
    ...(translate.kind === 'decided') ? [] : ['translate',] as const,
  ];
  if ((repair.kind !== 'decided') || (translate.kind !== 'decided'))
    return {
      kind: 'not-comparable',
      undecidedLanes,
    };
  return {
    kind: 'comparable',
    verdict: (repair.acceptedText === translate.acceptedText) ? 'same' : 'different',
  };
}

/**
 Names how one slice's two carried wordings relate.

 @param repairText - wording the repair document carries

 @param translateText - wording the translate document carries

 @param incumbentKind - whether the archive holds any wording here, which is
 what separates its wording standing from a passage still missing

 @param incumbentText - archive wording both fall back to

 @returns Verdict for this slice

 @example
 ```ts
 const verdict = judgeSlice({ repairText, translateText, incumbentKind, incumbentText, },);
 ```
 */
function judgeSlice(
  {
    repairText,
    translateText,
    incumbentKind,
    incumbentText,
  }: {
    readonly repairText: string;
    readonly translateText: string;
    readonly incumbentKind: 'present' | 'absent';
    readonly incumbentText: string;
  },
): SliceLaneVerdict {
  /**
   Whether the repair document moved off the archive wording.
   */
  const repairMoved = repairText !== incumbentText;

  /**
   Whether the translate document did.
   */
  const translateMoved = translateText !== incumbentText;
  if (repairMoved && translateMoved)
    return (repairText === translateText) ? 'both-agree' : 'both-differ';
  if (repairMoved)
    return 'repair-only';
  if (translateMoved)
    return 'translate-only';

  // NEITHER DOCUMENT MOVED, which means the archive's wording stands only where
  // the archive HAS wording. At an anchor it means the passage is still
  // missing, and reporting that as the archive standing told a grader a
  // translation was being kept where none has ever existed.
  return (incumbentKind === 'absent') ? 'gap-remains' : 'archive-stands';
}

/**
 Compares what two lanes' documents carry, slice by slice.

 @param repair - repair lane's delivery ledger with the slicing it was built
 over, already checked against that lane's own document

 @param translate - translate lane's, which must name the SAME slicing

 @returns One row per slice, in the order the REPAIR ledger reports them,
 which is document order wherever that lane built it from a preparation and is
 not re-sorted here

 @throws {@link LaneComparisonError} when the two name different slicings,
 when either ledger repeats a slice, when they disagree about which slice sits
 at a position, or when they disagree about a slice's original, its archive
 wording, or whether the archive translates it at all

 @example
 ```ts
 const comparison = compareDocumentLanes({ repair, translate, },);
 ```
 */
export function compareDocumentLanes(
  {
    repair,
    translate,
  }: {
    readonly repair: IdentifiedDeliveryLedger;
    readonly translate: IdentifiedDeliveryLedger;
  },
): LaneComparison {
  // THE FIRST THING CHECKED, because every other step of this comparison joins on a slice index
  // and an index means nothing without the slicing that issued it. Two ledgers
  // from different artifacts of the same entry line up perfectly and describe
  // different passages.
  if (repair.preparationIdentity !== translate.preparationIdentity)
    throw new LaneComparisonError({ fault: { kind: 'different-slicings', }, },);

  /**
   Slicing both ledgers name, now proven to be one.
   */
  const { preparationIdentity, } = repair;

  /**
   Rows each ledger holds, named once so every count this function takes reads
   as one step.
   */
  const { records: repairRecords, } = repair;

  /**
   Rows the translate ledger holds.
   */
  const { records: translateRecords, } = translate;
  if (repairRecords.length !== translateRecords.length)
    throw new LaneComparisonError({
      fault: {
        kind: 'slice-counts-differ',
        repair: repairRecords.length,
        translate: translateRecords.length,
      },
    },);

  /**
   Translate row for each slice index.
   */
  const translateByIndex = new Map(translateRecords.map(function toEntry(record,): [
    number,
    SliceDeliveryRecord,
  ] {
    return [
      record.sliceIndex,
      record,
    ];
  },),);

  // Equal lengths are not equal COVERAGE: repair rows for slices 1 and 1 and
  // translate rows for 1 and 2 both count two, and the join would then emit two
  // rows for slice 1 and silently drop slice 2. BOTH sides are checked, because
  // the join walks the repair rows and looks the translate ones up, so a repeat
  // on either side produces that same wrong answer from the other end.
  if (translateByIndex.size !== translateRecords.length)
    throw new LaneComparisonError({
      fault: {
        kind: 'rows-repeat',
        lane: 'translate',
        rows: translateRecords.length,
        distinct: translateByIndex.size,
      },
    },);

  /**
   Distinct slices the repair rows name.
   */
  const repairDistinct = new Set(repairRecords.map(function toIndex(record,): number {
    return record.sliceIndex;
  },),).size;
  if (repairDistinct !== repairRecords.length)
    throw new LaneComparisonError({
      fault: {
        kind: 'rows-repeat',
        lane: 'repair',
        rows: repairRecords.length,
        distinct: repairDistinct,
      },
    },);
  return {
    preparationIdentity,
    slices: repairRecords.map(function toRow(
      mine,
      position,
    ): SliceLaneComparison {
      /**
       Same slice as the other lane left it.
       */
      const theirs = translateByIndex.get(mine.sliceIndex,);
      if (theirs === undefined)
        throw new LaneComparisonError({
          fault: {
            kind: 'missing-from-translate',
            sliceIndex: mine.sliceIndex,
          },
        },);
      /**
       Row the other ledger holds at this POSITION, as against the one it
       holds for this index: a ledger built over one preparation has them in
       the same order, and two that do not were not built over one. The two
       hold the same number of rows, so there is one.
       */
      const alongside = nonNullishOrThrow(translateRecords[position],);
      if (theirs.sliceIndex !== alongside.sliceIndex)
        throw new LaneComparisonError({
          fault: {
            kind: 'position-differs',
            sliceIndex: mine.sliceIndex,
            position,
          },
        },);
      if (theirs.sourceText !== mine.sourceText)
        throw new LaneComparisonError({
          fault: {
            kind: 'source-differs',
            sliceIndex: mine.sliceIndex,
          },
        },);
      if (theirs.incumbentText !== mine.incumbentText)
        throw new LaneComparisonError({
          fault: {
            kind: 'incumbent-differs',
            sliceIndex: mine.sliceIndex,
          },
        },);

      // TEXT IS NOT ENOUGH. A blank content slice and a place the archive never
      // translated both carry the empty string, so equal text leaves exactly
      // the pair this comparison must not confuse still equal. Every row's kind
      // was taken from the repair lane, which decided the gap verdict for both.
      if (theirs.incumbentKind !== mine.incumbentKind)
        throw new LaneComparisonError({
          fault: {
            kind: 'incumbent-kind-differs',
            sliceIndex: mine.sliceIndex,
            repair: mine.incumbentKind,
            translate: theirs.incumbentKind,
          },
        },);
      assertWordingCoherent({ wording: mine, },);
      assertWordingCoherent({ wording: theirs, },);

      // AND THE DELIVERY AGAINST THE REST OF ITS ROW. A record reaching here is
      // a structural type rather than proof that `buildSliceDelivery` made it,
      // and a row shipping wording it never decided is well formed in every
      // field on its own.
      assertDeliveryCoherent({ record: mine, },);
      assertDeliveryCoherent({ record: theirs, },);
      return {
        sliceIndex: mine.sliceIndex,
        incumbentKind: mine.incumbentKind,
        incumbentText: mine.incumbentText,

        // WHAT EACH DOCUMENT CARRIES, stated by the ledger rather than inferred
        // from membership of an index set. An omitted shipped index used to
        // read here as the lane having kept the archive, which is an absence
        // read as a choice.
        repairText: mine.shippedText,
        translateText: theirs.shippedText,

        // THE OUTCOMES THEMSELVES, not two booleans derived from them. A
        // boolean pair answers whether anyone looked and erases the difference
        // between a lane that decided, one that reached the slice and could not
        // fill it, one that heard no voice at all, and one with no work to do.
        repairOutcome: mine.outcome,
        translateOutcome: theirs.outcome,
        verdict: judgeSlice({
          repairText: mine.shippedText,
          translateText: theirs.shippedText,
          incumbentKind: mine.incumbentKind,
          incumbentText: mine.incumbentText,
        },),
        decisionComparison: compareDecisions({
          repair: mine.outcome,
          translate: theirs.outcome,
        },),
        repairDelivery: mine.delivery,
        translateDelivery: theirs.delivery,
      };
    },),
  };
}

//endregion Lane comparison
