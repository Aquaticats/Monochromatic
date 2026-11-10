// Reads of a license text for its copyright line, as the bundle checks ask for them.
// What: whether one line of the text opens with "Copyright " and names the project authors further along it.
// Why: the font notices are shipped verbatim, so the check can only demand that the text still states who holds the
// copyright; a line folded in the middle by an editor would name nobody, and a phrase occurring mid-line is a sentence
// about copyright rather than the notice itself.
// The functions are linear passes over the text and are tested in copyright-lines.test.ts.

/**
 The word a copyright line of a project-held notice puts before the holders: "The Inter Project Authors".
 */
const projectAuthors = 'Project Authors';

/**
 The label a copyright line opens with, followed by the year and the holders.
 */
const copyrightLabel = 'Copyright ';

/**
 Whether a character ends the line a match may run along.
 *
 A JavaScript regex `.` in a pattern without the `s` flag reaches every character but these four, and its multiline
 anchor sits after each of them.
 *
 @param character - one character of the text.
 *
 @returns whether the character is a line terminator in the language's sense.
 */
const isLineTerminator = (character: string): boolean => (character === '\n')
  || (character === '\r')
  || (character === '\u2028')
  || (character === '\u2029');

/**
 Whether some line of `text` opens with "Copyright " and names the project authors on that same line.
 *
 The line the label opens runs to the next line terminator of any kind, so a notice folded with a carriage return or a
 paragraph separator names nobody even when the phrase follows on the next line.
 *
 @param text - the license text to read.
 *
 @returns whether the text carries such a line.
 */
export const hasProjectAuthorsCopyrightLine = (text: string): boolean => {
  // A match began only at the start of the text or right after a line terminator, so those are the only candidates.
  for (let at = 0; at < text.length; at++) {
    if (at > 0 && !isLineTerminator(text[at - 1] ?? '')) {
      continue;
    }
    if (!text.startsWith(copyrightLabel, at)) {
      continue;
    }
    const afterLabel = at + copyrightLabel.length;
    let lineEnd = afterLabel;
    while (lineEnd < text.length && !isLineTerminator(text[lineEnd] ?? '')) {
      lineEnd++;
    }
    if (text.slice(afterLabel, lineEnd).includes(projectAuthors)) {
      return true;
    }
  }
  return false;
};
