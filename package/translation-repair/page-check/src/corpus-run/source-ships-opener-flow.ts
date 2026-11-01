import {
  endingAt,
  type OpenerInCode,
  writtenAsText,
} from './source-ships-opener-text.ts';
import type {
  TextEdit,
  TextRange,
} from './source-ships-ranges.ts';

//region Source ships opener flow
// THE LINES OF A FENCE OR A DISPLAY MATH THAT HOLD AN OPENER OF A COMMENT THAT NEVER CLOSES, taken out of the block: the
// block is closed before the line, the line is written as a paragraph with the opener as the character reference, and the
// block is opened again with the same mark. A reader of the page sees the same characters in the code before the line, in
// the paragraph and in the code after it. The info string of the opening is not repeated, which no reader of the text sees.

/**
 The lines of a fence or display math, after its opening line and before its closing one, that hold an opener.

 @param source - text

 @param range - where the block stands

 @param starts - index of the `<` of each opener of a comment that never closes

 @returns Where each such line stands, without its line ending

 @example
 ```ts
 const lines = linesHolding({ source, range, starts: [9,], },);
 ```
 */
function linesHolding(
  {
    source,
    range,
    starts,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly starts: readonly number[];
  },
): readonly TextRange[] {
  /**
   Lines found.
   */
  const lines: TextRange[] = [];
  /**
   The line feed that ends the opening line.
   */
  const first = source.indexOf(
    '\n',
    range.start,
  );
  if (first === (-1)) {
    return lines;
  }
  for (
    let start = first + 1;
    (start > 0) && (start < range.end);
    start = source.indexOf(
      '\n',
      start,
    ) + 1
  ) {
    /**
     Index of the line feed that ends the line, past the end of the text for the last line.
     */
    const feed = source.indexOf(
      '\n',
      start,
    );
    /**
     Where the line ends, before its line ending.
     */
    const end = Math.min(
      (feed === (-1)) ? source.length : feed,
      range.end,
    );
    /**
     The line, without a carriage return.
     */
    const line = {
      start,
      end: ((end > start) && (source.charAt(end - 1,) === '\r')) ? (end - 1) : end,
    };
    if (starts.some(function inside(opener,): boolean {
      return (opener >= line.start) && (opener < line.end);
    },))
      lines.push(line,);
  }
  return lines;
}

/**
 Takes the lines that hold an opener out of one fence or display math.

 @param source - text

 @param range - where the block stands, its opening mark first

 @param mark - characters that open the block, as they are written

 @param carried - what every later line of the block starts with: the marks of its containers, list markers as spaces, and
 nothing for a block at the top, whose indentation no later line need repeat

 @param starts - index of the `<` of each opener of a comment that never closes

 @param closed - comments that close

 @returns Edits and the openers they write

 @example
 ```ts
 const split = splitFlow({ source, range, mark: '```', carried: '', starts: [9,], closed: [], },);
 ```
 */
export function splitFlow(
  {
    source,
    range,
    mark,
    carried,
    starts,
    closed,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly mark: string;
    readonly carried: string;
    readonly starts: readonly number[];
    readonly closed: readonly TextRange[];
  },
): OpenerInCode {
  /**
   Edits written.
   */
  const edits: TextEdit[] = [];
  /**
   Openers the edits write.
   */
  const handled: number[] = [];
  for (const line of linesHolding({
    source,
    range,
    starts,
  },)) {
    if (!source.startsWith(
      carried,
      line.start,
    ))
      continue;
    /**
     Where the content of the line stands, less its indentation.
     */
    const content = { start: line.start + carried.length, };
    while ((content.start < line.end) && (' \t'.includes(source.charAt(content.start,),)))
      content.start += 1;
    /**
     The openers on the line.
     */
    const openers = starts.filter(function onLine(start,): boolean {
      return (start >= line.start) && (start < line.end);
    },);
    /**
     The line ending after the line.
     */
    const ending = endingAt({
      source,
      feed: source.indexOf(
        '\n',
        line.end,
      ),
    },);
    edits.push({
      start: line.start,
      end: line.end,
      replacement: [
        `${carried}${mark}`,
        carried.trimEnd(),
        `${carried}${
          writtenAsText({
            source,
            range: {
              start: content.start,
              end: line.end,
            },
            openers,
            closed,
          },)
        }`,
        carried.trimEnd(),
        `${carried}${mark}`,
      ].join(ending,),
    },);
    handled.push(...openers,);
  }
  return {
    edits,
    handled,
  };
}

//endregion Source ships opener flow
