import type {
  ListItem,
  Root,
} from 'mdast';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  spliceDisjointEdits,
  type SpliceEdit,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type DeepReadonlyData,
  parseMarkdownBody,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  pageTextBySlice,
  pageTextOf,
  type SliceReplacement,
  slicesInOrder,
  withRewrittenText,
} from '@monochromatic-dev/translation-repair-page-pass/ts';

//region List spread restore
// CLASS ONE HUNDRED SEVENTEEN (CuspariaKLSY13, 2026-09-24). The original and
// the archive write a four-item ordered list with a blank line between its
// items, and all six consolidation proposals wrote it with none, so the page
// shipped a tight list where both references are loose. Loose or tight is
// how the list renders (a loose list sets each item as a paragraph), not a
// wording any judge weighs, and no floor read it: the block shape names a
// list's kind and whether it is ordered, never its spacing. Measured over
// the pinned corpus, every list an original writes is loose, and no archive
// list differs in spacing from the original's list in its place. THE PAGE
// DECIDES ONCE, HERE, from the archive: where the page carries the same
// number of top-level lists as the archive's text for the slice, each with
// the archive's ordering and item count, every list takes the archive's
// spacing between its items. A list whose own items hold blank lines cannot
// be made tight by its gaps and is left, as is a slice the archive never
// carried.

/**
 Line ending the corpus read folds every page to.
 */
const LF = '\n';

/**
 Text between two neighbouring items of one list, by offset in the slice.
 */
type ItemGap = {
  /**
   Offset just past the earlier item's last character.
   */
  readonly start: number;

  /**
   Offset of the later item's marker.
   */
  readonly end: number;
};

/**
 One top-level list, as far as its spacing goes.
 */
type ListSpacing = {
  /**
   Whether the list is numbered.
   */
  readonly ordered: boolean;

  /**
   How many items it holds.
   */
  readonly items: number;

  /**
   Gaps between neighbouring items, in order.
   */
  readonly gaps: readonly ItemGap[];

  /**
   How many of those gaps hold a blank line.
   */
  readonly blankGaps: number;

  /**
   Whether any item holds a blank line between its own blocks, which makes
   the list loose whatever its gaps say.
   */
  readonly itemsSpread: boolean;
};

/**
 Spacing of a list, as one of the two settled shapes or neither.
 */
type SpacingShape = 'loose' | 'tight' | 'mixed';

/**
 Counts the line endings in a stretch of text by index scan.

 @param text - stretch between two items

 @returns How many line endings it holds

 @example
 ```ts
 lineEndings({ text: '\n\n', },); // 2
 ```
 */
function lineEndings({ text, }: { readonly text: string; },): number {
  return text
    .split(LF,)
    .length
    - 1;
}

/**
 Number of lines a stretch of text between two items holds, blank ones
 included.

 @param text - whole slice text

 @param gap - stretch between two items

 @returns Line endings inside the stretch

 @example
 ```ts
 gapLineEndings({ text, gap, },); // 2 for a blank line between items
 ```
 */
function gapLineEndings(
  {
    text,
    gap,
  }: {
    readonly text: string;
    readonly gap: ItemGap;
  },
): number {
  /**
   Stretch between the two items.
   */
  const between = text.slice(
    gap.start,
    gap.end,
  );
  return lineEndings({ text: between, },);
}

/**
 Offsets of a parsed list item's first and last character.

 EVERY NODE THE PARSER BUILDS CARRIES POSITIONS, so an item without them is
 no input a parse produces. `mdast-util-from-markdown` sets `position.start`
 when it enters a node and `position.end` when it leaves it
 (`lib/index.js`, `enter` and `exit`), and a read of every list item of the
 184 pinned pages and of 13 hand-built lists found none without. Skipping the
 item, and with it the whole list, would move every later list's position
 and let a page pass with a list the archive's spacing was never applied to.

 @param item - one item of a parsed list

 @returns Offsets of the item's first and last character

 @throws Error when the item carries no start or end offset

 @example
 ```ts
 const extent = listItemExtent({ item, },);
 ```
 */
export function listItemExtent({ item, }: { readonly item: DeepReadonlyData<ListItem>; },): ItemGap {
  /**
   Item's first offset.
   */
  const start = item.position
    ?.start
    .offset;

  /**
   Item's last offset.
   */
  const end = item.position
    ?.end
    .offset;
  if ((start === undefined) || (end === undefined))
    throw new Error(
      'unreachable: a parsed list item carries no start or end offset, though the parser sets a position on every '
        + 'node it builds, so skipping it would drop its whole list and move every later list',
    );
  return {
    start,
    end,
  };
}

