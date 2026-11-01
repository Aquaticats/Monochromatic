import { edgeUnitsOf, } from './consistency-edge-units.ts';
import { isSingleMark, } from './consistency-loose-marks.ts';
import { isClosing, } from './consistency-quote-bounds.ts';
import { continuesLatinWordOrCompatibilityForm, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Consistency edge marks
// WHICH CHARACTERS AT THE TWO EDGES OF A QUOTE ARE NOT ITS WORDS. A judge copies
// the words of a place's English with the sentence punctuation that closed them
// (`consistency-quote-bounds.ts`), and also with the quotation marks or the
// brackets the English happened to hold around them: `"So she was kind".` and
// `So she was kind` name one span. A double quotation mark or a bracket is an
// edge mark only where the words it encloses do not need it: one that pairs with
// one INSIDE the quote belongs to the words (`"So she was kind," she said`, `So
// she was (kind)`), since moving the words without the mark would leave the other
// one stranded, and no floor reads quotation marks. The same holds for the end of
// the quote: a backslash escape and a character reference the parser decodes are
// each one unit with the character they show (`consistency-edge-units.ts`), and
// the unit is kept or taken off exactly as that character would be (`cats\.` goes
// like `cats.`, `dogs&#39;` like `dogs'`, `cats &amp;` stays like `cats &`), so an
// edge never falls between a backslash and its character or inside a reference.
//
// THE FOLD HAS TURNED CURLY QUOTES, SINGLE AND DOUBLE, AND THE CORNER BRACKETS
// INTO THE STRAIGHT ONES (`quote-normalize.ts`), so only the straight marks and
// the ASCII brackets are met here, and the modifier letter apostrophe (U+02BC),
// which the fold leaves as it is.
//
// EVERY SINGLE MARK AT AN EDGE IS TAKEN OFF, whether or not it pairs with
// anything in the quote, and no guess about what it is decides an edge. The quote
// alone cannot say whether `'The cat naps` opens a quotation the judge copied half
// of or `the cats'` ends a plural, and a mark that stays on the words because a
// guess paired it with one inside (`'the cats' bowl`) is inside the span and is
// deleted by a replacement. The words a quote names therefore never carry a single
// mark at an edge, two judges who copied the same words with and without it name
// the same words. The mark is not lost: `edgeFreeRangeOf` says where the words
// stand, so the locator (`consistency-fold.ts`) can read the mark that stood
// directly beside them. A mark that is a proven apostrophe
// (`consistency-proven-apostrophes.ts`) is taken into the span with its word, and
// every other mark is left where the English holds it, outside the span and
// recorded beside it; the splice (`consistency-splice.ts`) refuses to move words
// between places that differ in a mark beside them, in a proven apostrophe at an
// edge or in a quotation mark inside them.
//
// ONE READING OF THE WHOLE QUOTE, THEN ONE STEP PER CHARACTER TAKEN OFF. Which
// bracket closes which, and how many double marks and brackets of each kind stand
// before each position are worked out once; every question an edge asks after
// that (does this mark pair inside what is left, is what is left balanced) is
// answered from those tables without reading the words again, so the time taken
// grows with the length of the quote and not with its square.
//
// A DOUBLE MARK BETWEEN TWO WORD CHARACTERS PAIRS WITH NOTHING, and a word character is the
// neighbour test of the consistency pass (`continuesLatinWordOrCompatibilityForm`,
// `latin-letters.ts`: a Latin letter, an ASCII digit or a compatibility character of those, so
// `𝓠"𝓾` is read as inside a word like `s"u`); the rest of the package tests a neighbour with
// `continuesLatinWord`, as written.
//
// A FIXED POINT: edges are taken off one character at a time until neither edge
// moves. A mark exposed by taking another off (`("cat").`) goes too, and the
// words of a quote that has been read once are read again unchanged.

/**
 A bracket pair: an opening bracket and its closing bracket.

 @example
 ```ts
 const pair: BracketPair = { opener: '(', closer: ')', };
 ```
 */
type BracketPair = {
  /**
   The opening bracket.
   */
  readonly opener: string;

  /**
   The closing bracket.
   */
  readonly closer: string;
};

/**
 The bracket pairs the words may hold.
 */
const BRACKET_PAIRS: readonly BracketPair[] = [
  {
    opener: '(',
    closer: ')',
  },
  {
    opener: '[',
    closer: ']',
  },
  {
    opener: '{',
    closer: '}',
  },
];

/**
 The straight double quotation mark, which opens and closes with one character and pairs only by counting.
 */
const DOUBLE_MARK = '"';

/**
 Position value of a bracket that pairs with no other.
 */
const NO_PARTNER = -1;

/**
 How many characters of one kind stand before each position of the words: the
 count at a position covers the positions before it, so the count between two
 positions is one subtraction.

 @param points - the words, one whole character each

 @param counts - whether the character at a position is of the kind

 @returns The counts, one longer than the words

 @example
 ```ts
 const before = countsBefore({ points: Array.from('(a)(',), counts: ({ point, },) => point === '(', },); // [0, 1, 1, 1, 2]
 ```
 */
function countsBefore(
  {
    points,
    counts,
  }: {
    readonly points: readonly string[];
    readonly counts: ({
      point,
      at,
    }: {
      readonly point: string;
      readonly at: number;
    },) => boolean;
  },
): readonly number[] {
  /**
   The counts so far, mutated only inside this function.
   */
  const before = [0,];
  for (const [at, point,] of points.entries()) {
    before.push((before[at] ?? 0) + (counts({
      point,
      at,
    },)
      ? 1
      : 0),);
  }
  return before;
}

/**
 The bracket each bracket of one pair closes or is closed by, read in order with
 the openers on a stack.

 @param points - the words, one whole character each

 @param pair - the bracket pair read

 @returns The position of each bracket's partner, `-1` where the character is
 no bracket of the pair or pairs with none

 @example
 ```ts
 const partners = bracketPartnersOf({ points: Array.from('(a)',), pair: { opener: '(', closer: ')', }, },); // [2, -1, 0]
 ```
 */
function bracketPartnersOf(
  {
    points,
    pair,
  }: {
    readonly points: readonly string[];
    readonly pair: BracketPair;
  },
): readonly number[] {
  /**
   The partner of each position, mutated only inside this function.
   */
  const partners = points.map(function noPartner(): number {
    return NO_PARTNER;
  },);
  /**
   Positions of the openers not yet closed, innermost last, mutated only inside this function.
   */
  const open: number[] = [];
  for (const [at, point,] of points.entries()) {
    if (point === pair.opener) {
      open.push(at,);
    }
    else if (point === pair.closer) {
      /**
       The innermost opener still waiting, none when the closer closes nothing.
       */
      const opener = open.pop();
      if (opener !== undefined) {
        partners[opener] = at;
        partners[at] = opener;
      }
    }
  }
  return partners;
}

/**
 What one bracket pair's brackets are among the words.
 */
type BracketTable = {
  /**
   The pair.
   */
  readonly pair: BracketPair;

  /**
   The position of each bracket's partner, `-1` where there is none.
   */
  readonly partners: readonly number[];

  /**
   How many openers of the pair stand before each position.
   */
  readonly openersBefore: readonly number[];

  /**
   How many closers of the pair stand before each position.
   */
  readonly closersBefore: readonly number[];
};

/**
 Everything an edge asks about the words, worked out once.
 */
type EdgeTables = {
  /**
   The words, one whole character each.
   */
  readonly points: readonly string[];

  /**
   How many double quotation marks that can pair (those not between two word characters) stand before each position.
   */
  readonly doublesBefore: readonly number[];

  /**
   One table per bracket pair.
   */
  readonly brackets: readonly BracketTable[];
};

/**
 Reads the words once for everything their edges will ask.

 @param points - the words, one whole character each

 @returns The tables

 @example
 ```ts
 const tables = edgeTablesOf({ points: Array.from('"cat"',), },);
 ```
 */
function edgeTablesOf({ points, }: { readonly points: readonly string[]; },): EdgeTables {
  return {
    points,
    doublesBefore: countsBefore({
      points,
      counts: function pairsByCounting({
        point,
        at,
      },): boolean {
        return (point === DOUBLE_MARK)
          && (!(continuesLatinWordOrCompatibilityForm({ character: points[at - 1] ?? '', },)
            && continuesLatinWordOrCompatibilityForm({ character: points[at + 1] ?? '', },)));
      },
    },),
    brackets: BRACKET_PAIRS.map(function tableOf(pair,): BracketTable {
      return {
        pair,
        partners: bracketPartnersOf({
          points,
          pair,
        },),
        openersBefore: countsBefore({
          points,
          counts: function isOpener({ point, },): boolean {
            return point === pair.opener;
          },
        },),
        closersBefore: countsBefore({
          points,
          counts: function isCloser({ point, },): boolean {
            return point === pair.closer;
          },
        },),
      };
    },),
  };
}

/**
 How many counted characters stand in a stretch of the words.

 @param before - the counts before each position

 @param from - first position of the stretch

 @param to - exclusive position the stretch stops at

 @returns The count, none for an empty stretch

 @example
 ```ts
 const inside = countBetween({ before: [0, 1, 1, 2,], from: 1, to: 3, },); // 1
 ```
 */
function countBetween(
  {
    before,
    from,
    to,
  }: {
    readonly before: readonly number[];
    readonly from: number;
    readonly to: number;
  },
): number {
  return (to <= from) ? 0 : ((before[to] ?? 0) - (before[from] ?? 0));
}

/**
 The stretch of the words that is left: what an edge question is asked about.
 */
type Stretch = {
  /**
   The tables of all the words.
   */
  readonly tables: EdgeTables;

  /**
   First position left.
   */
  readonly from: number;

  /**
   Exclusive position what is left stops at.
   */
  readonly to: number;
};

/**
 Whether the first and last characters left are one mark and its pair, around
 words that hold that mark in order: the mark encloses all of them.

 @param tables - the tables of all the words

 @param from - first position left

 @param to - exclusive position what is left stops at

 @returns Whether the first and last characters enclose what is between them

 @example
 ```ts
 const encloses = enclosesAll({ tables: edgeTablesOf({ points: Array.from('"cat"',), },), from: 0, to: 5, },); // true
 ```
 */
function enclosesAll(
  {
    tables,
    from,
    to,
  }: Stretch,
): boolean {
  if ((to - from) < 2)
    return false;
  /**
   The first character left.
   */
  const first = tables.points[from] ?? '';
  /**
   Position of the last character left.
   */
  const lastAt = to - 1;
  if (first === DOUBLE_MARK) {
    return (tables.points[lastAt] === DOUBLE_MARK) && ((countBetween({
      before: tables.doublesBefore,
      from: from + 1,
      to: lastAt,
    },) % 2) === 0);
  }
  return tables.brackets
    .some(function enclosesWith({
      pair,
      partners,
    },): boolean {
      return (first === pair.opener) && (partners[from] === lastAt);
    },);
}

/**
 Whether the first character left is not one of the words.

 @param tables - the tables of all the words

 @param from - first position left

 @param to - exclusive position what is left stops at

 @returns Whether to take it off: closing punctuation, space or an invisible
 character; a double quotation mark that pairs with nothing in the rest by
 counting; any single mark; an opening bracket with no closing one in the rest;
 or a closing bracket, which can close nothing before it

 @example
 ```ts
 const off = dropsAtStart({ tables: edgeTablesOf({ points: Array.from('"cat naps',), },), from: 0, to: 9, },); // true
 ```
 */
function dropsAtStart(
  {
    tables,
    from,
    to,
  }: Stretch,
): boolean {
  /**
   The first character left.
   */
  const point = tables.points[from] ?? '';
  if (isClosing({ point, },))
    return true;
  if (point === DOUBLE_MARK) {
    return (countBetween({
      before: tables.doublesBefore,
      from: from + 1,
      to,
    },) % 2) === 0;
  }
  if (isSingleMark({ character: point, },))
    return true;
  return tables.brackets
    .some(function dropsBracket({
      pair,
      closersBefore,
    },): boolean {
      if (point === pair.closer)
        return true;
      return (point === pair.opener) && (countBetween({
        before: closersBefore,
        from: from + 1,
        to,
      },) === 0);
    },);
}

/**
 Whether the last character left is not one of the words.

 @param tables - the tables of all the words

 @param from - first position left

 @param to - exclusive position what is left stops at

 @returns Whether to take it off: closing punctuation, space or an invisible
 character; a double quotation mark that pairs with nothing before it by
 counting; any single mark; a closing bracket with no opening one before it;
 or an opening bracket, which can open nothing after it. The tables hold what each
 unit shows, so an escape or a reference is taken off whole or kept whole

 @example
 ```ts
 const off = dropsAtEnd({ tables: edgeTablesOf({ points: Array.from('cat naps."',), },), from: 0, to: 10, },); // true
 ```
 */
function dropsAtEnd(
  {
    tables,
    from,
    to,
  }: Stretch,
): boolean {
  /**
   Position of the last character left.
   */
  const lastAt = to - 1;
  /**
   The last character left.
   */
  const point = tables.points[lastAt] ?? '';
  if (isClosing({ point, },))
    return true;
  if (point === DOUBLE_MARK) {
    return (countBetween({
      before: tables.doublesBefore,
      from,
      to: lastAt,
    },) % 2) === 0;
  }
  if (isSingleMark({ character: point, },))
    return true;
  return tables.brackets
    .some(function dropsBracket({
      pair,
      openersBefore,
    },): boolean {
      if (point === pair.opener)
        return true;
      return (point === pair.closer) && (countBetween({
        before: openersBefore,
        from,
        to: lastAt,
      },) === 0);
    },);
}

/**
 Where the words of a quote stand among its characters once the marks and
 punctuation at its two edges are taken off, read to a fixed point.

 @param points - the folded quote, one whole character each

 @param originals - the characters as the judge wrote them, one for each of the points, which tell an escape from a backslash
 before a curly mark the fold turned straight; the points themselves where absent

 @returns The position of the first character that is one of the words and the
 exclusive position after the last, equal when the quote holds none, an escape or a reference never cut in two

 @example
 ```ts
 const { from, to, } = edgeFreeRangeOf({ points: Array.from('"cat naps".',), },); // { from: 1, to: 9 }
 ```
 */
export function edgeFreeRangeOf(
  {
    points,
    originals = points,
  }: {
    readonly points: readonly string[];
    readonly originals?: readonly string[];
  },
): {
  readonly from: number;
  readonly to: number;
} {
  /**
   The quote one unit at a time: a backslash with the character it escapes and a reference the parser decodes are one.
   */
  const units = edgeUnitsOf({
    points,
    originals,
  },);
  /**
   Everything the edges will ask about the words, over what each unit shows.
   */
  const tables = edgeTablesOf({
    points: units.map(function shownBy({ shown, },): string {
      return shown;
    },),
  },);
  /**
   What is left of the words, as indexes among the units, and whether an edge moved in the last reading, mutated only inside this
   function.
   */
  const left = {
    from: 0,
    to: units.length,
    moved: true,
  };
  while (left.moved && (left.from < left.to)) {
    // A MARK THAT ENCLOSES ALL THE WORDS GOES WITH ITS PAIR, both at once: taken
    // one at a time, the first off would leave the other pairing with nothing
    // inside and the pair split.
    if (enclosesAll({
      tables,
      from: left.from,
      to: left.to,
    },)) {
      left.from += 1;
      left.to -= 1;
      continue;
    }
    /**
     Whether the start is taken off in this reading.
     */
    const startOff = dropsAtStart({
      tables,
      from: left.from,
      to: left.to,
    },);
    if (startOff)
      left.from += 1;
    /**
     Whether the end is, read after the start.
     */
    const endOff = (left.from < left.to) && dropsAtEnd({
      tables,
      from: left.from,
      to: left.to,
    },);
    if (endOff)
      left.to -= 1;
    left.moved = startOff || endOff;
  }
  /**
   The first unit left, as a list of zero or one.
   */
  const [firstLeft,] = units.slice(
    left.from,
    left.from + 1,
  );
  /**
   Index among the characters where the first unit left starts, past the last character when none is left.
   */
  const from = (firstLeft === undefined) ? points.length : firstLeft.from;
  /**
   The last unit left, as a list of zero or one; none when the words are empty.
   */
  const [lastLeft,] = (left.to > left.from)
    ? units.slice(
      left.to - 1,
      left.to,
    )
    : [];
  return {
    from,
    to: (lastLeft === undefined) ? from : lastLeft.to,
  };
}

/**
 The words of a quote once the marks and punctuation at its two edges are taken
 off, read to a fixed point.

 @param points - the folded quote, one whole character each

 @param originals - the characters as the judge wrote them, one for each of the points; the points themselves where absent

 @returns The words, empty when the quote holds none

 @example
 ```ts
 const words = edgeFreeWordsOf({ points: Array.from('"cat naps".',), },); // ['c', 'a', 't', ' ', 'n', 'a', 'p', 's']
 ```
 */
export function edgeFreeWordsOf(
  {
    points,
    originals = points,
  }: {
    readonly points: readonly string[];
    readonly originals?: readonly string[];
  },
): readonly string[] {
  /**
   Where the words stand.
   */
  const {
    from,
    to,
  } = edgeFreeRangeOf({
    points,
    originals,
  },);
  return points.slice(
    from,
    to,
  );
}

//endregion Consistency edge marks
