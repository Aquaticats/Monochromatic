import type { LaneSliceText, } from './lane-slice-text.ts';
import { SliceDeliveryError, } from './slice-delivery-fault.ts';
import { assertWordingCoherent, } from './wording-coherence.ts';

//region Slice delivery decision
// What one slice's delivery is, decided from its wording, the two index sets
// and whether the run was blocked. Moved out of `slice-delivery.ts`, which
// keeps the ledger row and its builder, when that file crossed the length
// limit.

/**
 What the returned document carries at one slice.

 ONE AXIS, and deliberately not the only one a record needs. This says what
 the DOCUMENT ends up with; {@link LaneSliceOutcome} says what the LANE did,
 and they are independent facts one word cannot hold. A repair lane blocked
 before an anchor never evaluated that slice AND leaves a gap there; the
 single vocabulary this replaced had to report one of those and lose the
 other.

 @example
 ```ts
 const delivery: SliceDelivery = { kind: 'replacement-shipped', };
 ```
 */
export type SliceDelivery = {
  /**
   Document carries what the lane decided, which differs from the archive.
   */
  readonly kind: 'replacement-shipped';
} | {
  /**
   Lane decided a replacement and the document does not carry it.
   */
  readonly kind: 'replacement-withdrawn';

  /**
   Which mechanism took it back.

   `assembly-integrity` is the guard, per slice, after splicing.
   `blocked-non-translation` is the whole-document refusal, which returns the
   archive untouched whatever any slice decided.
   */
  readonly reason: 'assembly-integrity' | 'blocked-non-translation';
} | {
  /**
   Document carries the archive's own wording for this slice.

   Says nothing about WHY, which is the outcome's job: the lane may have
   examined the slice and kept it, may never have reached it, or may have
   heard no voice at all. All three leave the same text in the document and
   mean three different things about the run.
   */
  readonly kind: 'incumbent-retained';
} | {
  /**
   Passage is MISSING from the document, and nothing could have kept it there:
   the archive holds no wording for this slice and this lane wrote none.
   */
  readonly kind: 'gap-remains';
};

/**
 What one slice's document carries, and the wording that is.

 @example
 ```ts
 const decision: DeliveryDecision = { delivery: { kind: 'replacement-shipped', }, documentText: 'The cat naps.', };
 ```
 */
export type DeliveryDecision = {
  /**
   What the document carries here, and by which route.
   */
  readonly delivery: SliceDelivery;

  /**
   Wording the document carries here before any trim the assembly guard made:
   the lane's decision where it shipped, the archive's own wording elsewhere.

   RETURNED BESIDE THE DELIVERY because only the branch deciding that a slice
   shipped holds the proof that its outcome is a decision. Reading the text
   afterwards took a helper that threw when the outcome was not one, which no
   slice could reach (T8's seventeenth batch).
   */
  readonly documentText: string;
};

/**
 Decides what one slice's document text is, from what the lane reported.

 READS THE LANE'S OWN OUTCOME. It used to infer this from whether the slice
 was an anchor, which is a fact about the PREPARATION and cannot say whether
 a lane ran: a repair lane blocked at an anchor was reported as reached and
 unfillable when nobody had looked at it.

 @param sliceIndex - slice being described

 @param wording - what the lane reported for it

 @param shipped - whether the document carries this slice's change

 @param withdrawn - whether the assembly guard took that change back

 @param blocked - whether the whole run refused before assembly

 @returns What the document carries here, by which route, and its wording

 @throws {@link WordingCoherenceError} when the record's outcome and archive
 contradict each other, by way of {@link assertWordingCoherent}

 @throws {@link SliceDeliveryError} when the reports contradict each other

 @example
 ```ts
 const { delivery, documentText, } = decideDelivery({ sliceIndex, wording, shipped, withdrawn, blocked, },);
 ```
 */
