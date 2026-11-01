import { isDefinitionTrim, } from '@monochromatic-dev/translation-repair-assembly/ts';
import type {
  LaneSliceOutcome,
  SliceDeliveryRecord,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Delivery coherence
// The rule tying one row's DELIVERY to the rest of the row, for readers that
// did not build it.
//
// `buildSliceDelivery` decides these four cases and can only produce coherent
// ones. It is not the only way a row reaches a consumer: the type is exported,
// the lane comparison takes ledgers from a caller, and an artifact reader takes
// them from disk. At each of those the builder's guarantee is somebody else's
// assumption, and a row that contradicts itself is individually well formed in
// every field.
//
// So the four cases are stated once here as a rule, and every boundary that
// accepts a row rather than building one asserts it.

/**
 What a delivery row claims that cannot all be true.

 @example
 ```ts
 const fault: DeliveryCoherenceFault = { kind: 'gap-with-wording', };
 ```
 */
export type DeliveryCoherenceFault = {
  /**
   Row reports a replacement while its outcome is not a decision.
   */
  readonly kind: 'replacement-without-decision';

  /**
   Outcome the row reports instead.
   */
  readonly outcomeKind: LaneSliceOutcome['kind'];
} | {
  /**
   Row reports a replacement whose wording is the archive's own.
   */
  readonly kind: 'replacement-of-archive-wording';
} | {
  /**
   Row's text is not the one its delivery says it carries.
   */
  readonly kind: 'delivery-text-mismatch';
} | {
  /**
   Row decided wording of its own and reports the document unchanged.
   */
  readonly kind: 'hidden-decision';
} | {
  /**
   Row reports the archive's wording retained where the archive holds none.
   */
  readonly kind: 'retained-without-archive';
} | {
  /**
   Row reports the archive's wording retained and carries different text.
   */
  readonly kind: 'retained-differs';
} | {
  /**
   Row reports a gap where the archive holds wording.
   */
  readonly kind: 'gap-with-archive';
} | {
  /**
   Row reports a gap and carries wording anyway.
   */
  readonly kind: 'gap-with-wording';
};

/**
 Sentence for each fault that carries no field, keyed by kind.
 */
const COHERENCE_SENTENCES: Record<Exclude<DeliveryCoherenceFault['kind'], 'replacement-without-decision'>, string> = {
  'replacement-of-archive-wording': "reports a replacement whose wording is the archive's own, so nothing was replaced",
  'delivery-text-mismatch': 'reports a delivery whose text is not the one that delivery carries',
  'hidden-decision': 'decided wording of its own and reports the document unchanged, with nothing saying what '
    + 'took the decision back',
  'retained-without-archive': "reports the archive's wording retained where the archive holds none",
  'retained-differs': "reports the archive's wording retained and carries different text",
  'gap-with-archive': 'reports a gap where the archive holds wording',
  'gap-with-wording': 'reports a gap and carries wording anyway',
};

/**
 Words a coherence fault, after the slice the class prefixes.

 @param fault - what the row claims that cannot all be true

 @returns Sentence written here, naming at most an outcome kind

 @example
 ```ts
 const sentence = coherenceSentence({ fault: { kind: 'gap-with-wording', }, },);
 ```
 */
export function coherenceSentence({ fault, }: { readonly fault: DeliveryCoherenceFault; },): string {
  if (fault.kind === 'replacement-without-decision')
    return `reports a replacement and an outcome of ${
      fault.outcomeKind
    }, so there is no decision for the delivery to describe`;
  return COHERENCE_SENTENCES[fault.kind];
}

/**
 Refusal of a delivery row that contradicts itself.

 MARKED: its message is a slice index and the sentence `coherenceSentence`
 writes from a fault kind and, at most, an outcome kind.

 @example
 ```ts
 throw new DeliveryCoherenceError({ sliceIndex: 4, fault: { kind: 'gap-with-wording', }, },);
 ```
 */
export class DeliveryCoherenceError extends Error {
  /**
   Declares this message safe to forward: a slice index and kinds, in a
   sentence written here.
   */
  readonly messageNamesOnly: true = true;

  /**
   Slice whose row contradicts itself.
   */
  readonly sliceIndex: number;

  /**
   What the row claims that cannot all be true.
   */
  readonly fault: DeliveryCoherenceFault;

  /**
   @param sliceIndex - slice whose row contradicts itself

   @param fault - what the row claims that cannot all be true
   */
  constructor(
    {
      sliceIndex,
      fault,
    }: {
      readonly sliceIndex: number;
      readonly fault: DeliveryCoherenceFault;
    },
  ) {
    super(`slice ${String(sliceIndex,)} ${coherenceSentence({ fault, },)}`,);
    this.name = 'DeliveryCoherenceError';
    this.sliceIndex = sliceIndex;
    this.fault = fault;
  }
}

/**
 Refuses a shipped or withdrawn row whose decision cannot support it.

 Both cases need the same two things and differ only in which text the
 document ends up with, so they are checked together rather than twice.

 @param record - row being checked

 @param sliceIndex - slice whose row is checked

 @param carries - which text this delivery leaves the document with

 @throws {@link DeliveryCoherenceError} when the row decided nothing, decided
 the archive's own wording, or carries text the delivery does not allow

 @example
 ```ts
 assertReplacementRow({ record, sliceIndex, carries: 'incumbent', },);
 ```
 */
function assertReplacementRow(
  {
    record,
    sliceIndex,
    carries,
  }: {
    readonly record: SliceDeliveryRecord;
    readonly sliceIndex: number;
    readonly carries: 'accepted' | 'incumbent';
  },
): void {
  if (record.outcome
    .kind
    !== 'decided') {
    throw new DeliveryCoherenceError({
      sliceIndex,
      fault: {
        kind: 'replacement-without-decision',
        outcomeKind: record.outcome
          .kind,
      },
    },);
  }
  if (record.outcome
    .acceptedText
    === record.incumbentText) {
    throw new DeliveryCoherenceError({
      sliceIndex,
      fault: { kind: 'replacement-of-archive-wording', },
    },);
  }
  /**
   Text this delivery leaves the document with, which the
   `replacement-without-decision` check proves the decision can supply.
   */
  const expected = (carries === 'accepted')
    ? record.outcome
      .acceptedText
    : record.incumbentText;
  // A SHIPPED ROW MAY CARRY ITS DECISION TRIMMED: the assembly guard cuts an
  // orphan definition block out of a definitions-only replacement and lets the
  // rest stand, so the document carries the decision with that block gone and
  // nothing else changed. Any other difference is still a mismatch.
  if ((carries === 'accepted') && isDefinitionTrim({
    decided: expected,
    carried: record.shippedText,
  },))
    return;
  if (record.shippedText !== expected) {
    throw new DeliveryCoherenceError({
      sliceIndex,
      fault: { kind: 'delivery-text-mismatch', },
    },);
  }
}

/**
 Refuses a row that keeps the archive while hiding a decision to change it.

 @param record - row being checked

 @param sliceIndex - slice whose row is checked

 @throws {@link DeliveryCoherenceError} when a changed decision sits behind an
 unchanged document with nothing saying what took it back

 @example
 ```ts
 assertNothingHidden({ record, sliceIndex, },);
 ```
 */
function assertNothingHidden(
  {
    record,
    sliceIndex,
  }: {
    readonly record: SliceDeliveryRecord;
    readonly sliceIndex: number;
  },
): void {
  /**
   What the lane decided here, if anything.
   */
  const { outcome, } = record;
  if ((outcome.kind === 'decided') && (outcome.acceptedText !== record.incumbentText)) {
    throw new DeliveryCoherenceError({
      sliceIndex,
      fault: { kind: 'hidden-decision', },
    },);
  }
}

/**
 Refuses a delivery row whose four fields cannot describe one slice.

 @param record - one row of a delivery ledger, from wherever it came

 @throws {@link DeliveryCoherenceError} when the delivery, the outcome, the
 archive state and the shipped text cannot all be true at once

 @example
 ```ts
 assertDeliveryCoherent({ record, },);
 ```
 */
export function assertDeliveryCoherent(
  { record, }: { readonly record: SliceDeliveryRecord; },
): void {
  /**
   Slice this record describes.
   */
  const { sliceIndex, } = record;

  /**
   What the document ended up with here.
   */
  const { delivery, } = record;
  if (delivery.kind === 'replacement-shipped') {
    assertReplacementRow({
      record,
      sliceIndex,
      carries: 'accepted',
    },);
    return;
  }
  if (delivery.kind === 'replacement-withdrawn') {
    assertReplacementRow({
      record,
      sliceIndex,
      carries: 'incumbent',
    },);
    return;
  }
  if (delivery.kind === 'incumbent-retained') {
    if (record.incumbentKind === 'absent') {
      throw new DeliveryCoherenceError({
        sliceIndex,
        fault: { kind: 'retained-without-archive', },
      },);
    }
    if (record.shippedText !== record.incumbentText) {
      throw new DeliveryCoherenceError({
        sliceIndex,
        fault: { kind: 'retained-differs', },
      },);
    }
    assertNothingHidden({
      record,
      sliceIndex,
    },);
    return;
  }
  if (record.incumbentKind === 'present') {
    throw new DeliveryCoherenceError({
      sliceIndex,
      fault: { kind: 'gap-with-archive', },
    },);
  }
  if ((record.incumbentText !== '') || (record.shippedText !== '')) {
    throw new DeliveryCoherenceError({
      sliceIndex,
      fault: { kind: 'gap-with-wording', },
    },);
  }
  assertNothingHidden({
    record,
    sliceIndex,
  },);
}

//endregion Delivery coherence
