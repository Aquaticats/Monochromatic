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
 The edit that puts a blank line between the end of a footnote and a definition line, or a paragraph, that follows it
 directly.

 THE FOOTNOTE ENDS WHERE A PARSE WITH GFM ENDS IT, and the line after it starts a block of the top: a link reference
 definition, whose destination and title are not touched, or, where the page goes on with the footnote past that end, a
 paragraph. The page writes the last block of the footnote as text when it is raw HTML or indented code, so the line right
 after it would be a lazy line of the footnote's paragraph and its braces would be an expression, or the backticks of a
 code span would pair with those of the footnote. A blank line ends the paragraph, and the block is one on the page as it
 is in the original.

 @param source - whole text

 @param readable - whole text with its comments blanked, the parse's own input

 @param range - lines after the end of the footnote, from the line ending of the footnote's last line

 @param pageGoesOn - whether the page goes on with the footnote's paragraph past that end, which a line of raw HTML makes
 it do

 @returns Edit that doubles the line ending, none when a blank line stands there already or the first line after
 it is no definition, nor a paragraph where the page goes on

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
   Whether the block that follows is one the footnote's end must be kept from: a definition, or a paragraph where the
   page goes on with the footnote past that end.
   */
  const needsSeparation = (first?.type === 'definition') || (pageGoesOn && (first?.type === 'paragraph'));
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

//endregion Source ships separation