export function decideDelivery(
  {
    sliceIndex,
    wording,
    shipped,
    withdrawn,
    blocked,
  }: {
    readonly sliceIndex: number;
    readonly wording: LaneSliceText;
    readonly shipped: boolean;
    readonly withdrawn: boolean;
    readonly blocked: boolean;
  },
): DeliveryDecision {
  // THE TWO AXES AGAINST EACH OTHER, which no lane-against-preparation check
  // covers: those compare the lane record against the preparation, and this
  // compares the record against itself. `buildLaneSliceTexts` refuses all of
  // these while building, and a wording reaching here need not have come from
  // it. Checked HERE rather than by the caller, because this function's outcome
  // arms rely on it: at a slice the archive never translated, its wording is
  // empty and a decision of empty text is refused.
  assertWordingCoherent({ wording, },);
  if (wording.outcome
    .kind
    !== 'decided') {
    if (shipped || withdrawn) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'named-without-decision',
          sliceIndex,
          set: shipped ? 'shipped' : 'withdrawn',
        },
      },);
    }
    // WHAT THE DOCUMENT CARRIES, which the archive answers and the outcome does
    // not: an unreached slice and an unheard one both leave the incumbent
    // standing wherever there is one, and leave the gap wherever there is not.
    return {
      delivery: (wording.incumbentKind === 'absent')
        ? { kind: 'gap-remains', }
        : { kind: 'incumbent-retained', },
      documentText: wording.incumbentText,
    };
  }

  /**
   Whether the lane's decision moved off the archive at all.
   */
  const decided = wording.outcome
    .acceptedText
    !== wording.incumbentText;
  if (shipped) {
    if (!decided) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'shipped-archive-wording',
          sliceIndex,
        },
      },);
    }
    // A BLOCKED RUN RETURNS THE ARCHIVE UNTOUCHED, whatever any slice decided,
    // so no slice of one can be carrying a replacement. Accepting this pair
    // reported a change as shipped by a document that was never assembled.
    if (blocked) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'shipped-on-blocked',
          sliceIndex,
        },
      },);
    }
    return {
      delivery: { kind: 'replacement-shipped', },
      documentText: wording.outcome
        .acceptedText,
    };
  }
  if (withdrawn) {
    // ASSEMBLY NEVER RAN. The blocked exit returns the archive before anything
    // is assembled, so nothing there can have been taken back BY assembly, and
    // the two withdrawals are the events a reader counting integrity damage has
    // to tell apart. Naming both files a refusal under the guard's name.
    if (blocked) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'withdrawn-on-blocked',
          sliceIndex,
        },
      },);
    }

    // A WITHDRAWAL NEEDS SOMETHING TO WITHDRAW. Naming a slice whose decision
    // is the archive's own wording says assembly took back a replacement that
    // was never written, which reads downstream as a lane that tried and was
    // overruled rather than as one that left the slice alone.
    if (!decided) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'withdrawn-archive-wording',
          sliceIndex,
        },
      },);
    }
    return {
      delivery: {
        kind: 'replacement-withdrawn',
        reason: 'assembly-integrity',
      },
      documentText: wording.incumbentText,
    };
  }

  // THE ARCHIVE'S OWN WORDING STANDS. This asked whether the archive held any
  // wording here and reported a gap where it held none, but agreeing with an
  // archive that holds nothing means deciding empty text where the archive
  // never translated, which the coherence check refuses before this line, so
  // no slice reached that gap (T8's seventeenth batch).
  if (!decided) {
    return {
      delivery: { kind: 'incumbent-retained', },
      documentText: wording.incumbentText,
    };
  }
  if (blocked) {
    return {
      delivery: {
        kind: 'replacement-withdrawn',
        reason: 'blocked-non-translation',
      },
      documentText: wording.incumbentText,
    };
  }
  throw new SliceDeliveryError({
    fault: {
      kind: 'decided-unstated',
      sliceIndex,
    },
  },);
}

//endregion Slice delivery decision
