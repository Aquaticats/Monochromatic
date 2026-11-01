import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Slice delivery faults
// What a lane's slice reports say that cannot both be true, as a union the
// class words itself. Its own file because `slice-delivery.ts` holds the
// delivery decision and the ledger builder and has no room left under the
// file-length limit.

/**
 Which index set named a slice.

 @example
 ```ts
 const set: DeliverySetName = 'withdrawn';
 ```
 */
export type DeliverySetName = 'shipped' | 'withdrawn';

/**
 What a lane's slice reports say that cannot both be true.

 @example
 ```ts
 const fault: SliceDeliveryFault = { kind: 'decided-unstated', sliceIndex: 4, };
 ```
 */
export type SliceDeliveryFault = {
  /**
   A set names a slice whose outcome is not a decision.
   */
  readonly kind: 'named-without-decision';

  /**
   Slice named.
   */
  readonly sliceIndex: number;

  /**
   Set naming it.
   */
  readonly set: DeliverySetName;
} | {
  /**
   A slice named as shipped decided the archive's own wording.
   */
  readonly kind: 'shipped-archive-wording';

  /**
   Slice named.
   */
  readonly sliceIndex: number;
} | {
  /**
   A slice is named as shipped by a blocked run.
   */
  readonly kind: 'shipped-on-blocked';

  /**
   Slice named.
   */
  readonly sliceIndex: number;
} | {
  /**
   A slice is named as withdrawn by assembly on a blocked run.
   */
  readonly kind: 'withdrawn-on-blocked';

  /**
   Slice named.
   */
  readonly sliceIndex: number;
} | {
  /**
   A slice named as withdrawn decided the archive's own wording.
   */
  readonly kind: 'withdrawn-archive-wording';

  /**
   Slice named.
   */
  readonly sliceIndex: number;
} | {
  /**
   A slice decided wording of its own and neither set names it.
   */
  readonly kind: 'decided-unstated';

  /**
   Slice decided.
   */
  readonly sliceIndex: number;
} | {
  /**
   A trimmed replacement names a slice the document does not ship.
   */
  readonly kind: 'trim-names-unshipped';

  /**
   Slice named.
   */
  readonly sliceIndex: number;
} | {
  /**
   Two trimmed replacements name one slice.
   */
  readonly kind: 'trim-repeats';

  /**
   Slice named twice.
   */
  readonly sliceIndex: number;
} | {
  /**
   A set counts a slice twice.
   */
  readonly kind: 'set-repeats';

  /**
   Set that repeats.
   */
  readonly set: DeliverySetName;

  /**
   Slices it names.
   */
  readonly named: number;

  /**
   Distinct slices among them.
   */
  readonly distinct: number;
} | {
  /**
   Lane reported a different number of wordings than prepared slices.
   */
  readonly kind: 'wording-count';

  /**
   Wordings reported.
   */
  readonly wordings: number;

  /**
   Slices prepared.
   */
  readonly slices: number;
} | {
  /**
   Both sets name one slice.
   */
  readonly kind: 'both-shipped-and-withdrawn';

  /**
   Slice named twice.
   */
  readonly sliceIndex: number;
} | {
  /**
   A set names a slice the preparation never produced.
   */
  readonly kind: 'set-names-unproduced';

  /**
   Slice named.
   */
  readonly sliceIndex: number;

  /**
   Slices prepared.
   */
  readonly sliceCount: number;
} | {
  /**
   Wording at a position names another slice than the one there.
   */
  readonly kind: 'wording-index-differs';

  /**
   Position in question.
   */
  readonly position: number;

  /**
   Slice the preparation holds there.
   */
  readonly sliceIndex: number;

  /**
   Slice the wording names.
   */
  readonly wordingIndex: number;
} | {
  /**
   Wording's archive text disagrees with the prepared slice's.
   */
  readonly kind: 'archive-wording-differs';

  /**
   Slice affected.
   */
  readonly sliceIndex: number;
} | {
  /**
   Wording and prepared chunk disagree about whether the archive holds text.
   */
  readonly kind: 'incumbent-kind-differs';

  /**
   Slice affected.
   */
  readonly sliceIndex: number;

  /**
   What the lane record says.
   */
  readonly recorded: 'present' | 'absent';
};

/**
 Words a delivery fault from set names, kinds and numbers.

 @param fault - what the reports say that cannot both be true

 @returns Sentence written here

 @throws {@link UnhandledMemberInvariantError} when a member of `SliceDeliveryFault` has no branch here, which the compiler rules out

 @example
 ```ts
 const sentence = deliverySentence({ fault: { kind: 'wording-count', wordings: 2, slices: 3, }, },);
 ```
 */
