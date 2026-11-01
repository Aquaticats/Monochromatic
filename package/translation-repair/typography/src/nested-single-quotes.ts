import { codePointAt, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  bestPairsOf,
  distinctDepths,
  EMPTY_LINE,
  type LineState,
  type PairedMarks,
  withMark,
  withQuotedNumber,
} from './nested-single-line-readings.ts';
import {
  bindsWord,
  isElisionPair,
  isElisionPairEnd,
  isLeadingElision,
  sequenceBefore,
} from './quote-neighbours.ts';

//region Nested single quotes
// CLASS ONE HUNDRED FORTY-SEVEN (XingZ6013, 2026-09-26). The butterfly speech
// quotes a law inside its own quotation and the page shipped the inner pair
// straight ('to be awake is to keep living.') inside curly doubles on a page
// every other mark of which is curly. The restoration read a straight single
// quote only as an apostrophe. The corpus writes a nested quotation ‘…’ on
// every page that has one (13 marks over 6 archive pages) and never straight.
// A straight single quote that opens after a space, an opening mark or an emphasis delimiter in opening position (`*'hi'*`),
// with any character that is not white space after it (a word, or markup and punctuation as in `'(hi)'`, `'*hi*'`, `'$5'`),
// waits on a stack for a straight single quote on its line that closes after a non-space with no word
// after it; a closing mark pairs with the innermost opening mark that waits, so a quotation inside a quotation pairs
// both (`'she said 'hi' to me'`); each pair is a quotation, and the two marks of a pair are curled together or not at all.
// A mark before a
// leading elision (`'tis`, `'em`, `'99`, `'90s`, `isLeadingElision`, `quote-neighbours.ts`)
// opens a quotation when a later mark closes it and no other opening mark is waiting
// (`'Tis the season,' she said`), and is an apostrophe otherwise, which `restore-typography.ts` curls to the
// closing single mark: when no mark closes it, and inside a quotation, where it never joins the
// marks that wait (`'give 'em hell'`). A two-digit elision directly followed by a closing mark (`'42'`, `'90s'`) is a quoted
// number, not an elision: it opens a quotation that the mark right after it closes. Inside a quotation both readings are
// possible (the number a quotation of its own, or an elision whose closing mark closes the quotation around it), and the
// reading that leaves fewer marks unpaired on the line wins, a tie keeping the elision: `'he is '42' years old'` pairs the number,
// and `'the '90s'` reads `‘the ’90s’` (`nested-single-line-readings.ts`). A doubled opener, a mark right before such an elision (`''tis the season'`), opens
// the quotation, and the mark after it is the apostrophe. The `'n'` of `rock 'n' roll`
// (`isElisionPair`, `isElisionPairEnd`) is one unit whose two marks neither open nor close a quotation.

/**
 Straight single quote.
 */
const STRAIGHT_SINGLE = '\'';

/**
 Newline, which no quotation pair crosses.
 */
const LINE_END = '\n';

/**
 How many digits the number of an elision that may be quoted has: `'42`, `'90s`.
 */
const QUOTED_NUMBER_DIGITS = 2;

/**
 The ASCII digits.
 */
const DIGITS = '0123456789';

/**
 Whether the character at an offset is an ASCII digit.

 @param text - text being read

 @param at - offset of the character

 @returns False at the text's end

 @example
 ```ts
 isDigitAt({ text: 'in 42', at: 3, },); // true
 ```
 */
function isDigitAt(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  /**
   The character, empty past the text's end.
   */
  const character = text.charAt(at,);
  return (character !== '') && DIGITS.includes(character,);
}

/**
 Marks a quotation may open right after, besides a space or the text's start.
 */
const OPENING_CONTEXT: ReadonlySet<string> = new Set([
  '“',
  '‘',
  '"',
  '(',
  '[',
  '—',
  '–',
],);

/**
 Offsets of the straight single quotes that open and close a quotation.
 */
