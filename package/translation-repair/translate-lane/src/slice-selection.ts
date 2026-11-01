import type { CandidateProducer, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { TranslateSliceRecord, } from './translate-document-contract.ts';

//region Slice selection
// Who won each slice, and whether the document kept it.
//
// WHY A COUNT WAS NOT ENOUGH. The lane already reported how many slices changed
// and which indices shipped, and neither says who the text came from or how the
// judges got there. Every question anyone has asked of this lane since is per
// slice and per producer: how often the archive's English was kept, whether a
// producer favours its own rendering, whether ballot position moves a verdict.
// Recording who won each slice was asked for, and the judge-quality bench could
// only answer its half of those questions on a bench because no settled
// artifact carried it.
//
// TWO FACTS THAT COME APART, and keeping them apart is the point. "The incumbent
// won selection" is a decision the judges made. "The incumbent shipped
// unchanged" is what the document ended up carrying, and the assembly guard can
// withdraw a replacement the judges chose. A reader with one number cannot tell
// a slice the archive won from a slice where it lost and was reinstated.

/**
 What one slice decided, and what the document did with it.

 @example
 ```ts
 const selection: SliceSelection = { sliceIndex: 7, origin: 'fresh', decision: 'judged', voteWeight: 2, shipped: true, producer, };
 ```
 */
export type SliceSelection = {
  /**
   Slice this describes, in document order.
   */
  readonly sliceIndex: number;

  /**
   Whether the winning text was the archive's or freshly written.
   */
  readonly origin: string;

  /**
   Who wrote the winning text, carrying every model with a stake in it.
   */
  readonly producer: CandidateProducer;

  /**
   How the round ended: judged, sole candidate, no candidate, or which decline.
   */
  readonly decision: string;

  /**
   Weight the winner drew, zero on every path that did not reach a ballot.
   */
  readonly voteWeight: number;

  /**
   Whether the DOCUMENT carries this slice's decision.

   False on two different slices and the difference matters: one whose judges
   kept the archive, so there was never anything to ship, and one whose
   replacement the assembly guard withdrew. {@link SliceSelection.origin}
   separates them, which is why both fields are here.
   */
  readonly shipped: boolean;

  /**
   The WHOLE round: every candidate's provenance in slate order, and every
   ballot cast over it with its reason and weight.

   KEPT IN FULL RATHER THAN SUMMARISED, on the owner's instruction that disk
   is not the constraint. The summary fields (`origin`, `producer`, `decision`,
   `voteWeight`, `shipped`) answer what shipped; only the
   round answers who was asked and what they said, and three separate
   measurements need exactly that. `selfPreference` reads it as-is, which is
   why it is this shape: the artifact carries what the instrument consumes
   rather than something a reader has to reshape.

   THE REASONS ARE PART OF IT and are the bulk of the bytes. They earn it: the
   Kimi-K3 channel-marker defect and the `Dethelly` relocation were both found
   by reading what judges said rather than what they chose, and neither is
   recoverable from a tally.

   POSITION IS HERE TOO, without a field for it. `producers` is in slate
   order and a ballot names a one-based position into it, so a ballot's
   position and the candidate it fell on are both readable, which is what a
   position-bias measurement needs.
   */
  readonly round: SelectionRound;
};

/**
 Pairs every settled record with whether the document carries its decision.

 TAKES THE SHIPPED SET RATHER THAN RE-DERIVING IT, because the set is derived
 from the assembled bytes and a second derivation from the records would
 answer a different question: a record says what the slice CHOSE, and the
 document says what it CARRIES. Those disagree exactly where the assembly
 guard intervened, which is the case this ledger exists to make visible.

 @param records - settled slice records in document order

 @param changedSliceIndices - slices the assembled document carries a
 replacement for

 @returns One entry per record, in the order the records arrived

 @example
 ```ts
 const selections = buildSliceSelections({ records: settled, changedSliceIndices: ordered.shipped, },);
 ```
 */
export function buildSliceSelections(
  {
    records,
    changedSliceIndices,
  }: {
    readonly records: readonly TranslateSliceRecord[];
    readonly changedSliceIndices: readonly number[];
  },
): readonly SliceSelection[] {
  /**
   Shipped indices keyed for membership, since a document of any size makes
   repeated scans of an array the wrong shape.
   */
  const shipped = new Set(changedSliceIndices,);

  return records.map(function toSelection(record,): SliceSelection {
    return {
      sliceIndex: record.sliceIndex,
      origin: record.stageResult
        .origin,
      producer: record.stageResult
        .producer,
      decision: record.stageResult
        .decision,
      voteWeight: record.stageResult
        .voteWeight,
      shipped: shipped.has(record.sliceIndex,),
      round: {
        // SLATE ORDER, which is the order the judges saw and the order their
        // one-based ballot indices point into. Taking the producers off the
        // slate rather than off the candidate set is what keeps a ballot's
        // position meaningful: the slate is rotated per slice and the candidate
        // set is not.
        producers: record.stageResult
          .slate
          .map(function toProducer(entry,) {
            return entry.producer;
          },),
        ballots: record.stageResult
          .ballots,
      },
    };
  },);
}

//endregion Slice selection
