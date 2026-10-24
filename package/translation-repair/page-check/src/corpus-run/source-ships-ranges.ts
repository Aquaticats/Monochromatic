import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Source ships ranges
// STRETCHES OF A TEXT AND THE ARITHMETIC ON THEM that the readers of a parse (`source-ships-spans.ts`,
// `source-ships-footnote.ts`) share: where a node stands, where a bracket closes, which characters of an original a
// nested text was cut from.

/**
 A stretch of the text, the end exclusive, in UTF-16 units.
 */
export type TextRange = {
  /**
   Index of the first character.
   */
  readonly start: number;

  /**
   Index after the last character.
   */
  readonly end: number;
};

/**
 A stretch of the text and what stands in its place.
 */
export type TextEdit = TextRange & {
  /**
   Text written instead of the stretch.
   */
  readonly replacement: string;
};

/**
 What a parse says of a text.
 */
export type SourceSpans = {
  /**
   Stretches a reader sees as text, in which a brace or a less-than sign is syntax for the strict grammar.
   */
  readonly text: readonly TextRange[];

  /**
   Stretches of raw HTML blocks and indented code, which the strict grammar reads as a paragraph and a reader must see
   as they stand: every character Markdown reads as syntax there is escaped, and no address is read in them.
   */
  readonly literal: readonly TextRange[];

  /**
   Stretches rewritten whole, in text order and not overlapping.
   */
  readonly edits: readonly TextEdit[];

  /**
   Links and link references, each from its first bracket to its last
   character, inside whose text GFM reads no bare address.
   */
  readonly insideLinks: readonly TextRange[];

  /**
   Where each run of inline content starts (a paragraph, a heading, a table cell), in the order the parse gives them:
   a label opened in one is not open in the next.
   */
  readonly streamStarts: readonly number[];
};

/**
 Characters a backslash escapes in Markdown: the ASCII punctuation.
 */
export const ESCAPABLE = '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~';

/**
 What a fenced code block starts with, as the parse positions one: the fence itself.
 */
export const FENCE_MARKS: readonly string[] = [
  '```',
  '~~~',
];

/**
 Where a node's characters start and end.

 @param node - node of the parse

 @returns Range its position gives

 @throws {@link Error} when the node has no position, which the reading used never builds

 @example
 ```ts
 const { start, end, } = rangeOf({ node, },);
 ```
 */
export function rangeOf({ node, }: { readonly node: TreeNode; },): TextRange {
  /**
   Where the node starts.
   */
  const start = node.position
    ?.start
    .offset;
  /**
   Where the node ends.
   */
  const end = node.position
    ?.end
    .offset;
  if ((start === undefined) || (end === undefined)) {
    throw new Error(
      `unreachable: a ${node.type} node of the plain reading carries no position, though that reading builds `
        + 'every one with one',
    );
  }
  return {
    start,
    end,
  };
}

/**
 Whether the text has one of the strings at an index.

 @param text - text

 @param at - index to look at

 @param candidates - strings to try

 @returns Whether any of them starts at the index

 @example
 ```ts
 const fenced = startsAny({ text: '```ts', at: 0, candidates: ['```', '~~~',], },);
 ```
 */
export function startsAny(
  {
    text,
    at,
    candidates,
  }: {
    readonly text: string;
    readonly at: number;
    readonly candidates: readonly string[];
  },
): boolean {
  return candidates.some(function startsHere(candidate,): boolean {
    return text.startsWith(
      candidate,
      at,
    );
  },);
}

/**
 Where the bracket closing a label stands.

 @param source - text

 @param from - index just past the opening bracket

 @param limit - index the label cannot run past

 @returns Index of the closing bracket, or the limit when none closes within it

 @example
 ```ts
 const close = closingBracket({ source: '![a [b]](u)', from: 2, limit: 11, },);
 ```
 */
