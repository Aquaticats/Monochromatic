import { indentedFenceMarkEnd, } from './source-ships-fence.ts';
import type {
  TextEdit,
  TextRange,
} from './source-ships-ranges.ts';

//region Source ships fenced
// A FENCED BLOCK THAT HOLDS A MARK INDENTED FOUR SPACES OR MORE. The plain reading closes a fence only with a mark
// indented three spaces or fewer, so such a mark is a line of the code; the strict grammar closes it at any
// indentation (`source-ships-fence.ts`), and what follows the mark is then read as the page, a brace among it as an
// expression. A fence closes only with a mark at least as long as its opening, so the opening and the closing are
// written longer than the longest mark the block holds, which changes no character of the code and leaves a reader
// the same block. A block that never closes has only its opening to lengthen. The last line is the closing by its
// column against the column of the opening and not against the start of the line, since the list items and block quotes
// a fence stands in add the same columns to both and four of them are no indentation of the code.

/**
 Characters a fence is made of.
 */
const FENCE_CHARACTERS = '`~';

/**
 Characters that stand before the content of a line inside a container: the indentation and the mark of a block quote.
 */
const CONTAINER_PREFIX = ' \t>';

/**
 Columns a tab counts for in the indentation of a line.
 */
const TAB_COLUMNS = 4;

/**
 Columns past the opening mark of a fence at which the plain reading still takes a mark for the closing.
 */
const CLOSING_REACH = 3;

/**
 Index of the first character of a line after its indentation and the marks of block quotes.

 @param source - text

 @param from - index the line starts at

 @param limit - index the line does not pass

 @returns Index, `limit` for a line of nothing else

 @example
 ```ts
 const at = contentAt({ source: '>   ```', from: 0, limit: 8, },);
 ```
 */
function contentAt(
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
   The index being read.
   */
  const reading = { at: from, };
  while ((reading.at < limit) && CONTAINER_PREFIX.includes(source.charAt(reading.at,),))
    reading.at += 1;
  return reading.at;
}

/**
 Whether a text holds nothing but spaces, tabs and carriage returns.

 @param text - text

 @returns Whether it is empty or all of them

 @example
 ```ts
 const alone = isSpaceOnly({ text: '  \r', },);
 ```
 */
function isSpaceOnly({ text, }: { readonly text: string; },): boolean {
  for (const character of text) {
    if (!' \t\r'.includes(character,))
      return false;
  }
  return true;
}

/**
 The column a character stands at, counting the characters before it on its line, a tab as four.

 @param source - text

 @param at - index of the character

 @returns Zero-based column

 @example
 ```ts
 const column = columnOf({ source: '- a\n  - b\n    ```', at: 14, },);
 ```
 */
function columnOf(
  {
    source,
    at,
  }: {
    readonly source: string;
    readonly at: number;
  },
): number {
  /**
   Columns counted so far.
   */
  const columns = { count: 0, };
  for (let before = at - 1; (before >= 0) && (source.charAt(before,) !== '\n'); before -= 1)
    columns.count += (source.charAt(before,) === '\t') ? TAB_COLUMNS : 1;
  return columns.count;
}

/**
 Where the closing mark of a fenced block stands: the last line, when it is a run of the mark as long as the opening,
 alone on its line, and no more than three columns past the opening. A block that never closes has none.

 The plain reading closes a fence at a mark indented three spaces or fewer in the container the fence stands in, so a
 mark that stands as far from the opening as that is the closing, whatever columns the container's own indentation adds
 to both; a mark four columns past it is a line of the code.

 @param source - text

 @param range - where the block stands, its opening fence first

 @param lineStarts - where each line after the opening one starts

 @param mark - character the fence is made of

 @param openLength - length of the opening

 @returns Run of the mark that closes the block, as the one stretch of a list that is empty for a block that never
 closes

 @example
 ```ts
 const range = { start: 0, end: 9, };
 const [closing,] = closingMarksOf({ source: '```\na\n```', range, lineStarts: [4, 6,], mark: '`', openLength: 3, },);
 ```
 */
