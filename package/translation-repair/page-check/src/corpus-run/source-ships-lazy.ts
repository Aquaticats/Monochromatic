import type { TextRange, } from './source-ships-ranges.ts';

//region Source ships lazy
// THE LINES OF A FOOTNOTE'S PARAGRAPH THAT ARE NOT INDENTED are lazy lines to the strict grammar: a line made only of
// equals signs or hyphens that follows paragraph text is the underline of a heading in a text of its own, and in a
// footnote it is a line of the paragraph, since a lazy line is never a setext underline. A footnote body read as a text
// of its own reads it as the underline, turns the paragraph above it into a heading and reads every line after it as a
// block that starts there, where the strict grammar reads them as lines of the paragraph, with braces in them.

/**
 The most columns of indentation before a line that is still not indented under a footnote.
 */
const LAZY_INDENT = 3;

/**
 What a line that is not an underline is read as in its place: a letter, which starts no block.
 */
const PARAGRAPH_TEXT = 'x';

/**
 Characters a setext underline is made of.
 */
const UNDERLINE_MARKS: ReadonlySet<string> = new Set([
  '=',
  '-',
],);

/**
 What {@link underlineIn} answers for a line that holds no underline.
 */
const NO_UNDERLINE: TextRange = {
  start: 0,
  end: 0,
};

/**
 Where the marks of a setext underline stand in a line, when the line is made of nothing else.

 @param line - one line, without its line ending

 @returns The marks' stretch of the line, an empty stretch when the line is not an underline or is indented under a
 footnote

 @example
 ```ts
 const marks = underlineIn({ line: '  ==', },);
 ```
 */
function underlineIn({ line, }: { readonly line: string; }): TextRange {
  /**
   The line without white space after it.
   */
  const content = line.trimEnd();
  /**
   The line's characters after its indentation.
   */
  const marks = content.trimStart();
  /**
   How many characters of indentation the line starts with.
   */
  const indent = content.length - marks.length;
  /**
   The mark the line starts with.
   */
  const mark = marks.charAt(0,);
  /**
   The line made of that mark alone.
   */
  const run = mark.repeat(marks.length,);
  /**
   Whether the line is indented under the footnote.
   */
  const isIndented = (indent > LAZY_INDENT) || content.startsWith('\t',);
  /**
   Whether the line is a run of one underline mark.
   */
  const isRun = UNDERLINE_MARKS.has(mark,) && (marks === run);
  if (isIndented || (!isRun))
    return NO_UNDERLINE;
  return {
    start: indent,
    end: indent + marks.length,
  };
}

/**
 Writes a letter over each character of a line of a footnote's paragraph that the strict grammar reads as paragraph
 text and a text of its own would read as a setext underline, so that the paragraph keeps the lines after it.

 @param readable - whole text with its comments blanked

 @param range - where the body of the footnote stands, from the first character after its label

 @returns The text with those characters replaced, the same length, or the text itself when no line is one

 @example
 ```ts
 const kept = lazyUnderlinesBlanked({ readable: '[^1]: x\n===\n', range: { start: 6, end: 12, }, },);
 ```
 */
export function lazyUnderlinesBlanked(
  {
    readable,
    range,
  }: {
    readonly readable: string;
    readonly range: TextRange;
  },
): string {
  /**
   The text in pieces, those that are lines to blank written with their letters.
   */
  const pieces: string[] = [];
  /**
   Where the piece being read starts.
   */
  const written = { upTo: 0, };
  /**
   Where the first line after the label ends, which is a line of the footnote's first paragraph and holds the label.
   */
  const firstBreak = readable.indexOf(
    '\n',
    range.start,
  );
  for (
    let lineStart = (firstBreak === (-1)) ? range.end : (firstBreak + 1);
    lineStart < range.end;
  ) {
    /**
     Index of the line ending, or the end of the body for its last line.
     */
    const ending = readable.indexOf(
      '\n',
      lineStart,
    );
    /**
     Where the line ends, its line ending not included.
     */
    const lineEnd = ((ending === (-1)) || (ending > range.end)) ? range.end : ending;
    /**
     The marks of the line when it is an underline.
     */
    const underline = underlineIn({
      line: readable.slice(
        lineStart,
        lineEnd,
      ),
    },);
    if (underline.end > underline.start) {
      pieces.push(
        readable.slice(
          written.upTo,
          lineStart + underline.start,
        ),
        PARAGRAPH_TEXT.repeat(underline.end - underline.start,),
      );
      written.upTo = lineStart + underline.end;
    }
    lineStart = lineEnd + 1;
  }
  if (pieces.length === 0)
    return readable;
  return pieces.join('',) + readable.slice(written.upTo,);
}

//endregion Source ships lazy
