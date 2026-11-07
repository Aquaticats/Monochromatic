/**
 Small checks on text, written as character scans because the linter bans regular expressions without a scoped
 disable: every input here is short and the scans are linear.

 @module
 */

/**
 Characters a command word needs no quoting for when the command is shown.
 */
const SAFE_WORD_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_@%+=:,./-';

/**
 Characters of a plain path or folder name: word characters, dot, slash, at, plus and dash.
 */
const PLAIN_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_./@+-';

/**
 Tells whether every character of a text is in an allowed set, and the text is not empty.

 @param text - the text
 
 @param allowed - the allowed characters
 
 @returns true when the text is made only of allowed characters
 */
function isMadeOf(
  {
    text,
    allowed,
  }: {
    readonly text: string;
    readonly allowed: string;
  },
): boolean {
  return (text !== '')
    && Array.from(text,)
      .every(function isAllowed(character,) {
        return allowed.includes(character,);
      },);
}

/**
 Tells whether a text is one or more ASCII digits.

 @param text - the text
 
 @returns true for a non-empty run of digits
 */
export function isDigits(text: string,): boolean {
  return isMadeOf({
    text,
    allowed: '0123456789',
  },);
}

/**
 Tells whether a command word can be shown unquoted.

 @param word - the word
 
 @returns true when it has only safe characters
 */
export function isSafeWord(word: string,): boolean {
  return isMadeOf({
    text: word,
    allowed: SAFE_WORD_CHARACTERS,
  },);
}

/**
 Tells whether a folder name is plain: it has only characters the tool quotes nothing for, and no slash.

 @param name - the folder name
 
 @returns true for a plain name
 */
export function isPlainName(name: string,): boolean {
  return isMadeOf({
    text: name,
    allowed: PLAIN_CHARACTERS,
  },) && (!name.includes('/',));
}

/**
 Tells whether an absolute path is plain: it starts with a slash and has only plain characters.

 @param path - the path
 
 @returns true for a plain absolute path
 */
export function isPlainPath(path: string,): boolean {
  return path.startsWith('/',) && isMadeOf({
    text: path,
    allowed: PLAIN_CHARACTERS,
  },);
}

/**
 Splits a line into its space-separated words, dropping empty ones.

 @param line - the line
 
 @returns the words
 */
export function wordsOf(line: string,): readonly string[] {
  return line.replaceAll(
    '\t',
    ' ',
  )
    .split(' ',)
    .filter(function notEmpty(word,) {
      return word !== '';
    },);
}
