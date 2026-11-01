import { ConsistencyInvariantError, } from './consistency-invariant-error.ts';
import { isNeighbouringSingleMark, } from './consistency-mark-neighbours.ts';
import {
  closingRunMarksOf,
  EMPHASIS_CHARACTERS,
} from './consistency-markup-runs.ts';
import { emphasisDelimitersOf, } from './consistency-prose-text.ts';
import { continuesLatinWordOrCompatibilityForm, } from '@monochromatic-dev/translation-repair-markup/ts';

// The category test of a letter or a decimal digit of any script lives with the neighbour test that asks it
// (`latin-letters.ts`); the addresses and the carry crossing read it from here.

//region Consistency loose marks
// WHICH MARKS OF A PLACE'S ENGLISH A SPLICE MAY NOT BRING IN OR TAKE OUT. Four
// characters are SINGLE MARKS: the straight apostrophe, the opening and the
// closing curly single mark, and the modifier letter apostrophe (U+02BC). A
// single mark is ATTACHED when a letter or a digit stands directly on both sides
// of it (`don't`, `cat’s`): it is part of a word and moves with the word. Every
// other single mark, every double quotation mark (`"`, `“`, `”`, `„`, `‟`) and every
// corner bracket (`「`, `」`, `『`, `』`), is a LOOSE MARK: it opens or closes
// something, or marks a letter left out. A splice that carries words from one
// place to another keeps the receiving place's loose marks in their places,
// brings none from the chosen place, and writes them in the receiving place's
// own form; whether that leaves a single mark beside the span attached or loose
// as it was is decided at the span's edge (`edgeClassOf`, `consistency-splice.ts`).
//
// "A LETTER OR A DIGIT" HERE IS ONE TEST FOR EVERY READING OF A SINGLE MARK: a Latin
// letter with its accent, composed or combining, or an ASCII digit, or a whole character that is itself a letter or a
// decimal digit and whose NFKC form is those alone (a mathematical script letter, a fullwidth letter or digit, a ligature;
// `continuesLatinWordOrCompatibilityForm`, `latin-letters.ts`; a trade mark sign, a superscript or circled digit and a Roman
// numeral are none), the one neighbour test of the consistency
// pass: the locator's word joints (`consistency-word-joints.ts`), the proof of an apostrophe
// (`consistency-proven-apostrophes.ts`), the closing marks (`consistency-closing-marks.ts`), the edge
// marks (`consistency-edge-marks.ts`) and the form reader (`consistency-apostrophe-forms.ts`) ask it of a
// neighbour too. The rest of the package reads a word with `continuesLatinWord`, as written, and a
// compatibility character beside a mark changes nothing there. A Greek letter, a Han
// character and an Arabic-Indic digit beside a mark attach nothing, so `猫’猫` and `αβ’γδ`
// hold a loose mark, which only keeps a mark from being moved: the safe direction.
//
// WHAT THIS READS IS CHARACTERS, NOT ROLES. Whether a loose mark opens a
// quotation, closes one or stands for a dropped letter is never decided here (the
// edge reading of a quote once guessed it, and a wrong guess deleted a quotation
// mark from a page); the two places are compared by the marks they hold and by
// where those stand.
//
// THE FOLDS THE COMPARISONS READ. The straight apostrophe, the closing curly
// single mark and the modifier letter apostrophe are one mark, since a place
// written with one and a place written with another hold the same punctuation;
// the opening curly single mark is a mark of its own, which only equals itself;
// the five double quotation marks (`"`, `“`, `”`, the low `„` U+201E and the reversed `‟` U+201F) are one mark,
// so a carry that adds, removes or moves one of them is refused like any double mark; the okina `ʻ`, the
// prime `′` and the double prime `″` are a letter and unit marks, not quotation marks, and are no loose
// mark at all; each of the four corner brackets
// is a mark of its own and equals only itself, though the evidence fold of
// `quote-normalize.ts` writes them as a double or a single mark, since two places
// that differ in which bracket they hold differ on the page; so is each of the low
// and the reversed single mark, the two single angle marks and the two guillemets
// (`‚ ‛ ‹ › « »`), a quotation mark of its own wherever it stands inside the words, so a
// carry that changes their number or order is refused as it is for the double marks.
// The fold changes how two marks are compared and never what the page holds.
//
// THE OPENING CURLY SINGLE MARK BETWEEN LETTERS (`cat‘s`) IS A QUOTATION MARK HERE, never
// part of a word: this reader counts it as a loose mark, so a carry that adds, removes or
// moves one is refused. The locator reads the same mark as a joint, so words that run into
// it (`cat` in `cat‘s`) are not found and no carry is attempted there, and the proof reads
// it as a partner wherever it stands; the three headers (this one, the joints header and the
// proof's) say the same thing.
//
// A PROVEN APOSTROPHE AT AN EDGE OF A SPAN IS PART OF ITS WORD. A locator that has
// proven a single mark beside the words an apostrophe (`parents’` in `the parents’
// preference`, `consistency-proven-apostrophes.ts`) takes it into the span, so the
// words it names start or end with one apostrophe-like mark and a letter or a digit
// stands directly inside it at the start, and directly inside it or, at the end only, before a reading run of closing
// emphasis or a link's destination (`*kittens*’`: `closingRunMarksOf` in `consistency-markup-runs.ts`, the way the proof
// reads a mark past markup; `’*tis*` holds none, since a mark before opening markup is read where it stands, D4).
// A single mark with another single mark directly beside it holds none either (D5: `cats’’` ends in a pair of free marks).
// Read as the words of a span (`looseMarksOf`,
// `holdsEdgeApostrophe`) that mark is attached, like `don't`: it moves with its
// word and is no loose mark. Every caller reads the words of a rendering, so the
// functions of this file read all their text that way.
//
// THE WORDS ARE READ ALONE UNLESS A CALLER HANDS THE PAGE'S DEFINITIONS. The mark that ends a reading run is found by the parse of the
// words, and the parse of a fragment pairs the delimiters of emphasis by the definitions it can see: `*a [b*] c*’` ends in a
// reading run where the page defines `[b*]` and does not where it does not. Every function of this file that reads the words
// takes the definitions for that reason, and the planner, the apply stage and the reader of a record pass none, since the three
// share these functions, the reader of a record holds no page to read them from, and a definition handed to two of them and not
// the third would let the pass plan a splice that the reader of the record refuses. RECORDING THE DEFINITIONS WOULD NOT MAKE THE WORDS
// READ AS THE PAGE DOES, and that is the reason that holds when the first one does not (ledger B673): a definition cannot supply a
// delimiter that stands outside the span. In `Take *[*]a*’ now.` with `[*]` defined, the page reads the mark after `a*` as an apostrophe
// of its word, because the `*` after `a` closes the emphasis the `*` before `[*]` opened; the span `[*]a*’` read alone agrees, and read
// with the definition it makes the `*` inside `[*]` plain text and the `*` after `a` a closer of nothing, so the mark reads loose. Of
// the 84 spans that `carriesOrCutsMarkup` passes and that read another way with the definitions of their brackets, among the
// strings of three to seven characters over `* _ ~ [ ] ( ) a b ’` and a space that start or end in `’` and hold a bracket with a
// label, all 84 are of that kind (attached alone, loose with the definition). A span that only a definition balances
// (`*a [b*] c*’`) is refused as cutting markup, since `carriesOrCutsMarkup` counts delimiters and reads the span alone.
//
// A CORNER BRACKET BESIDE A SPAN IS NOT RECORDED. A single mark directly beside
// a located span that is no proven apostrophe is kept in the rendering
// (`markBefore`, `markAfter`), and a generation 17 record stores one of the eight
// single marks (`isCountedSingleMark`: the four single marks and the low and reversed single marks and the single angle
// marks) and refuses anything else
// (`corpus-run/artifact-two-lane-read-consistency-rendering.ts`). Storing a
// corner bracket there would widen the record, so only the corner brackets inside
// the words of a span are compared; the ones beside it stay where they are on
// the page, since a replacement never reaches outside the span. The refusal of a gain beside a kept mark (D1,
// `gainsBesideKeptMark`, `consistency-splice-readings.ts`) sees all eight, directly beside the span or one refusal run
// away. The locator trims only the four from the edges of a judge's quote (`isSingleMark`), so a quote that names
// `‹cats›` is spanned with the angle marks inside it, as loose marks of their own.
//
// A GAIN IS A SINGLE MARK AT AN EDGE WHATEVER STANDS INSIDE IT (the ruling on gains, `holdsEdgeMark`): a chosen word set
// whose first or last character is one of the eight single marks (`isCountedSingleMark`) brings a mark that the page must prove, though no reader of
// the words can say how the page reads it (`_cats_ʼ` has no closing underscore, since the modifier letter apostrophe is a
// letter to the parse, so its mark reads free; `_cats_’` is real emphasis with an edge apostrophe after it). The words the
// mark replaces hold one only as an edge apostrophe (`holdsEdgeApostrophe`), which is the mark a span can take in.

