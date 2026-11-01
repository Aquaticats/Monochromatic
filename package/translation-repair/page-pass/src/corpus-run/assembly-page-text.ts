import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { SliceReplacement, } from '../splice-slices.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Assembly page text
// WHAT THE PAGE CARRIES PER SLICE, read the same way by every page-assembly
// pass: the replacement a lane wrote, else the archive's own text. A pass
// that changes a slice the archive alone carried writes a replacement for
// it, so the assembler and the override row both see the page's text.

/**
 Text the page carries for every slice, by slice index.

 @param slices - prepared pairs, whose archive text is the fallback

 @param replacements - what the lanes wrote per slice

 @returns Page text per slice index

 @example
 ```ts
 const text = pageTextBySlice({ slices, replacements, },);
 ```
 */
export function pageTextBySlice(
  {
    slices,
    replacements,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): ReadonlyMap<number, string> {
  /**
   Page text per slice, the archive's first.
   */
  const text = new Map<number, string>(slices.map(function fromArchive(slice,): readonly [
    number,
    string,
  ] {
    /**
     Archive side of the slice.
     */
    const { target, } = slice;
    return [
      target.sliceIndex,
      target.text,
    ];
  },),);
  for (const replacement of replacements) {
    text.set(
      replacement.sliceIndex,
      replacement.replacementText,
    );
  }
  return text;
}

/**
 Refusal to read page text for a slice the page text was never built from.

 @example
 ```ts
 throw new SliceNotOnPageError({ sliceIndex: 7, },);
 ```
 */
export class SliceNotOnPageError extends Error {
  /**
   Declares this message safe to forward: it names a slice index and nothing
   else.
   */
  readonly messageNamesOnly: true = true;

  /**
   Slice the page text lacks.
   */
  readonly sliceIndex: number;

  /**
   Names the slice the page text lacks and what that means.

   @param sliceIndex - slice asked for, which the map was not built from
   */
  constructor({ sliceIndex, }: { readonly sliceIndex: number; },) {
    super(
      `slice ${String(sliceIndex,)} is not among the slices this page text was built from, so a page-assembly `
        + 'pass asked for a slice of some other list; reading it as empty text would ship that section as nothing',
    );
    this.name = 'SliceNotOnPageError';
    this.sliceIndex = sliceIndex;
  }
}

/**
 Text the page carries for one slice, read from what `pageTextBySlice` built.

 THROWS RATHER THAN READING A MISSING SLICE AS EMPTY TEXT. The map holds
 every slice it was built from, so a missing one is a slice from some other
 list, and a pass that took it for empty text once wrote an empty override
 for a section the page never replaced (ledger B83).

 @param pageText - page text per slice, as `pageTextBySlice` built it

 @param sliceIndex - slice of the same list the map was built from

 @returns The slice's page text

 @throws SliceNotOnPageError where the map lacks the slice

 @example
 ```ts
 const text = pageTextOf({ pageText, sliceIndex: slice.target.sliceIndex, },);
 ```
 */
export function pageTextOf(
  {
    pageText,
    sliceIndex,
  }: {
    readonly pageText: ReadonlyMap<number, string>;
    readonly sliceIndex: number;
  },
): string {
  /**
   The slice's page text, if the map holds the slice.
   */
  const text = pageText.get(sliceIndex,);
  if (text === undefined)
    throw new SliceNotOnPageError({ sliceIndex, },);
  return text;
}

/**
 Slices in slice-index order.

 @param slices - prepared pairs in any order

 @returns The same pairs sorted by their archive slice index

 @example
 ```ts
 for (const slice of slicesInOrder({ slices, },)) { /* ... *\/ }
 ```
 */
export function slicesInOrder({ slices, }: { readonly slices: readonly ChunkPair[]; },): readonly ChunkPair[] {
  return slices.toSorted(function byIndex(
    left,
    right,
  ): number {
    /**
     Left slice's index.
     */
    const leftIndex = left.target
      .sliceIndex;
    /**
     Right slice's index.
     */
    const rightIndex = right.target
      .sliceIndex;
    return leftIndex - rightIndex;
  },);
}

/**
 Rows a pass changed, read position by position against the rows it was
 given.

 The casing restore and the name gloss restore each kept their own copy of
 this comparison (audit area six, 2026-09-28).

 @param before - rows as the pass received them

 @param after - the same rows, in the same order, as the pass left them

 @returns Rows of `after` whose text differs from the row at their position

 @example
 ```ts
 const restored = rowsChangedBy({ before: replacements, after: rewritten, },);
 ```
 */
export function rowsChangedBy(
  {
    before,
    after,
  }: {
    readonly before: readonly SliceReplacement[];
    readonly after: readonly SliceReplacement[];
  },
): readonly SliceReplacement[] {
  return after.filter(function changed(
    row,
    index,
  ): boolean {
    return row.replacementText
      !== before[index]
        ?.replacementText;
  },);
}

/**
 Replacements with a pass's rewritten slices folded in: an existing
 replacement is rewritten in place, a slice the archive alone carried gains
 one.

 @param replacements - what the page would write before the pass

 @param rewritten - text the pass settled per slice it changed

 @returns Replacements after the pass, and the rewritten rows alone

 @example
 ```ts
 const folded = withRewrittenText({ replacements, rewritten, },);
 ```
 */
export function withRewrittenText(
  {
    replacements,
    rewritten,
  }: {
    readonly replacements: readonly SliceReplacement[];
    readonly rewritten: ReadonlyMap<number, string>;
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
} {
  /**
   Slices an existing replacement already covers.
   */
  const covered = new Set(replacements.map(function indexOf(replacement,): number {
    return replacement.sliceIndex;
  },),);
  /**
   Existing replacements, rewritten where the pass changed them.
   */
  const kept = replacements.map(function rewrite(replacement,): SliceReplacement {
    /**
     Text the pass settled for this slice, if any.
     */
    const text = rewritten.get(replacement.sliceIndex,);
    return (text === undefined)
      ? replacement
      : {
        sliceIndex: replacement.sliceIndex,
        replacementText: text,
      };
  },);
  /**
   Replacements for slices the archive alone carried until now.
   */
  const added = [...rewritten.entries(),]
    .filter(function uncovered([sliceIndex,]: ReadonlyEntry<number, string>,): boolean {
      return !covered.has(sliceIndex,);
    },)
    .map(function toReplacement([
      sliceIndex,
      replacementText,
    ]: ReadonlyEntry<number, string>,): SliceReplacement {
      return {
        sliceIndex,
        replacementText,
      };
    },);
  return {
    replacements: [
      ...kept,
      ...added,
    ],
    restored: [...rewritten.entries(),].map(function toRow([
      sliceIndex,
      replacementText,
    ]: ReadonlyEntry<number, string>,): SliceReplacement {
      return {
        sliceIndex,
        replacementText,
      };
    },),
  };
}

/**
 Whether a line is an ATX heading.

 @param line - one line of a document

 @returns True when the line opens with a heading mark

 @example
 ```ts
 isHeadingLine({ line: '### 猫', },); // true
 ```
 */
export function isHeadingLine({ line, }: { readonly line: string; },): boolean {
  return line.startsWith('#',);
}

/**
 Offset where a heading line's marks end.

 @param line - heading line

 @returns Count of leading heading marks

 @example
 ```ts
 markEnd({ line: '### Cat', },); // 3
 ```
 */
function markEnd({ line, }: { readonly line: string; },): number {
  /**
   Offset scanned to.
   */
  let end = 0;
  while ((end < line.length) && (line[end] === '#'))
    end += 1;
  return end;
}

/**
 Splits an ATX heading line into its marks and its title.

 @param line - heading line

 @returns Marks up to the first space, and the title after it, trimmed

 @example
 ```ts
 splitHeading({ line: '### One: Ginger', },); // { marks: '###', title: 'One: Ginger', }
 ```
 */
export function splitHeading({ line, }: { readonly line: string; },): {
  readonly marks: string;
  readonly title: string;
} {
  /**
   Where the marks end.
   */
  const end = markEnd({ line, },);
  return {
    marks: line.slice(
      0,
      end,
    ),
    title: line.slice(end,)
      .trim(),
  };
}

//endregion Assembly page text

//region Heading colons
// A HEADING TITLE'S NUMBER OR NAME SITS AROUND A COLON, fullwidth in the
// original and ASCII on the page; both count.

/**
 Colons a heading title may carry.
 */
const COLONS: readonly string[] = [
  '：',
  ':',
];

/**
 Position of the first colon in a title, or -1.

 @param title - heading title

 @returns Index of the first fullwidth or ASCII colon

 @example
 ```ts
 colonAt({ title: 'One: Ginger', },); // 3
 ```
 */
export function colonAt({ title, }: { readonly title: string; },): number {
  /**
   Earliest colon found so far, -1 for none.
   */
  let earliest = -1;
  for (const colon of COLONS) {
    /**
     Where this colon stands.
     */
    const at = title.indexOf(colon,);
    /**
     Whether this colon stands before any found so far.
     */
    const isEarlier = (earliest === (-1)) || (at < earliest);
    if ((at !== (-1)) && isEarlier)
      earliest = at;
  }
  return earliest;
}

//endregion Heading colons
