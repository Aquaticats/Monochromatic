import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { readsAsDisplayMath, } from './source-ships-display-math.ts';
import { indentedFenceMarkEnd, } from './source-ships-fence.ts';
import { hasFootnoteLabel, } from './source-ships-footnote.ts';
import {
  closingBracket,
  rangeOf,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships lines
// LINE ENDINGS THE STRICT GRAMMAR READS A BLOCK AT. MDX starts a flow expression at a `{` and a flow tag at a `<` that
// stand first on a line (after the indentation and the marks of the containers), and a fence at a mark indented four
// spaces or more (`source-ships-fence.ts`), and each ends the paragraph the line would continue. The plain reading
// keeps such a line in the paragraph, so a code span, inline math or the title of a link that runs over the line stays
// whole there, and its brace, tag or mark is no syntax. The strict grammar cuts the span at the line, and the brace,
// tag or mark is read as syntax there with no way to escape it, since a backslash written into code or math shows. What
// is written instead is the line ending, and the indentation and container marks after it, as one space: a line ending
// in a code span or in math is read as a space already, and in a title it is white space.
//
// A FOOTNOTE START ONLY THE PAGE READS is the same case. The strict grammar disables indented code, so a line
// `[^label]:` indented four columns or more after a paragraph starts a footnote definition there, where CommonMark and
// GFM read a lazy line of the paragraph. The span the original pairs across that line (a code span, inline math, emphasis,
// strong emphasis, strikethrough, the text of a link or the alternative text of an image, the title of a link, a hard
// break written with a backslash) is cut on the page, so the line is joined to the one before it and the footnote does
// not start; `footnoteStartsInSpans` says which starts a span holds. A footnote start on the line after a row of a table
// is no start of this kind (`source-ships-footnote-ranges.ts`, `startsOffTables`): a reader of the source takes that line
// for a block of its own, and joining it writes it into the row.

/**
 Node types whose characters are code or math, which no backslash may be written into.
 */
const SPAN_TYPES: ReadonlySet<string> = new Set([
  'inlineCode',
  'inlineMath',
],);

/**
 Node types whose lines are joined at a footnote start only the page reads and nowhere else: emphasis, strong emphasis,
 strikethrough, and the references, whose text a backslash can be written into, so a brace or a less-than sign that
 starts a line in them is escaped as text and needs no join.
 */
const FOOTNOTE_ONLY_TYPES: ReadonlySet<string> = new Set([
  'delete',
  'emphasis',
  'strong',
  'linkReference',
  'imageReference',
],);

/**
 What a search for the start of a line answers when there is none.
 */
const NO_START = -1;

/**
 Characters that stand before the content of a line inside a container: the indentation and the mark of a block quote.
 */
const CONTAINER_PREFIX = ' \t>';

/**
 Characters that start a flow expression or a flow tag.
 */
const FLOW_STARTS = '{<';

/**
 Where the content of the line after a hard break written with a backslash starts.

 @param source - text

 @param node - node of the parse

 @param range - where the node stands

 @returns Index of the first character after the container marks that follow the break when the node is a hard break
 written with a backslash; -1 otherwise

 @example
 ```ts
 const at = contentAfterBreak({ source: 'a\\\n    [^1]: y', node, range: { start: 1, end: 3, }, },); // 7
 ```
 */
function contentAfterBreak(
  {
    source,
    node,
    range,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
  },
): number {
  if ((node.type !== 'break') || (source.charAt(range.start,) !== '\\'))
    return NO_START;
  /**
   Index of the first character after the container marks.
   */
  const content = { at: range.end, };
  while ((content.at < source.length) && CONTAINER_PREFIX.includes(source.charAt(content.at,),))
    content.at += 1;
  return content.at;
}

/**
 Edits that join the lines of a stretch whose later line starts with a
 brace, a less-than sign, an indented fence mark or a footnote start only the page reads.

 @param source - text

 @param range - stretch to look in, whose first character no line ending joins to

 @param footnoteStarts - offsets where a footnote definition starts on the page and not for a reader of the source

 @param joinsFlowStarts - whether a brace, a less-than sign or an indented fence mark starts a block here, which it does in
 a stretch no backslash can be written into and does not in text, where the encoder escapes the brace and the sign

 @returns One edit for each line ending followed, after the container marks, by a brace, a less-than sign, an indented
 fence mark (where those start a block) or one of the footnote starts inside the stretch, replacing the ending and the
 marks with a space

 @example
 ```ts
 const edits = joinedLines({ source: '`a\n{b`', range: { start: 0, end: 6, }, },);
 ```
 */
export function joinedLines(
  {
    source,
    range,
    footnoteStarts = [],
    joinsFlowStarts = true,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly footnoteStarts?: readonly number[];
    readonly joinsFlowStarts?: boolean;
  },
): readonly TextEdit[] {
  /**
   Edits found.
   */
  const edits: TextEdit[] = [];
  for (let at = range.start; at < range.end; at += 1) {
    if (source.charAt(at,) !== '\n')
      continue;
    /**
     Index of the first character after the container marks.
     */
    const content = { at: at + 1, };
    while ((content.at < range.end) && CONTAINER_PREFIX.includes(source.charAt(content.at,),))
      content.at += 1;
    if (content.at >= range.end)
      continue;
    /**
     Whether the strict grammar starts a block at this line.
     */
    const startsBlock = footnoteStarts.includes(content.at,)
      || (joinsFlowStarts
        && (FLOW_STARTS.includes(source.charAt(content.at,),)
          || (indentedFenceMarkEnd({
            source,
            at: content.at,
          },) > content.at)));
    if (!startsBlock)
      continue;
    edits.push({
      start: ((at > range.start) && (source.charAt(at - 1,) === '\r')) ? (at - 1) : at,
      end: content.at,
      replacement: ' ',
    },);
  }
  return edits;
}

/**
 The edit that joins a code span to the line before it when the span starts with a fence mark indented four spaces or
 more: the plain reading takes the run of backticks for the opening of the span, so no backslash can be written into it,
 and the strict grammar takes it for a fence that cuts the paragraph.

 @param source - text

 @param range - where the code span stands

 @param definitionEnds - where each link reference definition of the text ends, since a line that ends a definition
 ends its block in both readings, and the fence mark after it opens a fence in the strict grammar with no paragraph
 line to join to: joining it would make the definition line a paragraph, whose braces are then code

 @returns One edit replacing the line ending before the span and the marks before it with a space; none when the span
 does not start with such a mark, starts the text or follows the end of a definition

 @example
 ```ts
 const range = { start: 6, end: 20, };
 const edits = joinBeforeFenceMark({ source: 'a\n    ```\n`x\n    ```', range, definitionEnds: [], },);
 ```
 */
function joinBeforeFenceMark(
  {
    source,
    range,
    definitionEnds,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly definitionEnds: readonly number[];
  },
): readonly TextEdit[] {
  if (indentedFenceMarkEnd({
    source,
    at: range.start,
  },) <= range.start)
    return [];
  /**
   Index of the first character before the span that is no indentation or container mark.
   */
  const before = { at: range.start - 1, };
  while ((before.at >= 0) && CONTAINER_PREFIX.includes(source.charAt(before.at,),))
    before.at -= 1;
  if (source.charAt(before.at,) !== '\n')
    return [];
  if (definitionEnds.some(function endsTheLineBefore(end,): boolean {
    return source.indexOf(
      '\n',
      end,
    ) === before.at;
  },))
    return [];
  return [
    {
      start: ((before.at > 0) && (source.charAt(before.at - 1,) === '\r')) ? (before.at - 1) : before.at,
      end: range.start,
      replacement: ' ',
    },
  ];
}

/**
 Edits that join the lines of a node whose characters the plain reading keeps whole and the strict grammar cuts: a code
 span, inline math, and the destination and title of a link, an image or a definition, at a brace, a less-than sign, an
 indented fence mark or a footnote start only the page reads; and emphasis, strong emphasis, strikethrough, the text of a
 link or an image and a reference, and a hard break written with a backslash, at a footnote start only the page reads.

 @param source - text

 @param node - node of the parse

 @param range - where the node stands

 @param definitionEnds - where each link reference definition read so far ends, none where the caller reads no
 definitions

 @param footnoteStarts - offsets where a footnote definition starts on the page and not for a reader of the source, none
 where the caller reads none

 @returns Edits, none for a node of another kind and none for a footnote definition

 @example
 ```ts
 const edits = lineJoinsOf({ source, node, range, },);
 ```
 */
export function lineJoinsOf(
  {
    source,
    node,
    range,
    definitionEnds = [],
    footnoteStarts = [],
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
    readonly definitionEnds?: readonly number[];
    readonly footnoteStarts?: readonly number[];
  },
): readonly TextEdit[] {
  // INDENTED DISPLAY MATH after a line of a paragraph is display math to the strict grammar, and is copied as it stands.
  if ((node.type === 'inlineMath') && readsAsDisplayMath({
    source,
    range,
  },))
    return [];
  if (SPAN_TYPES.has(node.type,)) {
    return [
      ...joinBeforeFenceMark({
        source,
        range,
        definitionEnds,
      },),
      ...joinedLines({
        source,
        range,
        footnoteStarts,
      },),
    ];
  }
  if (FOOTNOTE_ONLY_TYPES.has(node.type,)) {
    return joinedLines({
      source,
      range,
      footnoteStarts,
      joinsFlowStarts: false,
    },);
  }
  // A HARD BREAK WRITTEN WITH A BACKSLASH renders as a line break to a reader of the source and is white space to the
  // account the comparison makes of it (`source-reading-account.ts` folds a break to white space), and the page shows a
  // backslash where the footnote cuts the paragraph, so the break, the line ending and the indentation are written as one
  // space: the account reads the same, and the rendered page has one line where the source has two. A break written with
  // two spaces is not joined and keeps its break.
  /**
   Where the line after a hard break starts, when the node is one.
   */
  const afterBreak = contentAfterBreak({
    source,
    node,
    range,
  },);
  if (afterBreak !== NO_START) {
    return footnoteStarts.includes(afterBreak,)
      ? [
        {
          start: range.start,
          end: afterBreak,
          replacement: ' ',
        },
      ]
      : [];
  }
  if (((node.type === 'link') && source.startsWith(
    '[',
    range.start,
  )) || (node.type === 'image')) {
    /**
     Where the text of the link, or the alternative text of the image, ends.
     */
    const textEnd = closingBracket({
      source,
      from: range.start + ((node.type === 'image') ? 2 : 1),
      limit: range.end,
    },);
    return [
      ...joinedLines({
        source,
        range: {
          start: range.start,
          end: textEnd,
        },
        footnoteStarts,
        joinsFlowStarts: false,
      },),
      ...joinedLines({
        source,
        range: {
          start: textEnd,
          end: range.end,
        },
        footnoteStarts,
      },),
    ];
  }
  if ((node.type === 'definition') && (!hasFootnoteLabel({
    source,
    node,
  },))) {
    return joinedLines({
      source,
      range: {
        start: closingBracket({
          source,
          from: range.start + 1,
          limit: range.end,
        },),
        end: range.end,
      },
      footnoteStarts,
    },);
  }
  return [];
}

/**
 The stretches of a node in which a line ending is joined to the line after it, none for a node that has none.

 @param source - text

 @param node - node of the parse

 @returns Whole code span, inline math, emphasis, strong emphasis, strikethrough, link, image or reference; a hard break
 written with a backslash, through the first character of the next line; none for a node of another kind

 @example
 ```ts
 const stretches = joinableStretchesOf({ source, node, },);
 ```
 */
function joinableStretchesOf(
  {
    source,
    node,
  }: {
    readonly source: string;
    readonly node: TreeNode;
  },
): readonly TextRange[] {
  if (node.position === undefined)
    return [];
  /**
   Where the node stands.
   */
  const range = rangeOf({ node, },);
  if (SPAN_TYPES.has(node.type,) || FOOTNOTE_ONLY_TYPES.has(node.type,))
    return [range,];
  /**
   Where the line after a hard break starts, when the node is one written with a backslash.
   */
  const afterBreak = contentAfterBreak({
    source,
    node,
    range,
  },);
  if (afterBreak !== NO_START) {
    // The stretch runs one character past the start, since a start inside a stretch must be before its end.
    return [
      {
        start: range.start,
        end: afterBreak + 1,
      },
    ];
  }
  if (((node.type === 'link') && source.startsWith(
    '[',
    range.start,
  )) || (node.type === 'image'))
    return [range,];
  return [];
}

/**
 Says which of the footnote starts only the page reads stand inside a code span, inline math, emphasis, strong emphasis,
 strikethrough, a link, an image or a reference of a block, where the line is joined to the one before it and the
 footnote does not start.

 @param source - text

 @param node - block of the parse, a paragraph or a heading

 @param starts - offsets where a footnote definition starts on the page and not for a reader of the source

 @returns Starts inside such a stretch of the block or of an inline node it holds

 @example
 ```ts
 const joined = footnoteStartsInSpans({ source, node: paragraph, starts: [13,], },);
 ```
 */
export function footnoteStartsInSpans(
  {
    source,
    node,
    starts,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly starts: readonly number[];
  },
): readonly number[] {
  if (starts.length === 0)
    return [];
  /**
   Nodes still to read, a work stack so that a deep tree is no deep recursion.
   */
  const pending: TreeNode[] = [node,];
  /**
   Stretches in which a line is joined.
   */
  const stretches: TextRange[] = [];
  for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
    stretches.push(...joinableStretchesOf({
      source,
      node: next,
    },),);
    if ('children' in next)
      pending.push(...next.children,);
  }
  return starts.filter(function insideOne(start,): boolean {
    return stretches.some(function holds({
      start: from,
      end,
    },): boolean {
      return (from < start) && (start < end);
    },);
  },);
}