/**
 The straight apostrophe, the form every apostrophe-like mark folds to.
 */
const APOSTROPHE = '\'';

/**
 The opening curly single quotation mark, which is only ever compared with itself.
 */
const OPENING_SINGLE = '‘';

/**
 The single marks that fold to one: the straight apostrophe, the closing curly
 single mark and the modifier letter apostrophe.
 */
const APOSTROPHE_LIKE: ReadonlySet<string> = new Set([
  APOSTROPHE,
  '’',
  '\u{02BC}',
],);

/**
 The four single marks.
 */
const SINGLE_MARKS: ReadonlySet<string> = new Set([
  ...APOSTROPHE_LIKE,
  OPENING_SINGLE,
],);

/**
 The straight double quotation mark, the form the three double marks fold to.
 */
const DOUBLE_MARK = '"';

/**
 The double quotation marks: the straight one, the two curly ones, the low double mark (U+201E) and the reversed double mark
 (U+201F). The okina (`ʻ`), the prime (`′`) and the double prime (`″`) are not among them: they are a letter and unit marks,
 never a quotation mark.
 */
const DOUBLE_MARKS: ReadonlySet<string> = new Set([
  DOUBLE_MARK,
  '“',
  '”',
  '\u{201E}',
  '\u{201F}',
],);

