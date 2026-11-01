//region Slice record agreement
// Whether a slice record's changed flag agrees with its own text, checked
// wherever a record is accepted: on the way out of a stage, and on the way back
// in from a cache.
//
// Both lanes store a decided text beside a boolean saying whether that text
// differs from the archive's, and those two can contradict each other. From a
// CACHE it is a truncated write that still parses, a hand edit, or a slicing
// that moved while the key did not. From a STAGE it is a derivation reading
// something other than the text, which is what both lanes did until they were
// changed to read the text itself.
//
// CHECKED WHERE THE RECORD IS ACCEPTED rather than only at assembly, and the
// two cases want opposite answers. A cached record is DISCARDED and the slice
// recomputed: one bad file should cost one slice rather than the whole document
// after every other slice has been bought. A fresh record is REFUSED outright,
// because recomputing it would ask the same code the same question.
//
// WHAT ASSEMBLY CATCHES IS ONLY HALF OF IT, which is why this exists at all. A
// record claiming a change it did not make becomes a replacement carrying the
// archive wording, and `assertReplacementsChange` refuses it. A record DENYING a
// change it did make never becomes a replacement, so assembly sees nothing at
// all: the text is dropped in silence while the per-slice ledger still reports
// it. That direction has no other check anywhere.

/**
 Lane a record belongs to, which its finding names.

 @example
 ```ts
 const lane: SliceRecordLane = 'translate';
 ```
 */
export type SliceRecordLane =
  /**
   Repair lane, whose records are `ChunkRepairOutcome`.
   */
  | 'repair'
  /**
   Translate lane, whose records are `TranslateSliceRecord`.
   */
    | 'translate';

/**
 Contradiction between a settled record's `changed` flag and its text.

 MARKED: its message names the lane, the slice index and the flag, in a
 sentence written here from those alone.

 @example
 ```ts
 throw new SliceRecordContradictionError({ lane: 'repair', sliceIndex: 4, changed: true, },);
 ```
 */
export class SliceRecordContradictionError extends Error {
  /**
   Declares this message safe to forward: a lane name, a slice index and a
   flag, in a sentence written here.
   */
  readonly messageNamesOnly: true = true;

  /**
   Lane whose record contradicts itself.
   */
  readonly lane: SliceRecordLane;

  /**
   Slice the record settled.
   */
  readonly sliceIndex: number;

  /**
   Flag the record carried, which its text belies.
   */
  readonly changed: boolean;

  /**
   @param lane - lane whose record contradicts itself

   @param sliceIndex - slice the record settled

   @param changed - flag the record carried, which its text belies
   */
  constructor(
    {
      lane,
      sliceIndex,
      changed,
    }: {
      readonly lane: SliceRecordLane;
      readonly sliceIndex: number;
      readonly changed: boolean;
    },
  ) {
    super(`${lane} slice ${String(sliceIndex,)} settled with changed=${String(changed,)} and carries the ${
      changed ? 'archive wording' : 'wording of a change'
    }: the stage read that flag off something other than its own text`,);
    this.name = 'SliceRecordContradictionError';
    this.lane = lane;
    this.sliceIndex = sliceIndex;
    this.changed = changed;
  }
}

/**
 Whether a record's changed flag agrees with its own text.

 @param changed - what the record claims about itself

 @param decidedText - wording the record carries

 @param incumbentText - archive wording of the slice

 @returns Whether the record describes itself truthfully

 @example
 ```ts
 const usable = sliceRecordAgrees({ changed, decidedText, incumbentText, },);
 ```
 */
export function sliceRecordAgrees(
  {
    changed,
    decidedText,
    incumbentText,
  }: {
    readonly changed: boolean;
    readonly decidedText: string;
    readonly incumbentText: string;
  },
): boolean {
  return changed === (decidedText !== incumbentText);
}

/**
 Refuses a freshly settled record that contradicts itself.

 REFUSED RATHER THAN DISCARDED, which is the opposite of what a cached record
 gets. A cache holds an artifact of an earlier run and may simply be wrong
 about it; a fresh record is what this run just decided, so a contradiction
 means the stage derived its flag from something other than its text. Asking
 again would produce the same answer, and persisting it would write the
 contradiction into the cache for every later run to discard.

 Called BEFORE the cache write on both lanes, so nothing self-contradicting is
 ever stored.

 @param lane - which lane settled the record

 @param sliceIndex - slice it was settled for

 @param changed - what the record claims about itself

 @param decidedText - wording the record carries

 @param incumbentText - archive wording of that slice

 @throws SliceRecordContradictionError naming the direction of the
 contradiction, since over-claiming and under-claiming are different defects

 @example
 ```ts
 assertSettledRecordAgrees({ lane: 'repair', sliceIndex, changed, decidedText, incumbentText, },);
 ```
 */
export function assertSettledRecordAgrees(
  {
    lane,
    sliceIndex,
    changed,
    decidedText,
    incumbentText,
  }: {
    readonly lane: SliceRecordLane;
    readonly sliceIndex: number;
    readonly changed: boolean;
    readonly decidedText: string;
    readonly incumbentText: string;
  },
): void {
  if (sliceRecordAgrees({
    changed,
    decidedText,
    incumbentText,
  },))
    return;
  throw new SliceRecordContradictionError({
    lane,
    sliceIndex,
    changed,
  },);
}

/**
 Names a discarded record so a reader can tell it from a cache miss.

 A slice recomputed because its cached record was refused costs the same calls
 as one never cached, and the two are indistinguishable in a run log without
 this. Which way the record contradicted itself is stated, since a record that
 over-claims and one that under-claims fail for different reasons.

 @param lane - which lane's cache the record came from

 @param sliceIndex - slice it was resumed for

 @param changed - what the record claimed, which the wording explains

 @returns Finding in scorecard-stable wording

 @example
 ```ts
 findings.push(resumedSliceDiscardFinding({ lane: 'translate', sliceIndex, changed, },),);
 ```
 */
export function resumedSliceDiscardFinding(
  {
    lane,
    sliceIndex,
    changed,
  }: {
    readonly lane: SliceRecordLane;
    readonly sliceIndex: number;
    readonly changed: boolean;
  },
): string {
  return `${lane}-discarded-contradictory-slice chunk ${
    String(sliceIndex,)
  }; cached record claims changed=${String(changed,)} and carries the ${
    changed ? 'archive wording' : 'wording of a change'
  }, so it was recomputed`;
}

//endregion Slice record agreement
