import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { autolinkAsText, } from './source-ships-links.ts';
import {
  imageTextRanges,
  rangeOf,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships image alternative text
// THE ALTERNATIVE TEXT OF AN IMAGE, read as the inline content it is. The plain parse holds it as a string, built from
// the inline content between the brackets: an angle autolink in it is its address alone, and a code span is its content
// as typed. The page must show the same, so in the alternative text an angle autolink is written as its address without
// the angle brackets MDX reads as a tag (the way an autolink inside the text of a link is written), and a code span is
// copied as it stands, since a backslash written inside one is a character of the code and shows. The rest of the
// alternative text is text, as it was before.
//
// The alternative text is parsed on its own, behind a no-break space that Markdown reads as white space, so that nothing
// in its first line reads as the start of a block. A text that is not one paragraph on its own (a line of it that a
// container's mark or a block start would turn into another block) is left as text throughout, which is the reading the
// text had before.

/**
 A character Markdown reads as white space that starts no block, put before the alternative text so its first line is a
 paragraph line whatever it begins with.
 */
const NO_BREAK_SPACE = ' ';

/**
 What the alternative text of an image holds apart from text.
 */
type HeldInAlternative = {
  /**
   Stretches copied as they stand (code spans) or rewritten (angle autolinks), in text order.
   */
  readonly stretches: readonly TextRange[];

  /**
   The stretches rewritten, in text order.
   */
  readonly edits: readonly TextEdit[];
};

/**
 What an alternative text holds apart from text when it holds none, or cannot be read as one paragraph.
 */
const NOTHING_HELD: HeldInAlternative = {
  stretches: [],
  edits: [],
};

/**
 What a reader sees of an image as text, and what is rewritten in it.
 */
export type ImageAltReading = {
  /**
   Stretches a reader sees as text, in text order.
   */
  readonly text: readonly TextRange[];

  /**
   Stretches rewritten, in text order.
   */
  readonly edits: readonly TextEdit[];
};

/**
 Finds the code spans and the angle autolinks of an alternative text.

 @param source - text

 @param alternative - where the alternative text stands in it

 @returns The stretches in text order, none where the alternative text holds none or is not one paragraph

 @example
 ```ts
 const held = heldInAlternativeOf({ source: '![<https://example.net/a>](u.png)', alternative: { start: 2, end: 25, }, },);
 ```
 */
function heldInAlternativeOf(
  {
    source,
    alternative,
  }: {
    readonly source: string;
    readonly alternative: TextRange;
  },
): HeldInAlternative {
  /**
   The alternative text as typed.
   */
  const typed = source.slice(
    alternative.start,
    alternative.end,
  );
  if ((!typed.includes('<',)) && (!typed.includes('`',)))
    return NOTHING_HELD;
  /**
   Blocks of the alternative text parsed on its own.
   */
  const { children, } = parseMarkdownMathBody({ body: `${NO_BREAK_SPACE}${typed}`, },);
  if (children.length !== 1)
    return NOTHING_HELD;
  /**
   The paragraph the alternative text is, when it is one and no more.
   */
  const [paragraph,] = children;
  if (paragraph?.type !== 'paragraph')
    return NOTHING_HELD;
  /**
   The stretches found, with what replaces them where something does.
   */
  const found: (TextRange & { readonly replacement?: string; })[] = [];
  /**
   Nodes still to read.
   */
  const pending: TreeNode[] = [paragraph,];
  for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
    /**
     Where the node stands in the parsed text.
     */
    const parsed = rangeOf({ node: next, },);
    /**
     Where it stands in the text, the no-break space taken off.
     */
    const range = {
      start: (parsed.start - NO_BREAK_SPACE.length) + alternative.start,
      end: (parsed.end - NO_BREAK_SPACE.length) + alternative.start,
    };
    if (next.type === 'inlineCode') {
      found.push(range,);
      continue;
    }
    if ((next.type === 'link') && (source.charAt(range.start,) === '<')) {
      found.push({
        ...range,
        replacement: autolinkAsText({
          source,
          range,
        },),
      },);
      continue;
    }
    if ('children' in next)
      pending.push(...next.children,);
  }
  /**
   The stretches found, in text order.
   */
  const sorted = found.toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
  return {
    stretches: sorted,
    edits: sorted.flatMap(function toEdit(stretch,): readonly TextEdit[] {
      return (stretch.replacement === undefined)
        ? []
        : [
          {
            start: stretch.start,
            end: stretch.end,
            replacement: stretch.replacement,
          },
        ];
    },),
  };
}

/**
 Reads what a reader sees of an image as text, and what is rewritten in it.

 @param source - text

 @param node - `image` or an `imageReference` node of the parse

 @param range - where the node stands

 @returns The alternative text less its code spans and angle autolinks, which are not text, and the second label of a
 full reference; and the autolinks written as their addresses

 @example
 ```ts
 const { text, edits, } = imageAltReading({ source: '![图](u.png)', node, range: { start: 0, end: 10, }, },);
 ```
 */
export function imageAltReading(
  {
    source,
    node,
    range,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
  },
): ImageAltReading {
  /**
   The alternative text first, and the second label of a full reference after it.
   */
  const [first, ...labels] = imageTextRanges({
    source,
    node,
    range,
  },);
  /**
   The alternative text, which every image has.
   */
  const alternative = nonNullishOrThrow(first,);
  /**
   The code spans and the angle autolinks of the alternative text.
   */
  const held = heldInAlternativeOf({
    source,
    alternative,
  },);
  /**
   The alternative text cut around the stretches held.
   */
  const text: TextRange[] = [];
  /**
   Where the next stretch of text starts.
   */
  const cursor = { at: alternative.start, };
  for (const stretch of held.stretches) {
    if (stretch.start > cursor.at) {
      text.push({
        start: cursor.at,
        end: stretch.start,
      },);
    }
    cursor.at = stretch.end;
  }
  if (alternative.end > cursor.at) {
    text.push({
      start: cursor.at,
      end: alternative.end,
    },);
  }
  return {
    text: [
      ...text,
      ...labels,
    ],
    edits: held.edits,
  };
}

//endregion Source ships image alternative text