/**
 The corner brackets, which are quotation marks: the corner bracket and the
 white corner bracket, each opening and closing. Each is a loose mark as itself,
 and none is ever part of a word.
 */
const CORNER_MARKS: ReadonlySet<string> = new Set([
  '「',
  '」',
  '『',
  '』',
],);

/**
 The low and the reversed single quotation marks, the single angle quotation marks and the guillemets, each a mark of its
 own that equals only itself, as the corner brackets are. None is an apostrophe in any use, and none is part of a word
 wherever it stands.
 */
const ANGLE_MARKS: ReadonlySet<string> = new Set([
  '\u{201A}',
  '\u{201B}',
  '‹',
  '›',
  '«',
  '»',
],);

/**
 Whether a character is one of the four single marks.

 @param character - one whole character, empty past an edge of a text

 @returns Whether it is a single mark

 @example
 ```ts
 const single = isSingleMark({ character: '’', },); // true
 ```
 */
export function isSingleMark({ character, }: { readonly character: string; },): boolean {
  return SINGLE_MARKS.has(character,);
}

/**
 Whether a character is one of the eight single marks the pass counts beside a span and the record stores: the four single
 marks and the low and the reversed single mark and both single angle marks (`consistency-mark-neighbours.ts` holds the
 set). The double guillemets are none of them. The locator trims only the four from the edges of a judge's quote
 (`isSingleMark`), so a quote that names `‹cats›` is spanned with the angle marks inside it as loose marks of their own;
 these four are single marks only where they stand beside a span (`singleMarkAt`) or pair up at an edge of the words.

 @param character - one whole character, empty past an edge of a text

 @returns Whether it is one of the eight

 @example
 ```ts
 const counted = isCountedSingleMark({ character: '‹', },); // true
 ```
 */