/**
 The stretches of a raw HTML block or of indented code that are its content: each line less the marks of the containers
 and the indentation before it.

 @param source - text, or the text the parse was given with its comments blanked, of the same length

 @param node - `code` or `html` node of the parse

 @param range - where the node stands

 @returns One stretch for each line that has content; the whole range when the lines of the node and of the text do not
 correspond, which a parse of this reading does not give

 @example
 ```ts
 const stretches = contentStretches({ source: '> <div>\n> a', node, range: { start: 2, end: 12, }, },);
 ```
 */
export function contentStretches(
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
  if (!('value' in node))
    return [range,];
  /**
   Stretches found.
   */
  const stretches: TextRange[] = [];
  /**
   Where the line being read starts.
   */
  const line = { start: range.start, };
  /**
   The value of the node.
   */
  const { value, } = node;
  /**
   The lines of the node's value.
   */
  const contents = value.split('\n',);
  for (const content of contents) {
    /**
     Index of the line ending, or the end of the node for its last line.
     */
    const ending = source.indexOf(
      '\n',
      line.start,
    );
    /**
     Index just past the line, its carriage return included.
     */
    const lineEnd = ((ending === (-1)) || (ending >= range.end)) ? range.end : ending;
    /**
     Where the content would start if every character of the value stood in the text: the node's value keeps the line's
     carriage return only where the text has one, and starts with spaces the text does not hold where a tab that a container
     takes in part is read as spaces, which the line's first character stands for.
     */
    const natural = lineEnd - content.length;
    /**
     How many characters the value has before the line's first character.
     */
    const missing = line.start - natural;
    /**
     What the value has there.
     */
    const lead = content.slice(
      0,
      Math.max(
        missing,
        0,
      ),
    );
    if (lead.trim() !== '')
      return [range,];
    /**
     Where the content starts.
     */
    const start = Math.max(
      line.start,
      natural,
    );
    if (start < lineEnd)
      stretches.push({
        start,
        end: lineEnd,
      },);
    line.start = lineEnd + 1;
  }
  return stretches;
}

//endregion Source ships lines
