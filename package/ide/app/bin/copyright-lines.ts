// Reads of a license text for its copyright line, as the bundle checks ask for them.
// What: whether one line of the text opens with "Copyright " and names the project authors further along it, and
// whether the text mentions copyright at all, in either case of the letters.
// Why: the font notices are shipped verbatim, so the check can only demand that the text still states who holds the
// copyright; a line folded in the middle by an editor would name nobody, and a phrase occurring mid-line is a sentence
// about copyright rather than the notice itself. A grammar's notice may spell the word in any case.
// The functions are linear passes over the text and are tested in copyright-lines.test.ts.

/**
 The phrase a notice mentions, in whatever case it spells it.
 */
const copyrightWord = 'copyright';

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
 The same character in lower case, for the ASCII letters and for nothing else.
 *
 A pattern without the `u` flag folds case by comparing each character's upper case, and refuses to fold a character
 outside ASCII to one inside it, so `COPYRIGHT` and `copyright` are the same word there while a dotted capital I or a
 long s is never the letter it resembles. Folding the whole text through the language's lower-case map would not keep
 that boundary: it turns a dotted capital I into an i and a combining mark.
 *
 @param character - one character of the text.
 *
 @returns the character, lower-cased when it is a capital ASCII letter.
 */
const asciiLowercase = (character: string): string => ((character >= 'A') && (character <= 'Z'))
  ? String.fromCharCode(character.charCodeAt(0) + 0x20)
  : character;

/**
 Whether `text` mentions the word "copyright", spelled in either case of its letters.
 *
 @param text - the notice text to read.
 *
 @returns whether the word occurs in the text.
 */
export const mentionsCopyright = (text: string): boolean => {
  for (let at = 0; at + copyrightWord.length <= text.length; at++) {
    let matched = true;
    for (let offset = 0; offset < copyrightWord.length; offset++) {
      if (asciiLowercase(text[at + offset] ?? '') !== copyrightWord[offset]) {
        matched = false;
        break;
      }
    }
    if (matched) {
      return true;
    }
  }
  return false;
};

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