export function isCountedSingleMark({ character, }: { readonly character: string; },): boolean {
  return isNeighbouringSingleMark({ character, },);
}

/**
 The single mark a text holds at an offset.

 @param text - the English

 @param at - offset asked about, which may lie outside the text

 @returns The mark as the text holds it, as a list of one, or none when the
 offset holds no single mark or lies outside the text

 @example
 ```ts
 const [mark,] = singleMarkAt({ text: 'the cats’ bowl', at: 8, },); // '’'
 ```
 */
export function singleMarkAt(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): readonly string[] {
  if ((at < 0) || (at >= text.length))
    return [];
  /**
   The unit at the offset, which no single mark shares with a surrogate pair.
   */
  const character = text.charAt(at,);
  return isCountedSingleMark({ character, },) ? [character,] : [];
}

/**
 A single mark in the form two marks are compared in: the straight apostrophe,
 the closing curly single mark and the modifier letter apostrophe are one.

 @param mark - a single mark as a text holds it

 @returns The mark to compare

 @example
 ```ts
 const folded = foldedSingleMark({ mark: '’', },); // '\''
 ```
 */
function foldedSingleMark({ mark, }: { readonly mark: string; },): string {
  return APOSTROPHE_LIKE.has(mark,) ? APOSTROPHE : mark;
}

/**
 Whether the single marks two places hold beside their words are the same
 mark: both absent, or both present and equal once the three apostrophe-like
 marks are folded to one. The opening curly single mark equals only itself, and
 an absent mark equals only an absent one.

 @param left - the mark one place holds, as a list of zero or one

 @param right - the mark the other place holds, as a list of zero or one

 @returns Whether the two places hold the same mark

 @example
 ```ts
 const same = sameBesideMark({ left: ['\'',], right: ['’',], },); // true
 ```
 */
export function sameBesideMark(
  {
    left,
    right,
  }: {
    readonly left: readonly string[];
    readonly right: readonly string[];
  },
): boolean {
  /**
   The mark one place holds.
   */
  const [one,] = left;
  /**
   The mark the other holds.
   */
  const [other,] = right;
  if ((one === undefined) || (other === undefined))
    return one === other;
  return foldedSingleMark({ mark: one, },) === foldedSingleMark({ mark: other, },);
}

/**
 One loose mark of a text: where its whole character stands among the text's
 whole characters, and the form it is compared in.
 */
type LoosePoint = {
  /**
   Position of the mark among the whole characters of the text.
   */
  readonly at: number;

  /**
   The mark in the form two marks are compared in.
   */
  readonly folded: string;
};

/**
 Whether the last whole character of the words ends a reading run (`consistency-markup-runs.ts`) after a Latin letter or a
 digit: the closing delimiters of emphasis, strong emphasis or strikethrough by the parse of the words, and a link's closing
 `](…)`. The words are read by the function the proof reads a page with (`closingRunMarksOf`), so the words and the page cannot
 disagree about a mark past markup; a lone `*`, `_` or `~` is a character of the words and no part of a run.

 @param points - the whole characters of the words of a span, the last of them the mark asked about

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for words read alone: a reference in the
 words whose definition stands elsewhere on the page is then plain text, and an unresolved `[b*]` has a `*` that can pair with an
 outer one, so the words' emphasis pairs another way (n173); read only where the last character may end a reading run

 @returns Whether the last character is a mark past closing markup

 @example
 ```ts
 const past = endsReadingRun({ points: Array.from('*kittens*’',), definitions: '', },); // true
 ```
 */
