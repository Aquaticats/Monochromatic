// The record the application prints when it prepared syntax highlighting for a source file, read for its span count.
// What: the count in the line that names the prepared file and the spans it prepared, `source syntax prepared
// path=<file> spans=<count>`, and absence when the output holds no such line for the file the check opened.
// Why: the bundle checks judge a session by two records of that line: its presence settles a session, and its count is
// what proves a lone copy highlighted from the runtime it carries rather than by accident. The path before the file
// name is whatever the application was told to open, so the line is found by the file name and read from there.
// The read is one linear pass and is tested in prepared-syntax-record.test.ts.

/**
 The words that open the record.
 */
const recordPrefix = 'source syntax prepared path=';

/**
 The file the bundle checks open, which the record names at the end of its path.
 */
const openedFile = 'fixture.sql';

/**
 The words between the file name and the count.
 */
const countPrefix = ' spans=';

/**
 Characters a JavaScript pattern's `\s` reaches, which end the path the record names.
 *
 The set is the language's WhiteSpace and LineTerminator productions, spelled as the ranges it is: the ASCII space and
 tab, the line terminators, the no-break space, the ogham space, the en and em quads and their neighbours, the narrow
 and ideographic spaces, and the byte-order mark.
 *
 @param character - one character of the output.
 *
 @returns whether the character is whitespace in that sense.
 */
const isWhitespace = (character: string): boolean => (character === ' ')
  || (character === '\t')
  || (character === '\n')
  || (character === '\v')
  || (character === '\f')
  || (character === '\r')
  || (character === '\u00A0')
  || (character === '\u1680')
  || (character >= '\u2000' && character <= '\u200A')
  || (character === '\u2028')
  || (character === '\u2029')
  || (character === '\u202F')
  || (character === '\u205F')
  || (character === '\u3000')
  || (character === '\uFEFF');

/**
 Whether a character is an ASCII digit, the only kind the count is written with.
 *
 @param character - one character of the output.
 *
 @returns whether the character is in `0` to `9`.
 */
const isDigit = (character: string): boolean => (character >= '0') && (character <= '9');

/**
 The spans the output says were prepared for the opened file, or absence when it holds no such record.
 *
 @param output - the session's combined output.
 *
 @returns The count of prepared spans, or absence when the output holds no record for the opened file.
 */
export const preparedSyntaxSpanCount = (output: string): number | undefined => {
  for (let at = output.indexOf(recordPrefix); at >= 0; at = output.indexOf(recordPrefix, at + 1)) {
    // The path runs to the first whitespace, and it must end with the file name, so the file name closes it.
    const pathStart = at + recordPrefix.length;
    let pathEnd = pathStart;
    while (pathEnd < output.length && !isWhitespace(output[pathEnd] ?? '')) {
      pathEnd++;
    }
    if (!output.startsWith(openedFile, pathEnd - openedFile.length) || pathEnd - openedFile.length < pathStart) {
      continue;
    }
    // The record states the count one space after the path, and the run of digits is the count.
    if (!output.startsWith(countPrefix, pathEnd)) {
      continue;
    }
    const countStart = pathEnd + countPrefix.length;
    let countEnd = countStart;
    while (countEnd < output.length && isDigit(output[countEnd] ?? '')) {
      countEnd++;
    }
    if (countEnd === countStart) {
      continue;
    }
    return Number(output.slice(countStart, countEnd));
  }
  return undefined;
};