function closingMarksOf(
  {
    source,
    range,
    lineStarts,
    mark,
    openLength,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly lineStarts: readonly number[];
    readonly mark: string;
    readonly openLength: number;
  },
): readonly TextRange[] {
  /**
   Start of the last line.
   */
  const lastStart = lineStarts.at(-1,);
  if (lastStart === undefined)
    return [];
  /**
   Index of the first character of the last line's content.
   */
  const content = contentAt({
    source,
    from: lastStart,
    limit: range.end,
  },);
  /**
   Index after the run of the mark at the start of the content.
   */
  const run = { end: content, };
  while ((source.charAt(run.end,) === mark) && (run.end < range.end))
    run.end += 1;
  /**
   Whether the run is as long as the opening.
   */
  const isLongEnough = (run.end - content) >= openLength;
  /**
   Whether nothing but white space follows the run on the line.
   */
  const isAlone = isSpaceOnly({
    text: source.slice(
      run.end,
      range.end,
    ),
  },);
  /**
   Whether the mark stands close enough to the opening to close it in the plain reading.
   */
  const isClose = (columnOf({
    source,
    at: content,
  },) - columnOf({
    source,
    at: range.start,
  },)) <= CLOSING_REACH;
  if (![
    isLongEnough,
    isAlone,
    isClose,
  ].every(Boolean,))
    return [];
  return [
    {
      start: content,
      end: run.end,
    },
  ];
}

/**
 Edits that lengthen the opening and the closing of a fenced block whose content holds a mark the strict grammar would
 close it at.

 @param source - text

 @param range - where the block stands, its opening fence first

 @returns Edits for the opening and for the closing when there is one; none for a block that holds no such mark or none
 as long as its opening, and none for a range that does not start with a fence

 @example
 ```ts
 const edits = fenceLengthEdits({ source: '```\n    ```\n{\n```\n', range: { start: 0, end: 18, }, },);
 ```
 */
export function fenceLengthEdits(
  {
    source,
    range,
  }: {
    readonly source: string;
    readonly range: TextRange;
  },
): readonly TextEdit[] {
  /**
   Character the fence is made of.
   */
  const mark = source.charAt(range.start,);
  if (!FENCE_CHARACTERS.includes(mark,))
    return [];
  /**
   Index after the run of the mark that opens the block.
   */
  const opening = { end: range.start, };
  while (source.charAt(opening.end,) === mark)
    opening.end += 1;
  /**
   Length of the opening.
   */
  const openLength = opening.end - range.start;
  /**
   Where each line after the opening one starts.
   */
  const lineStarts: number[] = [];
  for (
    let at = source.indexOf(
      '\n',
      range.start,
    ) + 1;
    (at > 0) && (at < range.end);
    at = source.indexOf(
      '\n',
      at,
    ) + 1
  )
    lineStarts.push(at,);
  /**
   Where the closing mark stands, no stretch for a block that never closes.
   */
  const closings = closingMarksOf({
    source,
    range,
    lineStarts,
    mark,
    openLength,
  },);
  /**
   Length of the longest indented mark in the block's lines, the opening line and the closing apart.
   */
  const longest = { length: 0, };
  for (const lineStart of lineStarts) {
    /**
     Index of the first character of the line's content.
     */
    const content = contentAt({
      source,
      from: lineStart,
      limit: range.end,
    },);
    // The closing is no mark of the code.
    if (closings.some(function isClosing({ start, },): boolean {
      return start === content;
    },))
      continue;
    /**
     Index after an indented mark of the fence's kind at the start of the content, `content` itself when none.
     */
    const markEnd = (source.charAt(content,) === mark)
      ? indentedFenceMarkEnd({
        source,
        at: content,
      },)
      : content;
    longest.length = Math.max(
      longest.length,
      markEnd - content,
    );
  }
  if (longest.length < openLength)
    return [];
  /**
   Length of the opening and the closing once lengthened.
   */
  const lengthened = longest.length + 1;
  /**
   Edits so far: the opening.
   */
  const edits: TextEdit[] = [
    {
      start: range.start,
      end: opening.end,
      replacement: mark.repeat(lengthened,),
    },
  ];
  for (const closing of closings) {
    edits.push({
      ...closing,
      replacement: mark.repeat(lengthened,),
    },);
  }
  return edits;
}

//endregion Source ships fenced
