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
// the same block. A block that never closes has only its opening to lengthen.

/**
 Characters a fence is made of.
 */
const FENCE_CHARACTERS = '`~';

/**
 Characters that stand before the content of a line inside a container: the indentation and the mark of a block quote.
 */
const CONTAINER_PREFIX = ' \t>';

/**
 Index of the first character of a line after its indentation and the marks of block quotes.

 @param source - the text

 @param from - index the line starts at

 @param limit - index the line does not pass

 @returns The index, `limit` for a line of nothing else

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

 @param text - the text

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
 Edits that lengthen the opening and the closing of a fenced block whose content holds a mark the strict grammar would
 close it at.

 @param source - the text

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
   Length of the longest indented mark in the block's lines, the opening line apart.
   */
  const longest = { length: 0, };
  /**
   Start of the line being read, and the start of the last line of the block.
   */
  const lines = {
    at: source.indexOf(
      '\n',
      range.start,
    ) + 1,
    last: range.start,
  };
  while ((lines.at > 0) && (lines.at < range.end)) {
    lines.last = lines.at;
    /**
     Index of the first character of the line's content.
     */
    const content = contentAt({
      source,
      from: lines.at,
      limit: range.end,
    },);
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
    lines.at = source.indexOf(
      '\n',
      lines.at,
    ) + 1;
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
  // The closing is the last line when it is a run of the mark as long as the opening, indented three spaces or fewer.
  if (lines.last > range.start) {
    /**
     Index of the first character of the last line's content.
     */
    const content = contentAt({
      source,
      from: lines.last,
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
     Whether the mark stands at an indentation that closes a fence in both grammars.
     */
    const isShallow = indentedFenceMarkEnd({
      source,
      at: content,
    },) === content;
    /**
     Whether nothing but white space follows the run on the line.
     */
    const isAlone = isSpaceOnly({
      text: source.slice(
        run.end,
        range.end,
      ),
    },);
    if ([
      isLongEnough,
      isShallow,
      isAlone,
    ].every(Boolean,)) {
      edits.push({
        start: content,
        end: run.end,
        replacement: mark.repeat(lengthened,),
      },);
    }
  }
  return edits;
}

//endregion Source ships fenced
