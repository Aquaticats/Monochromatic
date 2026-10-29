import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { fencedLineFlags, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Invisible line masking
// Turns a line that shows nothing into a line that IS nothing, before the
// Markdown parser reads it.
//
// The corpus case: `people/Toka_ls/page.en.md` carries lines holding U+FEFF and
// nothing else, sitting between two ordinary sentences with no blank line
// anywhere near them. A line carrying a byte-order mark is not blank, so
// CommonMark reads it as a CONTINUATION and merges the paragraphs either side
// of it into one block. That translation therefore parses to 29 blocks against
// the original's 33, they track one-to-one until the first such line, and from
// there every English block pairs with the WRONG Chinese one.
//
// The damage is invisible to every later stage. Handed a source paragraph
// beside the wrong target paragraph, the editor rewrote a correct rendering of
// 期盼中，她看见光穿透暗影 into a faithful rendering of 尽管前路漫布荆棘, a
// sentence two blocks away. Both texts are fluent, both translate something the
// original really says, and the critics, checkers and probe all compare against
// whatever paragraph the misalignment handed them.
//
// Masking preserves LENGTH, exactly as `maskHtmlComments` does, because node
// text, quotes, hashes and every claim anchor are sliced from the original body
// by absolute offset. Replacing the character rather than removing the line is
// what lets the paragraph break come back without moving anything.

/**
 Characters CommonMark counts as blank.

 The entire list. A line built from only these ends a paragraph; a line
 holding anything else does not, however little of it a reader can see.
 */
const BLANK_TO_COMMONMARK: ReadonlySet<string> = new Set([
  '\u{0020}',
  '\u{0009}',
],);

/**
 Characters that occupy a line while showing a reader nothing.

 The byte-order mark is the one that occurred. The rest are listed because
 they fail identically: each shows nothing, each keeps a line from being
 blank, and each therefore welds two paragraphs together.

 Includes the CommonMark blanks, so this set answers one question only, "can a
 reader see it", and blankness is asked separately against
 {@link BLANK_TO_COMMONMARK}. Splitting the two questions is what keeps the
 non-ASCII spaces from being skipped: U+00A0, U+202F and U+3000 are
 ECMAScript whitespace, so any check phrased with `trim()` calls them empty
 and passes over the very characters it exists to catch. That trap already
 broke the first draft of this file once, with U+FEFF.

 Membership requires being invisible UNCONDITIONALLY, which is why three near
 misses are absent. U+00AD renders as a hyphen wherever a line happens to
 break; U+2028 and U+2029 carry line and paragraph meaning of their own.
 Masking a line made only of one of those would be a judgement about
 rendering rather than the restoration of a lost paragraph break, and
 declining costs nothing but a weld nobody has observed.
 */
const SHOWS_NOTHING: ReadonlySet<string> = new Set([
  '\u{0020}',
  '\u{0009}',
  '\u{FEFF}',
  '\u{200B}',
  '\u{200C}',
  '\u{200D}',
  '\u{2060}',
  '\u{180E}',
  '\u{00A0}',
  '\u{202F}',
  '\u{2007}',
  '\u{3000}',
],);

/**
 Whether a line shows a reader nothing yet is not blank to the parser.

 Both halves are required. Showing nothing is what makes blanking the line
 lossless; not being blank already is what makes it worth doing, so an
 ordinary blank line is left exactly as it is rather than rebuilt into an
 identical one.

 @param line - one line without its terminator

 @returns Whether the line welds the paragraphs either side of it

 @example
 ```ts
 const welds = isInvisibleOnly({ line: '\u{FEFF}', },);
 ```
 */
function isInvisibleOnly({ line, }: { readonly line: string; },): boolean {
  /**
   Whether a character was seen that a reader cannot see but the parser can,
   which is what distinguishes this line from an ordinary blank one.
   */
  let sawInvisible = false;
  for (const character of line) {
    if (!SHOWS_NOTHING.has(character,))
      return false;

    if (!BLANK_TO_COMMONMARK.has(character,))
      sawInvisible = true;
  }
  return sawInvisible;
}

/**
 One line the mask blanked, in offsets into the text it was given.

 Reported so the tolerance is never silent. Both parser defects this pipeline
 has hit were found by accident rather than from an artifact, and a masked
 line that nobody can see is exactly the shape that hides the third one.

 @example
 ```ts
 const region: MaskedInvisibleLine = {
   startOffset: 12,
   endOffset: 13,
   codePoints: ['U+FEFF',],
 };
 ```
 */
export type MaskedInvisibleLine = {
  /**
   Offset of the line's first character.
   */
  readonly startOffset: number;

  /**
   Exclusive end offset, before the line's terminator.
   */
  readonly endOffset: number;

  /**
   Which invisible characters the line carried, named rather than embedded, so
   a finding can be read without the characters vanishing into it.
   */
  readonly codePoints: readonly string[];
};

/**
 Base code points are conventionally written in.
 */
const HEX_RADIX = 16;

/**
 Fewest digits a code point is written with.
 */
const CODE_POINT_DIGITS = 4;

/**
 Names an invisible character the way a reader can see.

 @param character - character to name

 @returns Code point in `U+XXXX` form

 @example
 ```ts
 const name = nameOf({ character: '\u{FEFF}', },);
 ```
 */
function nameOf({ character, }: { readonly character: string; },): string {
  /**
   Code point, which is what identifies the character to whoever reads this.
   */
  const point = nonNullishOrThrow(character.codePointAt(0,),);

  /**
   Hexadecimal digits, padded to the conventional minimum width.
   */
  const digits = point
    .toString(HEX_RADIX,)
    .toUpperCase();

  /**
   Same padded to the conventional minimum width.
   */
  const padded = digits.padStart(
    CODE_POINT_DIGITS,
    '0',
  );

  return `U+${padded}`;
}

/**
 Names every character on a line that a reader cannot see.

 Scans rather than spreading, because spreading a string is restricted here
 for splitting complex characters, and the loop states plainly that this walks
 code points.

 @param line - line already known to show a reader nothing

 @returns Code point names in the order they appear

 @example
 ```ts
 const points = unseenCodePoints({ line: '\u{FEFF}', },);
 ```
 */
function unseenCodePoints({ line, }: { readonly line: string; },): readonly string[] {
  return (function scan(): readonly string[] {
    /**
     Names found so far.
     */
    const names: string[] = [];
    for (const character of line) {
      if (BLANK_TO_COMMONMARK.has(character,))
        continue;

      names.push(nameOf({ character, },),);
    }

    return names;
  })();
}

/**
 Replaces every invisible-only line outside fenced code with spaces.

 Written as a scan over lines rather than a pattern: the rule is one predicate
 per line, and it must not backtrack over a document built to be pathological.

 Fenced code is exempt because a line holding a zero-width space is CONTENT
 there, and rewriting content is the one thing a length-preserving mask exists
 to avoid. INDENTED code is not exempt, deliberately. Four spaces then an
 invisible character is a paragraph continuation far more often than it is
 code, since indented code cannot interrupt a paragraph, so a guard on indent
 would decline the mask in the common case to protect the rare one.

 @param text - body text as written

 @returns Same text, same length, with invisible-only lines blanked, plus one
 region per blanked line

 @example
 ```ts
 const { masked, regions, } = maskInvisibleLines({ text: body, },);
 ```
 */
export function maskInvisibleLines(
  { text, }: { readonly text: string; },
): {
  /**
   Text with every invisible-only line blanked, at its original length.
   */
  readonly masked: string;

  /**
   One region per blanked line, in document order.
   */
  readonly regions: readonly MaskedInvisibleLine[];
} {
  /**
   Body split once, so the flags and the mask agree line for line.
   */
  const lines = text.split('\n',);

  /**
   Which lines belong to a fenced code block and are therefore left alone.
   */
  const fenced = fencedLineFlags({ lines, },);

  /**
   Lines blanked so far, recorded as the scan walks the text.
   */
  const regions: MaskedInvisibleLine[] = [];

  /**
   Where each line begins, so the scan carries no running cursor.
   */
  const lineStarts = (function measure(): readonly number[] {
    /**
     Start offsets, one per line, built in document order.
     */
    const starts: number[] = [];

    /**
     Offset of the line under the cursor.
     */
    let cursor = 0;
    for (const line of lines) {
      starts.push(cursor,);

      // Past the line and the newline that split it off. The last line has no
      // newline, and nothing reads the cursor after it, so the overshoot is
      // harmless and costs no special case.
      cursor += line.length + 1;
    }

    return starts;
  })();

  /**
   Rebuilt lines, blanked where they showed a reader nothing.
   */
  const rebuilt = lines.map(function blankInvisible(
    line,
    index,
  ) {
    if (fenced[index] === true)
      return line;

    /**
     Whether this line ends in a carriage return, which a CRLF page leaves
     on every line the split on `\n` produces.

     JUDGED WITHOUT IT AND BLANKED AROUND IT. The return is part of the line
     ending, not of what a reader sees, so it neither makes the line visible
     nor gets blanked: it stays exactly where it was, and the line stays
     exactly as long.
     */
    const carriage = line.endsWith('\r',);

    /**
     The line without its carriage return, which is what a reader sees.
     */
    const body = carriage
      ? line.slice(
        0,
        -1,
      )
      : line;
    if (!isInvisibleOnly({ line: body, },))
      return line;

    /**
     Where this line starts, measured before anything was rewritten.
     */
    const startOffset = nonNullishOrThrow(lineStarts[index],);
    regions.push({
      startOffset,
      endOffset: startOffset + body.length,
      codePoints: unseenCodePoints({ line: body, },),
    },);

    // Spaces rather than removal: a line of spaces is blank to CommonMark, so
    // the paragraph break returns. Repeating over the line's own UTF-16
    // length keeps the replacement exactly as long as what it replaces, which
    // is what every absolute offset downstream depends on, and it sidesteps
    // the question of how a code point maps to units entirely.
    return `${' '.repeat(body.length,)}${carriage ? '\r' : ''}`;
  },);

  return {
    masked: rebuilt.join('\n',),
    regions,
  };
}

//endregion Invisible line masking
