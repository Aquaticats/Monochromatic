import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type {
  LaneSliceOutcome,
  LaneSliceText,
} from './lane-slice-text.ts';
import {
  decideDelivery,
  type SliceDelivery,
} from './slice-delivery-decide.ts';
import {
  type DeliverySetName,
  SliceDeliveryError,
} from './slice-delivery-fault.ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';

/**
 No shipped slice trimmed, which is every lane whose guard cut nothing.
 */
const NO_TRIMS: readonly SliceReplacement[] = [];

// DECLARED IN SIBLINGS and re-exported here, so every caller of the builder
// keeps one import: the delivery shape and its decision live in
// `slice-delivery-decide.ts`, the refusal with its faults in
// `slice-delivery-fault.ts`, and this file holds the ledger row and its builder.
export { type SliceDelivery, } from './slice-delivery-decide.ts';
export { SliceDeliveryError, } from './slice-delivery-fault.ts';

//region Slice delivery
// What became of every slice, in one row: the original, the archive's English,
// what the lane decided, what the document ends up carrying, and WHY those last
// two are what they are.
//
// The lane results already carry the pieces, and a reader cannot join them. A
// wording says what a lane decided and says nothing about whether the document
// kept it; an index set says which slices the document carries a change for and
// nothing about the wording; the source text is in neither, so a grader holding
// an artifact cannot see what was being translated without re-preparing the
// corpus at the right commit with the right budget.
//
// DELIVERY IS ITS OWN VOCABULARY, deliberately not reusing either lane's. A
// repair disposition is the fate of ONE ISSUE's targeted edit, and a translate
// disposition is a slice-local acceptance decision; both can hold several
// values inside one slice whose text ships once. This names the fate of the
// slice TEXT and nothing else.
//
// TWO AXES, since 2026-08-16, because one word could not carry both. What the
// LANE did is its outcome and what the DOCUMENT carries is its delivery, and a
// row states each separately: a run blocked before an anchor never evaluated
// that slice and also leaves a gap there, and the single vocabulary this
// replaced had to report one of those facts and drop the other.

/**
 One slice as a grader needs to read it.

 @example
 ```ts
 const record: SliceDeliveryRecord = {
   sliceIndex: 3,
   sourceText: '猫猫在睡觉。',
   incumbentKind: 'present',
   incumbentText: 'The cat sleeps.',
   outcome: { kind: 'decided', acceptedText: 'The cat is asleep.', },
   shippedText: 'The cat is asleep.',
   delivery: { kind: 'replacement-shipped', },
 };
 ```
 */
export type SliceDeliveryRecord = {
  /**
   Global slice index, which every other per-slice record names it by.
   */
  readonly sliceIndex: number;

  /**
   Original this slice was translated from.

   Stored as TEXT rather than as offsets, which is not a preference: the
   artifact keeps neither document, only their lengths, so an offset would
   need a matching corpus checkout, matching preparation code and a matching
   slice budget before it meant anything, and could never recover an accepted
   wording that did not ship.
   */
  readonly sourceText: string;

  /**
   Whether the archive holds any wording at this slice at all, carried through
   from the preparation so a reader need not guess it from blank text.
   */
  readonly incumbentKind: 'present' | 'absent';

  /**
   Archive's own English for this slice.
   */
  readonly incumbentText: string;

  /**
   What the LANE did about this slice, exactly as the lane reported it.
   */
  readonly outcome: LaneSliceOutcome;

  /**
   Wording the returned document carries for this slice.

   Empty where {@link SliceDeliveryRecord.delivery} says the gap remains,
   which is why that field and not this one answers whether a passage is
   missing: an empty translation and no translation are the same string.
   */
  readonly shippedText: string;

  /**
   What the DOCUMENT ends up carrying, and by which route.
   */
  readonly delivery: SliceDelivery;
};

/**
 Refuses an index set that names one slice more than once.

 TAKES BOTH THE ARRAY AND THE SET rather than deriving the second here, so the
 caller's own deduplicated set is what the length is compared against. Building
 a second set to check the first would be checking this function's work instead
 of the work that matters.

 @param indices - index set as the lane reported it

 @param unique - same indices deduplicated, which the caller already built

 @param named - which set this is, for the message

 @throws {@link SliceDeliveryError} when a slice is named twice

 @example
 ```ts
 assertNoRepeat({ indices: changedSliceIndices, unique: shipped, named: 'shipped', },);
 ```
 */
