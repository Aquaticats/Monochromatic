import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  MARKUP_IDENTIFIER_KINDS,
  type MarkupAtom,
  type MarkupAtomKind,
  scanMarkupAtoms,
} from './markup-atom-scan.ts';

//region Markup atom preservation
// What an edit must carry through (ledger L4). The owner ruled 2026-09-28
// ("Markup atoms") that the atoms survive every edit except a removal an
// addition issue names, and the same day made the quality of the end result a
// standing directive; a replay of the ruling's wording then refused 24 recorded
// edits of which 17 re-marked or moved markup rather than lost it, five of them
// restoring the source's own `DottedNumber` props (`l4-source-carried.mjs`).
// The rule that keeps the ruling's protection and lets those through:
//
// - An atom inside a quote an addition claim made may go: removing the detail
//   an addition quotes is the fix (`addition-repair-rule.ts`).
// - An atom the source carries is copied markup and must survive.
// - An MDX expression, inline code or tag the translation authored (the source
//   does not carry it) may be RE-MARKED, one for one, into an atom of the same
//   kind or one the source carries; it may never simply be dropped.
// - A footnote reference or a link destination has no re-marking excuse: a
//   label or an address is an identifier no checker judges, and the source
//   often carries no footnote at all, so every archive reference would read
//   as authored.
// - An atom one edit drops and another edit of the same patch writes has
//   survived the patch (`settleMarkupMoves`): two shipped edits on
//   yuki418330012 swapped footnote references between clauses.
//
// Gains are always allowed: the atoms the replay found edits adding were
// omission fixes restoring a footnote reference, a link destination or a tag.

/**
 Separator between an atom's kind and value in a key, a character no kind
 name contains.
 */
const KEY_SEPARATOR = '\u0000';

/**
 What one edit did to the markup of its envelope.

 @example
 ```ts
 const delta: MarkupDelta = { unexcused: [], gained: [], };
 ```
 */
export type MarkupDelta = {
  /**
   Atoms the edit lost that no addition quote licensed and no re-marking
   excused, which only another edit of the patch writing them can excuse.
   */
  readonly unexcused: readonly MarkupAtom[];

  /**
   Atoms the edit wrote that the replaced text did not carry, less those a
   re-marking consumed, so one written atom excuses one loss at most.
   */
  readonly gained: readonly MarkupAtom[];
};

/**
 An atom's kind-and-value key.

 @param atom - atom to key

 @returns Key two atoms share exactly when they are the same markup

 @example
 ```ts
 const key = keyOf({ kind: 'tag', value: '<br />', },);
 ```
 */
function keyOf(atom: MarkupAtom,): string {
  return `${atom.kind}${KEY_SEPARATOR}${atom.value}`;
}

/**
 Atoms of the first list that the second does not match one for one, in
 the first list's order.

 @param left - atoms to keep where unmatched

 @param right - atoms each cancelling one equal atom of `left`

 @returns `left` less `right`, as multisets

 @example
 ```ts
 const lost = unmatched({ left: before, right: after, },);
 ```
 */
function unmatched(
  {
    left,
    right,
  }: {
    readonly left: readonly MarkupAtom[];
    readonly right: readonly MarkupAtom[];
  },
): readonly MarkupAtom[] {
  /**
   How many of each key `right` still has to cancel.
   */
  const remaining = right.reduce(
    function countOne(
      counts,
      atom,
    ) {
      return counts.set(
        keyOf(atom,),
        (counts.get(keyOf(atom,),) ?? 0) + 1,
      );
    },
    new Map<string, number>(),
  );
  return left.filter(function isUnmatched(atom,): boolean {
    /**
     Cancellations left for this atom's key.
     */
    const count = remaining.get(keyOf(atom,),) ?? 0;
    if (count === 0)
      return true;
    remaining.set(
      keyOf(atom,),
      count - 1,
    );
    return false;
  },);
}

/**
 Keys of every atom a source text carries, which is what makes an atom
 copied markup rather than markup the translation authored.

 @param sourceText - original, the whole document where the stage has it

 @returns Keys of its atoms

 @example
 ```ts
 const sourceKeys = markupSourceKeys({ sourceText, },);
 ```
 */
export function markupSourceKeys({ sourceText, }: { readonly sourceText: string; },): ReadonlySet<string> {
  return new Set(scanMarkupAtoms({ text: sourceText, },)
    .map(keyOf,),);
}

/**
 What an edit did to its envelope's markup: the losses nothing about the edit
 itself excuses, and the atoms it wrote that remain free to excuse a loss
 elsewhere in the patch.

 @param before - exact text the edit replaced

 @param after - exact text the edit wrote, empty for a deletion

 @param removableQuotes - quotes of the envelope's addition claims, whose
 atoms may go

 @param sourceKeys - keys of the atoms the source carries

 @returns Unexcused losses and free gains

 @example
 ```ts
 const delta = markupDelta({ before, after, removableQuotes: [], sourceKeys, },);
 ```
 */
