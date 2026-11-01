/**
 A reading of the single marks and quotation marks of an English written apart
 from the code under test, by the definitions alone and without any attention
 to speed, for the tests that check what a replacement does to the marks of a
 whole page: which marks are attached to a word, which are proven apostrophes,
 and which are the loose ones that are left. Fixtures read ASCII letters and
 digits and characters of the first plane only, and read the English as it is
 written, with no component, code or comment left out of it.

 THE RULES THE CODE UNDER TEST HOLDS (decisions D1 to D9 of the independent check; the rule text lives in the headers of
 `consistency-proven-apostrophes.ts`, `consistency-markup-runs.ts`, `consistency-mark-neighbours.ts` and
 `consistency-splice-readings.ts`), stated here so a test written against this reader knows what it is held to:
 D1, a gain (an apostrophe that arrives at an edge where the receiving words hold none) is unavailable when any single mark
 stands directly outside the span on that side or one wide run away, and when the words carry two marks at that edge;
 D2, a `*`, `_` or `~` counts as markup only where the Markdown parse makes it a delimiter; D3, a mark reads trailing
 only when what follows it is white space, punctuation that is no single mark or the edge of the text, and leading only
 when what precedes it is likewise, and a letter of another script, a symbol or a pictograph beside it makes it free;
 D4, the run a mark is read past holds closing delimiters of emphasis, strong emphasis or strikethrough from the parse and
 a link's closing `](…)`, on the trailing side only, so `"`, `”` and `)` leave it and a mark before opening markup
 (`’**cats**`, `'_cats_`) reads free; D5, a single mark directly beside another single mark reads free, both; D6, a
 straight leading `'` and a later trailing mark are possible partners, so neither is proven by the other's absence;
 D7, a splice that removes or adds a quotation mark of the page is refused; D8, U+02BC is a single mark like `’`;
 D9, an accepted limit: a bare leading mark no later mark could close reads as an elision (`'tis`), though a reader may see
 an unclosed quotation.

 WHERE THIS READER IS COARSER THAN THOSE RULES, because it asks no parse and reads ASCII only: it takes every `*`, `_` and
 `~` for markup, so a literal one reads as a delimiter here (D2); its run of closing markup past which a trailing mark is read
 still holds the backtick, closing brackets, double quotation marks and closing corner brackets (D4); its reading of a mark
 before opening markup keeps a leading mirror that D4 removed; it does not part a mark by a pictograph, a symbol or a letter
 of another script (D3). A test that depends on one of these differences must be asserted against the code under test and
 not against this reader.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

/**
 One single mark, double quotation mark or corner bracket of an English.
 */
type PageMark = {
  /**
   Offset of the mark in the English.
   */
  readonly at: number;

  /**
   The character.
   */
  readonly character: string;

  /**
   What the mark is: a single mark, a double quotation mark or a corner bracket.
   */
  readonly family: 'single' | 'double' | 'corner';

  /**
   For a single mark: attached when a letter or a digit stands on both sides,
   trailing when one stands before it and none after, leading when one stands
   after it and none before, free otherwise; absent for the other families. A mark with no letter or digit directly beside
   it is trailing when it ends a run of closing markup that one stands before (`*kittens*’`), leading when it starts a run
   of opening markup that one stands after (`’*tis*`), and free when neither or both.
   */
  readonly kind?: 'attached' | 'trailing' | 'leading' | 'free';

  /**
   Whether the mark is a proven apostrophe: a trailing mark of the three
   apostrophe-like spellings that no earlier quotation-like single mark anywhere
   in the English could have opened a quotation for, or a leading one that no
   later such mark could close a quotation for. The opening curly mark and the
   five other single quotation characters (the low and the reversed single mark,
   both angle marks and the backtick) are such a mark wherever they stand; the
   three apostrophe-like marks are one by where they stand among letters.
   */
  readonly proven: boolean;
};

/**
 The three apostrophe-like single marks and the opening curly one.
 */
const SINGLE_MARKS: ReadonlySet<string> = new Set([
  '\'',
  '’',
  '\u{02BC}',
  '‘',
],);

/**
 The single marks that can be proven apostrophes.
 */
const APOSTROPHE_LIKE: ReadonlySet<string> = new Set([
  '\'',
  '’',
  '\u{02BC}',
],);

/**
 The single quotation characters that can pair with any apostrophe-like mark
 wherever they stand, glued between letters or not: the opening curly mark, the
 low and the reversed single mark, both angle marks and the backtick.
 */
