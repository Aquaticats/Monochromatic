import { indentedFenceMarkEnd, } from './source-ships-fence.ts';
import type {
  TextEdit,
  TextRange,
} from './source-ships-ranges.ts';

//region Source ships fence join
// A CODE SPAN THAT STARTS WITH A FENCE MARK INDENTED FOUR SPACES OR MORE. The plain reading takes the run of backticks for
// the opening of the span, so no backslash can be written into it, and the strict grammar takes it for a fence that cuts
// the paragraph. The span is joined to the line before it, so that the mark no longer starts its line. After the end of a
// block the strict grammar ends at the line (a link reference definition, display math) there is no paragraph line to
// join to, and the space before the mark is written as a character reference instead, which the mark no longer follows.

/**
 Characters that stand before the content of a line inside a container: the indentation and the mark of a block quote.
 */
const CONTAINER_PREFIX = ' \t>';

/**
 The edit that joins a code span to the line before it when the span starts with a fence mark indented four spaces or
 more: the plain reading takes the run of backticks for the opening of the span, so no backslash can be written into it,
 and the strict grammar takes it for a fence that cuts the paragraph.

 @param source - text

 @param range - where the code span stands

 @param definitionEnds - where each link reference definition of the text ends, since a line that ends a definition
 ends its block in both readings, and the fence mark after it opens a fence in the strict grammar with no paragraph
 line to join to: joining it would make the definition line a paragraph, whose braces are then code

 @returns One edit replacing the line ending before the span and the marks before it with a space, or after the end of a
 definition the space before the mark with a character reference for it; none when the span does not start with such a
 mark or starts the text

 @example
 ```ts
 const range = { start: 6, end: 20, };
 const edits = joinBeforeFenceMark({ source: 'a\n    ```\n`x\n    ```', range, definitionEnds: [], },);
 ```
 */
export function joinBeforeFenceMark(
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
  },)) {
    return [
      {
        start: range.start - 1,
        end: range.start,
        replacement: '&#32;',
      },
    ];
  }
  return [
    {
      start: ((before.at > 0) && (source.charAt(before.at - 1,) === '\r')) ? (before.at - 1) : before.at,
      end: range.start,
      replacement: ' ',
    },
  ];
}

//endregion Source ships fence join