export type NestedSinglePairs = {
  /**
   Offsets of opening marks.
   */
  readonly openings: ReadonlySet<number>;

  /**
   Offsets of closing marks.
   */
  readonly closings: ReadonlySet<number>;
};

/**
 The emphasis delimiters, which open a span or close one by what stands before the run they form.
 */
const EMPHASIS_DELIMITERS: ReadonlySet<string> = new Set([
  '*',
  '_',
  '~',
],);

/**
 Whether a character is where a quotation may open right after it: the text's start, a space or an opening mark.

 @param character - code point before the run, empty at the text's start

 @returns True where a quotation may open

 @example
 ```ts
 opensAfter({ character: ' ', },); // true
 ```
 */
function opensAfter({ character, }: { readonly character: string; },): boolean {
  if (character.trim() === '')
    return true;
  return OPENING_CONTEXT.has(character,);
}

/**
 Whether a quotation may open at the quote at an offset by what stands before it: the text's start, a space, an opening mark,
 or an emphasis delimiter in opening position (`*'hi'*`). A delimiter that follows a word or a closing mark closes its span
 instead (`*Cat Chorus*'sad`, `[name](url)'s`), so a quote after it opens nothing, and so does a closing bracket or
 parenthesis.

 @param text - text being read

 @param at - offset of the quote

 @returns True where a quotation may open

 @example
 ```ts
 opensAfterQuote({ text: "He said *'hi'*", at: 9, },); // true
 ```
 */
function opensAfterQuote(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  /**
   Code point before the quote, with any combining marks on it.
   */
  const character = sequenceBefore({
    text,
    at,
  },);
  if (!EMPHASIS_DELIMITERS.has(character,))
    return opensAfter({ character, },);
  /**
   Where the run of delimiters that ends at the quote starts.
   */
  const run = { start: at - character.length, };
  while ((run.start > 0) && EMPHASIS_DELIMITERS.has(text.charAt(run.start - 1,),))
    run.start -= 1;
  return opensAfter({ character: sequenceBefore({
    text,
    at: run.start,
  },), },);
}

/**
 Whether the straight quote at an offset is the first of two straight marks whose second is a leading elision with a word
 after it (`''tis`, `''em`), and the first stands where a quotation may open.

 @param text - text being read

 @param at - offset of the first mark

 @param before - code point before the first mark, with any combining marks on it

 @returns Whether the first mark opens a quotation whose first word is the elision

 @example
 ```ts
 opensBeforeElision({ text: "He said ''tis fine'", at: 8, before: ' ', },); // true
 ```
 */
function opensBeforeElision(
  {
    text,
    at,
    before,
  }: {
    readonly text: string;
    readonly at: number;
    readonly before: string;
  },
): boolean {
  if (text.charAt(at + 1,) !== STRAIGHT_SINGLE)
    return false;
  if (bindsWord({ character: before, },) || (!opensAfterQuote({
    text,
    at,
  },)))
    return false;
  return bindsWord({ character: codePointAt({
    text,
    at: at + 2,
  },), },) && isLeadingElision({
    text,
    at: at + 1,
  },);
}

/**
 Characters right before a quote that may close a quotation as well as open one: the straight double mark and the dashes
 (`'she said "hi"'.`, `'Wait—'.`). A quote after one of them with markup or punctuation after it is read as closing.
 */
const OPENING_OR_CLOSING_BEFORE: ReadonlySet<string> = new Set([
  '"',
  '—',
  '–',
],);

/**
 Whether the straight quote at an offset opens a quotation by markup or punctuation after it: it stands where a quotation may
 open, and the character after it is neither white space, nor a word character, nor another straight single mark
 (`'(hi)'`, `'*hi*'`, `'“hi”'`, `'$5'`). A word after the mark is read by `singleShape`, and white space after it makes the mark
 no opening one (`' hi'`). `restore-typography.ts` reads the same shape to see which marks wait for a partner.

 @param text - text being read

 @param at - offset of the quote

 @returns Whether the quote has opening shape with a character that is no word character after it

 @example
 ```ts
 opensBeforeMarkup({ text: "says '(nap)'", at: 5, },); // true
 ```
 */
