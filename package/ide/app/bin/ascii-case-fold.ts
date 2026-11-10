// Whether a text holds a phrase, reading ASCII letters in either case.
// What: the fold a pattern carrying the `i` flag applies, and the scan that asks whether one phrase stands in a text.
// Why: two checks in this package ask it (a bare shutdown phrase in a log record, and a credential word in a variable
// name), and both answer a pattern written in lower-case ASCII, so the fold is one code path rather than two.
// The fold is ASCII alone because that is the fold the `i` flag applies to an ASCII phrase: a character above ASCII
// whose uppercase form is an ASCII letter is left unfolded by the pattern too, so a long s never stands for an `s` and
// a Kelvin sign never stands for a `k`.

/**
 One UTF-16 code unit folded to lower case, for a fold above ASCII that the `i` flag never made.
 
 What: the ASCII uppercase letters mapped to their lower case, every other code unit unchanged.
 
 Why: folding the whole text would answer a phrase the pattern never read, so the fold covers ASCII alone and reads the
 text where it lies instead of building a lower-cased copy of it.
 
 @param codeUnit - one code unit of the text, from `String#charCodeAt`
 @returns the code unit of its ASCII lower case, or the code unit itself
 */
function asciiLowerCase(codeUnit: number): number {
  return ((codeUnit >= 0x41) && (codeUnit <= 0x5a)) ? codeUnit + 0x20 : codeUnit;
}

/**
 Whether a text holds a phrase, reading ASCII letters in either case.
 
 What: a scan from each position of the text that compares one phrase's ASCII lower case with the text's, which is the
 answer an unanchored `phrase` pattern carrying the `i` flag gave over the same text.
 
 Why: the phrases are short and the texts are one record or one variable name, so the scan costs the text's length
 times the phrase's, exactly what the pattern's engine walked.
 
 @param text - the text to read
 @param phrase - the phrase to look for, in lower-case ASCII, one character or longer
 @returns whether the text holds the phrase in some case
 */
export function holdsIgnoringAsciiCase(text: string, phrase: string): boolean {
  const lastStart = text.length - phrase.length;
  for (let start = 0; start <= lastStart; start += 1) {
    let offset = 0;
    while ((offset < phrase.length)
      && (asciiLowerCase(text.charCodeAt(start + offset)) === phrase.charCodeAt(offset))) offset += 1;
    if (offset === phrase.length) return true;
  }
  return false;
}
