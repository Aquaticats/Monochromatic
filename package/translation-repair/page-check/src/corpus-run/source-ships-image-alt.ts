import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { autolinkAsText, } from './source-ships-links.ts';
import {
  imageTextRanges,
  rangeOf,
  type SourceSpans,
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
// The alternative text is parsed as a paragraph of its own, with what stands around it in the image written around it:
// a no-break space, which Markdown reads as white space and which starts no block, so that nothing in its first line
// reads as the start of one, and the closing bracket and a destination, so that its last line is followed by text, as it
// is in the image: a last line that reads as a setext underline, a thematic break or the delimiter row of a table on its
// own reads as none of them there. The marks of a container at the start of its lines are written as spaces first. A
// text that is not one paragraph on its own (a line of it that a block start would turn into another block) is left as
// text throughout, which is the reading the text had before.

/**
 The code point of a no-break space.
 */
const NO_BREAK_SPACE_CODE = 0xA0;

/**
 What stands before the alternative text when it is parsed, which starts no block and is white space to the inline
 syntax.
 */
const LEAD = String.fromCodePoint(NO_BREAK_SPACE_CODE,);

/**
 What stands after the alternative text when it is parsed: the bracket that closes it in the image, and a destination.
 */
const TRAIL = '](u)';

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
 Characters a container writes at the start of a line it holds: the mark of a block quote and the indentation of a list
 item, a footnote or a nested block.
 */
const CONTAINER_MARKS = ' \t>';

/**
 Counts the characters at the start of a line that a container writes there.

 @param line - one line of the alternative text

 @returns How many characters from the start of the line are marks, all of them where the line holds no other

 @example
 ```ts
 const marks = marksLengthOf({ line: '> b', },);
 ```
 */
function marksLengthOf({ line, }: { readonly line: string; },): number {
  for (let at = 0; at < line.length; at += 1) {
    if (!CONTAINER_MARKS.includes(line.charAt(at,),))
      return at;
  }
  return line.length;
}

/**
 Writes each character of the container's marks at the start of a line of the alternative text as a space, so that a line
 of an image inside a block quote is a line of a paragraph here as it is there and every offset stands. A greater-than
 sign cannot be text there, since a line that starts with one ends the paragraph the image stands in.

 @param typed - alternative text as typed

 @returns Text with the marks at the start of each line after the first written as spaces, of the same length

 @example
 ```ts
 const flat = withoutContainerMarks({ typed: 'a\n> b', },);
 ```
 */
function withoutContainerMarks({ typed, }: { readonly typed: string; },): string {
  return typed
    .split('\n',)
    .map(function flat(
      line,
      index,
    ): string {
      if (index === 0)
        return line;
      /**
       How many characters at the start of the line are marks.
       */
      const marks = marksLengthOf({ line, },);
      return `${' '.repeat(marks,)}${line.slice(marks,)}`;
    },)
    .join('\n',);
}

/**
 Finds the code spans and the angle autolinks of an alternative text.

 @param source - text

 @param alternative - where the alternative text stands in it

 @returns Stretches in text order, none where the alternative text holds none or is not one paragraph

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
  const { children, } = parseMarkdownMathBody({
    body: `${LEAD}${withoutContainerMarks({ typed, },)}${TRAIL}`,
  },);
  /**
   The paragraph the alternative text is, when it is one and no more.
   */
  const paragraph = (children.length === 1) ? children[0] : undefined;
  if (paragraph?.type !== 'paragraph')
    return NOTHING_HELD;
  /**
   The stretches found, with what replaces them where something does.
   */
  const found: (TextRange & { readonly replacement?: string; })[] = [];
  /**
   Nodes still to read, the last of the paragraph's children on top so they come out in order.
   */
  const pending: TreeNode[] = paragraph
    .children
    .toReversed();
  for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
    /**
     Where the node stands in the parsed text.
     */
    const parsed = rangeOf({ node: next, },);
    /**
     Where it stands in the text, what stands before it in the parsed text taken off.
     */
    const range = {
      start: (parsed.start - LEAD.length) + alternative.start,
      end: (parsed.end - LEAD.length) + alternative.start,
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
    if ('children' in next) {
      pending.push(...next
        .children
        .toReversed(),);
    }
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

 @returns Alternative text less its code spans and angle autolinks, which are not text, and the second label of a
 full reference as the stretches read as text; and the autolinks written as their addresses as the edits

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
): SourceSpans {
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
    literal: [],
    edits: held.edits,
    insideLinks: [],
    streamStarts: [],
  };
}

//endregion Source ships image alternative text