function endsReadingRun(
  {
    points,
    definitions,
  }: {
    readonly points: readonly string[];
    readonly definitions: string;
  },
): boolean {
  // A run can end at the mark only when the character before it closes a stretch of emphasis or a link's destination.
  if (!(EMPHASIS_CHARACTERS.has(points.at(-2,) ?? '',) || (points.at(-2,) === ')')))
    return false;
  /**
   The words as one text.
   */
  const text = points.join('',);
  /**
   The closing delimiters of emphasis in the words, by the parse.
   */
  const { closing, } = emphasisDelimitersOf({
    english: text,
    definitions,
  },);
  return closingRunMarksOf({
    text,
    closing,
  },)
    .has(text.length - (points.at(-1,) ?? '').length,);
}

/**
 Whether the whole character at an edge of a span is an apostrophe a proven
 apostrophe put there: one of the three apostrophe-like marks with a letter or a
 digit inside it, directly or, at the end, past a reading run of closing emphasis or a link
 (`*kittens*’`), the way the proof reads a mark past markup (`consistency-marks-past-markup.ts`). A mark at the start has no
 mirror: `’*tis*` holds none.

 @param points - the whole characters of the span

 @param at - position of the character asked about, an edge of the span

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for words read alone: a reference in the
 words whose definition stands elsewhere on the page is then plain text, and an unresolved `[b*]` has a `*` that can pair with an
 outer one, so the words' emphasis pairs another way (n173); read only where the last character may end a reading run

 @returns Whether it is an apostrophe at the edge of a span, part of the word beside it

 @example
 ```ts
 const edge = isEdgeApostropheAt({ points: Array.from('the cats’',), at: 8, definitions: '', },); // true
 ```
 */
function isEdgeApostropheAt(
  {
    points,
    at,
    definitions,
  }: {
    readonly points: readonly string[];
    readonly at: number;
    readonly definitions: string;
  },
): boolean {
  if (!APOSTROPHE_LIKE.has(points[at] ?? '',))
    return false;
  // At the start the letter must stand directly inside: a mark before opening markup is read where it stands.
  if (at === 0)
    return continuesLatinWordOrCompatibilityForm({ character: points[1] ?? '', },);
  if (at !== (points.length - 1))
    return false;
  // At the end the letter stands directly inside, or before a reading run of closing markup.
  return continuesLatinWordOrCompatibilityForm({ character: points[at - 1] ?? '', },) || endsReadingRun({
    points,
    definitions,
  },);
}

/**
 The loose marks among the whole characters of a text, each with its position.

 @param points - the whole characters of the words of a span, so a letter beyond the first plane is one neighbour

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for words read alone: a reference in the
 words whose definition stands elsewhere on the page is then plain text, and an unresolved `[b*]` has a `*` that can pair with an
 outer one, so the words' emphasis pairs another way (n173); read only where the last character may end a reading run

 @returns The loose marks in the order they stand

 @example
 ```ts
 const loose = looseMarkPointsOf({ points: Array.from('“It’s done,” she said',), definitions: '', },); // [{ at: 0, folded: '"' }, { at: 11, folded: '"' }]
 ```
 */
function looseMarkPointsOf(
  {
    points,
    definitions,
  }: {
    readonly points: readonly string[];
    readonly definitions: string;
  },
): readonly LoosePoint[] {
  return points.flatMap(function looseAt(
    point,
    at,
  ): readonly LoosePoint[] {
    if (DOUBLE_MARKS.has(point,))
      return [
        {
          at,
          folded: DOUBLE_MARK,
        },
      ];
    if (CORNER_MARKS.has(point,) || ANGLE_MARKS.has(point,))
      return [
        {
          at,
          folded: point,
        },
      ];
    if (!SINGLE_MARKS.has(point,))
      return [];
    // The opening curly single mark is never part of a word, between letters or not (`cat‘s`).
    if (APOSTROPHE_LIKE.has(point,)
      && continuesLatinWordOrCompatibilityForm({ character: points[at - 1] ?? '', },)
      && continuesLatinWordOrCompatibilityForm({ character: points[at + 1] ?? '', },))
      return [];
    if (isEdgeApostropheAt({
      points,
      at,
      definitions,
    },))
      return [];
    return [
      {
        at,
        folded: foldedSingleMark({ mark: point, },),
      },
    ];
  },);
}