const PARTNER_CHARACTERS: ReadonlySet<string> = new Set([
  '‘',
  '‚',
  '‛',
  '‹',
  '›',
  '`',
],);

/**
 The double quotation marks.
 */
const DOUBLE_MARKS: ReadonlySet<string> = new Set([
  '"',
  '“',
  '”',
],);

/**
 The corner brackets.
 */
const CORNER_MARKS: ReadonlySet<string> = new Set([
  '「',
  '」',
  '『',
  '』',
],);

/**
 The letters and digits of the grids.
 */
const WORD_CHARACTERS: ReadonlySet<string> = new Set(Array.from(
  'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789',
),);

/**
 The characters of a run of closing markup after a word: emphasis, the backtick, closing brackets and double quotation marks
 and the closing corner brackets.
 */
const CLOSING_RUN: ReadonlySet<string> = new Set(Array.from('*_~`)]"”」』',),);

/**
 The characters of a run of opening markup before a word: emphasis, the backtick, opening brackets and double quotation marks
 and the opening corner brackets.
 */
const OPENING_RUN: ReadonlySet<string> = new Set(Array.from('*_~`(["“「『',),);

/**
 How a single mark with no letter or digit directly beside it stands past markup: trailing when it ends a run of closing
 markup that a letter or a digit stands before, leading when it starts a run of opening markup that one stands after, free
 when neither or both. A mark with a letter or a digit directly beside it is not asked.

 @param text - the English

 @param at - offset of the mark

 @returns The kind the mark is read as

 @example
 ```ts
 const kind = kindPastMarkup({ text: 'The *kittens*’ bowl', at: 13, },); // 'trailing'
 ```
 */
function kindPastMarkup(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): 'trailing' | 'leading' | 'free' {
  /**
   The characters before the mark.
   */
  const before = Array.from(text.slice(
    0,
    at,
  ),);
  /**
   Where the last character that is no part of a run of closing markup stands, none when there is none.
   */
  const beforeRun = before.findLastIndex(function endsRun(character,): boolean {
    return !CLOSING_RUN.has(character,);
  },);
  /**
   The characters after the mark.
   */
  const after = Array.from(text.slice(at + 1,),);
  /**
   Where the first character that is no part of a run of opening markup stands, none when there is none.
   */
  const afterRun = after.findIndex(function endsRun(character,): boolean {
    return !OPENING_RUN.has(character,);
  },);
  /**
   The character right before the run of closing markup, empty when there is none.
   */
  const wordBeforeRun = before[beforeRun] ?? '';
  /**
   Whether a run of closing markup stands right before the mark.
   */
  const runStands = (beforeRun !== (-1)) && (beforeRun < (before.length - 1));
  /**
   Whether a closing run of markup stands right before the mark and a letter or a digit right before the run.
   */
  const trails = runStands && WORD_CHARACTERS.has(wordBeforeRun,);
  /**
   Whether an opening run of markup stands right after the mark and a letter or a digit right after the run.
   */
  const leads = (afterRun > 0) && WORD_CHARACTERS.has(after[afterRun] ?? '',);
  if (trails === leads)
    return 'free';
  return trails ? 'trailing' : 'leading';
}

/**
 One mark of the English before it is known to be proven, or one of the quotation
 characters that only act as partners and are no mark of the page.
 */
type FoundMark = Omit<PageMark, 'proven'> & { readonly partnerOnly: boolean; };

/**
 Whether a mark could have been opened by a quotation: it is a quotation character that pairs wherever it stands, or an
 apostrophe-like mark with no letter or digit directly before it.

 @param mark - the mark

 @returns Whether a trailing apostrophe-like mark after it could be its closer

 @example
 ```ts
 const opens = couldOpen({ mark, },);
 ```
 */
function couldOpen({ mark, }: { readonly mark: FoundMark; },): boolean {
  if (PARTNER_CHARACTERS.has(mark.character,))
    return true;
  return (mark.family === 'single') && ((mark.kind === 'leading') || (mark.kind === 'free'));
}

/**
 Whether a mark could close a quotation: it is a quotation character that pairs wherever it stands, or an apostrophe-like
 mark with no letter or digit directly after it.

 @param mark - the mark

 @returns Whether a leading apostrophe-like mark before it could be its opener

 @example
 ```ts
 const closes = couldClose({ mark, },);
 ```
 */
function couldClose({ mark, }: { readonly mark: FoundMark; },): boolean {
  if (PARTNER_CHARACTERS.has(mark.character,))
    return true;
  return (mark.family === 'single') && ((mark.kind === 'trailing') || (mark.kind === 'free'));
}

