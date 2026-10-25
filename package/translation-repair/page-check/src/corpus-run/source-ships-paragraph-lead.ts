import type {
  SourceSpans,
  TextEdit,
  TextRange,
} from './source-ships-ranges.ts';

//region Source ships paragraph lead
// A BLOCK READ ALONE THAT WAS A PARAGRAPH WHERE IT STOOD. A line indented four spaces or more right after a link reference
// definition opens a paragraph to a reader of the source, and the node of that paragraph starts after the indentation.
// Read alone, the same characters start at the first column, and a run of backticks or tildes there opens a fence, a
// number sign a heading. A lead of one word and a space before the characters keeps them in a paragraph, and the spans the
// reading finds are moved back by the length of the lead.

/**
 What is put before a block that stood in a paragraph, so that the parse reads a paragraph.
 */
export const PARAGRAPH_LEAD = 'x ';

/**
 Columns a tab counts for in the indentation of a line.
 */
const TAB_COLUMNS = 4;

/**
 Columns of indentation from which a line cannot start a fence, a heading or a quote in the source's reading.
 */
const MOST_COLUMNS = 4;

/**
 Whether the characters at an index stand after an indentation of four columns or more on their line.

 @param source - whole text

 @param at - index of the first character of the block

 @returns Whether only spaces and tabs stand before the index on its line, and they make four columns or more

 @example
 ```ts
 const indented = isIndentedFourOrMore({ source: '[r]: u\n    ```', at: 11, },);
 ```
 */
export function isIndentedFourOrMore(
  {
    source,
    at,
  }: {
    readonly source: string;
    readonly at: number;
  },
): boolean {
  /**
   Columns counted so far.
   */
  const columns = { count: 0, };
  for (let before = at - 1; before >= 0; before -= 1) {
    /**
     The character before the one read last.
     */
    const character = source.charAt(before,);
    if (character === '\n')
      break;
    if (character === ' ')
      columns.count += 1;
    else if (character === '\t')
      columns.count += TAB_COLUMNS;
    else
      return false;
  }
  return columns.count >= MOST_COLUMNS;
}

/**
 Moves ranges back by an amount and drops what lies wholly before the start.

 @param ranges - ranges to move

 @param by - characters to move back

 @returns Ranges moved, those that end at or before the start dropped, those that start before it cut to start there

 @example
 ```ts
 const moved = rangesBack({ ranges: [{ start: 2, end: 6, },], by: 2, },);
 ```
 */
function rangesBack(
  {
    ranges,
    by,
  }: {
    readonly ranges: readonly TextRange[];
    readonly by: number;
  },
): readonly TextRange[] {
  return ranges.flatMap(function back(range,): readonly TextRange[] {
    if ((range.end - by) <= 0)
      return [];
    return [
      {
        start: Math.max(
          0,
          range.start - by,
        ),
        end: range.end - by,
      },
    ];
  },);
}

/**
 Moves the spans of a block read behind a lead back by the length of the lead.

 @param spans - spans found in the block with the lead before it

 @param by - length of the lead

 @returns Spans at the offsets of the block alone; what lay in the lead is dropped

 @example
 ```ts
 const spans = spansWithoutLead({ spans: found, by: PARAGRAPH_LEAD.length, },);
 ```
 */
export function spansWithoutLead(
  {
    spans,
    by,
  }: {
    readonly spans: SourceSpans;
    readonly by: number;
  },
): SourceSpans {
  return {
    text: rangesBack({
      ranges: spans.text,
      by,
    },),
    literal: rangesBack({
      ranges: spans.literal,
      by,
    },),
    edits: spans
      .edits
      .filter(function afterLead(edit,): boolean {
        return edit.start >= by;
      },)
      .map(function back(edit,): TextEdit {
        return {
          ...edit,
          start: edit.start - by,
          end: edit.end - by,
        };
      },),
    insideLinks: rangesBack({
      ranges: spans.insideLinks,
      by,
    },),
    streamStarts: spans
      .streamStarts
      .filter(function afterLead(start,): boolean {
        return start >= by;
      },)
      .map(function back(start,): number {
        return start - by;
      },),
  };
}

//endregion Source ships paragraph lead