export function closingBracket(
  {
    source,
    from,
    limit,
  }: {
    readonly source: string;
    readonly from: number;
    readonly limit: number;
  },
): number {
  /**
   Brackets opened inside the label and not yet closed.
   */
  const open = { depth: 1, };
  for (let at = from; at < limit; at += 1) {
    /**
     Character read.
     */
    const character = source.charAt(at,);
    if (character === '\\') {
      at += 1;
      continue;
    }
    if (character === '[')
      open.depth += 1;
    if (character === ']')
      open.depth -= 1;
    if (open.depth === 0)
      return at;
  }
  return limit;
}

/**
 The indices of a stretch, each one.

 @param range - stretch

 @returns Every index from its start to before its end

 @example
 ```ts
 const indices = indicesOf({ range: { start: 3, end: 6, }, },);
 ```
 */
export function indicesOf({ range, }: { readonly range: TextRange; },): readonly number[] {
  return Array.from(
    { length: range.end - range.start, },
    function indexAt(
      _unused,
      offset,
    ): number {
      return range.start + offset;
    },
  );
}

/**
 Maps a stretch of a nested text back to the stretches of the text it was cut from.

 @param range - stretch of the nested text

 @param indices - for each index of the nested text, the index it came from

 @returns One stretch for each unbroken run of the original's characters

 @throws {@link Error} when an index of the stretch has no origin, which a stretch the nested parse gives never has

 @example
 ```ts
 const back = originalStretches({ range: { start: 0, end: 2, }, indices: [4, 5, 6,], },);
 ```
 */
export function originalStretches(
  {
    range,
    indices,
  }: {
    readonly range: TextRange;
    readonly indices: readonly number[];
  },
): readonly TextRange[] {
  /**
   Stretches found, the last of which is still growing.
   */
  const stretches: TextRange[] = [];
  for (let at = range.start; at < range.end; at += 1) {
    /**
     Index of the original the character came from.
     */
    const origin = nonNullishOrThrow(indices[at],);
    /**
     Stretch the character may continue.
     */
    const last = stretches.at(-1,);
    if ((last !== undefined) && (last.end === origin)) {
      stretches[stretches.length - 1] = {
        start: last.start,
        end: origin + 1,
      };
      continue;
    }
    stretches.push({
      start: origin,
      end: origin + 1,
    },);
  }
  return stretches;
}

/**
 How many characters `![` takes.
 */
const IMAGE_OPENER_LENGTH = 2;

/**
 Where the second label of a full reference stands: the brackets after the text.

 @param source - text

 @param range - where the reference stands

 @param textFrom - index just past the bracket that opens the text

 @returns Label between the second brackets

 @example
 ```ts
 const label = secondLabel({ source: '[x][a {b]', range: { start: 0, end: 9, }, textFrom: 1, },);
 ```
 */
export function secondLabel(
  {
    source,
    range,
    textFrom,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly textFrom: number;
  },
): TextRange {
  return {
    start: closingBracket({
      source,
      from: textFrom,
      limit: range.end,
    },) + 2,
    end: range.end - 1,
  };
}

/**
 The stretches of an image or an image reference that a reader sees as text: the alternative text, and the second label
 of a full reference. The destination and the title are not.

 @param source - text

 @param node - `image` or an `imageReference` node of the parse

 @param range - where the node stands

 @returns Stretches, in text order

 @example
 ```ts
 const stretches = imageTextRanges({ source: '![图](u.png)', node, range: { start: 0, end: 10, }, },);
 ```
 */
export function imageTextRanges(
  {
    source,
    node,
    range,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
  },
): readonly TextRange[] {
  /**
   The alternative text, from after the opener to the bracket that closes it.
   */
  const alternative = {
    start: range.start + IMAGE_OPENER_LENGTH,
    end: closingBracket({
      source,
      from: range.start + IMAGE_OPENER_LENGTH,
      limit: range.end,
    },),
  };
  if ((node.type === 'imageReference') && (node.referenceType === 'full')) {
    return [
      alternative,
      secondLabel({
        source,
        range,
        textFrom: range.start + IMAGE_OPENER_LENGTH,
      },),
    ];
  }
  return [alternative,];
}

//endregion Source ships ranges