/**
 Reads the top-level lists of a slice's text.

 @param text - slice text, folded to LF

 @returns Every top-level list's spacing, in document order

 @example
 ```ts
 const lists = readLists({ text: '1. Cat\n\n2. Dog', },);
 ```
 */
function readLists({ text, }: { readonly text: string; },): readonly ListSpacing[] {
  /**
   Slice under plain markdown, which reads any text.
   */
  const root: DeepReadonlyData<Root> = parseMarkdownBody({ body: text, },);
  return root.children
    .flatMap(function asList(node,): readonly ListSpacing[] {
      if (node.type !== 'list')
        return [];
      /**
       Items of the list.
       */
      const listItems = node.children;
      /**
       Offsets of each item's first and last character.
       */
      const extents = listItems
        .map(function extentOf(item,): ItemGap {
          return listItemExtent({ item, },);
        },);
      /**
       Gaps from each item's end to the next item's start.
       */
      const gaps = extents.slice(1,)
        .map(function gapBefore(
          later,
          at,
        ): ItemGap {
          /**
           Item before this one; present since `at` indexes the list from
           its first item.
           */
          const earlier = nonNullishOrThrow(extents[at],);
          return {
            start: earlier.end,
            end: later.start,
          };
        },);
      /**
       Gaps holding a blank line.
       */
      const blank = gaps.filter(function holdsBlankLine(gap,): boolean {
        return gapLineEndings({
          text,
          gap,
        },) >= 2;
      },);
      return [
        {
          ordered: node.ordered === true,
          items: listItems.length,
          gaps,
          blankGaps: blank.length,
          itemsSpread: listItems.some(function spread(item,): boolean {
            return item.spread === true;
          },),
        },
      ];
    },);
}

/**
 Reads a list's spacing as loose, tight, or neither.

 @param list - one list

 @returns Loose when every gap holds a blank line, tight when none does and
 no item is spread, mixed otherwise

 @example
 ```ts
 spacingOf({ list, },); // 'loose'
 ```
 */
function spacingOf({ list, }: { readonly list: ListSpacing; },): SpacingShape {
  /**
   How many gaps the list has.
   */
  const gapCount = list.gaps
    .length;
  // A ONE-ITEM LIST HAS NO SPACING TO RESTORE. No case can tell this line
  // from its absence: lists pair only at the same item count, so both sides
  // of a pair lack gaps together and read alike either way. It stays so the
  // shape says what a gapless list is, not what a fallthrough makes of it.
  if (gapCount === 0)
    return 'mixed';
  if (list.blankGaps === gapCount)
    return 'loose';
  if ((list.blankGaps === 0) && (!list.itemsSpread))
    return 'tight';
  return 'mixed';
}

/**
 Rewrites one gap to the wanted spacing.

 @param gapText - text between two items as the page wrote it

 @param wanted - spacing the archive writes

 @returns Gap text with the archive's spacing, the indentation before the
 later item kept

 @example
 ```ts
 respaceGap({ gapText: '\n', wanted: 'loose', },); // '\n\n'
 ```
 */
function respaceGap(
  {
    gapText,
    wanted,
  }: {
    readonly gapText: string;
    readonly wanted: 'loose' | 'tight';
  },
): string {
  /**
   Indentation between the last line ending and the later item's marker.
   */
  const indent = gapText.slice(gapText.lastIndexOf(LF,) + 1,);
  return (wanted === 'loose') ? `${LF}${LF}${indent}` : `${LF}${indent}`;
}

/**
 Page text with one list's gaps respaced, in one pass: the gaps lie between
 the list's items, so each is read from the text as given and written at its
 own offsets (ledger B70).

 @param text - page text of the slice

 @param list - list whose gaps to respace

 @param wanted - spacing the archive writes

 @returns Rewritten text

 @example
 ```ts
 const text = respaceList({ text, list, wanted: 'loose', },);
 ```
 */
function respaceList(
  {
    text,
    list,
    wanted,
  }: {
    readonly text: string;
    readonly list: ListSpacing;
    readonly wanted: 'loose' | 'tight';
  },
): string {
  return spliceDisjointEdits({
    text,
    edits: list.gaps
      .map(function respace(gap,): SpliceEdit {
        return {
          start: gap.start,
          end: gap.end,
          text: respaceGap({
            gapText: text.slice(
              gap.start,
              gap.end,
            ),
            wanted,
          },),
        };
      },),
  },);
}

/**
 One list the pass will respace.
 */
type ListRespace = {
  /**
   Position of the list among the slice's top-level lists.
   */
  readonly at: number;

  /**
   Page's list.
   */
  readonly list: ListSpacing;

  /**
   How many items the list holds, for the finding.
   */
  readonly items: number;

  /**
   Spacing the page wrote.
   */
  readonly from: SpacingShape;

  /**
   Spacing the archive writes.
   */
  readonly to: 'loose' | 'tight';
};

