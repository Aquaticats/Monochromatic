import {
  codePointBefore,
  continuesLatinWordOrCompatibilityForm,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Consistency markup runs
// THE RUNS OF MARKUP BESIDE A WORD, in one place, so the readings that step over such a run cannot drift apart. There are
// TWO RUNS, named for the job each does, and they are not one run read two ways.
//
// THE READING RUN is what a single mark is READ PAST: the marks that close emphasis, strong emphasis or strikethrough by the
// parse, and the closing `](…)` of a link, on the trailing side of a word and nowhere else. A mark that ends such a run after a
// Latin letter or a digit reads as the trailing edge of the word (`*kittens*’`), since the markup is no part of the word.
// Whether a `*`, `_` or `~` is a delimiter is the parse's to say (`EmphasisDelimiters`, `consistency-prose-text.ts`): a lone
// one is a character of the text and ends the run. A closing double quotation mark or bracket is NOT in the reading run: after
// `"ok"` or `(here)` a single mark is most often a closing quotation mark, so it is read where it stands. There is no leading
// mirror: a mark before opening markup (`’**cats**`) is read where it stands. The readers that step over the reading run are
// `closingRunMarksOf` (the proof of an apostrophe, through `marksPastMarkupOf`, `consistency-marks-past-markup.ts`) and the
// edge apostrophe of the words of a span (`consistency-loose-marks.ts`), which ask the one function, so they cannot differ.
//
// THE REFUSAL RUN is what a gain is refused beside (`markPastRunOf`, `consistency-closing-marks.ts`; decision D1, wide run J1):
// a splice that would bring an apostrophe to an edge of a span is unavailable when the receiving place keeps any single mark
// directly outside the span on that side or one refusal run away, whatever that mark reads as, since the two would stand as a
// doubled mark (`*dogs’*’`); it is also unavailable when the chosen words carry two single marks at that edge
// (`gainsBesideKeptMark`, `consistency-splice-readings.ts`). The refusal run stays wide, and it is made of these units: on the
// trailing side every `*`, `_` and `~`, a backtick, closing round and square brackets, closing double quotation marks and
// closing corner brackets, with a link's destination; on the leading side every `*`, `_` and `~`, a backtick, opening round and
// square brackets, opening double quotation marks and opening corner brackets; on both sides every unit a reader does not read
// as a word (`unreadUnitsOf`, `consistency-unread-units.ts`: a tag, the label of a footnote or reference link, a link's
// destination with its title, a comment, a character reference); horizontal white space (`HORIZONTAL_SPACE`, every character
// the proof's parting test reads as white space but the line terminators, the zero width no-break space U+FEFF among them); every
// Default_Ignorable_Code_Point, which renders as nothing (`isDefaultIgnorable`, `renders-as-nothing.ts`: the zero-width
// characters, the soft hyphen, the word joiner, the combining grapheme joiner, the variation selectors), the controls that
// show nothing (U+0001 to U+0008, U+000E to U+001F, U+007F) and the Braille blank U+2800 (`ignorableWidthAt`,
// `consistency-run-steps.ts`; U+0000, which a page shows as the replacement character, ends the run); a backslash escape, a
// backslash and the one ASCII punctuation character it escapes (`\*`, `\_`, `\~`, `\]`, `\)`, `\\`), read as that character is,
// so an escaped single mark (`\'`) is the mark that ends the run; and a single line break inside a paragraph or a setext
// heading, which renders as a space or a break between the marks: a line feed, a carriage return, the next-line control, the line
// separator or the paragraph separator, alone or as a soft break, two spaces and a break, or a backslash and a break. After
// such a break the run also steps over the markers of the container the paragraph stands in: a blockquote marker `>` with its
// optional space, one per level of nesting, the indentation of a list item's continuation line, and the indentation of a lazy
// continuation line. A mark that stands past any of these is still a second mark for the eye (`dogs’[^1] ’`, `dogs’\*’`,
// `dogs’` and `’` on the next line, or after `> ` on the next line). THE RUN ENDS at the first other character: a letter, a
// digit, any punctuation that is none of these units (a comma, a full stop), a single mark, which is the mark the run names, a
// backslash before any other character, a blank line (the line the page grammar calls blank, `consistency-paragraphs.ts`: a line
// of nothing but spaces and tabs, or of characters its mask of invisible lines blanks, between two line breaks; a line of other
// characters, such as an en space or a line separator, is no blank line to it; an English it refuses has no blank line to it
// either, every line break being stepped), a `>` that is text of the paragraph (`a > b`) or a
// `>` beyond the paragraph's blockquote depth, and the edge of the text. An emphasis character is a character of the refusal run WHETHER OR NOT the parse makes it a delimiter (the ruling
// on the J1 shapes: D2 governs how a mark reads, not when a gain is refused), unlike the reading run, which asks the parse.
// `TRAILING_REFUSAL_RUN` and `LEADING_REFUSAL_RUN` hold the characters of the first two kinds besides those three.
//
// A SPLICE IS ALSO REFUSED ON A PAGE THAT HOLDS A CHARACTER REFERENCE THAT IS A QUOTATION MARK (`quoteReferencesIn`,
// `consistency-quote-references.ts`), wherever the reference stands and whether or not the splice brings an apostrophe: no
// reader of the raw text sees the mark (`&#39;`, `&rsquo;`), the refusal run steps over the reference as an unread unit, so the
// run cannot say what stands past it, and the English of every slice of the page is read, a slice read alone included.

/**
 The characters emphasis and strikethrough are written with. Whether one of them is a delimiter is the parse's to say
 (`EmphasisDelimiters`); this set only tells a character that may be one from every other.
 */
export const EMPHASIS_CHARACTERS: ReadonlySet<string> = new Set([
  '*',
  '_',
  '~',
],);

/**
 The closing curly double quotation mark, which the fold writes as the straight
 one and the original English still holds.
 */
export const CLOSING_CURLY_DOUBLE = '\u{201D}';

/**
 Marks that close a bracketed or linked stretch: when one stands directly before
 an apostrophe that a letter follows, the apostrophe belongs to what the stretch
 names (`[the cat](x)'s`, `(grey cat)'s`). Declared here, beside the other marks the readers of a run share, so that the reading of
 a link's label (`consistency-link-label.ts`) tells a last mark that closes from one that does not by the same set.
 */
export const CLOSING_BRACKETS: ReadonlySet<string> = new Set([
  ')',
  ']',
],);

/**
 The characters of the refusal run on the trailing side of a span besides the delimiters of emphasis, as the English wrote
 them: the backtick of inline code, the closing round and square brackets and the double quotation marks that close, and the
 closing corner brackets.
 */
export const TRAILING_REFUSAL_RUN: ReadonlySet<string> = new Set([
  '`',
  ')',
  ']',
  '"',
  CLOSING_CURLY_DOUBLE,
  '」',
  '』',
],);

/**
 The characters of the refusal run on the leading side of a span besides the delimiters of emphasis, as the English wrote
 them: the backtick of inline code, the opening round and square brackets, the double quotation marks that open, and the
 opening corner brackets.
 */
export const LEADING_REFUSAL_RUN: ReadonlySet<string> = new Set([
  '`',
  '(',
  '[',
  '"',
  '“',
  '「',
  '『',
],);

/**
 The horizontal white space the refusal run steps over on either side of a span: every character the proof's parting test
 reads as white space (`partsMarkFromWord`, `consistency-mark-neighbours.ts`, JavaScript white space) but the line
 terminators. These are the tab, the vertical tab, the form feed, the space, the no-break space, the Ogham space mark, the
 spaces from the en quad to the hair space, the narrow no-break space, the medium mathematical space, the ideographic space
 and the byte order mark. A line break is none of them: the run steps over a line break only by the rule of
 `markPastRunOf` (`consistency-closing-marks.ts`).
 */
export const HORIZONTAL_SPACE: ReadonlySet<string> = new Set([
  '\t',
  '\u{000B}',
  '\u{000C}',
  ' ',
  '\u{00A0}',
  '\u{1680}',
  '\u{2000}',
  '\u{2001}',
  '\u{2002}',
  '\u{2003}',
  '\u{2004}',
  '\u{2005}',
  '\u{2006}',
  '\u{2007}',
  '\u{2008}',
  '\u{2009}',
  '\u{200A}',
  '\u{202F}',
  '\u{205F}',
  '\u{3000}',
  '\u{FEFF}',
],);

/**
 The apostrophe-like characters of an English as it was written: the straight apostrophe, the closing curly single mark and
 the modifier letter apostrophe. The opening curly single mark folds to the straight one and is none of them.
 */
export const WRITTEN_APOSTROPHES: ReadonlySet<string> = new Set([
  '\'',
  '’',
  '\u{02BC}',
],);

/**
 What can stand before a mark so that the mark opens and closes nothing: white space, and the empty answer for the start
 of the text. In a link destination, which white space ends, it is also what a destination cannot hold.
 */
export const MARK_PARTING_SPACE: ReadonlySet<string> = new Set([
  '',
  ' ',
  '\t',
  '\n',
  '\r',
],);

/**
 Where the nearest unit that ends a link destination stands, read from the end of the text to its start: the nearest
 closing round bracket or white space at or after the offset the reading has reached, and whether it is the bracket.
 */
export type DestinationStop = {
  /**
   Offset of the nearest closing round bracket or white space, the length of the text when neither stands.
   */
  at: number;

  /**
   Whether the unit there is a closing round bracket, so a destination that reaches it is closed.
   */
  isBracket: boolean;
};

/**
 Where a reading run that starts at each offset of a text ends, read once from the end of the text to its start, so a link
 destination is skipped in constant time and no run is walked again for each letter before it. The run is made of closing
 delimiters of emphasis (the offsets the caller names, from the parse) and the closing `](…)` of a link; it ends at the first
 unit that is none of them. A `](` whose destination holds white space or is never closed is no part of a run.

 @param text - the English

 @param closing - the offsets of the closing delimiters of emphasis, strong emphasis and strikethrough by the parse

 @returns For each offset and one past the end, the offset where the run that starts there ends (the offset itself when no run
 starts there)

 @example
 ```ts
 const ends = runEndsOf({ text: 'the *cats*’ bowl', closing: new Set([9,],), },); // ends[9] is 10
 ```
 */
function runEndsOf(
  {
    text,
    closing,
  }: {
    readonly text: string;
    readonly closing: ReadonlySet<number>;
  },
): Int32Array {
  /**
   The end of the run from each offset, written once for each offset from the end of the text back.
   */
  const ends = new Int32Array(text.length + 1,);
  ends[text.length] = text.length;
  /**
   The nearest end of a link destination after the offset the reading has reached, mutated only inside this function.
   */
  const stop: DestinationStop = {
    at: text.length,
    isBracket: false,
  };
  for (
    let at = text.length - 1;
    at >= 0;
    at -= 1
  ) {
    /**
     The unit at the offset.
     */
    const unit = text.charAt(at,);
    /**
     The unit after it.
     */
    const next = text.charAt(at + 1,);
    /**
     Whether the unit opens a link's destination that a closing round bracket ends.
     */
    const opensClosedDestination = (unit === ']')
      && (next === '(')
      && stop.isBracket;
    if (closing.has(at,))
      ends[at] = ends[at + 1] ?? at;
    else if (opensClosedDestination)
      ends[at] = ends[stop.at + 1] ?? at;
    else
      ends[at] = at;
    if ((unit === ')') || MARK_PARTING_SPACE.has(unit,)) {
      stop.at = at;
      stop.isBracket = unit === ')';
    }
  }
  return ends;
}

/**
 The apostrophe-like marks of a text that end a reading run after a Latin letter or a digit (`*kittens*’`, `[the cat](x)’`),
 which read as the trailing edge of the word before the run. One pass reads the text from its end to its start, a constant
 number of reads for each unit, whatever the length of a run or of a destination.

 @param text - the English as it was written, where the markup is not blanked yet

 @param closing - the offsets of the closing delimiters of emphasis, strong emphasis and strikethrough by the parse; a `*`, `_`
 or `~` at no such offset is a character of the text and no part of a run

 @returns The offsets of the marks that end a reading run

 @example
 ```ts
 const marks = closingRunMarksOf({ text: 'The *kittens*’ bowl', closing: new Set([12,],), },); // holds 13
 ```
 */
export function closingRunMarksOf(
  {
    text,
    closing,
  }: {
    readonly text: string;
    readonly closing: ReadonlySet<number>;
  },
): ReadonlySet<number> {
  /**
   The offsets of the marks that end a run, mutated only inside this function.
   */
  const marks = new Set<number>();
  /**
   Where the run that starts at each offset ends.
   */
  const ends = runEndsOf({
    text,
    closing,
  },);
  for (let at = 1; at < text.length; at += 1) {
    /**
     Where the run that starts here ends, the offset itself when none starts here.
     */
    const end = ends[at] ?? at;
    // The unit before a run is a letter or a digit, which is none of the run's characters, so the run starts here.
    if ((end > at) && WRITTEN_APOSTROPHES.has(text.charAt(end,),)
      && continuesLatinWordOrCompatibilityForm({
        character: codePointBefore({
          text,
          at,
        },),
      },))
      marks.add(end,);
  }
  return marks;
}

/**
 Where the destination of a link ends, when it does: the offset after the closing round bracket of a `(` at an offset, read
 only for a destination with no white space in it.

 @param text - the English

 @param open - offset of the `(`

 @returns The offset after the `)` as a list of one, none for a destination that holds white space or is never closed

 @example
 ```ts
 const [after,] = destinationEndOf({ text: '[a](b)’', open: 3, },); // 6
 ```
 */
export function destinationEndOf(
  {
    text,
    open,
  }: {
    readonly text: string;
    readonly open: number;
  },
): readonly number[] {
  for (let at = open + 1; at < text.length; at += 1) {
    /**
     The unit at the offset.
     */
    const unit = text.charAt(at,);
    if (unit === ')')
      return [at + 1,];
    if (MARK_PARTING_SPACE.has(unit,))
      return [];
  }
  return [];
}

//endregion Consistency markup runs