function assertNoRepeat(
  {
    indices,
    unique,
    named,
  }: {
    readonly indices: readonly number[];
    readonly unique: ReadonlySet<number>;
    readonly named: DeliverySetName;
  },
): void {
  if (indices.length === unique.size)
    return;
  throw new SliceDeliveryError({
    fault: {
      kind: 'set-repeats',
      set: named,
      named: indices.length,
      distinct: unique.size,
    },
  },);
}

/**
 Builds one delivery record per prepared slice.

 BUILT FROM WHAT THE LANE REPORTED rather than recomputed beside it. Every
 field here is a join of the preparation, the lane's per-slice wordings and
 its two index sets, so a ledger that disagrees with the returned document is
 a contradiction inside one result rather than two independent derivations
 drifting apart.

 @param slices - prepared slice pairs, which supply the original

 @param wordings - what the lane decided per slice

 @param changedSliceIndices - slices the returned document carries a change
 for

 @param withdrawnSliceIndices - slices whose change the assembly guard took
 back

 @param trimmedReplacements - shipped slices whose text the assembly guard
 trimmed, with the text the document carries; a shipped row reads its text
 here before it reads the decision

 @param blocked - whether the run refused the whole document before assembly,
 which makes an unshipped decision a withdrawal rather than a contradiction

 @returns One record per prepared slice, in document order

 @throws {@link SliceDeliveryError} when the wordings do not cover the
 preparation one for one, when an index set names a slice twice or names one
 the preparation never produced, when two trimmed replacements name one
 slice, or when a slice's reports contradict

 @throws {@link WordingCoherenceError} when a slice's record contradicts
 itself, by way of {@link decideDelivery}

 @example
 ```ts
 const ledger = buildSliceDelivery({ slices, wordings, changedSliceIndices, withdrawnSliceIndices, blocked, },);
 ```
 */
