// The crate license sections `monochromatic-ide --licenses` prints, read as the records the bundle check compares.
// What: every section of the listing that states whose terms a text carries, how many crates those terms cover, the
// crates by name and release, and where the text comes from, each with the whole text of the section, so the text
// printed below it can be compared with the crate list.
// Why: the crate-licenses check demands that every crate the executable links has its license text printed in full, so
// it must read the section headings the application writes (src/runtime/notices.rs and
// src/runtime/crate_licenses.rs) and the texts below them. One section is a heading line, the crates it names, the line
// naming where the text comes from, a rule of 78 equals signs, and a blank line.
// The read is one pass over the listing and is tested in crate-license-section.test.ts.

/**
 One printed section: the text of it, and the fields its heading and origin lines state.
 */
export type CrateLicenseSection = {
  readonly whole: string
  readonly name: string
  readonly id: string
  readonly crateCount: number
  readonly usedBy: string
};

/**
 The words a section heading opens with.
 */
const headingPrefix = 'Rust crates under ';

/**
 The words between a heading's crate count and its line end.
 */
const crateNounPrefix = ' crate';

/**
 The letter the crate noun carries when the count is not one.
 */
const pluralSuffix = 's';

/**
 The label the first line of a crate list opens with, and the indentation that continues it.
 *
 The two are the same width, so a wrapped list lines up under the names it starts with.
 */
const usedByLabel = 'Used by: ';
const usedByIndent = '         ';

/**
 The words the origin line opens with when the text is a crate's own file.
 */
const crateFilePrefix = 'Text from the crate file ';

/**
 The origin line of a text that is the standard text of its license, with no crate file recognized.
 */
const standardTextLine = 'Text: the standard text of this license (no crate file was recognized)';

/**
 The character a section's rule is drawn with, and how many of them it holds.
 */
const ruleCharacter = '=';
const ruleLength = 78;

/**
 The line terminator every line of a section is ended by.
 */
const sectionLineTerminator = '\n';

/**
 Whether a character ends a line.
 *
 A JavaScript pattern's `.` reaches every character but these four, and its multiline anchor sits after each of them,
 so both the line a field may run along and where a section may begin follow this set.
 *
 @param character - one character of the listing.
 *
 @returns whether the character is a line terminator in the language's sense.
 */
const isLineTerminator = (character: string): boolean => (character === '\n')
  || (character === '\r')
  || (character === '\u2028')
  || (character === '\u2029');

/**
 Whether a character ends a section heading's license identifier.
 *
 An identifier is written between parentheses, so neither parenthesis and no line feed can stand inside it; the other
 line terminators can, because the pattern that read identifiers before reached them.
 *
 @param character - one character of the listing.
 *
 @returns whether the character ends an identifier.
 */
const endsIdentifier = (character: string): boolean => (character === '(')
  || (character === ')')
  || (character === '\n');

/**
 Whether a character is an ASCII digit, the only kind a crate count is written with.
 *
 @param character - one character of the listing.
 *
 @returns whether the character is in `0` to `9`.
 */
const isDigit = (character: string): boolean => (character >= '0') && (character <= '9');

/**
 Whether `at` is the start of a line of the listing.
 *
 @param options - `printed` is the listing; `at` the character to read.
 *
 @returns whether a line begins at `at`.
 */
const isLineStart = ({ printed, at }: { printed: string; at: number }): boolean => (at === 0)
  || isLineTerminator(printed[at - 1] ?? '');

/**
 The start of the line after the one `at` stands on.
 *
 @param options - `printed` is the listing; `at` the character to read from.
 *
 @returns The index just past the next line terminator, or the end of the listing.
 */
const nextLineStart = ({ printed, at }: { printed: string; at: number }): number => {
  // The line that begins at `at` may be empty, and then its own terminator is the one that ends it.
  for (let after = at; after < printed.length; after++) {
    if (isLineTerminator(printed[after] ?? '')) {
      return after + 1;
    }
  }
  return printed.length;
};

/**
 The line that begins at `at`, and the index just past its terminator.
 *
 A line here is ended by a line feed and by nothing else, as every line of a section is: a carriage return or a
 paragraph separator before the line feed would leave the pattern that read these sections with no line feed to find.
 *
 @param options - `printed` is the listing; `at` the character the line begins at.
 *
 @returns The line's text and where the next line begins, or absence when no line feed ends it.
 */
const readLine = ({ printed, at }: { printed: string; at: number }): { content: string; end: number } | undefined => {
  let end = at;
  while (end < printed.length && !isLineTerminator(printed[end] ?? '')) {
    end++;
  }
  if (printed[end] !== sectionLineTerminator) {
    return undefined;
  }
  return { content: printed.slice(at, end), end: end + 1 };
};

/**
 The ASCII digits that begin at `at`, as a number, and the index just past them.
 *
 @param options - `printed` is the listing; `at` the character the digits begin at.
 *
 @returns The number and where it ends, or absence when no digit begins at `at`.
 */
const readDigits = ({ printed, at }: { printed: string; at: number }): { count: number; end: number } | undefined => {
  let end = at;
  while (end < printed.length && isDigit(printed[end] ?? '')) {
    end++;
  }
  if (end === at) {
    return undefined;
  }
  return { count: Number(printed.slice(at, end)), end };
};

