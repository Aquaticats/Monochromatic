import {
  howOften,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Lane slice coverage error
// The failure both halves of the wording builder raise, in its own file so they
// can share it without one importing the other.
//
// It was declared beside `buildLaneSliceTexts` while that function owned every
// check. The per-list checks now live in `lane-slice-sets.ts`, which the builder
// calls, so leaving the class behind would have made the two files import each
// other.
//
// EVERY COUNT TAKES ITS OWN WORD (ledger B66). A repeat fault's distinct count
// is 1 whenever two entries share one index, and a fixed plural after it read
// "under 1 distinct indices".

/**
 Sets a lane may name slices under, beside its decisions.

 @example
 ```ts
 const label: NamedSliceSetLabel = 'unfilled';
 ```
 */
export type NamedSliceSetLabel = 'unfilled' | 'unheard' | 'not-applicable';

/**
 What each set says about a slice it names beside a decision, and what the
 archive must hold for it to be named at all.
 */
const SET_CLAUSES: Record<NamedSliceSetLabel, {
  readonly decided: string;
  readonly incumbent: string;
}> = {
  'unfilled': {
    decided: 'so what it accepted there is unstated',
    incumbent: 'and the archive holds wording for it: only a slice with none can be unfilled',
  },
  'unheard': {
    decided: 'so whether anyone answered for it is unstated',
    incumbent: 'and the archive holds no wording for it to fall back on',
  },
  'not-applicable': {
    decided: 'so whether this lane had anything to do there is unstated',
    incumbent: 'and the archive holds wording for it, which is exactly what this lane works on',
  },
};

/**
 Why a lane's slice report does not cover its preparation.

 @example
 ```ts
 const fault: LaneSliceCoverageFault = { kind: 'left-undecided', sliceIndex: 4, };
 ```
 */
export type LaneSliceCoverageFault = {
  /**
   A set names more slices than distinct indices.
   */
  readonly kind: 'set-repeats';

  /**
   Set that repeats.
   */
  readonly set: NamedSliceSetLabel;

  /**
   Slices it claims.
   */
  readonly claimed: number;

  /**
   Distinct indices among them.
   */
  readonly distinct: number;
} | {
  /**
   A set names a slice the preparation never produced.
   */
  readonly kind: 'set-names-unproduced';

  /**
   Set naming it.
   */
  readonly set: NamedSliceSetLabel;

  /**
   Slice named.
   */
  readonly sliceIndex: number;
} | {
  /**
   A set names a slice the lane also decided.
   */
  readonly kind: 'set-and-decided';

  /**
   Set naming it.
   */
  readonly set: NamedSliceSetLabel;

  /**
   Slice named.
   */
  readonly sliceIndex: number;
} | {
  /**
   A set names a slice whose archive wording contradicts the set.
   */
  readonly kind: 'set-against-archive';

  /**
   Set naming it.
   */
  readonly set: NamedSliceSetLabel;

  /**
   Slice named.
   */
  readonly sliceIndex: number;
} | {
  /**
   Two sets name one slice.
   */
  readonly kind: 'two-sets';

  /**
   First set naming it.
   */
  readonly set: NamedSliceSetLabel;

  /**
   Second set naming it.
   */
  readonly other: NamedSliceSetLabel;

  /**
   Slice named twice.
   */
  readonly sliceIndex: number;
} | {
  /**
   Lane left a prepared slice undecided under a policy that refuses that.
   */
  readonly kind: 'left-undecided';

  /**
   Slice left.
   */
  readonly sliceIndex: number;
} | {
  /**
   Preparation produced more slices than distinct indices.
   */
  readonly kind: 'preparation-repeats';

  /**
   Slices produced.
   */
  readonly slices: number;

  /**
   Distinct indices among them.
   */
  readonly distinct: number;
} | {
  /**
   Lane decided more times than distinct slices.
   */
  readonly kind: 'decisions-repeat';

  /**
   Decisions made.
   */
  readonly decisions: number;

  /**
   Distinct slices among them.
   */
  readonly distinct: number;
} | {
  /**
   Lane decided a slice the preparation never produced.
   */
  readonly kind: 'decided-unproduced';

  /**
   Slice decided.
   */
  readonly sliceIndex: number;
} | {
  /**
   Lane reports reaching a slice after leaving an earlier one unexamined.
   */
  readonly kind: 'reached-after-stop';

  /**
   Slice reached.
   */
  readonly sliceIndex: number;
};

/**
 Words a lane slice coverage fault from set labels, kinds and numbers.

 @param fault - why the lane's report does not cover its preparation

 @returns Sentence written here

 @throws {@link UnhandledMemberInvariantError} when a member of `LaneSliceCoverageFault` has no branch here, which the compiler rules out

 @example
 ```ts
 const sentence = laneCoverageSentence({ fault: { kind: 'left-undecided', sliceIndex: 4, }, },);
 ```
 */
export function laneCoverageSentence({ fault, }: { readonly fault: LaneSliceCoverageFault; },): string {
  if (fault.kind === 'set-repeats')
    return `lane reports ${String(fault.claimed,)} ${fault.set} ${
      wordForCount({
        count: fault.claimed,
        one: 'slice',
        many: 'slices',
      },)
    } under ${String(fault.distinct,)} distinct ${
      wordForCount({
        count: fault.distinct,
        one: 'index',
        many: 'indices',
      },)
    }`;
  if (fault.kind === 'set-names-unproduced')
    return `lane reports slice ${String(fault.sliceIndex,)} ${fault.set}, which this preparation never produced`;
  if (fault.kind === 'set-and-decided') {
    /**
     What this set says about a slice it names beside a decision.
     */
    const { decided, } = SET_CLAUSES[fault.set];
    return `lane reports slice ${String(fault.sliceIndex,)} as ${fault.set} and decided at once, ${decided}`;
  }
  if (fault.kind === 'set-against-archive') {
    /**
     What the archive must hold for this set to name a slice.
     */
    const { incumbent, } = SET_CLAUSES[fault.set];
    return `lane reports slice ${String(fault.sliceIndex,)} ${fault.set}, ${incumbent}`;
  }
  if (fault.kind === 'two-sets')
    return `lane reports slice ${String(fault.sliceIndex,)} as ${fault.set} and ${
      fault.other
    } at once, so what it did there is stated twice and differently`;
  if (fault.kind === 'left-undecided')
    return `lane left prepared slice ${String(fault.sliceIndex,)} undecided`;
  if (fault.kind === 'preparation-repeats')
    return `preparation produced ${String(fault.slices,)} ${
      wordForCount({
        count: fault.slices,
        one: 'slice',
        many: 'slices',
      },)
    } under ${String(fault.distinct,)} distinct ${
      wordForCount({
        count: fault.distinct,
        one: 'index',
        many: 'indices',
      },)
    }`;
  if (fault.kind === 'decisions-repeat')
    return `lane decided ${howOften({ count: fault.decisions, },)} over ${String(fault.distinct,)} distinct ${
      wordForCount({
        count: fault.distinct,
        one: 'slice',
        many: 'slices',
      },)
    }`;
  if (fault.kind === 'decided-unproduced')
    return `lane decided slice ${String(fault.sliceIndex,)}, which this preparation never produced`;
  if (fault.kind === 'reached-after-stop') {
    return `lane reports reaching slice ${
      String(fault.sliceIndex,)
    } after leaving an earlier one unexamined, which no early stop produces`;
  }
  throw new UnhandledMemberInvariantError({
    what: 'lane slice coverage fault',
    member: fault,
  },);
}

/**
 Refusal of a lane report that does not cover its preparation.

 MARKED: its message is the sentence `laneCoverageSentence` writes from set
 labels, kinds and numbers.

 @example
 ```ts
 throw new LaneSliceCoverageError({ fault: { kind: 'left-undecided', sliceIndex: 4, }, },);
 ```
 */
export class LaneSliceCoverageError extends Error {
  /**
   Declares this message safe to forward: set labels, kinds and numbers in
   a sentence written here.
   */
  readonly messageNamesOnly: true = true;

  /**
   Why the report does not cover the preparation.
   */
  readonly fault: LaneSliceCoverageFault;

  /**
   @param fault - why the report does not cover the preparation
   */
  constructor({ fault, }: { readonly fault: LaneSliceCoverageFault; },) {
    super(laneCoverageSentence({ fault, },),);
    this.name = 'LaneSliceCoverageError';
    this.fault = fault;
  }
}

//endregion Lane slice coverage error