/**
 Every loose mark of the words of a span in order, each in the form two marks
 are compared in: the double marks as the straight one, the apostrophe-like
 single marks as the straight apostrophe, and the opening curly single mark and
 each of the four corner brackets as itself. A single mark with a letter or a
 digit directly on both sides is attached to its word and not a loose mark; a
 corner bracket is never attached. An apostrophe-like single mark at an edge of
 the words with a letter or a digit directly inside it is a proven apostrophe
 the span took in with its word (`consistency-fold.ts`,
 `consistency-proven-apostrophes.ts`), part of the word and no loose mark. A
 span the locator makes never holds any other single mark at an edge, since the
 words a quote names have none at theirs and the span is widened over no mark but a
 proven apostrophe, so the opening curly single mark and a mark with no letter or
 digit inside it stay loose marks wherever they stand. Words a record gives are held
 to the same by `edgeMarksContradictedBySpan`, which refuses any other single mark
 at an edge.

 @param text - the words of a span

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for words read alone: a reference in the
 words whose definition stands elsewhere on the page is then plain text, and an unresolved `[b*]` has a `*` that can pair with an
 outer one, so the words' emphasis pairs another way (n173); read only where the last character may end a reading run

 @returns The loose marks as one text per mark, in the order they stand

 @example
 ```ts
 const marks = looseMarksOf({ text: '“It’s done,” she said', },); // ['"', '"']
 ```
 */
export function looseMarksOf(
  {
    text,
    definitions = '',
  }: {
    readonly text: string;
    readonly definitions?: string;
  },
): readonly string[] {
  return looseMarkPointsOf({
    points: Array.from(text,),
    definitions,
  },)
    .map(function foldedOf({ folded, },): string {
      return folded;
    },);
}

/**
 Whether the words of a span hold a proven apostrophe at one edge: an
 apostrophe-like single mark with a letter or a digit directly inside it.

 @param text - the words of a span

 @param side - `start` for the first whole character, `end` for the last

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for words read alone: a reference in the
 words whose definition stands elsewhere on the page is then plain text, and an unresolved `[b*]` has a `*` that can pair with an
 outer one, so the words' emphasis pairs another way (n173); read only where the last character may end a reading run

 @returns Whether that edge is a proven apostrophe the span took in with its word

 @example
 ```ts
 const held = holdsEdgeApostrophe({ text: 'the parents’', side: 'end', },); // true
 ```
 */
export function holdsEdgeApostrophe(
  {
    text,
    side,
    definitions = '',
  }: {
    readonly text: string;
    readonly side: 'start' | 'end';
    readonly definitions?: string;
  },
): boolean {
  /**
   The whole characters of the words.
   */
  const points = Array.from(text,);
  return isEdgeApostropheAt({
    points,
    at: (side === 'start') ? 0 : (points.length - 1),
    definitions,
  },);
}

