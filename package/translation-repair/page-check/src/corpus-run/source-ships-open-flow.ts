import {
  commentRegionsOf,
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { firstNestingExcess, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  asTheParseReadsIt,
  blanking,
} from './source-ships-readable.ts';
import {
  rangeOf,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships open flow
// A FENCE OR DISPLAY MATH A TEXT LEAVES OPEN AT ITS END, which the strict grammar ends where the page ends and not
// where the text does: the rest of the page falls inside it, and the first line of a later run that looks like its
// closing line ends it and leaves the text after that line outside, where a brace or a tag in it is code. The
// original of a slice ends where the slice does, so the text is closed here, by a line of the same mark, when the
// encoder writes it as literal text.

/**
 The most spaces a fence or display math may be indented by and still open or close one.
 */
const MAX_INDENT = 3;

/**
 The characters whose runs open a fence or a display math.
 */
const FLOW_CHARACTERS = '`~$';

/**
 How many dollar signs open a display math, where three backticks or tildes open a fence.
 */
const MINIMUM_DOLLARS = 2;

/**
 How many backticks or tildes open a fence.
 */
const MINIMUM_FENCE_MARKS = 3;

/**
 What the functions that read a mark answer for a line, a block or a text that holds none.
 */
export const NO_MARK: unique symbol = Symbol(
  'No fence or display math mark (backticks, tildes or dollar signs) in the line, block or text read.',
);

/**
 A mark, or that none stands.
 */
type MarkOrNone = string | typeof NO_MARK;

/**
 The line ending a closing mark is put after when the text it closes has none at its end.
 */
const LINE_ENDING = '\n';

/**
 How many spaces a line starts with.

 @param line - line

 @returns Count of leading spaces

 @example
 ```ts
 const indent = indentOf({ line: '  ```', },);
 ```
 */
function indentOf({ line, }: { readonly line: string; },): number {
  /**
   Where the first character that is no space stands.
   */
  const at = { first: 0, };
  while ((at.first < line.length) && (line.charAt(at.first,) === ' '))
    at.first += 1;
  return at.first;
}

/**
 The run of one character a line holds after its indentation.

 @param line - line, without its line ending

 @returns Run, `NO_MARK` when the indentation is more than three spaces or the line starts with no character that a
 flow is made of

 @example
 ```ts
 const run = runOf({ line: '  ```js', },);
 ```
 */
function runOf({ line, }: { readonly line: string; },): MarkOrNone {
  /**
   Spaces before the run.
   */
  const indent = indentOf({ line, },);
  /**
   The character the run is made of.
   */
  const character = line.charAt(indent,);
  if (indent > MAX_INDENT)
    return NO_MARK;
  if (indent >= line.length)
    return NO_MARK;
  if (!FLOW_CHARACTERS.includes(character,))
    return NO_MARK;
  /**
   Where the run ends.
   */
  const end = { at: indent, };
  while (line.charAt(end.at,) === character)
    end.at += 1;
  return line.slice(
    indent,
    end.at,
  );
}

/**
 What a flow's opening line opens: the run, when it is long enough to open one.

 @param line - first line of a code or math block

 @returns Mark, `NO_MARK` when the line opens no fence or display math

 @example
 ```ts
 const mark = openingMark({ line: '```js', },);
 ```
 */
function openingMark({ line, }: { readonly line: string; },): MarkOrNone {
  /**
   The run the line starts with.
   */
  const run = runOf({ line, },);
  if (run === NO_MARK)
    return NO_MARK;
  /**
   How many characters of the run open a flow.
   */
  const minimum = run.startsWith('$',) ? MINIMUM_DOLLARS : MINIMUM_FENCE_MARKS;
  return (run.length >= minimum) ? run : NO_MARK;
}

/**
 Whether a line holds nothing but spaces and tabs from an index on.

 @param line - line

 @param from - index to look from

 @returns Whether every character from the index on is a space or a tab

 @example
 ```ts
 const blank = onlySpacesFrom({ line: '```  ', from: 3, },);
 ```
 */
function onlySpacesFrom(
  {
    line,
    from,
  }: {
    readonly line: string;
    readonly from: number;
  },
): boolean {
  for (let at = from; at < line.length; at += 1) {
    /**
     Character read.
     */
    const character = line.charAt(at,);
    if ((character !== ' ') && (character !== '\t'))
      return false;
  }
  return true;
}

/**
 Whether the last line of a flow is a closing line for a mark: the same character, at least as many, and nothing but
 spaces and tabs after them.

 @param line - last line of the node's source, without its line ending

 @param mark - characters the flow opened with

 @returns Whether the line closes the flow

 @example
 ```ts
 const closed = closesFlow({ line: '```', mark: '```', },);
 ```
 */
function closesFlow(
  {
    line,
    mark,
  }: {
    readonly line: string;
    readonly mark: string;
  },
): boolean {
  /**
   The run the line starts with.
   */
  const run = runOf({ line, },);
  if (run === NO_MARK)
    return false;
  if (!onlySpacesFrom({
    line,
    from: indentOf({ line, },) + run.length,
  },))
    return false;
  return run.startsWith(mark.charAt(0,),) && (run.length >= mark.length);
}

/**
 The lines of a node's source, without their line endings.

 @param source - text the node was read from

 @param range - where the node stands

 @returns Lines in order

 @example
 ```ts
 const lines = linesOf({ source: '```\nx', range: { start: 0, end: 5, }, },);
 ```
 */
function linesOf(
  {
    source,
    range,
  }: {
    readonly source: string;
    readonly range: TextRange;
  },
): readonly string[] {
  return source
    .slice(
    range.start,
    range.end,
    )
    .split('\n',)
    .map(function withoutReturn(line,): string {
      return line.endsWith('\r',)
        ? line.slice(
          0,
          -1,
        )
        : line;
    },);
}

/**
 Says how a block that the plain reading ends inside a fence or display math at the top is closed.

 @param source - text the parse read

 @param root - parse of the text by the plain reading with the site's math

 @returns Mark that closes the last block of the text, `NO_MARK` when that block is no fence or display math, or is
 closed

 @example
 ```ts
 const closer = openFlowCloser({ source: '$$\nx', root: parseMarkdownMathBody({ body: '$$\nx', },), },);
 ```
 */
function openFlowCloser(
  {
    source,
    root,
  }: {
    readonly source: string;
    readonly root: TreeNode;
  },
): MarkOrNone {
  if (!('children' in root))
    return NO_MARK;
  /**
   The blocks at the top.
   */
  const { children, } = root;
  /**
   The block that stands last.
   */
  const last = children.at(-1,);
  if ((last === undefined) || ((last.type !== 'code') && (last.type !== 'math')))
    return NO_MARK;
  /**
   Lines of the block's source.
   */
  const lines = linesOf({
    source,
    range: rangeOf({ node: last, },),
  },);
  /**
   What the block's first line opens with.
   */
  const mark = openingMark({ line: lines[0] ?? '', },);
  if (mark === NO_MARK)
    return NO_MARK;
  return ((lines.length > 1) && closesFlow({
    line: lines.at(-1,) ?? '',
    mark,
  },))
    ? NO_MARK
    : mark;
}

/**
 Says how a text that the plain reading ends inside a fence or display math at the top is closed.

 @param text - text with its escapes made, in which no comment stays open

 @returns Mark that closes the last block of the text as the page reads it, `NO_MARK` when that block is no fence or
 display math, is closed, or the text is nested past the bound the plain reading keeps and is read as holding no block

 @example
 ```ts
 const closer = closerOfWritten({ text: '$$\nx', },);
 ```
 */
export function closerOfWritten({ text, }: { readonly text: string; },): MarkOrNone {
  /**
   The text as the parse reads it: its comments blanked, a byte order mark that opens it an ordinary character.
   */
  const readable = asTheParseReadsIt({
    text: blanking({
      text,
      ranges: commentRegionsOf({ text, },)
        .map(function toRange(
          {
            startOffset,
            endOffset,
          },
        ): TextRange {
          return {
            start: startOffset,
            end: endOffset,
          };
        },),
    },),
  },);
  if (firstNestingExcess({
    body: readable,
    grammar: 'markdown',
  },)
    .kind
    === 'beyond') {
    return NO_MARK;
  }
  return openFlowCloser({
    source: readable,
    root: parseMarkdownMathBody({ body: readable, },),
  },);
}

/**
 Puts a closing mark after a text.

 @param text - text the mark closes

 @param closer - mark, `NO_MARK` for a text that leaves nothing open

 @returns Text with the mark on a line of its own at the end, or the text itself for `NO_MARK`

 @example
 ```ts
 const closed = closedWith({ text: '$$\nx', closer: '$$', },);
 ```
 */
export function closedWith(
  {
    text,
    closer,
  }: {
    readonly text: string;
    readonly closer: MarkOrNone;
  },
): string {
  if (closer === NO_MARK)
    return text;
  if (text.endsWith('\n',))
    return `${text}${closer}`;
  return `${text}${text.includes('\r\n',) ? '\r\n' : LINE_ENDING}${closer}`;
}

//endregion Source ships open flow