/**
 Every single mark, double quotation mark and corner bracket of an English,
 with what each is.

 @param text - the English

 @returns The marks in the order they stand

 @example
 ```ts
 const marks = pageMarksOf({ text: 'The parents’ bowl', },); // one trailing proven single mark
 ```
 */
export function pageMarksOf({ text, }: { readonly text: string; },): readonly PageMark[] {
  /**
   The marks and the partner-only quotation characters before their proven status is known.
   */
  const found = Array.from(text,)
    .flatMap(function markAt(
      character,
      at,
    ): readonly FoundMark[] {
      if (DOUBLE_MARKS.has(character,))
        return [{
          at,
          character,
          family: 'double',
          partnerOnly: false,
        },];
      if (CORNER_MARKS.has(character,))
        return [{
          at,
          character,
          family: 'corner',
          partnerOnly: false,
        },];
      if (!SINGLE_MARKS.has(character,)) {
        return PARTNER_CHARACTERS.has(character,)
          ? [{
            at,
            character,
            family: 'single',
            kind: 'free',
            partnerOnly: true,
          },]
          : [];
      }
      /**
       Whether a letter or a digit stands directly before the mark.
       */
      const before = WORD_CHARACTERS.has(text.charAt(at - 1,),);
      /**
       Whether a letter or a digit stands directly after it.
       */
      const after = WORD_CHARACTERS.has(text.charAt(at + 1,),);
      /**
       How the mark stands among the letters and digits directly beside it.
       */
      const beside = (before && after) ? 'attached' : (before ? 'trailing' : (after ? 'leading' : 'free'));
      /**
       How the mark stands among its neighbours: a free mark is read past the markup beside it.
       */
      const kind = (beside === 'free')
        ? kindPastMarkup({
          text,
          at,
        },)
        : beside;
      return [{
        at,
        character,
        family: 'single',
        kind,
        partnerOnly: false,
      },];
    },);
  return found
    .flatMap(function withProof(
      mark,
      position,
    ): readonly PageMark[] {
      if (mark.partnerOnly)
        return [];
      /**
       Whether a quotation character before this mark could have opened a quotation it closes, or one after it could close a
       quotation it opens.
       */
      const partnered = (mark.kind === 'trailing')
        ? found.slice(
          0,
          position,
        )
          .some(function opensBefore(other,): boolean {
            return couldOpen({ mark: other, },);
          },)
        : found.slice(position + 1,)
          .some(function closesAfter(other,): boolean {
            return couldClose({ mark: other, },);
          },);
      return [{
        at: mark.at,
        character: mark.character,
        family: mark.family,
        ...((mark.kind === undefined) ? {} : { kind: mark.kind, }),
        proven: (mark.family === 'single') && APOSTROPHE_LIKE.has(mark.character,)
          && ((mark.kind === 'trailing') || (mark.kind === 'leading'))
          && (!partnered),
      },];
    },);
}

/**
 The loose marks of an English that are not proven apostrophes, in order, each
 in the form two marks are compared in: every double quotation mark as the
 straight one, every corner bracket as itself, the three apostrophe-like single
 marks as the straight apostrophe and the opening curly mark as itself. A
 single mark with a letter or a digit on both sides is attached, and a proven
 apostrophe is a part of its word, so neither is loose.

 @param text - the English

 @returns The loose marks, one text per mark

 @example
 ```ts
 const marks = unprovenLooseMarksIn({ text: 'the parents’ "bowl"', },); // ['"', '"']
 ```
 */
export function unprovenLooseMarksIn({ text, }: { readonly text: string; },): readonly string[] {
  return pageMarksOf({ text, },)
    .filter(function isLoose({
      family,
      kind,
      proven,
    },): boolean {
      return (family !== 'single') || ((kind !== 'attached') && (!proven));
    },)
    .map(function foldedOf({
      character,
      family,
    },): string {
      if (family === 'double')
        return '"';
      if (family === 'corner')
        return character;
      return (character === '‘') ? '‘' : '\'';
    },);
}

/**
 The offsets of the single marks of an English that are attached to a word.

 @param text - the English

 @returns The offsets, in order

 @example
 ```ts
 const attached = attachedOffsetsIn({ text: 'the cat’s bowl', },); // [7]
 ```
 */
export function attachedOffsetsIn({ text, }: { readonly text: string; },): readonly number[] {
  return pageMarksOf({ text, },)
    .filter(function isAttached({ kind, },): boolean {
      return kind === 'attached';
    },)
    .map(function offsetOf({ at, },): number {
      return at;
    },);
}