export function opensBeforeMarkup(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  /**
   Code point after the quote, empty at the text's end.
   */
  const after = codePointAt({
    text,
    at: at + 1,
  },);
  if (after.trim() === '')
    return false;
  if ((after === STRAIGHT_SINGLE) || bindsWord({ character: after, },))
    return false;
  /**
   Character before the quote, with any combining marks on it.
   */
  const before = sequenceBefore({
    text,
    at,
  },);
  if (bindsWord({ character: before, },) || OPENING_OR_CLOSING_BEFORE.has(before,))
    return false;
  return opensAfterQuote({
    text,
    at,
  },);
}

/**
 Offset of the closing mark of a quoted two-digit number whose opening mark is at an offset (`'42'`, `'90s'`): the two digits, an
 optional `s`, and a straight single quote that no word follows. None where the text after the mark reads any other way.

 @param text - text being read

 @param at - offset of the opening mark

 @returns The offset of the closing mark as a list of one, none when the text after the mark reads any other way

 @example
 ```ts
 closerOfQuotedNumber({ text: "he is '42' old", at: 6, },); // [9]
 ```
 */
function closerOfQuotedNumber(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): readonly number[] {
  if (!isLeadingElision({
    text,
    at,
  },))
    return [];
  /**
   The offset of the second digit.
   */
  const lastDigit = at + QUOTED_NUMBER_DIGITS;
  /**
   The offset of the character after the two digits.
   */
  const afterDigits = lastDigit + 1;
  if (!isDigitAt({
    text,
    at: at + 1,
  },))
    return [];
  if (!isDigitAt({
    text,
    at: lastDigit,
  },))
    return [];
  if (isDigitAt({
    text,
    at: afterDigits,
  },))
    return [];
  /**
   Where the closing mark would stand: past an `s` that makes the number a decade.
   */
  const closer = (text.charAt(afterDigits,) === 's') ? (afterDigits + 1) : afterDigits;
  if (text.charAt(closer,) !== STRAIGHT_SINGLE)
    return [];
  /**
   The code point after the closing mark.
   */
  const following = codePointAt({
    text,
    at: closer + 1,
  },);
  return bindsWord({ character: following, },) ? [] : [closer,];
}

/**
 How a straight single quote at an offset reads by its neighbours: opening a
 quotation, closing one, or neither (an apostrophe, a leading elision or
 unreadable).

 @param text - text being read

 @param at - offset of the quote

 @returns Shape of the quote

 @example
 ```ts
 singleShape({ text: "says 'nap.'", at: 5, },); // 'opening'
 ```
 */
function singleShape(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): 'opening' | 'closing' | 'neither' {
  /**
   Character before the quote, with any combining marks on it.
   */
  const before = sequenceBefore({
    text,
    at,
  },);
  /**
   Whether a word follows the quote.
   */
  const boundAfter = bindsWord({ character: codePointAt({
    text,
    at: at + 1,
  },), },);
  if (boundAfter) {
    /**
     Whether the quote stands where a quotation may open, with a word after it.
     */
    const mayOpen = (!bindsWord({ character: before, },)) && opensAfterQuote({
      text,
      at,
    },);
    // A leading elision (`'tis`, `'90s`) opens a quotation when a later mark closes it, as any mark does, and is an apostrophe
    // when none does (`restore-typography.ts` reads the unpaired mark). The `'n'` of `rock 'n' roll` opens none.
    return (mayOpen && (!isElisionPair({
      text,
      at,
    },))) ? 'opening' : 'neither';
  }
  // The second mark of the `'n'` of `rock 'n' roll` closes none: the unit is two apostrophes.
  if (isElisionPairEnd({
    text,
    at,
  },))
    return 'neither';
  // A DOUBLED OPENER: a mark that opens a quotation whose first word is an elision (`''tis the season'`). The mark right after
  // it is the apostrophe of the elision, so this one opens and the next one never takes its place.
  if (opensBeforeElision({
    text,
    at,
    before,
  },))
    return 'opening';
  // MARKUP OR PUNCTUATION AFTER THE MARK: a mark that stands where a quotation may open and has any character that is not white
  // space after it opens one (`'(hi)'`, `'*hi*'`, `'$5'`), so its partner is never curled alone.
  if (opensBeforeMarkup({
    text,
    at,
  },))
    return 'opening';
  return (before.trim() === '') ? 'neither' : 'closing';
}