/**
 The ways the name and the identifier of a heading can be divided, the last space before a parenthesis first.
 *
 A pattern that reads a heading takes as much of it as it can for the name, so it tries the last ` (` of the line
 before the ones before it; the name is at least one character and never crosses a line terminator.
 *
 @param options - `printed` is the listing; `at` the line start the heading begins at.
 *
 @returns Each candidate name with the index just past its ` (`, in the order a greedy read tries them.
 */
const headingSplits = ({ printed, at }: { printed: string; at: number }): { name: string; afterParen: number }[] => {
  const nameStart = at + headingPrefix.length;
  const splits: { name: string; afterParen: number }[] = [];
  for (let space = nameStart + 1; space < printed.length; space++) {
    if (isLineTerminator(printed[space] ?? '')) {
      break;
    }
    if (printed[space] === ' ' && printed[space + 1] === '(') {
      splits.push({ name: printed.slice(nameStart, space), afterParen: space + 2 });
    }
  }
  return splits.reverse();
};

/**
 The identifier and the crate count on a heading line, and where the line ends.
 *
 @param options - `printed` is the listing; `at` the character after the heading's ` (`.
 *
 @returns The identifier, the crate count and the index just past the line's terminator, or absence when the line
 states neither.
 */
const readHeadingTail = ({ printed, at }: { printed: string; at: number }): { id: string; crateCount: number; end: number } | undefined => {
  let idEnd = at;
  while (idEnd < printed.length && !endsIdentifier(printed[idEnd] ?? '')) {
    idEnd++;
  }
  if (idEnd === at || !printed.startsWith('): ', idEnd)) {
    return undefined;
  }
  const digits = readDigits({ printed, at: idEnd + 3 });
  if (digits === undefined || !printed.startsWith(crateNounPrefix, digits.end)) {
    return undefined;
  }
  const afterNoun = digits.end + crateNounPrefix.length;
  if (printed[afterNoun] === pluralSuffix && printed[afterNoun + 1] === sectionLineTerminator) {
    return { id: printed.slice(at, idEnd), crateCount: digits.count, end: afterNoun + 2 };
  }
  if (printed[afterNoun] === sectionLineTerminator) {
    return { id: printed.slice(at, idEnd), crateCount: digits.count, end: afterNoun + 1 };
  }
  return undefined;
};

/**
 The lines that name where a section's text comes from, and where the section ends.
 *
 A crate list is one line that opens with its label, then the lines that continue it under the label's width; the line
 after them names where the text comes from, and the rule and the blank line close the section.
 *
 @param options - `printed` is the listing; `at` the line start the crate list begins at.
 @returns The crate list's lines and the index just past the blank line, or absence when no origin line follows
 the list.
 */
const readOrigin = ({ printed, at }: { printed: string; at: number }): { usedBy: string; end: number } | undefined => {
  const first = readLine({ printed, at });
  if (first === undefined) {
    return undefined;
  }
  if (!first.content.startsWith(usedByLabel) && !first.content.startsWith(usedByIndent)) {
    return undefined;
  }
  let listEnd = first.end;
  // A line that continues the list is indented to the label's width and states crates, so it opens with that indent.
  while (true) {
    const continuation = readLine({ printed, at: listEnd });
    if (continuation === undefined || !continuation.content.startsWith(usedByIndent)) {
      break;
    }
    listEnd = continuation.end;
  }
  const origin = readLine({ printed, at: listEnd });
  if (origin === undefined) {
    return undefined;
  }
  const namesTextFile = origin.content.startsWith(crateFilePrefix) && (origin.content.length > crateFilePrefix.length);
  if (!namesTextFile && origin.content !== standardTextLine) {
    return undefined;
  }
  const rule = ruleCharacter.repeat(ruleLength) + sectionLineTerminator + sectionLineTerminator;
  if (!printed.startsWith(rule, origin.end)) {
    return undefined;
  }
  return { usedBy: printed.slice(at, listEnd), end: origin.end + rule.length };
};

/**
 The section that begins at a line start, or absence when no section does.
 *
 @param options - `printed` is the listing; `at` the line start to read a section at.
 @returns The section and the index just past it, or absence when no section begins at `at`.
 */
const readSection = ({ printed, at }: { printed: string; at: number }): { section: CrateLicenseSection; end: number } | undefined => {
  if (!printed.startsWith(headingPrefix, at)) {
    return undefined;
  }
  for (const split of headingSplits({ printed, at })) {
    const tail = readHeadingTail({ printed, at: split.afterParen });
    if (tail === undefined) {
      continue;
    }
    const origin = readOrigin({ printed, at: tail.end });
    if (origin === undefined) {
      continue;
    }
    const section = {
      whole: printed.slice(at, origin.end),
      name: split.name,
      id: tail.id,
      crateCount: tail.crateCount,
      usedBy: origin.usedBy,
    };
    return { section, end: origin.end };
  }
  return undefined;
};

/**
 Every crate license section of a printed listing, in the order the listing prints them.
 *
 @param printed - the whole output of `--licenses`.
 *
 @returns The sections the listing holds.
 */
export const crateLicenseSections = (printed: string): CrateLicenseSection[] => {
  const sections: CrateLicenseSection[] = [];
  let at = 0;
  while (at <= printed.length) {
    const read = isLineStart({ printed, at }) ? readSection({ printed, at }) : undefined;
    if (read !== undefined) {
      sections.push(read.section);
      at = read.end;
      continue;
    }
    if (at === printed.length) {
      break;
    }
    at = nextLineStart({ printed, at });
  }
  return sections;
};