export function deliverySentence({ fault, }: { readonly fault: SliceDeliveryFault; },): string {
  if (fault.kind === 'named-without-decision')
    return `slice ${String(fault.sliceIndex,)} is named as ${
      fault.set
    } and reports no decision, so the lane both did and did not reach it`;
  if (fault.kind === 'shipped-archive-wording')
    return `slice ${
      String(fault.sliceIndex,)
    } is named as shipped and its decision is the archive's own wording, so the document would carry a `
      + 'change nobody made';
  if (fault.kind === 'shipped-on-blocked')
    return `slice ${
      String(fault.sliceIndex,)
    } is named as shipped by a blocked run, which returns the archive untouched, so no slice of it carries `
      + 'a replacement';
  if (fault.kind === 'withdrawn-on-blocked')
    return `slice ${
      String(fault.sliceIndex,)
    } is named as withdrawn by assembly on a blocked run, which returns the archive without assembling `
      + 'anything';
  if (fault.kind === 'withdrawn-archive-wording')
    return `slice ${
      String(fault.sliceIndex,)
    } is named as withdrawn and its decision is the archive's own wording, so there was no replacement for `
      + 'assembly to take back';
  if (fault.kind === 'decided-unstated')
    return `slice ${
      String(fault.sliceIndex,)
    } decided wording of its own and is named as neither shipped nor withdrawn by a run that was not `
      + 'blocked, so what the document carries there is unstated';
  if (fault.kind === 'trim-names-unshipped')
    return `slice ${
      String(fault.sliceIndex,)
    } carries a trimmed replacement and is not named as shipped, so the trim describes text the document `
      + 'does not carry';
  if (fault.kind === 'trim-repeats')
    return `slice ${
      String(fault.sliceIndex,)
    } carries more than one trimmed replacement, so which text the document holds there cannot be read`;
  if (fault.kind === 'set-repeats')
    return `the ${fault.set} set names ${String(fault.named,)} ${
      wordForCount({
        count: fault.named,
        one: 'slice',
        many: 'slices',
      },)
    } and ${
      String(fault.distinct,)
    } of them are distinct, so it counts at least one slice twice`;
  if (fault.kind === 'wording-count') {
    return `lane reported ${String(fault.wordings,)} slice wordings against ${
      String(fault.slices,)
    } prepared ${
      wordForCount({
        count: fault.slices,
        one: 'slice',
        many: 'slices',
      },)
    }, so the two describe different preparations`;
  }
  if (fault.kind === 'both-shipped-and-withdrawn')
    return `slice ${
      String(fault.sliceIndex,)
    } is named as both shipped and withdrawn, so the lane reports the document both carrying its change `
      + 'and having taken it back';
  if (fault.kind === 'set-names-unproduced')
    return `an index set names slice ${String(fault.sliceIndex,)}, which this preparation of ${
      String(fault.sliceCount,)
    } ${
      wordForCount({
        count: fault.sliceCount,
        one: 'slice',
        many: 'slices',
      },)
    } never produced`;
  if (fault.kind === 'wording-index-differs')
    return `slice at position ${String(fault.position,)} is indexed ${
      String(fault.sliceIndex,)
    } and its wording names slice ${String(fault.wordingIndex,)}`;
  if (fault.kind === 'archive-wording-differs')
    return `slice ${
      String(fault.sliceIndex,)
    } carries archive wording its own lane record disagrees with, so the two were built from different `
      + 'preparations';
  if (fault.kind === 'incumbent-kind-differs') {
    return `slice ${String(fault.sliceIndex,)}'s lane record says archive wording is ${
      fault.recorded
    } there, and its prepared chunk says the opposite`;
  }
  throw new UnhandledMemberInvariantError({
    what: 'slice delivery fault',
    member: fault,
  },);
}

/**
 Refusal of lane slice reports that cannot both be true.

 MARKED: its message is the sentence `deliverySentence` writes from set
 names, kinds and numbers.

 @example
 ```ts
 throw new SliceDeliveryError({ fault: { kind: 'decided-unstated', sliceIndex: 2, }, },);
 ```
 */
export class SliceDeliveryError extends Error {
  /**
   Declares this message safe to forward: set names, kinds and numbers in a
   sentence written here.
   */
  readonly messageNamesOnly: true = true;

  /**
   What the reports say that cannot both be true.
   */
  readonly fault: SliceDeliveryFault;

  /**
   @param fault - what the reports say that cannot both be true
   */
  public constructor({ fault, }: { readonly fault: SliceDeliveryFault; },) {
    super(deliverySentence({ fault, },),);
    this.name = 'SliceDeliveryError';
    this.fault = fault;
  }
}

//endregion Slice delivery faults