export function markupDelta(
  {
    before,
    after,
    removableQuotes,
    sourceKeys,
  }: {
    readonly before: string;
    readonly after: string;
    readonly removableQuotes: readonly string[];
    readonly sourceKeys: ReadonlySet<string>;
  },
): MarkupDelta {
  /**
   Atoms the replaced text carried.
   */
  const beforeAtoms = scanMarkupAtoms({ text: before, },);

  /**
   Atoms the edit wrote.
   */
  const afterAtoms = scanMarkupAtoms({ text: after, },);

  /**
   Losses less those an addition quote licensed.
   */
  const lost = unmatched({
    left: unmatched({
      left: beforeAtoms,
      right: afterAtoms,
    },),
    right: removableQuotes.flatMap(function toAtoms(quote,) {
      return scanMarkupAtoms({ text: quote, },);
    },),
  },);

  // LOSSES ARE PAIRED, one for one, with the atom that re-marked them, and the
  // written atoms left over are the gains. SAME KIND FIRST, so an authored tag
  // rewritten as a tag does not use up the source's own markup another loss
  // could pair with. One pass in order that appends and removes in place,
  // since copying both lists at every loss bought nothing (ledger B70).
  /**
   Losses no written atom re-marked.
   */
  const unexcused: MarkupAtom[] = [];
  /**
   Written atoms no loss has paired with yet.
   */
  const gained = [
    ...unmatched({
      left: afterAtoms,
      right: beforeAtoms,
    },),
  ];
  for (const atom of lost) {
    /**
     Whether this loss can be re-marked at all.
     */
    const authored = (!MARKUP_IDENTIFIER_KINDS.has(atom.kind,)) && (!sourceKeys.has(keyOf(atom,),));

    /**
     Written atom of the same kind, -1 where none.
     */
    const sameKind = gained.findIndex(function isSameKind(written,): boolean {
      return written.kind === atom.kind;
    },);

    /**
     Written atom that re-marks this loss, same kind first, else one the
     source carries; -1 where none does or the loss cannot be re-marked.
     */
    const pairedAt = (!authored)
      ? -1
      : ((sameKind === (-1))
        ? gained.findIndex(function isSourceMarkup(written,): boolean {
          return sourceKeys.has(keyOf(written,),);
        },)
        : sameKind);
    if (pairedAt === (-1)) {
      unexcused.push(atom,);
      continue;
    }
    gained.splice(
      pairedAt,
      1,
    );
  }
  return {
    unexcused,
    gained,
  };
}

/**
 Keys the patch loses more often than it writes, among the edits standing.

 @param deltas - every edit's delta

 @param standing - which edits still stand

 @returns Keys some standing edit loses that no other standing edit writes

 @example
 ```ts
 const short = shortKeys({ deltas, standing, },);
 ```
 */
function shortKeys(
  {
    deltas,
    standing,
  }: {
    readonly deltas: readonly MarkupDelta[];
    readonly standing: readonly boolean[];
  },
): ReadonlySet<string> {
  /**
   Deltas of the edits still standing.
   */
  const kept = deltas.filter(function stands(
    _delta,
    index,
  ): boolean {
    return standing[index] === true;
  },);
  return new Set(unmatched({
    left: kept.flatMap(function toLosses(delta,) {
      return delta.unexcused;
    },),
    right: kept.flatMap(function toGains(delta,) {
      return delta.gained;
    },),
  },)
    .map(keyOf,),);
}

/**
 Settles which edits of one patch lose markup the patch writes nowhere else,
 and refuses them.

 A FIXED POINT, reached in at most one round per edit: each round refuses
 every standing edit that loses a key the standing edits lose more often than
 they write, and a refused edit's written atoms stop excusing anything, so
 the next round may refuse more. A key two edits both lose and one sibling
 writes refuses both, since nothing says which of the two the sibling
 answered.

 @param deltas - each edit's delta, in patch order

 @returns For each edit, the kinds it was refused for, empty where it stands

 @example
 ```ts
 const refusals = settleMarkupMoves({ deltas, },);
 ```
 */
export function settleMarkupMoves(
  { deltas, }: { readonly deltas: readonly MarkupDelta[]; },
): readonly (readonly MarkupAtomKind[])[] {
  /**
   Which edits stand once no round refuses anything more.
   */
  const standing = Array.from({ length: deltas.length + 1, },)
    .reduce<readonly boolean[]>(
    function refineOnce(current,) {
        /**
         Keys the standing edits cannot account for.
         */
      const short = shortKeys({
        deltas,
        standing: current,
      },);
      return current.map(function stillStands(
        isStanding,
        index,
      ): boolean {
          /**
           Losses this edit has not been excused, present by the equal
           lengths of the deltas and the standings.
           */
        const { unexcused, } = nonNullishOrThrow(deltas[index],);
        return isStanding
          && (!unexcused.some(function isShort(atom,): boolean {
            return short.has(keyOf(atom,),);
          },));
      },);
    },
    deltas.map(function startsStanding(): boolean {
      return true;
    },),
    );
  return deltas.map(function toRefusal(
    delta,
    index,
  ): readonly MarkupAtomKind[] {
    return (standing[index] === true)
      ? []
      : [...new Set(delta.unexcused
        .map(function toKind(atom,): MarkupAtomKind {
          return atom.kind;
        },),),];
  },);
}

//endregion Markup atom preservation
