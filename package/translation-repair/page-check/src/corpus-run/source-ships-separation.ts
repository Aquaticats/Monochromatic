import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type {
  TextEdit,
  TextRange,
} from './source-ships-ranges.ts';

//region Source ships separation
// THE BLANK LINE BETWEEN THE END OF A FOOTNOTE AND A DEFINITION LINE THAT FOLLOWS IT DIRECTLY, written where the strict
// grammar ends the footnote and the plain reading goes on in a block of its own that a footnote's paragraph would
// otherwise take for a lazy line.

/**
 What `findIndex` answers when no line is blank.
 */
const NO_BLANK_LINE = -1;

/**
 The characters a blank line may hold.
 */
const BLANK_CHARACTERS = ' \t\r';

/**
 Whether a line holds nothing but spaces, tabs and a carriage return.

 @param line - line, without its line ending

 @returns Whether every character of the line is a space, a tab or a carriage return

 @example
 ```ts
 const blank = isBlankLine({ line: '  ', },);
 ```
 */
function isBlankLine({ line, }: { readonly line: string; },): boolean {
  for (let at = 0; at < line.length; at += 1) {
    if (!BLANK_CHARACTERS.includes(line.charAt(at,),))
      return false;
  }
  return true;
}

/**
 The edit that puts a blank line between the end of a footnote and the block that follows it directly.

 THE FOOTNOTE ENDS WHERE A PARSE WITH GFM ENDS IT, and the line after it starts a block of the top: a link reference
 definition, whose destination and title are not touched, or, where the page goes on with the footnote past that end, any
 block. The page writes the last block of the footnote as text when it is raw HTML or indented code, so the line right
 after it would be a lazy line of the footnote's paragraph and its braces would be an expression, the backticks of a
 code span would pair with those of the footnote, a definition would show as text and the underline of a heading as a row
 of equals signs. A blank line ends the paragraph, and the block is one on the page as it is in the original.

 @param source - whole text

 @param readable - whole text with its comments blanked, the parse's own input

 @param range - lines after the end of the footnote, from the line ending of the footnote's last line

 @param pageGoesOn - whether the page goes on with the footnote's paragraph past that end, which a line of raw HTML makes
 it do

 @returns Edit that doubles the line ending, none when a blank line stands there already, or when the page does not go
 on and the first line after it is no definition

 @example
 ```ts
 const edits = separatingEdits({ source, readable, range: { start: 20, end: 30, }, pageGoesOn: false, },);
 ```
 */
export function separatingEdits(
  {
    source,
    readable,
    range,
    pageGoesOn,
  }: {
    readonly source: string;
    readonly readable: string;
    readonly range: TextRange;
    readonly pageGoesOn: boolean;
  },
): readonly TextEdit[] {
  /**
   The line ending that ends the footnote's last line, absent where the text ends.
   */
  const ending = [
    '\r\n',
    '\n',
  ].find(function endsHere(candidate,): boolean {
    return source.startsWith(
      candidate,
      range.start,
    );
  },);
  if (ending === undefined)
    return [];
  /**
   Where the next line starts.
   */
  const lineStart = range.start + ending.length;
  /**
   The lines after the footnote up to the first blank one, as the parse reads them: a definition does not cross a blank
   line.
   */
  const lines = readable
    .slice(
    lineStart,
    range.end,
    )
    .split('\n',);
  /**
   The line right after the footnote.
   */
  const [nextLine = '',] = lines;
  if (isBlankLine({ line: nextLine, },))
    return [];
  /**
   Which line is the first blank one, negative where none is.
   */
  const blankAt = lines.findIndex(function isBlank(line,): boolean {
    return isBlankLine({ line, },);
  },);
  /**
   The lines up to the first blank one, joined.
   */
  const rest = lines
    .slice(
    0,
    (blankAt === NO_BLANK_LINE) ? lines.length : blankAt,
    )
    .join('\n',);
  /**
   The blocks of those lines, as the parse reads them.
   */
  const { children, } = parseMarkdownMathBody({ body: rest, },);
  /**
   The first of them.
   */
  const [first,] = children;
  /**
   Whether the block that follows is one the footnote's end must be kept from: a definition, or any block where the page
   goes on with the footnote past that end.
   */
  const needsSeparation = pageGoesOn || (first?.type === 'definition');
  if (!needsSeparation)
    return [];
  return [
    {
      start: range.start,
      end: lineStart,
      replacement: `${ending}${ending}`,
    },
  ];
}

/**
 The edit that puts a blank line after a raw HTML block of the top that ends at a line of its own.

 A block of the kinds that end at a line of their own (a processing instruction, a declaration, a CDATA section, a script,
 a pre or a style element) is followed by a new block on the very next line. The page writes the lines of the block as a
 paragraph, so the next line would be a lazy line of it: a definition would show as text and the underline of a heading as
 a row of equals signs. A block of the other kinds ends at a blank line and needs nothing.

 @param readable - whole text with its comments blanked, the parse's own input

 @param range - where the raw HTML block stands

 @returns Edit that doubles the line ending after the block, none when the text ends there or a blank line stands there

 @example
 ```ts
 const edits = afterHtmlBlockEdits({ readable: '<?x?>\n[r]: u\n', range: { start: 0, end: 5, }, },);
 ```
 */
export function afterHtmlBlockEdits(
  {
    readable,
    range,
  }: {
    readonly readable: string;
    readonly range: TextRange;
  },
): readonly TextEdit[] {
  /**
   The line ending that ends the block's last line, absent where the text ends.
   */
  const ending = [
    '\r\n',
    '\n',
  ].find(function endsHere(candidate,): boolean {
    return readable.startsWith(
      candidate,
      range.end,
    );
  },);
  if (ending === undefined)
    return [];
  /**
   The line after the block, without its line ending.
   */
  const [nextLine = '',] = readable
    .slice(range.end + ending.length,)
    .split('\n',);
  if (isBlankLine({ line: nextLine, },))
    return [];
  return [
    {
      start: range.end,
      end: range.end + ending.length,
      replacement: `${ending}${ending}`,
    },
  ];
}

//endregion Source ships separation
