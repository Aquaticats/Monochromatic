// The readers a language-server lifetime check applies to the application's log, written as cursor scans.
// What: the text of a record without the colour escapes it carries, the level a record states, and whether the text
// reports its own shutdown as a failure.
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

/// The phrases a process prints when it reports its own shutdown as a failure, in lower case.
const BARE_SHUTDOWN_PHRASES = ['context canceled', 'streamclosed', 'broken pipe'] as const;

/**
 One UTF-16 code unit folded to lower case, for a fold above ASCII that the `i` flag never made.
 
 What: the ASCII uppercase letters mapped to their lower case, every other code unit unchanged.
 
 Why: a `key` alternative with the `i` flag folds a character with an ASCII letter only when the character itself is
 ASCII, so a long s never stands for an `s` and a Kelvin sign never stands for a `k`; folding ASCII alone keeps that
 answer, and a fold of the whole text would answer a phrase this check must not read.
 
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
 
 Why: the phrases are short and the text is one record, so the scan costs the record's length times the phrase's,
 exactly what the pattern's engine walked, and it reads the text where it lies instead of folding a copy of it.
 
 @param text - the text to read
 @param phrase - the phrase to look for, in lower case ASCII
 @returns whether the text holds the phrase in some case
 */
function holdsIgnoringAsciiCase(text: string, phrase: string): boolean {
  const lastStart = text.length - phrase.length;
  for (let start = 0; start <= lastStart; start += 1) {
    let offset = 0;
    while ((offset < phrase.length)
      && (asciiLowerCase(text.charCodeAt(start + offset)) === phrase.charCodeAt(offset))) offset += 1;
    if (offset === phrase.length) return true;
  }
  return false;
}

/**
 Whether a text reports a process's own shutdown as a failure.
 
 What: a search for each of the three phrases a process prints when it does, the answer an unanchored
 `context canceled|StreamClosed|broken pipe` pattern carrying the `i` flag gave to the same text.
 
 Why: a phrase is a bare shutdown error when it reaches the log at WARN or ERROR, or outside any record, and a phrase
 below WARN is labelled for what it is; the caller decides that by reading the record's level, so this reader answers
 the phrase alone and never the level.
 
 @param text - one record of the application's log, or text that reached the log outside any record
 @returns whether the text holds one of the phrases in some case
 */
export function isBareShutdownError(text: string): boolean {
  return BARE_SHUTDOWN_PHRASES.some(phrase => holdsIgnoringAsciiCase(text, phrase));
}