/**
 Whether two lists agree in ordering and item count, so one can take the
 other's spacing.

 @param left - one list

 @param right - other

 @returns True when both are numbered or both bulleted, with as many items

 @example
 ```ts
 sameListShape({ left: archived, right: list, },); // true
 ```
 */
function sameListShape(
  {
    left,
    right,
  }: {
    readonly left: ListSpacing;
    readonly right: ListSpacing;
  },
): boolean {
  return (left.ordered === right.ordered) && (left.items === right.items);
}

/**
 Lists of one slice whose spacing the archive settles and the page missed.

 @param pageLists - page's top-level lists

 @param archiveLists - archive's top-level lists

 @returns One respace per list to rewrite, empty where the two sides do not
 carry the same lists

 @example
 ```ts
 const respaces = listRespaces({ pageLists, archiveLists, },);
 ```
 */
function listRespaces(
  {
    pageLists,
    archiveLists,
  }: {
    readonly pageLists: readonly ListSpacing[];
    readonly archiveLists: readonly ListSpacing[];
  },
): readonly ListRespace[] {
  if (pageLists.length !== archiveLists.length)
    return [];
  return pageLists.flatMap(function respaceOf(
    list,
    at,
  ): readonly ListRespace[] {
    /**
     Archive's list in the same place; present since the counts were
     compared and they run together.
     */
    const archived = nonNullishOrThrow(archiveLists[at],);
    if (!sameListShape({
      left: archived,
      right: list,
    },))
      return [];
    /**
     Spacing the archive writes.
     */
    const to = spacingOf({ list: archived, },);
    /**
     Spacing the page wrote.
     */
    const from = spacingOf({ list, },);
    if ((to === 'mixed') || (from === to))
      return [];
    if ((to === 'tight') && list.itemsSpread)
      return [];
    return [
      {
        at,
        list,
        items: list.items,
        from,
        to,
      },
    ];
  },);
}

/**
 Restores the archive's spacing between list items, slice by slice, where
 the page carries the archive's lists in order.

 @param slices - prepared pairs, whose archive side fixes the spacing

 @param replacements - what the page would write per slice

 @returns Replacements with the spacing restored, the rewritten rows alone,
 and one finding per respaced list

 @example
 ```ts
 const restored = restoreListSpread({ slices, replacements, },);
 ```
 */
export function restoreListSpread(
  {
    slices,
    replacements,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  /**
   Page text per slice.
   */
  const pageText = pageTextBySlice({
    slices,
    replacements,
  },);
  /**
   Slices a replacement covers.
   */
  const replaced = new Set(replacements.map(function indexOf(replacement,): number {
    return replacement.sliceIndex;
  },),);
  /**
   Page text per slice after the respacing.
   */
  const rewritten = new Map<number, string>();
  /**
   One finding per respaced list.
   */
  const findings: string[] = [];
  for (const slice of slicesInOrder({ slices, },)) {
    /**
     Index of this slice.
     */
    const { sliceIndex, } = slice.target;
    if (!replaced.has(sliceIndex,))
      continue;
    /**
     Lists the archive writes for this slice.
     */
    const archiveLists = readLists({ text: slice.target
      .text, },);
    if (archiveLists.length === 0)
      continue;
    /**
     Page text of this slice.
     */
    const text = pageTextOf({
      pageText,
      sliceIndex,
    },);
    /**
     Lists the archive's spacing rewrites, last first so each rewrite
     leaves the earlier lists' offsets true.
     */
    const respaces = listRespaces({
      pageLists: readLists({ text, },),
      archiveLists,
    },);
    if (respaces.length === 0)
      continue;
    rewritten.set(
      sliceIndex,
      respaces.toReversed()
        .reduce(
        function respace(
          current,
          one,
        ): string {
          return respaceList({
            text: current,
            list: one.list,
            wanted: one.to,
          },);
        },
        text,
        ),
    );
    for (const one of respaces) {
      findings.push(
        `list-spread-restored (slice ${String(sliceIndex,)}: list ${String(one.at,)} of ${
          String(one.items,)
        } ${
          wordForCount({
            count: one.items,
            one: 'item',
            many: 'items',
          },)
        } from ${one.from} to ${one.to}; the archive writes this list ${
          (one.to === 'loose') ? 'with' : 'without'
        } a blank line between its items)`,
      );
    }
  }
  /**
   Replacements with the rewritten slices folded in.
   */
  const folded = withRewrittenText({
    replacements,
    rewritten,
  },);
  return {
    ...folded,
    findings,
  };
}

//endregion List spread restore
