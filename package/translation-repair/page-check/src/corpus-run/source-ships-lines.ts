import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { indentedFenceMarkEnd, } from './source-ships-fence.ts';
import {
  closingBracket,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships lines
// LINE ENDINGS THE STRICT GRAMMAR READS A BLOCK AT. MDX starts a flow expression at a `{` and a flow tag at a `<`
// that stand first on a line (after the indentation and the marks of the containers), and a fence at a mark indented
// four spaces or more (`source-ships-fence.ts`), and each ends the paragraph the line would continue. The plain
// reading keeps such a line in the paragraph, so a code span, inline math or the title of a link that runs over the
// line stays whole there, and its brace, tag or mark is no syntax. The strict grammar cuts the span at the line, and
// the brace, tag or mark is read as syntax there with no way to escape it, since a backslash written into code or math
// shows. What is written instead is the line ending, and the indentation and container marks
// after it, as one space: a line ending in a code span or in math is read as a space already, and in a title it is
// white space.

/**
 Node types whose characters are code or math, which no backslash may be written into.
 */
const SPAN_TYPES: ReadonlySet<string> = new Set([
  'inlineCode',
  'inlineMath',
],);

/**
 Characters that stand before the content of a line inside a container: the indentation and the mark of a block quote.
 */
const CONTAINER_PREFIX = ' \t>';

/**
 Characters that start a flow expression or a flow tag.
 */
const FLOW_STARTS = '{<';

/**
 Edits that join the lines of a stretch whose later line starts with a
 brace, a less-than sign or an indented fence mark.

 @param source - text

 @param range - stretch to look in, whose first character no line ending joins to

 @returns One edit for each line ending followed, after the container marks, by a brace, a less-than sign or an indented
 fence mark inside the stretch, replacing the ending and the marks with a space

 @example
 ```ts
 const edits = joinedLines({ source: '`a\n{b`', range: { start: 0, end: 6, }, },);
 ```
 */
export function joinedLines(
  {
    source,
    range,
  }: {
    readonly source: string;
    readonly range: TextRange;
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
    const startsBlock = FLOW_STARTS.includes(source.charAt(content.at,),)
      || (indentedFenceMarkEnd({
        source,
        at: content.at,
      },) > content.at);
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
 Edits that join the lines of a node whose characters the plain reading keeps whole and the strict grammar cuts: a code
 span, inline math, and the destination and title of a link, an image or a definition.

 @param source - text

 @param node - node of the parse

 @param range - where the node stands

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
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
  },
): readonly TextEdit[] {
  if (SPAN_TYPES.has(node.type,))
    return joinedLines({
      source,
      range,
    },);
  if (((node.type === 'link') && source.startsWith(
    '[',
    range.start,
  )) || (node.type === 'image')) {
    return joinedLines({
      source,
      range: {
        start: closingBracket({
          source,
          from: range.start + ((node.type === 'image') ? 2 : 1),
          limit: range.end,
        },),
        end: range.end,
      },
    },);
  }
  if ((node.type === 'definition') && (!node
    .identifier
    .startsWith('^',))) {
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
    },);
  }
  return [];
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
     Where the content starts: the node's value keeps the line's carriage return only where the text has one.
     */
    const start = lineEnd - content.length;
    if (start < line.start)
      return [range,];
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
