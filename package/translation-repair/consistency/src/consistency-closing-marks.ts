import {
  codePointAt,
  continuesLatinWordOrCompatibilityForm,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { isCornerBracket, } from './consistency-corner-brackets.ts';
import { singleMarkAt, } from './consistency-loose-marks.ts';
import {
  CLOSING_BRACKETS,
  CLOSING_CURLY_DOUBLE,
  destinationEndOf,
  type DestinationStop,
  EMPHASIS_CHARACTERS,
  HORIZONTAL_SPACE,
  LEADING_REFUSAL_RUN,
  MARK_PARTING_SPACE,
  TRAILING_REFUSAL_RUN,
} from './consistency-markup-runs.ts';
import {
  containerStepAt,
  ignorableWidthAt,
  isSingleBreakAt,
  markersAfterBreakOf,
  startsEscape,
} from './consistency-run-steps.ts';
import { unreadUnitsOf, } from './consistency-unread-units.ts';

//region Consistency closing marks
// THE FORWARD READING OF `consistency-word-joints.ts`: from the end of a stand, over the
// marks that close a quotation, a bracketed stretch, a link or a stretch of emphasis, to an
// apostrophe that a letter or a digit follows. The rule and its examples are in the header of
// that file; this file holds the marks the reading steps over and the reading itself. A STAND is
// one place where the words of a quote lie in an English (`consistency-fold.ts`, `SpanSearch`),
// and a JOINT is a mark that makes one word of the letters on its two sides, here across markup
// (`consistency-word-joints.ts` defines the joints that cut a word).
//
// WHAT CLOSES, AND WHO SAYS SO. The reading steps over a closing round or square bracket, a
// closing curly double quotation mark (always), a straight double quotation mark and a
// delimiter of emphasis. A straight double quotation mark is read by its SHAPE: it closes when a
// character that is neither white space nor an opening mark (an opening round, square or curly
// bracket, an opening curly or corner quotation mark, a guillemet) stands before it, since a
// straight mark cannot say whether it opens or closes. A delimiter of emphasis is read by the
// PARSE, never by its shape: `*`, `_` and `~` close only where they are the closing delimiter of
// an emphasis, strong or strikethrough node of the Markdown grammar, the delimiters the prose
// view blanks (`consistency-prose-text.ts`); a lone asterisk, a stray underscore and an
// underscore inside a word are characters of the text and close nothing. The caller hands the
// closing delimiters (`closing`), at the offsets of the folded English.
//
// "A LETTER OR A DIGIT" AFTER THE APOSTROPHE is the neighbour test of the consistency pass
// (`continuesLatinWordOrCompatibilityForm`, `latin-letters.ts`), which also reads a mathematical or
// fullwidth letter as its Latin form; the rest of the package tests a neighbour with `continuesLatinWord`.
//
// A MARK BEYOND A RUN OF MARKUP is read by `markPastRunOf`, the mark that stands past the markup or
// closing characters directly beside a span: the REFUSAL RUN of the counter, which stays wide (decision D1 and the ruling on
// the J1 shapes). Every `*`, `_` and `~` is a character of that run, whether or not the parse makes it a delimiter: D2
// governs how a mark READS (the reading run, `marksPastMarkupOf`), not when a gain is REFUSED. The run also steps over every
// unit no reader reads as a word (`unreadUnitsOf`: a tag, a footnote label, a link's destination with its title, a comment, a
// character reference), over horizontal white space, every Default_Ignorable_Code_Point, the controls that show nothing and the
// Braille blank (`ignorableWidthAt`, `consistency-run-steps.ts`), and over one line break inside a
// paragraph or a setext heading (a line feed, a carriage return, the next-line control, the line separator or the paragraph
// separator) with the markers of the containers after it (a blockquote `>` and its optional space, one per level of nesting,
// and the indentation of a list item or a lazy continuation line), and a single mark at its far end is the mark past it. A
// blank line, a `>` that is text, and a `>` beyond the blockquote depth of the paragraph end the run; an English the grammar
// refuses has no blocks to tell, so there every line break is stepped, a blank line included.
//
// THE READING RUN IS ANOTHER, NARROWER RUN, read for the proof by `marksPastMarkupOf`
// (`consistency-marks-past-markup.ts`), which names the marks that end a run of closing delimiters of emphasis by the
// parse, and a link's closing `](…)`, after a letter, on the trailing side only (D4). The sets of characters the two runs
// are made of (`TRAILING_REFUSAL_RUN`, `LEADING_REFUSAL_RUN`, `EMPHASIS_CHARACTERS`) and `closingRunMarksOf` are in
// `consistency-markup-runs.ts`, so the readings that step over a run cannot drift.

/**
 Marks that join the letter or digit before them to the letter or digit after
 them: the straight apostrophe every curly single mark folds to, and the modifier
 letter apostrophe.
 */
export const APOSTROPHE_JOINERS: ReadonlySet<string> = new Set([
  '\'',
  '\u{02BC}',
],);

/**
 The characters before a straight double quotation mark that make it open a stretch rather than close one, as the English
 wrote them (before the fold, which writes a curly opening mark as the straight one): an opening round, square or curly
 bracket, the opening curly double and single quotation marks, the opening corner brackets and the opening guillemet.
 */
const OPENING_MARKS: ReadonlySet<string> = new Set([
  '(',
  '[',
  '{',
  '“',
  '‘',
  '「',
  '『',
  '«',
],);

/**
 The characters the English held before the fold at a double quotation mark that closes a
 quotation: the straight one, which closes when a character that is neither white space nor an
 opening mark stands before it, and the closing curly one. The opening curly one and the corner brackets fold to
 the same straight mark and are left out.
 */
const CLOSING_DOUBLE_ORIGINALS: ReadonlySet<string> = new Set([
  '"',
  CLOSING_CURLY_DOUBLE,
],);

/**
 Whether a unit closes a stretch: a closing round or square bracket, a delimiter of emphasis that the parse makes a closing
 one, or a double quotation mark that closes (the English held a straight or closing curly mark there, and a character that
 is neither white space nor an opening mark stands before it).

 @param mark - the unit asked about, empty past an edge of the text

 @param previous - the unit before it, empty at the start of the text

 @param original - the character the English held before the fold at the unit

 @param previousOriginal - the character the English held before the fold at the unit before it

 @param isClosingEmphasis - whether the parse makes the unit a closing delimiter of emphasis, strong emphasis or strikethrough

 @returns Whether the unit closes a stretch

 @example
 ```ts
 const closes = closesAfter({ mark: '"', previous: 't', original: '"', previousOriginal: 't', isClosingEmphasis: false, },); // true
 ```
 */
function closesAfter(
  {
    mark,
    previous,
    original,
    previousOriginal,
    isClosingEmphasis,
  }: {
    readonly mark: string;
    readonly previous: string;
    readonly original: string;
    readonly previousOriginal: string;
    readonly isClosingEmphasis: boolean;
  },
): boolean {
  if (CLOSING_BRACKETS.has(mark,))
    return true;
  if (EMPHASIS_CHARACTERS.has(mark,))
    return isClosingEmphasis;
  if (!((mark === '"') && CLOSING_DOUBLE_ORIGINALS.has(original,)))
    return false;
  return !(MARK_PARTING_SPACE.has(previous,) || OPENING_MARKS.has(previousOriginal,));
}

/**
 Whether a unit is an apostrophe that a letter or a digit follows: the straight apostrophe or the modifier letter
 apostrophe, never a corner bracket the fold wrote as one.

 @param folded - the English in the fold

 @param originals - the character the English held before the fold at each UTF-16 unit of it

 @param at - offset of the unit asked about

 @param mark - the unit there, which the caller has read already

 @returns Whether the unit is such an apostrophe

 @example
 ```ts
 const text = 'the cat\'s';
 const joins = apostropheBeforeLetter({ folded: text, originals: Array.from(text,), at: 7, mark: '\'', },); // true
 ```
 */
function apostropheBeforeLetter(
  {
    folded,
    originals,
    at,
    mark,
  }: {
    readonly folded: string;
    readonly originals: readonly string[];
    readonly at: number;
    readonly mark: string;
  },
): boolean {
  if (!APOSTROPHE_JOINERS.has(mark,))
    return false;
  if (isCornerBracket({ original: originals[at] ?? '', },))
    return false;
  /**
   The whole character after the apostrophe.
   */
  const after = codePointAt({
    text: folded,
    at: at + 1,
  },);
  return continuesLatinWordOrCompatibilityForm({ character: after, },);
}

/**
 The offsets of a folded English at which a run of closing marks ends in an apostrophe that a letter or a digit
 follows, so that a stand ending at one of them lies inside a word made with the apostrophe (`"cat"'s`,
 `(grey cat)'s`, `[the cat](x)'s`). The offsets are those where the English continues with a run of closing marks
 (emphasis `*` `_` `~` and a straight `"` when a character that is no white space stands before it, a closing curly
 `”`, a closing `)` or `]`, and the destination `(...)` of a link right after a `]`) and then the apostrophe, whatever
 the length of the run or of the destination; and the offsets where the apostrophe itself stands when the unit before
 it closes a stretch, so that a stand which ends in a closing mark is inside the word too.

 One pass reads the text from its end to its start, a constant number of reads for each unit. A link destination that
 has white space in it (a title), or is never closed, is not read, so the `]` before it starts no run that ends in an
 apostrophe.

 @param folded - the English in the fold

 @param originals - the character the English held before the fold at each UTF-16 unit of it

 @param closing - the offsets in the fold of the units that are the closing delimiter of an emphasis, strong or strikethrough
 stretch by the parse (`consistency-prose-text.ts`); an emphasis character at no such offset closes nothing

 @returns The offsets at which a stand may end to lie inside such a word

 @example
 ```ts
 const text = 'Fill the "cat"\'s bowl.';
 const inside = closingJointsInText({ folded: text, originals: Array.from(text,), closing: new Set(), },).has(14,); // true
 ```
 */
export function closingJointsInText(
  {
    folded,
    originals,
    closing,
  }: {
    readonly folded: string;
    readonly originals: readonly string[];
    readonly closing: ReadonlySet<number>;
  },
): ReadonlySet<number> {
  /**
   The offsets recorded, mutated only inside this function.
   */
  const joints = new Set<number>();
  /**
   Whether, from each offset, a run of closing marks (possibly none) ends in an apostrophe that a letter or a digit
   follows; one entry past the end of the text and one more for the offset after a link destination that ends the text.
   Mutated only inside this function.
   */
  const reaches = new Uint8Array(folded.length + 2,);
  /**
   The nearest end of a link destination at or after the offset the reading has reached, mutated only inside this
   function.
   */
  const stop: DestinationStop = {
    at: folded.length,
    isBracket: false,
  };
  /**
   The unit at the offset read next and the unit after it, both empty past the end of the text, mutated only inside this
   function.
   */
  const reading = {
    mark: '',
    next: '',
  };
  for (
    let at = folded.length;
    at >= 0;
    at -= 1
  ) {
    /**
     The unit before the one read, empty at the start of the text.
     */
    const previous = folded.charAt(at - 1,);
    /**
     Whether the unit is an apostrophe a letter or a digit follows.
     */
    const isApostrophe = apostropheBeforeLetter({
      folded,
      originals,
      at,
      mark: reading.mark,
    },);
    /**
     Whether the unit closes a stretch.
     */
    const closes = (!isApostrophe) && closesAfter({
      mark: reading.mark,
      previous,
      original: originals[at] ?? '',
      previousOriginal: originals[at - 1] ?? '',
      isClosingEmphasis: closing.has(at,),
    },);
    if (isApostrophe) {
      reaches[at] = 1;
      // THE STAND'S OWN LAST CHARACTER MUST CLOSE when the apostrophe stands right after it,
      // since nothing was stepped over (`"hi"'s` is inside a word, `hi'` + `s` is not).
      if (closesAfter({
        mark: previous,
        previous: folded.charAt(at - 2,),
        original: originals[at - 1] ?? '',
        previousOriginal: originals[at - 2] ?? '',
        isClosingEmphasis: closing.has(at - 1,),
      },))
        joints.add(at,);
    }
    else if (closes) {
      /**
       Offset where the run goes on: the next unit, or past the destination of a link.
       */
      const onward = ((reading.mark === ']') && (reading.next === '('))
        ? (stop.isBracket ? (stop.at + 1) : (-1))
        : (at + 1);
      reaches[at] = ((onward !== (-1)) && (reaches[onward] === 1)) ? 1 : 0;
      if (reaches[at] === 1)
        joints.add(at,);
    }
    if ((reading.mark === ')') || MARK_PARTING_SPACE.has(reading.mark,)) {
      stop.at = at;
      stop.isBracket = reading.mark === ')';
    }
    reading.next = reading.mark;
    reading.mark = previous;
  }
  return joints;
}

/**
 Whether the unit at an offset is a unit of the refusal run beside a span, as one unit can say: one of the run's characters, any
 `*`, `_` or `~`, horizontal white space, a unit that no reader reads as a word, the backslash of an escape or of a hard break, or
 a single line break inside a paragraph or a setext heading. What stands beside a unit decides the markers of a quotation at the
 start of a line and a character that renders nothing, which `markPastRunOf` reads itself. The run is wide on purpose (the ruling
 on the J1 shapes, the ruling on the footnote labels, tags and spaces, and the ruling on escapes, spaces and line breaks): a
 literal emphasis character that the parse makes no delimiter, a footnote label, a tag, a space, an escape or a soft break still
 stands between the span and a kept mark, and an apostrophe that arrives beside that mark would stand as a doubled mark for the
 eye (`'*'5 cats`, `dogs’[^1] ’`, `dogs’\*’`). How a mark READS is the reading run's business and asks the parse
 (`closingRunMarksOf`, `consistency-markup-runs.ts`); when a gain is REFUSED is this run's.

 @param text - the English

 @param index - offset of the unit asked about

 @param characters - the characters the run is made of on the side read

 @param unread - the offsets of the units no reader reads as words (`unreadUnitsOf`)

 @param definitions - the page's definitions, which the parse of the English reads references against; empty for English read alone

 @returns Whether the unit is part of the run

 @example
 ```ts
 const inRun = isRunUnit({ text: 'the *kittens*’', index: 12, characters: TRAILING_REFUSAL_RUN, unread: new Set(), definitions: '', },); // true
 ```
 */
function isRunUnit(
  {
    text,
    index,
    characters,
    unread,
    definitions,
  }: {
    readonly text: string;
    readonly index: number;
    readonly characters: ReadonlySet<string>;
    readonly unread: ReadonlySet<number>;
    readonly definitions: string;
  },
): boolean {
  /**
   The unit at the offset.
   */
  const unit = text.charAt(index,);
  return characters.has(unit,)
    || EMPHASIS_CHARACTERS.has(unit,)
    || HORIZONTAL_SPACE.has(unit,)
    || unread.has(index,)
    || startsEscape({
      text,
      index,
    },)
    || isSingleBreakAt({
      text,
      index,
      definitions,
    },);
}

/**
 The single mark that stands past a run of markup beside a span, which the mark directly beside the span does not name. The run
 is made of the characters of the side (on the trailing side emphasis, closing brackets, double quotation marks and the
 destination of a link; on the leading side emphasis, opening brackets and opening double quotation marks), of every unit no
 reader reads as a word (a tag, a footnote or reference label, a link's destination with its title, a comment, a character
 reference), of horizontal white space of every width the proof reads as white space, of a backslash escape (a backslash and the
 ASCII punctuation character it escapes, read as that character is, so an escaped single mark is the mark that ends the run),
 of a single line break inside a paragraph or a setext heading (a soft break, two spaces and a break, a backslash and a break,
 the next-line control, a line separator, a paragraph separator), of the quotation markers at the start of the line after such a
 break (`>` with its space or without, no more than the quotations around the block), and of every default-ignorable code
 point (a zero-width or format character, the soft hyphen, a variation selector), of the controls that show nothing (U+0001 to
 U+0008, U+000E to U+001F, U+007F) and of the Braille blank U+2800; a letter, a digit, a blank line, a `>` that is text, the NUL
 character U+0000 or any other character ends it. Every `*`, `_` and `~` belongs to the run, whether or not the parse makes it
 a delimiter (the ruling on the J1 shapes: the run is the one a gain is refused beside, not the one a mark is read past). The
 characters are read by shape, since a mark that stands beside them is a second apostrophe for the eye (`*dogs’*’`,
 `'*'5 cats`, `dogs’[^1] ’`, `dogs’\*’`).

 @param text - the English

 @param at - on the trailing side the offset after the span, on the leading side the offset where the span starts

 @param side - `after` to read forward from the span's end, `before` to read backward from its start

 @param definitions - the page's definitions, which the parse of the English reads references against; empty for English read alone

 @returns The mark as the English writes it, as a list of one; none when the run is empty (the mark beside the span is then
 the caller's `singleMarkAt`), when a character that is no part of a run stands first, or when no single mark ends the run

 @example
 ```ts
 const [mark,] = markPastRunOf({ text: 'the *kittens*’ bowl', at: 12, side: 'after', },); // '’'
 ```
 */
export function markPastRunOf(
  {
    text,
    at,
    side,
    definitions = '',
  }: {
    readonly text: string;
    readonly at: number;
    readonly side: 'after' | 'before';
    readonly definitions?: string;
  },
): readonly string[] {
  /**
   The characters the run is made of on this side besides the delimiters of emphasis, the white space and the unread units.
   */
  const characters = (side === 'after') ? TRAILING_REFUSAL_RUN : LEADING_REFUSAL_RUN;
  /**
   The step from one unit to the next of the run: forward after the span, backward before it.
   */
  const step = (side === 'after') ? 1 : (-1);
  /**
   The offsets of the units no reader reads as words, which a run steps over.
   */
  const unread = unreadUnitsOf({
    english: text,
    definitions,
  },);
  /**
   Where the reading is: the offset of the unit asked about next, whether any unit of the run was stepped over, and, reading
   forward, how many quotation markers the line the reading is at the start of may still start with (none anywhere else).
   */
  const reading = {
    at: (side === 'after') ? at : (at - 1),
    steppedOver: false,
    markers: 0,
  };
  while ((reading.at >= 0) && (reading.at < text.length)) {
    /**
     How many units the character that renders nothing at the reading takes, none where the character renders.
     */
    const ignorable = ignorableWidthAt({
      text,
      index: reading.at,
      side,
    },);
    if (ignorable > 0) {
      reading.at += step * ignorable;
      reading.steppedOver = true;
      reading.markers = 0;
      continue;
    }
    /**
     Whether the unit closes a link's text and a `(` follows, so a destination may start there.
     */
    const opensDestination = (text.charAt(reading.at,) === ']')
      && (text.charAt(reading.at + 1,) === '(');
    if ((side === 'after') && opensDestination) {
      /**
       Where the link's destination ends, none when it is not read (white space in it, or never closed), when the run goes on
       with the bracket alone.
       */
      const [end,] = destinationEndOf({
        text,
        open: reading.at + 1,
      },);
      reading.at = end ?? (reading.at + 1);
      reading.steppedOver = true;
      reading.markers = 0;
      continue;
    }
    // An escaped backslash reads forward as one character, so the backslash it escapes is no escape of what follows it.
    if ((side === 'after') && text.startsWith(
      String.raw`\\`,
      reading.at,
    )) {
      reading.at += 2;
      reading.steppedOver = true;
      reading.markers = 0;
      continue;
    }
    /**
     Where the markers of the container go on the run, none where the unit is no marker for this reading: a quotation marker at
     the start of a line, as many as the quotations around the paragraph that the break before the line left, read forward;
     read backward, the markers and the white space among them are known only from the line break before them, so the run
     jumps to that break when it is a single one inside a paragraph with as many quotations around it.
     */
    const [container,] = containerStepAt({
      text,
      index: reading.at,
      side,
      markers: reading.markers,
      definitions,
    },);
    if (container !== undefined) {
      reading.at = container.at;
      reading.steppedOver = true;
      reading.markers = container.markers;
      continue;
    }
    if (!isRunUnit({
      text,
      index: reading.at,
      characters,
      unread,
      definitions,
    },))
      break;
    if (side === 'after') {
      /**
       How many markers the line after the unit may start with, none (an empty list) when the unit is no single line break.
       */
      const [afterBreak,] = markersAfterBreakOf({
        text,
        index: reading.at,
        definitions,
      },);
      reading.markers = afterBreak ?? (HORIZONTAL_SPACE.has(text.charAt(reading.at,),) ? reading.markers : 0);
    }
    reading.at += step;
    reading.steppedOver = true;
  }
  return reading.steppedOver
    ? singleMarkAt({
      text,
      at: reading.at,
    },)
    : [];
}

//endregion Consistency closing marks