/**
 Whether the words of a span end or begin in a single mark, whatever stands inside it: the whole character at the asked edge
 is one of the eight single marks the counter and the record reader know (`isCountedSingleMark`), with a letter, markup that
 closes nothing, a space or another mark beside it. This is the mark a gain brings to an edge (the ruling on gains): the parse
 of the words and the letter inside the mark decide how the page reads it, and no reader of the words can say that before the
 page is made (`_cats_ʼ` holds no real emphasis, so its mark reads free there, though `_cats_’` is real emphasis with an edge
 apostrophe after it). The low and the reversed single mark and the two angle marks are gains too, though no page can prove one
 an apostrophe: the planner and the apply stage ask this one definition (`bringsAtEdge`, `consistency-splice-readings.ts`), and the
 counter and the record reader know the same eight marks, so a splice that ends in one is left out by the check of the
 apostrophes and never made without a proof. It reads no markup run: the mark that ends a run is the last character of the
 words, as the mark that begins them is the first.

 @param text - the words of a span

 @param side - `start` for the first whole character, `end` for the last

 @returns Whether that edge is a single mark

 @example
 ```ts
 const held = holdsEdgeMark({ text: '_cats_ʼ', side: 'end', },); // true
 ```
 */
export function holdsEdgeMark(
  {
    text,
    side,
  }: {
    readonly text: string;
    readonly side: 'start' | 'end';
  },
): boolean {
  /**
   The whole characters of the words.
   */
  const points = Array.from(text,);
  return isCountedSingleMark({ character: ((side === 'start') ? points[0] : points.at(-1,)) ?? '', },);
}

/**
 The positions among the whole characters of a text of its apostrophes that are
 part of a word: the apostrophe-like marks a letter or a digit attaches on both
 sides, and the one at an edge of the words with a letter or a digit directly
 inside it.

 @param points - the whole characters of the words of a span

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for words read alone: a reference in the
 words whose definition stands elsewhere on the page is then plain text, and an unresolved `[b*]` has a `*` that can pair with an
 outer one, so the words' emphasis pairs another way (n173); read only where the last character may end a reading run

 @returns The positions, in the order they stand

 @example
 ```ts
 const at = apostrophePositionsOf({ points: Array.from('the cat’s bowl',), definitions: '', },); // [7]
 ```
 */
function apostrophePositionsOf(
  {
    points,
    definitions,
  }: {
    readonly points: readonly string[];
    readonly definitions: string;
  },
): readonly number[] {
  /**
   The positions of the loose marks, which an apostrophe-like mark that is no part of a word is.
   */
  const loose = new Set(looseMarkPointsOf({
    points,
    definitions,
  },)
    .map(function atOf({ at, },): number {
      return at;
    },),);
  return points.flatMap(function apostropheAt(
    point,
    at,
  ): readonly number[] {
    return (APOSTROPHE_LIKE.has(point,) && (!loose.has(at,))) ? [at,] : [];
  },);
}

/**
 The apostrophes of the words of a span that are part of a word, as written, in
 order: the marks a letter or a digit attaches on both sides (`dog's`) and the one
 at an edge of the words with a letter or a digit directly inside it (`dogs’`).

 @param text - the words of a span

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for words read alone: a reference in the
 words whose definition stands elsewhere on the page is then plain text, and an unresolved `[b*]` has a `*` that can pair with an
 outer one, so the words' emphasis pairs another way (n173); read only where the last character may end a reading run

 @returns The characters of those marks, one text per mark, none for words with no apostrophe of a word

 @example
 ```ts
 const marks = apostropheMarksOf({ text: 'the dog\'s and the cat’s bowl', },); // ['\'', '’']
 ```
 */
export function apostropheMarksOf(
  {
    text,
    definitions = '',
  }: {
    readonly text: string;
    readonly definitions?: string;
  },
): readonly string[] {
  /**
   The whole characters of the words.
   */
  const points = Array.from(text,);
  return apostrophePositionsOf({
    points,
    definitions,
  },)
    .map(function markAt(at,): string {
      return points[at] ?? '';
    },);
}

/**
 The words with every apostrophe that is part of a word written in one form. A
 loose mark is never touched: it is written in the form of the place it goes to
 by {@link withLooseMarksOf}.

 @param text - the words of a span

 @param form - the character every apostrophe of a word is written as, one of the three apostrophe-like marks

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for words read alone: a reference in the
 words whose definition stands elsewhere on the page is then plain text, and an unresolved `[b*]` has a `*` that can pair with an
 outer one, so the words' emphasis pairs another way (n173); read only where the last character may end a reading run

 @returns The words with those apostrophes in that form

 @example
 ```ts
 const written = withApostropheForm({ text: 'the dog\'s bowl', form: '’', },); // 'the dog’s bowl'
 ```
 */
