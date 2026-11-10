// The readers a language-server lifetime check applies to the application's log, written as cursor scans.
// What: the text of a record without the colour escapes it carries.
// Why: the application colours every record for a terminal, so a lifetime check that reads a record's level or the
// shape helix-lsp wrote would read the escape instead of the field when the escape is left in place.

/// The escape character that starts a colour sequence.
const ESCAPE = '\u001b';

/// The control sequence introducer that follows the escape character.
const CONTROL_SEQUENCE_INTRODUCER = '[';

/// The select-graphic-rendition command, the only control command a colour sequence ends with.
const SELECT_GRAPHIC_RENDITION = 'm';

/**
 Whether one UTF-16 code unit is a parameter of a colour sequence.
 
 What: a decimal digit or the separator between parameters, the set `[0-9;]` reached.
 
 Why: every other character, a space included, ends the parameter list, and a sequence whose list is followed by one of
 those is not a colour sequence at all.
 
 @param codeUnit - one code unit of the text, from `String#charCodeAt`
 @returns whether the character may stand inside a colour sequence's parameter list
 */
function isGraphicRenditionParameter(codeUnit: number): boolean {
  return ((codeUnit >= 0x30) && (codeUnit <= 0x39)) || (codeUnit === 0x3b);
}

/**
 The text of a log without the colour sequences it carries.
 
 What: one pass that keeps the span between colour sequences, the answer a global `\u001b\[[0-9;]*m` replacement gave to
 the same text: a sequence that ends in another command, or that ends nowhere, stays in the text, and an escape the
 introducer does not follow stays too.
 
 Why: a record states its level after its timestamp, so the escapes have to go before the level is read; the scan keeps
 the spans between them rather than joining one character at a time.
 
 @param text - the standard error of a run, holding one coloured record per line
 @returns the same text without its colour sequences
 */
export function withoutColourSequences(text: string): string {
  if (!text.includes(ESCAPE)) return text;
  const kept: string[] = [];
  let keptFrom = 0;
  let index = 0;
  while (index < text.length) {
    if ((text[index] !== ESCAPE) || (text[index + 1] !== CONTROL_SEQUENCE_INTRODUCER)) {
      index += 1;
      continue;
    }
    let end = index + 2;
    while (end < text.length && isGraphicRenditionParameter(text.charCodeAt(end))) end += 1;
    if (text[end] !== SELECT_GRAPHIC_RENDITION) {
      // Not this command: the scan resumes one character on, so an escape inside the rejected text still matches.
      index += 1;
      continue;
    }
    kept.push(text.slice(keptFrom, index));
    index = end + 1;
    keptFrom = index;
  }
  if (keptFrom === 0) return text;
  kept.push(text.slice(keptFrom));
  return kept.join('');
}
