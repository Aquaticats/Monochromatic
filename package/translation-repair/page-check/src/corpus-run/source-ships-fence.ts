//region Source ships fence
// A FENCE MARK THAT ONLY THE STRICT GRAMMAR READS. CommonMark takes a line indented four spaces or more for indented
// code, or for the continuation of the paragraph above it, and a fence mark there is no fence. MDX turns indented code
// off, and with it the limit of three spaces: a fence opens at any indentation, it interrupts a paragraph, and a
// closing fence is read at any indentation (`micromark-core-commonmark@2.0.4`, `lib/code-fenced.js`, the closing
// fence takes its line prefix with no bound where `codeIndented` is disabled; the opening and the interruption were
// measured by the reader cases of `source-ships-encoding.unit.test.ts`). So a mark the plain reading keeps as text
// opens a code block in the page, which swallows what follows it up to a mark in the page that closes it, a later run
// among the places that does.

/**
 Characters a fence mark is made of.
 */
const FENCE_CHARACTERS = '`~';

/**
 Characters that stand before the content of a line inside a container: the indentation and the mark of a block quote.
 */
const CONTAINER_PREFIX = ' \t>';

/**
 Characters in a fence mark at the least.
 */
const MARK_LENGTH = 3;

/**
 Columns a tab counts for in the indentation of a line.
 */
const TAB_COLUMNS = 4;

/**
 Columns of indentation from which the plain reading no longer takes a fence mark for one.
 */
const INDENT_WITHOUT_FENCE = 4;

/**
 Where the indented fence mark that starts at an index ends.

 @param source - the text

 @param at - index of the first character of the mark

 @returns The index after the mark, which is `at` itself when no indented fence mark starts there: that is, when the
 character is no fence character, fewer than three of it follow, or the characters before it on the line are other than
 the indentation and the marks of block quotes, or hold fewer than four columns of indentation

 @example
 ```ts
 const end = indentedFenceMarkEnd({ source: 'a\n    ```\n', at: 6, },);
 ```
 */
export function indentedFenceMarkEnd(
  {
    source,
    at,
  }: {
    readonly source: string;
    readonly at: number;
  },
): number {
  /**
   Character the mark would be made of.
   */
  const character = source.charAt(at,);
  if (at >= source.length)
    return at;
  if (!FENCE_CHARACTERS.includes(character,))
    return at;
  /**
   Columns of indentation before the mark.
   */
  const columns = { count: 0, };
  for (let before = at - 1; (before >= 0) && (source.charAt(before,) !== '\n'); before -= 1) {
    /**
     Character before the mark on its line.
     */
    const prefix = source.charAt(before,);
    if (!CONTAINER_PREFIX.includes(prefix,))
      return at;
    if (prefix === ' ')
      columns.count += 1;
    if (prefix === '\t')
      columns.count += TAB_COLUMNS;
  }
  if (columns.count < INDENT_WITHOUT_FENCE)
    return at;
  /**
   Index after the run of the character.
   */
  const end = { at, };
  while (source.charAt(end.at,) === character)
    end.at += 1;
  return ((end.at - at) >= MARK_LENGTH) ? end.at : at;
}

/**
 Whether a text has a line that starts, after its indentation, with an indented fence mark.

 @param text - text of a run

 @returns Whether the strict grammar reads a fence where the plain reading reads text

 @example
 ```ts
 const holds = holdsIndentedFenceMark({ text: '    ```\n', },);
 ```
 */
export function holdsIndentedFenceMark({ text, }: { readonly text: string; },): boolean {
  /**
   Index of the start of the line being read.
   */
  const line = { start: 0, };
  for (const content of text.split('\n',)) {
    /**
     Index of the first character after the indentation and the marks of block quotes.
     */
    const mark = { at: line.start, };
    while ((mark.at < (line.start + content.length)) && CONTAINER_PREFIX.includes(text.charAt(mark.at,),))
      mark.at += 1;
    if (indentedFenceMarkEnd({
      source: text,
      at: mark.at,
    },) > mark.at)
      return true;
    line.start += content.length + 1;
  }
  return false;
}

//endregion Source ships fence