export function withApostropheForm(
  {
    text,
    form,
    definitions = '',
  }: {
    readonly text: string;
    readonly form: string;
    readonly definitions?: string;
  },
): string {
  /**
   The whole characters of the words.
   */
  const points = Array.from(text,);
  /**
   The positions of the apostrophes of a word.
   */
  const apostrophes = new Set(apostrophePositionsOf({
    points,
    definitions,
  },),);
  return points
    .map(function inForm(
      point,
      at,
    ): string {
      return apostrophes.has(at,) ? form : point;
    },)
    .join('',);
}

/**
 A text with each of its loose marks written as the loose mark of another text
 that stands at the same position in that text's sequence of loose marks: the
 first loose mark of the text takes the character of the first loose mark of the
 other, and so on. Nothing is guessed about what a mark is: the two sequences are
 equal once folded, so the marks correspond one to one, and only the character
 changes (a straight quotation mark becomes the curly one the other text wrote
 there). An apostrophe inside a word is no loose mark and stays as written.

 @param text - the words whose loose marks are rewritten

 @param like - the words whose loose marks give the characters, with a sequence of loose marks that equals the text's once folded

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for words read alone; both texts are read
 with them: a reference in the words whose definition stands elsewhere on the page is then plain text, and an unresolved `[b*]` has a
 `*` that can pair with an outer one, so the words' emphasis pairs another way (n173); read only where the last character may end a
 reading run

 @returns The text with its loose marks in the form `like` wrote them

 @throws {@link ConsistencyInvariantError} when the two texts hold different numbers of loose marks, since the caller has compared the sequences

 @example
 ```ts
 const written = withLooseMarksOf({ text: '"So kind," she said', like: '“Be kind,” he said', },); // '“So kind,” she said'
 ```
 */
export function withLooseMarksOf(
  {
    text,
    like,
    definitions = '',
  }: {
    readonly text: string;
    readonly like: string;
    readonly definitions?: string;
  },
): string {
  /**
   The whole characters of the text.
   */
  const points = Array.from(text,);
  /**
   The whole characters of the text whose marks give the forms.
   */
  const modelPoints = Array.from(like,);
  /**
   The loose marks of the text.
   */
  const own = looseMarkPointsOf({
    points,
    definitions,
  },);
  /**
   The loose marks of the other text.
   */
  const model = looseMarkPointsOf({
    points: modelPoints,
    definitions,
  },);
  if (own.length !== model.length) {
    throw new ConsistencyInvariantError({ invariant:
      `the words hold ${String(own.length,)} loose marks and the words they take the marks' form from hold ${
        String(model.length,)
      }, and the sequences are compared before the marks are written`, },);
  }
  /**
   The character each loose mark of the text is written as, by its position among the text's characters.
   */
  const written = new Map<number, string>(own.map(function formOf(
    mark,
    order,
  ): readonly [
    number,
    string,
  ] {
    /**
     The loose mark of the other text at the same place in its sequence.
     */
    const counterpart = model[order];
    /**
     The character the other text wrote there.
     */
    const character = (counterpart === undefined) ? undefined : modelPoints[counterpart.at];
    if (character === undefined) {
      throw new ConsistencyInvariantError({ invariant:
        `loose mark ${String(order,)} of the words has a counterpart, since both sequences hold ${String(own.length,)}`, },);
    }
    return [
      mark.at,
      character,
    ];
  },),);
  return points
    .map(function inForm(
      point,
      at,
    ): string {
      return written.get(at,) ?? point;
    },)
    .join('',);
}

//endregion Consistency loose marks
