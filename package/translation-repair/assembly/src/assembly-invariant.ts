import type { AdjacentSliceText, } from './assembly-adjacent-repetition.ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type AssemblyContractFault,
  assemblySentence,
} from './assembly-contract-fault.ts';
import {
  type SliceReplacement,
  spliceSlices,
} from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Assembly invariant
// The checks both lanes run around assembly, because both lanes can otherwise
// report a change the returned document does not carry.
//
// WHAT THESE DEFEND AGAINST HAS MOVED, and the old answer is worth stating so
// nobody re-derives it. It used to be the slice cache: a resumed record was
// trusted on its chunk index alone, so one claiming a change while carrying the
// archive's own wording reached the guard as a replacement and landed in the
// shipped set beside a document nobody changed. Both lanes now refuse that
// record where they accept it, in `slice-record-agreement.ts`, and both derive `changed`
// from their own text rather than from a vote, so no fresh record reaches here
// contradicting itself either.
//
// So these are now a BACKSTOP: for a defect in a stage nobody has changed yet,
// for a future caller of the exported guard, and for the one relation no single
// slice can see, which is what the document-level check reads.
//
// ASSERTIONS rather than repairs on purpose: silently dropping a suspect
// replacement would leave a run reporting counts nobody can reproduce. The one
// place assembly DOES repair rather than refuse is the net-zero canonicalization
// in `guardFootnoteAssembly`, and it repairs because nobody did anything wrong
// there.

/**
 Refusal of an assembly whose change sets or document contradict each other.

 MARKED: its message is the sentence `assemblySentence` writes from set
 names, kinds and numbers.

 @example
 ```ts
 throw new AssemblyContractError({ fault: { kind: 'reassembly-differs', survivors: 0, }, },);
 ```
 */
export class AssemblyContractError extends Error {
  /**
   Declares this message safe to forward: set names, kinds and numbers in a
   sentence written here.
   */
  readonly messageNamesOnly: true = true;

  /**
   What the sets or document contradict.
   */
  readonly fault: AssemblyContractFault;

  /**
   @param fault - what the sets or document contradict
   */
  constructor({ fault, }: { readonly fault: AssemblyContractFault; },) {
    super(assemblySentence({ fault, },),);
    this.name = 'AssemblyContractError';
    this.fault = fault;
  }
}

/**
 Refuses a replacement whose text is what the slice already said.

 Writing a slice back over itself changes no byte of the document and still
 counts, in every later rate and index set, as a slice the lane changed. That
 is worse than a crash: the run settles, the artifact records it, and the
 number is wrong wherever it is read afterwards.

 @param slices - prepared slice pairs, which supply each incumbent

 @param replacements - what the lane wants assembly to write

 @throws AssemblyContractError when a replacement names an unknown slice, or
 repeats its incumbent verbatim

 @example
 ```ts
 assertReplacementsChange({ slices, replacements, },);
 ```
 */