export function buildSliceDelivery(
  {
    slices,
    wordings,
    changedSliceIndices,
    withdrawnSliceIndices,
    trimmedReplacements = NO_TRIMS,
    blocked,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly wordings: readonly LaneSliceText[];
    readonly changedSliceIndices: readonly number[];
    readonly withdrawnSliceIndices: readonly number[];
    readonly trimmedReplacements?: readonly SliceReplacement[];
    readonly blocked: boolean;
  },
): readonly SliceDeliveryRecord[] {
  if (wordings.length !== slices.length) {
    throw new SliceDeliveryError({
      fault: {
        kind: 'wording-count',
        wordings: wordings.length,
        slices: slices.length,
      },
    },);
  }

  /**
   Slices the document carries a change for.
   */
  const shipped = new Set(changedSliceIndices,);

  /**
   Slices whose change was taken back.
   */
  const withdrawn = new Set(withdrawnSliceIndices,);

  // CHECKED AGAINST THE ARRAYS, not the sets built from them. Building a set is
  // what makes a repeated index disappear, so a check that reads the set is
  // asking a question whose answer it has already thrown away, and this
  // function's own contract promised the throw while the sets quietly deduped.
  // A lane naming one slice twice has two derivations that disagree about how
  // many slices it changed, and neither the count nor the ledger would show it.
  assertNoRepeat({
    indices: changedSliceIndices,
    unique: shipped,
    named: 'shipped',
  },);
  assertNoRepeat({
    indices: withdrawnSliceIndices,
    unique: withdrawn,
    named: 'withdrawn',
  },);

  for (const sliceIndex of shipped) {
    // SHIPPED AND WITHDRAWN AT ONCE is a contradiction rather than a precedence
    // question. The branch order used to answer it silently, in shipped's
    // favour, which reported a change the assembly guard had taken back as one
    // the document carries.
    if (withdrawn.has(sliceIndex,)) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'both-shipped-and-withdrawn',
          sliceIndex,
        },
      },);
    }
  }
  /**
   Slices a trimmed replacement has named so far.
   */
  const trimmedSlices = new Set<number>();
  for (const replacement of trimmedReplacements) {
    // ONE TRIM PER SLICE, CHECKED AGAINST THE LIST for the reason the two
    // index sets are: keying the texts by slice is what makes a second
    // replacement for one slice disappear, and the ledger would report the
    // later text as what the document carries with nothing here knowing which
    // the assembly wrote. The guard that trims hands over rows the splice
    // already held to one per slice, which is a fact about that caller and
    // not about this signature.
    if (trimmedSlices.has(replacement.sliceIndex,)) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'trim-repeats',
          sliceIndex: replacement.sliceIndex,
        },
      },);
    }
    trimmedSlices.add(replacement.sliceIndex,);
  }
  /**
   Text the document carries at each trimmed slice, by slice.
   */
  const trimmedText = new Map(trimmedReplacements.map(function toEntry(replacement,) {
    return [
      replacement.sliceIndex,
      replacement.replacementText,
    ] as const;
  },),);
  for (const sliceIndex of trimmedText.keys()) {
    // A TRIM IS A FACT ABOUT A SHIPPED SLICE, since the guard trims what it
    // lets stand; a trim naming any other slice describes text the document
    // does not carry.
    if (!shipped.has(sliceIndex,)) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'trim-names-unshipped',
          sliceIndex,
        },
      },);
    }
  }

  /**
   Indices the preparation actually produced.

   MEMBERSHIP, not a numeric range. A range check assumes the prepared indices
   are exactly `0` to `length - 1`, which is a property of today's stamping
   rather than a contract, so a renumbered preparation would let an index that
   names no slice pass as in range.
   */
  const preparedIndices = new Set(slices.map(function toIndex(slice,): number {
    return slice.target
      .sliceIndex;
  },),);
  for (const sliceIndex of [
    ...shipped,
    ...withdrawn,
  ]) {
    if (!preparedIndices.has(sliceIndex,)) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'set-names-unproduced',
          sliceIndex,
          sliceCount: slices.length,
        },
      },);
    }
  }

  return slices.map(function toRecord(
    slice,
    position,
  ): SliceDeliveryRecord {
    /**
     What the lane decided for this position, present at every position since
     the counts were checked equal.
     */
    const wording = nonNullishOrThrow(wordings[position],);

    /**
     Index and archive wording this slice carries.
     */
    const {
      sliceIndex,
      text: incumbentText,
    } = slice.target;
    if (wording.sliceIndex !== sliceIndex) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'wording-index-differs',
          position,
          sliceIndex,
          wordingIndex: wording.sliceIndex,
        },
      },);
    }
    if (wording.incumbentText !== incumbentText) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'archive-wording-differs',
          sliceIndex,
        },
      },);
    }

    // CHECKED RATHER THAN TAKEN, because the lane and the preparation are two
    // derivations of the same fact and this is the one place holding both. The
    // lane reads it off the prepared chunk today, which makes this agree by
    // construction; what it pins is that it keeps doing so.
    if (
      wording.incumbentKind
        !== (isInsertionChunk(slice.target,) ? 'absent' : 'present')
    ) {
      throw new SliceDeliveryError({
        fault: {
          kind: 'incumbent-kind-differs',
          sliceIndex,
          recorded: wording.incumbentKind,
        },
      },);
    }

    /**
     What the document carries here, by which route, and its wording; the
     record's own two axes are checked against each other there.
     */
    const decision = decideDelivery({
      sliceIndex,
      wording,
      shipped: shipped.has(sliceIndex,),
      withdrawn: withdrawn.has(sliceIndex,),
      blocked,
    },);

    return {
      sliceIndex,
      sourceText: slice.source
        .text,
      incumbentKind: wording.incumbentKind,
      incumbentText: wording.incumbentText,
      outcome: wording.outcome,
      // A TRIM NAMES ONLY A SHIPPED SLICE, refused otherwise, so the guard's
      // trimmed text stands in for a shipped decision and for nothing else.
      shippedText: trimmedText.get(sliceIndex,) ?? decision.documentText,
      delivery: decision.delivery,
    };
  },);
}

//endregion Slice delivery
