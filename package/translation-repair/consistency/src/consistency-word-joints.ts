import {
  codePointAt,
  codePointBefore,
  continuesLatinWordOrCompatibilityForm,
  isAsciiDigit,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { APOSTROPHE_JOINERS, } from './consistency-closing-marks.ts';
import { isCornerBracket, } from './consistency-corner-brackets.ts';
import {
  type FoldedLinks,
  NO_FOLDED_LINKS,
} from './consistency-folded-links.ts';
import {
  CLOSING_BRACKETS,
  CLOSING_CURLY_DOUBLE,
} from './consistency-markup-runs.ts';

//region Consistency word joints
// WHERE A WORD GOES ON PAST WHAT LOOKS LIKE ITS EDGE, for the locator of a
// judge's quote (`consistency-fold.ts`) alone; the glossary's reading in
// `word-bounds.ts` is unchanged. A judge quotes a place, and a quote must not
// be found inside a longer token, so each mark that makes one token of what
// stands on both of its sides is a joint. A STAND is the stretch of the page's English that
// a judge's quote is found as (`consistency-fold.ts`, with its start and end offsets); a
// stand that a joint cuts is no stand, and the quote is not found there.
//
// "A LETTER OR A DIGIT" BELOW IS THE ONE NEIGHBOUR TEST OF THE CONSISTENCY PASS: a Latin letter with
// its accents, composed or as a combining mark, or an ASCII digit `0` to `9`, or a whole
// character that is itself a letter or a decimal digit and whose NFKC form is made of those alone (a mathematical script
// letter, a fullwidth letter or digit, a ligature; `continuesLatinWordOrCompatibilityForm` in
// `latin-letters.ts`; D3: a trade mark sign, a superscript or circled digit and a Roman numeral are none). The rest of the
// package, the glossary's reading in `word-bounds.ts` among it, tests
// a neighbour with `continuesLatinWord`, as written. A
// Greek letter, a Han character, an Arabic-Indic digit, a symbol and an emoji beside a hyphen or an
// apostrophe make no joint. A joint asks only whether a letter stands on each side, never what parts a mark from a word, so
// it keeps the wide run of closing marks of the rule for an apostrophe after markup (D4 narrows the run a trailing mark is READ past, in
// `consistency-markup-runs.ts`, and leaves this one). The
// loose-mark reader (`consistency-loose-marks.ts`), the proof of an apostrophe
// (`consistency-proven-apostrophes.ts`) and the form reader ask the same question of a
// character beside a single mark, so a mark this file reads as joining is a mark they
// read as attached, and the word the locator finds is the word the splice moves.
// A LINK THAT SHOWS A READER NOTHING (`[](u)`, `![](u)`, a reference with an empty label) NEITHER ENDS A WORD NOR STARTS ONE (n169):
// every reading beside a stand, forward and backward, steps over a run of such links as it steps over the delimiters of emphasis, in
// any order with them, so `kit[](u)'kit` reads as `kit'kit` and `kit-[](u)kit` as `kit-kit` (`pastHiddenUnits`).
// A LINK WITH A LABEL SHOWS A READER ITS LABEL AND NOTHING ELSE (n173): its opening square bracket and its resource (`](u)`,
// `][r]`, `]`) are stepped over the same way, so `kit[kit](u)kit` reads as `kitkitkit` (no stand of `kit`), `kit[-](u)kit` as
// `kit-kit` and `kit [kit](u)kit` as `kit kitkit`; the label's own characters are the neighbours there.
// THE RULE, by mark:
//
// - A hyphen joins when each side is a letter or a digit: `so` is no stand
//   inside `so-called`, `Catch` and `22` none inside `Catch-22`, `cat` none
//   inside `cat-5`, `1000` none inside `T-1000`, `3` none inside `3-D`, `10`
//   none inside `2026-10-13` and, a range being written the same way, `10`
//   none inside `pages 10-12`. A hyphen beside a space (`cats - and dogs`,
//   `cat- nap`) or the edge of the text joins nothing.
// - An apostrophe joins when each side is a letter or a digit, as the hyphen
//   does: `t` is no stand inside `don't`, `dog` none inside `dog's`, `2 tall`
//   none inside `5'2 tall` (a span starting there would strand the mark between
//   the two digits), `5` none inside `cat'5` and `90s` none inside `mid'90s`.
//   The plural of a decade is one such word: `1990` is no stand inside `1990's`,
//   the same answer `cat` gets inside `cat's`. An apostrophe with nothing joined before it (`'90s`, `'em`, `'Tis`), or a
//   space or the edge of the text on a side, joins nothing.
// - An apostrophe after markup joins too, so that a stand which ends in the markup, or
//   stands before it, is cut as the bare word is. The reading goes forward from the end
//   of a stand and backward from an apostrophe.
//   FORWARD (`consistency-closing-marks.ts`): from the end of a stand, the reading steps over a run of closing marks, which
//   are the closing emphasis marks (`*`, `_` and `~`), a straight or curly closing double quotation
//   mark, a closing round or square bracket, and the destination `(...)` of a link directly
//   after a `]`. When an apostrophe followed by a letter or a digit comes next, the stand is
//   inside a word and is no stand, whether or not the stand itself ends in one of those marks.
//   So `cat` is no stand inside `"cat"'s`, `“cat”’s`, `(grey cat)'s`, `[grey cat]'s` or
//   `**cat**'s`, `the cat` none inside `[the cat](x)'s`, `"hi"` none inside `“hi”’s` and
//   `the **cat**` none inside `the **cat**'s bowl`. WHICH EMPHASIS MARKS CLOSE IS THE MARKDOWN
//   PARSE'S ANSWER (`emphasisDelimitersOf` in `consistency-prose-text.ts`): a delimiter closes
//   when the parse of the English ends an emphasis, strong or delete node there, and a lone
//   asterisk or underscore that closes nothing is a character (`The cat*'s bowl` holds none, so
//   `cat` is a stand in it, while `*cat*'s` cuts it). A STRAIGHT DOUBLE QUOTATION MARK KEEPS A
//   TEST OF SHAPE, since a parse cannot pair straight marks across a page: it closes when the
//   character before it is neither white space nor an opening mark (`(`, `[`, `{`, `“`, `‘`,
//   `「`, `『`, `«`), so `"cat"'s` cuts `cat` and `("'Tis")` does not. A curly opening double
//   quotation mark and a corner bracket close nothing. A run of
//   any number of closing marks is stepped over, and so is a link destination of any length
//   (one with white space in it, a title, or with no closing bracket is not read). The
//   reading is made once for the whole English, from its end to its start, and a stand asks
//   it in constant time, so a long run costs each stand nothing and the unit test of this
//   file counts the reads for the whole English.
//   BACKWARD: an apostrophe directly followed by a letter is a joint when, reading backward
//   over any run of emphasis delimiters of the parse, the next character is a letter or a digit,
//   or when the character directly before the apostrophe is `)`, `]` or a closing curly double
//   quotation mark. So `Yorushika` is no stand inside `*Yorushika*’s` and `s bowl` none inside
//   `[the cat](x)'s bowl`. A straight `"` before the apostrophe is not read backward, since
//   `"'Tis"` shows it can open a quotation, so a stand that starts right after the
//   apostrophe of `"cat"'s bowl` (`s bowl`) is located.
//   Not a joint: `*'Tis the season*` (beyond the emphasis stands a space or the start of the
//   text), `('em)`, `“'Tis”`, an apostrophe with a space or a quotation mark after it
//   (`"hi"' now`), and a link destination that is not closed. The backward reading steps
//   over emphasis marks only from an apostrophe, so a long run of them beside none costs a
//   constant number of reads for each position, which the unit test of this file counts.
// - A full stop, a comma, a colon or a slash joins when a digit stands on both
//   sides: `5 dollars` is no stand inside `0.5 dollars`, `500` none inside
//   `1,500`, `30` none inside `10:30` and `1` none inside `1/2`. Between
//   letters, or beside a space, these four join nothing (`cat/dog`).
// - A Greek letter, a Han character or an emoji beside a word joins nothing.
//
// THE HYPHEN is the ASCII one, the Unicode hyphen, the non-breaking hyphen or
// the soft hyphen; THE APOSTROPHE is the straight one every curly single mark
// folds to, or the modifier letter, and never a corner bracket, which the fold
// writes as the same character (`『cat』and` holds the word `cat`). The opening curly
// single mark folds to the straight one too, so the locator refuses a quote that
// stops inside `cat‘s`, although the loose-mark reader
// (`consistency-loose-marks.ts`) reads a `‘` between two letters as a quotation
// mark and never as an apostrophe attached to its word: the two agree that the
// quote is not a whole word, and differ on what the mark is. The backtick,
// the acute accent U+00B4 and the prime U+2032 are no apostrophe here, and join
// nothing. Emphasis marks between a word and its joint hide nothing: `so` is no
// stand in `**so**-called`. A dash that parts two clauses (U+2013 and U+2014,
// which stay no joint even between two numbers) and a slash between two
// alternatives join nothing, since each side is a whole word.

/**
 Marks that join the letter or digit before them to the letter or digit after
 them: the ASCII hyphen, the Unicode hyphen, the non-breaking hyphen and the
 soft hyphen.
 */
const HYPHEN_JOINERS: ReadonlySet<string> = new Set([
  '-',
  '\u{2010}',
  '\u{2011}',
  '\u{00AD}',
],);

/**
 Marks that join the two digits beside them into one number: the full stop, the
 comma, the colon and the slash.
 */
const DIGIT_JOINERS: ReadonlySet<string> = new Set([
  '.',
  ',',
  ':',
  '/',
],);

/**
 Every mark that can join the characters beside it, whether it is a hyphen, an apostrophe or one of the marks that join
 digits, so that a unit that is none of them is known to be no joint without reading its neighbours.
 */
const JOINING_MARKS: ReadonlySet<string> = new Set([
  ...HYPHEN_JOINERS,
  ...APOSTROPHE_JOINERS,
  ...DIGIT_JOINERS,
],);

/**
 Whether a letter or a digit stands on each side of a hyphen or an apostrophe.

 @param before - the whole character before the mark

 @param after - the whole character after the mark

 @returns Whether the mark has a Latin letter or a digit on each side, so a mark
 with a space, an edge of the text or any other character on a side joins nothing

 @example
 ```ts
 const beside = besideLettersOrDigits({ before: '5', after: '2', },); // true
 ```
 */
function besideLettersOrDigits(
  {
    before,
    after,
  }: {
    readonly before: string;
    readonly after: string;
  },
): boolean {
  return continuesLatinWordOrCompatibilityForm({ character: before, },)
    && continuesLatinWordOrCompatibilityForm({ character: after, },);
}

/**
 Whether a character joins the two characters beside it into one word: a hyphen
 or an apostrophe between two letters or digits, a full stop, a comma, a colon
 or a slash between two digits. A corner bracket that the fold wrote as the
 apostrophe joins nothing.

 @param joiner - the character asked about, one UTF-16 unit, empty past an edge of the text

 @param original - the character the English held there before the fold

 @param before - the whole character before it

 @param after - the whole character after it

 @returns Whether the character is a joint

 @example
 ```ts
 const joint = joins({ joiner: '-', original: '-', before: 'o', after: 'c', },); // true
 ```
 */
function joins(
  {
    joiner,
    original,
    before,
    after,
  }: {
    readonly joiner: string;
    readonly original: string;
    readonly before: string;
    readonly after: string;
  },
): boolean {
  if (HYPHEN_JOINERS.has(joiner,)) {
    return besideLettersOrDigits({
      before,
      after,
    },);
  }
  if (APOSTROPHE_JOINERS.has(joiner,)) {
    return (!isCornerBracket({ original, },)) && besideLettersOrDigits({
      before,
      after,
    },);
  }
  return DIGIT_JOINERS.has(joiner,)
    && isAsciiDigit({ character: before, },)
    && isAsciiDigit({ character: after, },);
}

/**
 No delimiter of emphasis: the reading that steps over links that show nothing alone passes this in place of the parse's delimiters.
 */
const NO_EMPHASIS: ReadonlySet<number> = new Set();

/**
 The first offset, from one offset on in one direction, that holds neither a delimiter of emphasis nor a unit of a link that shows
 nothing. A link that shows nothing (`[](u)`, `![](u)`) is stepped over whole, from its first unit forward or from its last unit
 backward, and may stand between delimiters of emphasis or between runs of them, so the reading takes them in any order and any
 number (n169). The opening square bracket and the resource of a link with a label are stepped over the same way, one stretch at a
 time, and the label between them is left to be read (n173).

 @param text - folded English

 @param emphasis - the offsets in the fold of the delimiters of emphasis, strong emphasis and strikethrough by the parse; a
 lone asterisk or underscore is none

 @param links - the links the parse holds, at the offsets of the fold

 @param from - offset the reading starts at

 @param step - `1` to read on, `-1` to read back

 @returns The offset, which may lie outside the text when delimiters and links run to its edge

 @example
 ```ts
 const at = pastHiddenUnits({ text: '**so**-called', emphasis: new Set([0, 1, 4, 5,],), links: NO_FOLDED_LINKS, from: 4, step: 1, },); // 6
 ```
 */
function pastHiddenUnits(
  {
    text,
    emphasis,
    links,
    from,
    step,
  }: {
    readonly text: string;
    readonly emphasis: ReadonlySet<number>;
    readonly links: FoldedLinks;
    readonly from: number;
    readonly step: -1 | 1;
  },
): number {
  /**
   The links that show nothing, by their last unit and by their first.
   */
  const {
    emptyStarts,
    emptyEnds,
    hiddenStarts,
    hiddenEnds,
  } = links;
  /**
   The offset reached, mutated only inside this function.
   */
  const reached = { at: from, };
  while ((reached.at >= 0) && (reached.at < text.length)) {
    if (emphasis.has(reached.at,)) {
      reached.at += step;
      continue;
    }
    /**
     The first unit of the link, or of the stretch of a link with a label, that ends at the offset reached, absent where none ends there.
     */
    const first = emptyStarts.get(reached.at,) ?? hiddenStarts.get(reached.at,);
    /**
     The offset past the link or the stretch that starts at the offset reached, reading on, or before the one that ends at it, reading
     back, absent where none shows nothing there.
     */
    const past = (step === 1)
      ? (emptyEnds.get(reached.at,) ?? hiddenEnds.get(reached.at,))
      : ((first === undefined) ? undefined : (first - 1));
    if (past === undefined)
      break;
    reached.at = past;
  }
  return reached.at;
}

/**
 Where the run of empty links directly before a unit starts: the offset itself when no empty link ends directly before it.
 Each step moves to the start of one link, so the reading is one pass over the run.

 @param links - the links the parse holds, at the offsets of the fold

 @param at - offset of the unit asked about

 @returns The offset of the first unit of the run, `at` for no run

 @example
 ```ts
 const start = startOfEmptyLinksBefore({ links: { ...NO_FOLDED_LINKS, emptyStarts: new Map([[6, 2,],],), }, at: 7, },); // 2
 ```
 */
function startOfEmptyLinksBefore(
  {
    links,
    at,
  }: {
    readonly links: FoldedLinks;
    readonly at: number;
  },
): number {
  /**
   For the last unit of each empty link, the offset of its first unit.
   */
  const { emptyStarts, } = links;
  /**
   Where the run starts so far, moved back over one link at a time, mutated only inside this function.
   */
  const run = { start: at, };
  for (
    let first = emptyStarts.get(run.start - 1,);
    first !== undefined;
    first = emptyStarts.get(run.start - 1,)
  )
    run.start = first;
  return run.start;
}

/**
 Whether the unit at an offset is an apostrophe that joins what stands before it
 to the letter after it, where what stands before it may be markup. An apostrophe
 (the characters {@link APOSTROPHE_JOINERS} lists, never a corner bracket the fold
 wrote as one) directly followed by a letter or a digit is a joint when, reading
 backward over any run of emphasis marks, the next character is a letter or a
 digit, so `s bowl` starts after the joint of `*Yorushika*’s bowl`; and it is a joint
 when a letter follows it and the character directly before it is a closing round or
 square bracket or a closing curly double quotation mark, so `s bowl` starts after
 the joint of `[the cat](x)'s bowl`. Not a joint: an apostrophe with a space, an
 opening mark or the start of the text where the backward reading stops
 (`*'Tis the season*`, `('em)`), one after a straight double quotation mark
 (`"'Tis"`, and `s bowl` in `"cat"'s bowl`), which cannot say whether it closes or
 opens, one after a link whose label shows marks or spaces and no letter or digit (`[-](u)'kit`), one after a link whose label ends in an open mark, neither a closing mark nor a letter or a digit (`[kit!](u)'s`, `[kit ](u)'s`), and one that no letter follows.
 A link whose label shows nothing (`[](u)`, `[](u "t")`, `[]()`, a reference with an empty label, an image with no alt text) is no
 neighbour of the apostrophe at all: the characters before the run of such links directly before the apostrophe are read as if the
 links were not there, so `a [](u)'kit` is no joint and `kit[](u)'kit` is one. A stand that ends before the apostrophe is
 read forward instead, by {@link closingMarksThenApostrophe}.

 @param folded - the English in the fold

 @param originals - the character the English held before the fold at each UTF-16 unit of it

 @param emphasis - the offsets in the fold of the delimiters of emphasis by the parse, the only marks read past

 @param links - the links the parse holds, at the offsets of the fold

 @param at - offset of the unit asked about, which may lie outside the text

 @returns Whether the unit is an apostrophe that is a joint

 @example
 ```ts
 const text = 'the **cat**\'s bowl';
 const joint = apostropheJoinsAt({ folded: text, originals: Array.from(text,), emphasis: new Set([4, 5, 9, 10,],), links: NO_FOLDED_LINKS, at: 11, },); // true
 ```
 */
function apostropheJoinsAt(
  {
    folded,
    originals,
    emphasis,
    links,
    at,
  }: {
    readonly folded: string;
    readonly originals: readonly string[];
    readonly emphasis: ReadonlySet<number>;
    readonly links: FoldedLinks;
    readonly at: number;
  },
): boolean {
  if (!APOSTROPHE_JOINERS.has(folded.charAt(at,),))
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
  /**
   Offset where the units a reader sees before the apostrophe end: the apostrophe's own, or the start of the run of empty links
   directly before it.
   */
  const seen = startOfEmptyLinksBefore({
    links,
    at,
  },);
  /**
   Offset of the last character before the apostrophe that is no emphasis mark and no unit of a link that shows nothing, none when
   they run to the start.
   */
  const beforeHidden = pastHiddenUnits({
    text: folded,
    emphasis,
    links,
    from: at - 1,
    step: -1,
  },);
  if (continuesLatinWordOrCompatibilityForm({
    character: codePointBefore({
      text: folded,
      at: beforeHidden + 1,
    },),
  },) && continuesLatinWordOrCompatibilityForm({ character: after, },))
    return true;
  // A LINK THAT SHOWS NO WORD CONTINUES NO WORD RUN (n166 Z9, n168): the `)` of `[-](u)` is a closing bracket that nothing a reader
  // sees as a word stands before, so the apostrophe after it is no joint (`consistency-link-label.ts`). A link that shows nothing is
  // stepped over by `startOfEmptyLinksBefore`, so what is read here is what stands before it.
  /**
   The last unit of each link whose label shows marks and no letter or digit, and of each link whose label ends in an open mark.
   */
  const {
    markOnlyEnds,
    openLabelEnds,
  } = links;
  /**
   Whether the apostrophe stands after a closing bracket that continues the word run: a bracket that closes no link of marks and
   no link whose label ends in an open mark (`[kit!](u)'s` reads as `kit!'s`, and `!` closes nothing).
   */
  const afterBracket = CLOSING_BRACKETS.has(folded.charAt(seen - 1,),) && (!markOnlyEnds.has(seen - 1,))
    && (!openLabelEnds.has(seen - 1,));
  return continuesLatinWordOrCompatibilityForm({ character: after, },)
    && (afterBracket || (originals[seen - 1] === CLOSING_CURLY_DOUBLE));
}

/**
 Whether a stand of the needle in a folded English is cut out of a longer word:
 its first or last character is itself a joint, or the character just outside
 either edge is one, read past any emphasis marks that stand between.

 @param folded - the English in the fold

 @param originals - the character the English held before the fold at each UTF-16 unit of it

 @param joints - the offsets of the English at which a stand ends inside a word made with an apostrophe after closing
 marks, read once for the whole English by `closingJointsOf` (`consistency-closing-links.ts`)

 @param emphasis - the offsets in the fold of every delimiter of emphasis, strong emphasis and strikethrough by the parse
 (`consistency-prose-text.ts`), the only marks the reading steps over beside a joint

 @param links - the links the parse holds, at the offsets of the fold (`foldedLinksOf`, `consistency-folded-links.ts`), none where
 the caller read no link

 @param at - offset where the stand starts

 @param end - exclusive offset where it ends

 @returns Whether an edge of the stand is a joint of a longer word

 @example
 ```ts
 const text = 'the so-called cat';
 const originals = Array.from(text,);
 const cut = cutsWord({ folded: text, originals, joints: closingJointsOf({ folded: text, originals, closing: new Set(), },), emphasis: new Set(), at: 4, end: 6, },); // true
 ```
 */
export function cutsWord(
  {
    folded,
    originals,
    joints,
    emphasis,
    links = NO_FOLDED_LINKS,
    at,
    end,
  }: {
    readonly folded: string;
    readonly originals: readonly string[];
    readonly joints: ReadonlySet<number>;
    readonly emphasis: ReadonlySet<number>;
    readonly links?: FoldedLinks;
    readonly at: number;
    readonly end: number;
  },
): boolean {
  /**
   Whether the unit at an offset inside the stand joins its two neighbours.

   @param offset - offset of the unit asked about

   @returns Whether it is a joint
   */
  function isJointAt({ offset, }: { readonly offset: number; },): boolean {
    /**
     The unit asked about, whose neighbours are read only when it is a mark that joins.
     */
    const joiner = folded.charAt(offset,);
    if (!JOINING_MARKS.has(joiner,))
      return false;
    return joins({
      joiner,
      original: originals[offset] ?? '',
      before: codePointBefore({
        text: folded,
        at: offset,
      },),
      after: codePointAt({
        text: folded,
        at: offset + 1,
      },),
    },) || apostropheJoinsAt({
      folded,
      originals,
      emphasis,
      links,
      at: offset,
    },);
  }
  // A stand of one unit has one edge, so its unit is asked about once.
  if (isJointAt({ offset: at, },) || (((end - 1) !== at) && isJointAt({ offset: end - 1, },)))
    return true;
  /**
   The stand's last whole character.
   */
  const last = codePointBefore({
    text: folded,
    at: end,
  },);
  /**
   The stand's first whole character.
   */
  const first = codePointAt({
    text: folded,
    at,
  },);
  // EMPHASIS MARKS ARE READ PAST ONLY FROM AN EDGE THAT CAN BE JOINED. A joint
  // has a letter or a digit on each side, so a stand whose edge is neither has
  // no joint there whatever follows the marks, and a stand that is itself
  // emphasis marks (which stands at every mark of a long run of them) never
  // walks the run: the reading stays linear in the English.
  /**
   Whether a joint stands just after the stand, past any emphasis marks and any links that show nothing.

   @returns Whether the stand's last character is joined to what follows
   */
  function joinsOn(): boolean {
    /**
     Offset of the first character after the stand that is no emphasis mark and no unit of a link that shows nothing.
     */
    const after = pastHiddenUnits({
      text: folded,
      emphasis,
      links,
      from: end,
      step: 1,
    },);
    return joins({
      joiner: folded.charAt(after,),
      original: originals[after] ?? '',
      before: last,
      after: codePointAt({
        text: folded,
        at: pastHiddenUnits({
          text: folded,
          emphasis,
          links,
          from: after + 1,
          step: 1,
        },),
      },),
    },);
  }
  /**
   Whether a joint stands just before the stand, past any emphasis marks and any links that show nothing.

   @returns Whether the stand's first character is joined to what precedes
   */
  function joinsBack(): boolean {
    /**
     Offset of the last character before the stand that is no emphasis mark and no unit of a link that shows nothing.
     */
    const before = pastHiddenUnits({
      text: folded,
      emphasis,
      links,
      from: at - 1,
      step: -1,
    },);
    return joins({
      joiner: (before < 0) ? '' : folded.charAt(before,),
      original: originals[before] ?? '',
      before: codePointBefore({
        text: folded,
        at: pastHiddenUnits({
          text: folded,
          emphasis,
          links,
          from: before - 1,
          step: -1,
        },) + 1,
      },),
      after: first,
    },);
  }
  /**
   Whether a link that shows nothing stands just after the stand, with a letter or a digit directly beyond it, so that the text on
   the two sides of the link is one word to a reader (`kit[](u)tens`).

   @returns Whether the stand's last character runs on past a link that shows nothing
   */
  function runsOnPastLinks(): boolean {
    /**
     Offset of the first unit after the stand that is no unit of a link that shows nothing.
     */
    const after = pastHiddenUnits({
      text: folded,
      emphasis: NO_EMPHASIS,
      links,
      from: end,
      step: 1,
    },);
    return (after > end) && continuesLatinWordOrCompatibilityForm({
      character: codePointAt({
        text: folded,
        at: after,
      },),
    },);
  }
  /**
   Whether a link that shows nothing stands just before the stand, with a letter or a digit directly beyond it, so that the text on
   the two sides of the link is one word to a reader (`kit[](u)tens`).

   @returns Whether the stand's first character runs back past a link that shows nothing
   */
  function runsBackPastLinks(): boolean {
    /**
     Offset of the last unit before the stand that is no unit of a link that shows nothing.
     */
    const before = pastHiddenUnits({
      text: folded,
      emphasis: NO_EMPHASIS,
      links,
      from: at - 1,
      step: -1,
    },);
    return (before < (at - 1)) && continuesLatinWordOrCompatibilityForm({
      character: codePointBefore({
        text: folded,
        at: before + 1,
      },),
    },);
  }
  // AN APOSTROPHE AFTER THE STAND IS ASKED OF THE CLOSING JOINTS THE CALLER READ ONCE FOR
  // THE WHOLE TEXT, whatever the stand's last character is, since the stand may end in the
  // markup that stands between the word and its apostrophe (`the **cat**`, a link,
  // `"cat"`) or end before it (`cat` in `"cat"'s`). AN APOSTROPHE BEFORE THE STAND IS
  // READ BACKWARD FROM ITS OWN POSITION over emphasis marks and links that show nothing, which costs a
  // constant number of reads for a stand of closing marks beside no apostrophe.
  return (continuesLatinWordOrCompatibilityForm({ character: last, },) && (joinsOn() || runsOnPastLinks()))
    || (continuesLatinWordOrCompatibilityForm({ character: first, },) && (joinsBack() || runsBackPastLinks()))
    || joints.has(end,)
    || apostropheJoinsAt({
      folded,
      originals,
      emphasis,
      links,
      at: startOfEmptyLinksBefore({
        links,
        at,
      },) - 1,
    },);
}

//endregion Consistency word joints
