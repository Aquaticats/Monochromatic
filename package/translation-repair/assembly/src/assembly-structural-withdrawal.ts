import {
  introducedFootnoteFindings,
  introducedStructuralRegressions,
} from './assembly-regressions.ts';
import {
  type SliceReplacement,
  spliceSlices,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  type ChunkPair,
  strictRefusalOffset,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';

//region Structural withdrawal proof
// Before blanket withdrawal, test whether reverting one replacement repairs the
// actual whole document. This preserves unrelated insertions without changing
// the author's right to defend a candidate against per-slice findings.

/**
 Finds a single withdrawal that leaves no introduced structural or footnote defect.

 Tries replacements in their supplied document order and keeps the first proven
 repair. Each trial reparses an actual splice, not an isolated slice that may
 own only one half of a valid container. No trial result changes caller state.

 @param targetText - inherited document defining permissible existing defects

 @param slices - prepared locations for every replacement

 @param replacements - current assembly, before any speculative withdrawal

 @returns One proven slice index, or none when blanket fallback remains necessary

 @example
 ```ts
 const proven = singleStructuralWithdrawal({ targetText, slices, replacements, });
 ```
 */
export function singleStructuralWithdrawal(
  {
    targetText,
    slices,
    replacements,
  }: {
    readonly targetText: string;
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): readonly number[] {
  // A sole replacement has no sibling to preserve; the ordinary fallback already
  // produces exactly that result and retains its existing diagnostic.
  if (replacements.length < 2)
    return [];
  for (const replacement of replacements) {
    /**
     Counterfactual set retains every other accepted wording.
     */
    const remaining = replacements.filter(function other(candidate,): boolean {
      return candidate.sliceIndex !== replacement.sliceIndex;
    },);
    /**
     Whole page produced by this one withdrawal, including all slice joins.
     */
    const assembledText = spliceSlices({
      targetText,
      slices,
      replacements: remaining,
    },);
    if (introducedStructuralRegressions({
      incumbentText: targetText,
      assembledText,
    },)
      .length
      > 0)
      continue;
    // Restoring grammar can expose a footnote hidden by the malformed component.
    // It is not a proven repair until that graph also matches the inherited one.
    if (introducedFootnoteFindings({
      incumbentText: targetText,
      assembledText,
    },)
      .length
      > 0)
      continue;
    return [replacement.sliceIndex,];
  }
  return [];
}

/**
 One withdrawal that moves the first strict-grammar refusal later, both
 offsets read on the page as assembled.

 @example
 ```ts
 const step: AdvancingWithdrawal = { sliceIndex: 4, from: 120, to: 980, cleared: false, };
 ```
 */
export type AdvancingWithdrawal = {
  /**
   Slice whose replacement is withdrawn.
   */
  readonly sliceIndex: number;

  /**
   Offset of the first refusal on the page as assembled.
   */
  readonly from: number;

  /**
   Where the first refusal stands once the replacement is withdrawn, as an
   offset on the page as assembled: later than `from`, or the page's length
   when nothing refuses any more.
   */
  readonly to: number;

  /**
   Whether the strict grammar accepts the page once the replacement is
   withdrawn, which outranks a refusal moved however far: a grammar that
   stops at a page's very end (an expression left open) leaves `to` no
   later than a page it accepts.
   */
  readonly cleared: boolean;
};

/**
 Characters two texts share from their start.

 @param left - one text

 @param right - the other

 @returns Length of the longest common prefix, in UTF-16 units like every
 offset the strict reading names

 @example
 ```ts
 sharedPrefixLength({ left: 'The cat naps.', right: 'The cat sits.', },);
 // => 8
 ```
 */
export function sharedPrefixLength(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): number {
  /**
   Characters both texts have.
   */
  const shorter = Math.min(
    left.length,
    right.length,
  );
  for (let at = 0; at < shorter; at += 1) {
    if (left[at] !== right[at])
      return at;
  }
  return shorter;
}

/**
 Characters two texts share at their end, never reaching into a prefix
 already counted, so the two shared stretches never overlap.

 @param left - one text

 @param right - the other

 @param prefix - characters already counted as shared from the start

 @returns Length of the longest common suffix after the prefix

 @example
 ```ts
 sharedSuffixLength({ left: 'A cat naps.', right: 'A dog naps.', prefix: 2, },);
 // => 6
 ```
 */
export function sharedSuffixLength(
  {
    left,
    right,
    prefix,
  }: {
    readonly left: string;
    readonly right: string;
    readonly prefix: number;
  },
): number {
  /**
   Characters after the prefix in the shorter text, the most a suffix can
   share without overlapping it.
   */
  const room = Math.min(
    left.length,
    right.length,
  ) - prefix;
  for (let back = 0; back < room; back += 1) {
    if (left.at(-1 - back,) !== right.at(-1 - back,))
      return back;
  }
  return room;
}

/**
 Offset on the page as assembled of a refusal read on the page with one
 replacement withdrawn (ledger B85).

 THE TWO PAGES DIFFER IN ONE STRETCH, the withdrawn slice's: the text before
 it and after it is the same. A refusal in the shared text before it keeps
 its offset; one in the shared text after it moves by the difference in the
 pages' lengths; one inside the withdrawn slice's own archive text stands,
 on the assembled page, where that slice begins. Compared raw, the offsets
 index two texts, and a withdrawal that only lengthened the text before a
 break read as moving the break.

 @param assembledText - page as assembled, the frame both offsets are compared in

 @param withdrawnText - same page with one replacement withdrawn

 @param offset - where the strict grammar stops on the withdrawn page

 @returns The same place on the assembled page

 @example
 ```ts
 const to = assembledOffsetOf({ assembledText, withdrawnText, offset: reading.offset, },);
 ```
 */
function assembledOffsetOf(
  {
    assembledText,
    withdrawnText,
    offset,
  }: {
    readonly assembledText: string;
    readonly withdrawnText: string;
    readonly offset: number;
  },
): number {
  /**
   Text both pages carry before the withdrawn stretch.
   */
  const prefix = sharedPrefixLength({
    left: assembledText,
    right: withdrawnText,
  },);
  if (offset < prefix)
    return offset;
  /**
   Text both pages carry after it.
   */
  const suffix = sharedSuffixLength({
    left: assembledText,
    right: withdrawnText,
    prefix,
  },);
  /**
   Where that shared ending starts on the withdrawn page.
   */
  const suffixStart = withdrawnText.length - suffix;
  if (offset < suffixStart)
    return prefix;
  /**
   How much longer the assembled page is than the withdrawn one.
   */
  const lengthChange = assembledText.length - withdrawnText.length;
  return offset + lengthChange;
}

/**
 Finds the withdrawal that moves the first strict-grammar refusal furthest
 later, when no single withdrawal repairs the page (class fifty-eight).

 The parser names where it stopped; a withdrawal after which it stops later
 removed the break it named, and the guard's next round reads what is left.
 Nothing is chosen when the page parses already or when no withdrawal moves
 the refusal.

 BOTH OFFSETS ARE READ ON THE PAGE AS ASSEMBLED (ledger B85). The page with a
 replacement withdrawn is another text, longer or shorter by what the
 withdrawal changed, so a break after the withdrawn slice sits at another
 offset there without having moved; read raw, a whole replacement much
 shorter than its archive text ranked as moving a break it never touched,
 above the withdrawal that removed it. A withdrawal after which the grammar
 accepts the page outranks every one that only moves the refusal.

 @param targetText - inherited document the replacements are spliced over

 @param slices - prepared locations for every replacement

 @param replacements - current assembly, before any speculative withdrawal

 @returns Withdrawal that advances the refusal furthest, at most one, or none

 @example
 ```ts
 const [step,] = advancingStructuralWithdrawal({ targetText, slices, replacements, },);
 ```
 */
export function advancingStructuralWithdrawal(
  {
    targetText,
    slices,
    replacements,
  }: {
    readonly targetText: string;
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): readonly AdvancingWithdrawal[] {
  /**
   Page as assembled, the frame every offset here is read in.
   */
  const assembledText = spliceSlices({
    targetText,
    slices,
    replacements,
  },);
  /**
   What the grammar makes of the page as it stands.
   */
  const standing = strictRefusalOffset({ text: assembledText, },);
  if (!standing.refused)
    return [];
  /**
   Where the grammar stops before any withdrawal.
   */
  const from = standing.offset;
  /**
   Every withdrawal that moves the refusal later.
   */
  const advancing = replacements.flatMap(function advances(replacement,): readonly AdvancingWithdrawal[] {
    /**
     Page with this one replacement withdrawn.
     */
    const withdrawnText = spliceSlices({
      targetText,
      slices,
      replacements: replacements.filter(function other(candidate,): boolean {
        return candidate.sliceIndex !== replacement.sliceIndex;
      },),
    },);
    /**
     What the grammar makes of it then.
     */
    const reading = strictRefusalOffset({ text: withdrawnText, },);
    if (!reading.refused) {
      return [{
        sliceIndex: replacement.sliceIndex,
        from,
        to: assembledText.length,
        cleared: true,
      },];
    }
    /**
     Where it stops then, as a place on the assembled page.
     */
    const to = assembledOffsetOf({
      assembledText,
      withdrawnText,
      offset: reading.offset,
    },);
    if (to <= from)
      return [];
    return [{
      sliceIndex: replacement.sliceIndex,
      from,
      to,
      cleared: false,
    },];
  },);
  /**
   First candidate, absent when nothing advances the refusal.
   */
  const [first, ...rest] = advancing;
  if (first === undefined)
    return [];
  /**
   Candidate that moves the refusal furthest, one that clears it first.
   */
  const furthest = rest.reduce(
    function later(
      best: AdvancingWithdrawal,
      candidate: AdvancingWithdrawal,
    ): AdvancingWithdrawal {
      if (candidate.cleared !== best.cleared)
        return candidate.cleared ? candidate : best;
      return (candidate.to > best.to) ? candidate : best;
    },
    first,
  );
  return [furthest,];
}

//endregion Structural withdrawal proof