/**
 Every straight single quotation pair in a text's prose: a closing mark
 paired with the innermost opening mark on its line that still waits, so pairs nest, and a listed elision, which is an
 apostrophe inside a quotation, never waits behind another opening mark. A quoted two-digit number inside a quotation reads as a
 quotation or as an elision, whichever leaves fewer marks unpaired, an elision on a tie.

 @param text - text being read

 @param mask - which units of the text are prose

 @returns Offsets of paired opening and closing marks

 @example
 ```ts
 nestedSinglePairs({ text: "says 'nap.'", mask, },); // openings {5}, closings {10}
 ```
 */
export function nestedSinglePairs(
  {
    text,
    mask,
  }: {
    readonly text: string;
    readonly mask: readonly boolean[];
  },
): NestedSinglePairs {
  /**
   The pairs of every line read so far, mutated only inside this function.
   */
  const settled: PairedMarks[] = [];
  (function scan(): void {
    /**
     The readings of the line so far, one for each depth of waiting marks that a quoted number leaves open, the reading that
     keeps an elision first.
     */
    const line = { states: [EMPTY_LINE,] as readonly LineState[], };
    for (let index = 0; index < text.length; index += 1) {
      /**
       Unit under the scan.
       */
      const character = text.charAt(index,);
      if (character === LINE_END) {
        settled.push(...bestPairsOf({ states: line.states, },),);
        line.states = [EMPTY_LINE,];
      }
      if ((mask[index] !== true) || (character !== STRAIGHT_SINGLE))
        continue;
      /**
       How the quote reads.
       */
      const shape = singleShape({
        text,
        at: index,
      },);
      /**
       The closing mark of a quoted two-digit number that opens here, none for any other mark.
       */
      const [numberCloser,] = (shape === 'opening')
        ? closerOfQuotedNumber({
          text,
          at: index,
        },)
        : [];
      if ((numberCloser !== undefined) && (mask[numberCloser] === true)) {
        // A QUOTED NUMBER (`'42'`, `'90s'`) can be read two ways inside a quotation: as a quotation of its own, or as an elision
        // whose closing mark closes the quotation around it. Both readings are kept to the end of the line, and the one that
        // leaves fewer marks unpaired wins, a tie keeping the elision.
        line.states = withQuotedNumber({
          states: line.states,
          open: index,
          close: numberCloser,
          closeShape: singleShape({
            text,
            at: numberCloser,
          },),
        },);
        // The mark that closes the number is read with it.
        index = numberCloser;
        continue;
      }
      /**
       Whether the mark starts a listed elision.
       */
      const elision = isLeadingElision({
        text,
        at: index,
      },);
      line.states = distinctDepths({
        states: line.states
          .map(function marked(state,): LineState {
            return withMark({
              state,
              at: index,
              shape,
              elision,
            },);
          },),
      },);
    }
    settled.push(...bestPairsOf({ states: line.states, },),);
  })();
  return {
    openings: new Set(settled.map(function openOf(pair,): number {
      return pair.open;
    },),),
    closings: new Set(settled.map(function closeOf(pair,): number {
      return pair.close;
    },),),
  };
}

//endregion Nested single quotes
