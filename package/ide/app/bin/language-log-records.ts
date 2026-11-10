// The readers a language-server lifetime check applies to the application's log, written as cursor scans.
// What: the text of a record without the colour escapes it carries, and the level a record states.
// Why: the application colours every record for a terminal, so a lifetime check that reads a record's level or the
// shape helix-lsp wrote would read the escape instead of the field when the escape is left in place, and it accepts
// a run only when no record states ERROR.

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

/**
 Whether one UTF-16 code unit is a character `\s` reaches.
 
 What: the WhiteSpace and LineTerminator set, spelled as the ranges it is.
 
 Why: a record separates its fields with a space, a no-break space or an ideographic space depending on the writer, and
 a scan that knows only the ASCII space would read a field that starts with one of the others as part of the field
 before it.
 
 @param codeUnit - one code unit of the text, from `String#charCodeAt`
 @returns whether the character is whitespace
 */
function isPatternWhitespace(codeUnit: number): boolean {
  return ((codeUnit >= 0x0009) && (codeUnit <= 0x000d))
    || (codeUnit === 0x0020)
    || (codeUnit === 0x00a0)
    || (codeUnit === 0x1680)
    || ((codeUnit >= 0x2000) && (codeUnit <= 0x200a))
    || (codeUnit === 0x2028)
    || (codeUnit === 0x2029)
    || (codeUnit === 0x202f)
    || (codeUnit === 0x205f)
    || (codeUnit === 0x3000)
    || (codeUnit === 0xfeff);
}

/// The level names a record states, the five `tracing` writes.
const LEVELS = ['TRACE', 'DEBUG', 'INFO', 'WARN', 'ERROR'] as const;

/**
 The level a record states, or nothing when it states none.
 
 What: one pass that walks the first field, the whitespace after it, one of the five level names and the whitespace
 after that, the answer an anchored `^\S+\s+(TRACE|DEBUG|INFO|WARN|ERROR)\s` match gave to the same line.

 Why: the level is what decides whether a record reports a failure, so the line is read once from its start: a line
 whose first field is empty, whose level is spelled in another case, or whose level ends the line states no level.

 @param line - one record of the application's log, with its colour sequences already removed
 @returns the level name the record states, or `undefined`
 */
export function levelOf(line: string): string | undefined {
  let index = 0;
  const fieldStart = index;
  while ((index < line.length) && !isPatternWhitespace(line.charCodeAt(index))) index += 1;
  // A field is one or more non-whitespace characters, and the level follows one or more whitespace characters.
  if ((index === fieldStart) || (index === line.length)) return undefined;
  while ((index < line.length) && isPatternWhitespace(line.charCodeAt(index))) index += 1;
  for (const level of LEVELS) {
    if (!line.startsWith(level, index)) continue;
    const after = index + level.length;
    if ((after < line.length) && isPatternWhitespace(line.charCodeAt(after))) return level;
  }
  return undefined;
}
