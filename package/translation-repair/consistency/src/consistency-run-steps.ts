import {
  codePointAt,
  codePointBefore,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { ASCII_PUNCTUATION, } from './consistency-edge-units.ts';
import { HORIZONTAL_SPACE, } from './consistency-markup-runs.ts';
import {
  insideParagraphAt,
  quoteDepthAt,
} from './consistency-paragraphs.ts';
import { isDefaultIgnorable, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Consistency run steps
// THE STEPS OF THE REFUSAL RUN THAT ONE CHARACTER CANNOT SAY. The refusal run beside a span (`markPastRunOf`,
// `consistency-closing-marks.ts`) steps over every unit a reader sees no word in, and some of those units are known by what
// stands beside them: a backslash escape, a line break, the markers at the start of the line that follows a break, and a
// character that renders nothing. A BACKSLASH ESCAPE is a backslash and the one ASCII punctuation character it escapes (`\*`,
// `\_`, `\~`, `\]`, `\)`, `\\`, `\'`): the run reads it as the escaped character is read, and the parse reads the character as
// text and the backslash as no part of the page. A backslash before a line feed or a carriage return is a hard break. A LINE
// BREAK is a line feed, a carriage return, both together, the next-line control, a line separator or a paragraph separator; one
// inside a paragraph or a setext heading renders as a space, or as a break, between the two marks, and a blank line ends the
// block, so a mark on the far side of it is another block's, and so is a mark on the line of a list item, a heading or a
// quotation that follows the paragraph without a blank line. WHETHER A BREAK IS INSIDE A BLOCK IS THE PARSE'S TO SAY, never this
// file's: a line of nothing but an en space, a line separator, a next-line control or any one invisible or non-ASCII space
// character (an ideographic space, a zero-width space, a no-break space) is no blank line to CommonMark, which the site
// renders, and the run reads no character set of its own for it (`consistency-paragraphs.ts` says where the blocks are, from
// the parse of the prose read as a renderer reads it). THE MARKERS OF A QUOTATION at the start of the line after such
// a break (`>` with its space or without, as many as quotations enclose the block) render nothing, so the run steps over them
// and over the indentation of a list item or of a lazy line; a `>` anywhere else, or past more markers than the block has
// quotations, is text a reader sees. A CHARACTER THAT SHOWS A BLANK GAP OR NOTHING is a default-ignorable code point (a
// zero-width or format character, the soft hyphen, the combining grapheme joiner, a variation selector), a control (U+0001 to
// U+0008, U+000E to U+001F, U+007F, U+0080 to U+0084, U+0086 to U+009F) or the Braille blank U+2800, read by whole code point.
// A browser draws a C0 control as a blank gap wider than a space (15.55 px at 20 px serif in headless Chromium, measured in the
// n155 review), and U+007F and the C1 controls with no width; the default-ignorable code points show nothing, but for the Hangul
// fillers, which are drawn as a blank cell. Either way no letter stands
// between the two marks, so a gain carried beside the run's mark would print two marks with a gap or nothing between them,
// and stepping over them is the safe direction: it only refuses a gain and never carries one. A renderer that draws a control
// as a visible box shows a character there, and the run then refuses a gain it could have carried, which keeps the archive's
// mark. U+0000 is none of them: a page shows it as the replacement character U+FFFD, a character a reader sees, so it ends the
// run.

/**
 The characters that are a line break, or half of one when a carriage return and a line feed stand together.
 */
const LINE_BREAK_UNITS: ReadonlySet<string> = new Set([
  '\n',
  '\r',
],);

/**
 The characters the run reads as a line break: the two of a line feed and a carriage return, the next-line control, the line
 separator and the paragraph separator. A backslash before the last three is no hard break.
 */
const BREAK_UNITS: ReadonlySet<string> = new Set([
  ...LINE_BREAK_UNITS,
  '\u{0085}',
  '\u{2028}',
  '\u{2029}',
],);

/**
 The character that marks a quotation at the start of a line.
 */
const QUOTATION_MARKER = '>';

/**
 Whether the unit at an offset is a backslash that escapes the character after it, or ends a line as a hard break: a backslash
 directly before an ASCII punctuation character or a line break.

 @param text - the English

 @param index - offset of the unit asked about

 @returns Whether the unit is the backslash of an escape or of a hard break

 @example
 ```ts
 const escapes = startsEscape({ text: 'kittens\\*’', index: 7, },); // true
 ```
 */
export function startsEscape(
  {
    text,
    index,
  }: {
    readonly text: string;
    readonly index: number;
  },
): boolean {
  if (text.charAt(index,) !== '\\')
    return false;
  /**
   The unit the backslash is directly before.
   */
  const escaped = text.charAt(index + 1,);
  return ASCII_PUNCTUATION.has(escaped,) || LINE_BREAK_UNITS.has(escaped,);
}

/**
 Whether the unit at an offset is part of a single line break inside a paragraph or a setext heading: a line feed, a carriage
 return, both together, the next-line control, a line separator or a paragraph separator, inside a block of running text of the
 English as the pass's own parse reads it (`insideParagraphAt`). Whether a line next to the break is blank is the parse's to say: a
 blank line ends the block, so the breaks around it are outside every block, and a line of an en space or of a line separator,
 which the parse does not call blank, leaves the breaks around it inside.

 @param text - the English

 @param index - offset of the unit asked about

 @param definitions - the page's definitions, which the parse of the English reads references against; empty for English read alone

 @returns Whether the unit is a line break and a paragraph or the text of a heading holds it; for an English the grammar refuses,
 whether the unit is a line break

 @example
 ```ts
 const single = isSingleBreakAt({ text: 'kittens\n’ bowl', index: 7, },); // true
 const blank = isSingleBreakAt({ text: 'kittens\n\n’ bowl', index: 7, },); // false
 ```
 */
export function isSingleBreakAt(
  {
    text,
    index,
    definitions = '',
  }: {
    readonly text: string;
    readonly index: number;
    readonly definitions?: string;
  },
): boolean {
  return BREAK_UNITS.has(text.charAt(index,),)
    && insideParagraphAt({
      english: text,
      index,
      definitions,
    },);
}

/**
 How many quotation markers the line after a single line break may start with, which is how many quotations enclose the block
 that holds the break.

 @param text - the English

 @param index - offset of the unit asked about

 @param definitions - the page's definitions, which the parse of the English reads references against; empty for English read alone

 @returns The number as a list of one, none when the unit is no single line break; `Number.POSITIVE_INFINITY` for an English the
 grammar refuses

 @example
 ```ts
 const [markers,] = markersAfterBreakOf({ text: '> kittens\n> ’ bowl', index: 9, },); // 1
 ```
 */
export function markersAfterBreakOf(
  {
    text,
    index,
    definitions = '',
  }: {
    readonly text: string;
    readonly index: number;
    readonly definitions?: string;
  },
): readonly number[] {
  return isSingleBreakAt({
    text,
    index,
    definitions,
  },)
    ? [
      quoteDepthAt({
        english: text,
        index,
        definitions,
      },),
    ]
    : [];
}

/**
 Where the single line break stands that a run of quotation markers at the start of a line follows, read backward from the first
 marker found: the markers and the horizontal white space among them, back to a single line break inside a block that has at
 least as many quotations around it as the run has markers.

 @param text - the English

 @param index - offset of the unit asked about, which is the last marker of the run in reading order

 @param definitions - the page's definitions, which the parse of the English reads references against; empty for English read alone

 @returns The offset of the last unit of the line break as a list of one, none when the unit is no `>`, the markers do not start a
 line, the line break is not a single one inside a block, or the block has fewer quotations than the run has markers

 @example
 ```ts
 const [at,] = lineBreakBeforeMarkersAt({ text: '> kittens\n> ’ bowl', index: 10, definitions: '', },); // 9
 ```
 */
function lineBreakBeforeMarkersAt(
  {
    text,
    index,
    definitions,
  }: {
    readonly text: string;
    readonly index: number;
    readonly definitions: string;
  },
): readonly number[] {
  if (text.charAt(index,) !== QUOTATION_MARKER)
    return [];
  /**
   Where the reading is, and how many markers it has met.
   */
  const reading = {
    at: index,
    markers: 0,
  };
  while (
    (reading.at >= 0)
      && ((text.charAt(reading.at,) === QUOTATION_MARKER) || HORIZONTAL_SPACE.has(text.charAt(reading.at,),))
  ) {
    if (text.charAt(reading.at,) === QUOTATION_MARKER)
      reading.markers += 1;
    reading.at -= 1;
  }
  if (!isSingleBreakAt({
    text,
    index: reading.at,
    definitions,
  },))
    return [];
  return (reading.markers <= quoteDepthAt({
    english: text,
    index: reading.at,
    definitions,
  },))
    ? [reading.at,]
    : [];
}

/**
 The first control of the first range that shows a reader a blank gap, the start of heading, U+0001.
 */
const START_OF_HEADING = 0x01;

/**
 The last control of the first range, the backspace, U+0008.
 */
const BACKSPACE = 0x08;

/**
 The first control of the second range, the shift out, U+000E.
 */
const SHIFT_OUT = 0x0E;

/**
 The last control of the second range, the unit separator, U+001F.
 */
const UNIT_SEPARATOR = 0x1F;

/**
 The delete control, U+007F, which shows nothing.
 */
const DELETE = 0x7F;

/**
 The first C1 control, the padding character, U+0080, which shows nothing.
 */
const PADDING_CHARACTER = 0x80;

/**
 The last C1 control before the next-line control, the index, U+0084.
 */
const INDEX = 0x84;

/**
 The first C1 control after the next-line control, which is a line break, the single shift two, U+0086.
 */
const SINGLE_SHIFT_TWO = 0x86;

/**
 The last C1 control, the application program command, U+009F.
 */
const APPLICATION_PROGRAM_COMMAND = 0x9F;

/**
 The ranges of code points of the controls that show a reader a blank gap or nothing and are neither white space nor a line
 break: U+0001 to U+0008, U+000E to U+001F, U+007F, U+0080 to U+0084 and U+0086 to U+009F (the next-line control U+0085 is a line
 break). U+0000 is none of them, since a page shows it as the replacement character U+FFFD.
 */
const CONTROLS_THAT_SHOW_NOTHING: readonly (readonly [
  number,
  number,
])[] = [
  [
    START_OF_HEADING,
    BACKSPACE,
  ],
  [
    SHIFT_OUT,
    UNIT_SEPARATOR,
  ],
  [
    DELETE,
    DELETE,
  ],
  [
    PADDING_CHARACTER,
    INDEX,
  ],
  [
    SINGLE_SHIFT_TWO,
    APPLICATION_PROGRAM_COMMAND,
  ],
];

/**
 The Braille pattern blank, which a page shows as an empty cell, a blank that is white space to no Unicode property.
 */
const BRAILLE_BLANK = 0x28_00;

/**
 Whether a character shows a reader a blank gap or nothing without being white space or a line break: a default-ignorable code
 point, one of the controls of `CONTROLS_THAT_SHOW_NOTHING` (C0 and C1), or the Braille blank.

 @param character - one whole code point

 @returns Whether the run steps over it

 @example
 ```ts
 const blank = showsNothing({ character: '\u{0001}', },); // true
 ```
 */
function showsNothing({ character, }: { readonly character: string; },): boolean {
  /**
   The code point, none for an empty character.
   */
  const code = character.codePointAt(0,);
  if (code === undefined)
    return false;
  return isDefaultIgnorable({ character, },)
    || (code === BRAILLE_BLANK)
    || CONTROLS_THAT_SHOW_NOTHING.some(function holds([first, last,],): boolean {
      return (code >= first) && (code <= last);
    },);
}

/**
 How many UTF-16 units the code point at an offset takes when it shows a reader a blank gap or nothing (a default-ignorable code
 point, a C0 or C1 control that is no white space or line break, the Braille blank): the one that starts at the offset when the run is read forward, the one that ends with
 the unit there when it is read backward.

 @param text - the English

 @param index - offset of the unit asked about

 @param side - `after` to read the code point that starts at the offset, `before` the one that ends with the unit there

 @returns The number of units, zero when the code point shows something or the offset is past an edge of the text

 @example
 ```ts
 const width = ignorableWidthAt({ text: 'kittens\u{200B}’', index: 7, side: 'after', },); // 1
 ```
 */
export function ignorableWidthAt(
  {
    text,
    index,
    side,
  }: {
    readonly text: string;
    readonly index: number;
    readonly side: 'after' | 'before';
  },
): number {
  /**
   The whole code point asked about, empty past an edge of the text.
   */
  const character = (side === 'after')
    ? codePointAt({
      text,
      at: index,
    },)
    : codePointBefore({
      text,
      at: index + 1,
    },);
  return showsNothing({ character, },) ? character.length : 0;
}

/**
 Where the run goes on at the marker of a container, and how many markers it may still step over: a quotation marker at the
 start of a line that the break before it allowed, read forward, or the line break that a run of markers follows, read backward.

 @param text - the English

 @param index - offset of the unit asked about

 @param side - `after` to read forward, `before` to read backward

 @param markers - how many quotation markers the line the reading is on may still start with, which only a forward reading uses

 @param definitions - the page's definitions, which the parse of the English reads references against; empty for English read alone

 @returns The offset the reading goes on at and the markers still allowed there, as a list of one, none when the unit is no
 marker of a container for that reading

 @example
 ```ts
 const [step,] = containerStepAt({ text: '> kittens\n> ’ bowl', index: 10, side: 'after', markers: 1, },); // { at: 11, markers: 0, }
 ```
 */
export function containerStepAt(
  {
    text,
    index,
    side,
    markers,
    definitions = '',
  }: {
    readonly text: string;
    readonly index: number;
    readonly side: 'after' | 'before';
    readonly markers: number;
    readonly definitions?: string;
  },
): readonly {
  readonly at: number;
  readonly markers: number;
}[] {
  if (side === 'after') {
    return ((markers > 0) && (text.charAt(index,) === QUOTATION_MARKER))
      ? [
        {
          at: index + 1,
          markers: markers - 1,
        },
      ]
      : [];
  }
  /**
   Where the line break stands that the markers at the unit follow, none when they follow none.
   */
  const [breakAt,] = lineBreakBeforeMarkersAt({
    text,
    index,
    definitions,
  },);
  return (breakAt === undefined)
    ? []
    : [
      {
        at: breakAt,
        markers: 0,
      },
    ];
}

//endregion Consistency run steps