export function assertReplacementsChange(
  {
    slices,
    replacements,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): void {
  /**
   Archive wording of each prepared slice.
   */
  const incumbentByIndex = new Map(slices.map(function toEntry(slice,): [
    number,
    string,
  ] {
    return [
      slice.target
        .sliceIndex,
      slice.target
        .text,
    ];
  },),);
  for (const replacement of replacements) {
    /**
     What the archive said for this slice.
     */
    const incumbentText = incumbentByIndex.get(replacement.sliceIndex,);
    if (incumbentText === undefined)
      throw new AssemblyContractError({
        fault: {
          kind: 'replacement-unproduced',
          sliceIndex: replacement.sliceIndex,
        },
      },);
    if (replacement.replacementText === incumbentText)
      throw new AssemblyContractError({
        fault: {
          kind: 'replacement-unchanged',
          sliceIndex: replacement.sliceIndex,
        },
      },);
  }
}

/**
 Both index sets a lane result carries, checked and put in document order.

 @example
 ```ts
 const { shipped, withdrawn, } = orderedChangeSets({ sliceCount, shipped, withdrawn, },);
 ```
 */
export type OrderedChangeSets = {
  /**
   Slices the returned document carries a change for, ascending.
   */
  readonly shipped: readonly number[];

  /**
   Slices whose change was taken back, ascending.
   */
  readonly withdrawn: readonly number[];
};

/**
 Checks everything about both index sets that needs no slice count, and
 returns them in document order.

 SPLIT OUT OF {@link orderedChangeSets} rather than duplicated, for the one
 reader that has the sets without their denominator: a settled artifact
 written before `sliceCount` was recorded carries both index arrays and no way
 to bound them. Every OTHER rule still applies to it, and a reader that
 skipped them all for want of a count would accept a repeat or an overlap it
 can plainly see.

 @param shipped - slices the document carries a change for

 @param withdrawn - slices whose change was taken back

 @returns Both sets ascending

 @throws AssemblyContractError when an index is not a whole number, is
 negative, repeats within its set, or appears in both

 @example
 ```ts
 const checked = checkedChangeSets({ shipped, withdrawn, },);
 ```
 */
export function checkedChangeSets(
  {
    shipped,
    withdrawn,
  }: {
    readonly shipped: readonly number[];
    readonly withdrawn: readonly number[];
  },
): OrderedChangeSets {
  /**
   Every index either set names, since the integrality and sign rules are the
   same for both.
   */
  const named = [
    ...shipped,
    ...withdrawn,
  ];
  for (const index of named) {
    if (!Number.isInteger(index,))
      throw new AssemblyContractError({
        fault: {
          kind: 'index-not-integer',
          index,
        },
      },);
    if (index < 0)
      throw new AssemblyContractError({
        fault: {
          kind: 'index-negative',
          index,
        },
      },);
  }
  if (new Set(shipped,).size !== shipped.length)
    throw new AssemblyContractError({
      fault: {
        kind: 'set-repeats',
        set: 'shipped',
      },
    },);
  if (new Set(withdrawn,).size !== withdrawn.length)
    throw new AssemblyContractError({
      fault: {
        kind: 'set-repeats',
        set: 'withdrawn',
      },
    },);

  /**
   Slices claimed by both sets, which no slice can be.
   */
  const both = shipped.filter(function isWithdrawnToo(index,): boolean {
    return withdrawn.includes(index,);
  },);
  if (both.length > 0)
    throw new AssemblyContractError({
      fault: {
        kind: 'both-shipped-and-withdrawn',
        indices: both,
      },
    },);

  return {
    shipped: shipped.toSorted(function ascending(
      left,
      right,
    ): number {
      return left - right;
    },),
    withdrawn: withdrawn.toSorted(function ascending(
      left,
      right,
    ): number {
      return left - right;
    },),
  };
}

/**
 Checks both index sets against the slices they are out of, and returns them
 in document order.

 Both lane contracts claim these sets are disjoint, in range and free of
 repeats, and until now nothing checked any of it. The shipped set was sorted
 at each call site and the withdrawn set was passed through in whatever order
 the guard took slices back, so two lanes compared slice by slice were being
 read from lists ordered by different rules.

 WHICH FAILURE A CALLER SEES when a set breaks more than one rule is settled
 here rather than left to fall out of the code: everything a slice count is
 not needed for is checked FIRST, and the range check reads the sets after
 they are ascending. So `[2, 2]` against two prepared slices reports the
 repeat rather than the range, and an out-of-range index is named in document
 order rather than in the order the caller listed it. Nothing depends on the
 choice; it is pinned so that changing it is a decision rather than a side
 effect.

 @param sliceCount - slices the preparation produced, which bounds both sets

 @param shipped - slices the document carries a change for

 @param withdrawn - slices whose change was taken back

 @returns Both sets ascending

 @throws AssemblyContractError when an index is not a whole number, falls
 outside the prepared slices, repeats within its set, or appears in both

 @example
 ```ts
 const ordered = orderedChangeSets({ sliceCount, shipped, withdrawn, },);
 ```
 */
export function orderedChangeSets(
  {
    sliceCount,
    shipped,
    withdrawn,
  }: {
    readonly sliceCount: number;
    readonly shipped: readonly number[];
    readonly withdrawn: readonly number[];
  },
): OrderedChangeSets {
  /**
   Both sets under every rule a count is not needed for, already ascending.
   */
  const checked = checkedChangeSets({
    shipped,
    withdrawn,
  },);
  for (
    const index of [
      ...checked.shipped,
      ...checked.withdrawn,
    ]
  ) {
    if (index >= sliceCount)
      throw new AssemblyContractError({
        fault: {
          kind: 'index-beyond-count',
          index,
          sliceCount,
        },
      },);
  }
  return checked;
}

/**
 Names the slices a returned document carries a change for, refusing any
 document its own surviving replacements do not reconstruct.

 DERIVED RATHER THAN ACCEPTED, which is the whole point. Both lanes used to
 map the surviving replacements to indices themselves and hand the result here
 as an independent argument, so a caller passing a set that named the wrong
 slices was checked only for being empty or not. A document changed in slices
 2 and 3 while reporting only slice 1 passed. Taking the replacements instead
 makes the two impossible to disagree, and re-splicing them is what proves the
 returned text is the one those replacements make.

 ONE EMPTINESS CHECK SURVIVES THE RE-SPLICE, the one it cannot imply. A
 document that moved while nothing survived fails the re-splice itself, since
 no replacements reassemble to the archive text; a separate check for it never
 ran and went in T8's twenty-first batch. A net-zero set genuinely re-splices
 to the archive text, so exact reconstruction accepts it; what refuses it is
 the check that remains, which enforces `guardFootnoteAssembly`'s canonical
 answer that such a set ships nothing.
 That direction was unenforceable until the guard learned to canonicalize:
 two adjacent slices whose replacements each differ from their own incumbent
 can reassemble to the archive text, and refusing THAT would crash a run the
 models got right.

 PRECONDITION, and the only way a legitimate run reaches a refusal here: the
 replacements must be what `guardFootnoteAssembly` LET STAND, not what a lane
 proposed. The guard is where a net-zero set becomes no survivors, so calling
 this first, on a set that reassembles to the archive text, refuses a run
 nobody got wrong. The message says so, because the fix is the call order
 rather than anything about the document.

 @param incumbentText - archive document the lane started from

 @param assembledText - document the lane is about to return

 @param slices - prepared slices, which place every replacement

 @param survivingReplacements - what `guardFootnoteAssembly` let stand, which
 is the only admissible source for both the text and the index set; a set that
 has not been through the guard can be a legitimate net-zero this refuses

 @returns Slices the returned document carries a change for

 @throws AssemblyContractError when a surviving replacement repeats its own
 incumbent, when re-splicing them does not reproduce the returned document
 (a document that moved while nothing survived among them), or when it did
 not move while something did

 @example
 ```ts
 const shipped = deriveShippedIndices({ incumbentText, assembledText, slices, survivingReplacements, },);
 ```
 */
export function deriveShippedIndices(
  {
    incumbentText,
    assembledText,
    slices,
    survivingReplacements,
  }: {
    readonly incumbentText: string;
    readonly assembledText: string;
    readonly slices: readonly ChunkPair[];
    readonly survivingReplacements: readonly SliceReplacement[];
  },
): readonly number[] {
  // Sound when called on its own, rather than relying on every caller having
  // run this before the guard. A replacement repeating its incumbent survives
  // assembly untouched and would otherwise be named as shipped.
  assertReplacementsChange({
    slices,
    replacements: survivingReplacements,
  },);

  /**
   Document those replacements make, computed here rather than trusted.
   */
  const reconstructed = spliceSlices({
    targetText: incumbentText,
    slices,
    replacements: survivingReplacements,
  },);
  if (reconstructed !== assembledText)
    throw new AssemblyContractError({
      fault: {
        kind: 'reassembly-differs',
        survivors: survivingReplacements.length,
      },
    },);

  /**
   Slices those replacements name.
   */
  const shipped = survivingReplacements.map(function toIndex(replacement,): number {
    return replacement.sliceIndex;
  },);
  if ((assembledText === incumbentText) && (shipped.length > 0))
    throw new AssemblyContractError({
      fault: {
        kind: 'unchanged-with-claims',
        indices: shipped,
      },
    },);
  return shipped;
}

/**
 What each slice contributed to the assembled document, in document order.

 Read off the SURVIVING replacements rather than the lane's wishes, so a
 row the footnote guard took back is not counted as wording that shipped.
 Both lanes kept their own copy of this reading (audit area six,
 2026-09-28), and the adjacent-repetition check reads its output in each.

 @param slices - prepared slices in document order

 @param survivingReplacements - what `guardFootnoteAssembly` let stand

 @returns Each slice's shipped wording: its surviving replacement, else the
 archive's

 @example
 ```ts
 const shippedSlices = shippedSliceTexts({ slices, survivingReplacements: guarded.replacements, },);
 ```
 */
export function shippedSliceTexts(
  {
    slices,
    survivingReplacements,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly survivingReplacements: readonly SliceReplacement[];
  },
): readonly AdjacentSliceText[] {
  return slices.map(function shippedFor(slice,): AdjacentSliceText {
    /**
     This slice's index and the wording the archive had there.
     */
    const {
      sliceIndex,
      text: incumbentText,
    } = slice.target;

    /**
     Surviving replacement for this slice, absent when the incumbent stood.
     */
    const replacement = survivingReplacements.find(function atSlice(candidate,): boolean {
      return candidate.sliceIndex === sliceIndex;
    },);
    return {
      sliceIndex,
      text: (replacement === undefined)
        ? incumbentText
        : replacement.replacementText,
    };
  },);
}

//endregion Assembly invariant
