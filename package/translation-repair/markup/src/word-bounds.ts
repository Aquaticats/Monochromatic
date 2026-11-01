import {
  codePointAt,
  codePointBefore,
} from './code-points.ts';
import { continuesLatinWord, } from './latin-letters.ts';

//region Word bounds
// ONE READING OF A WORD OR PHRASE INSIDE A TEXT (ledger B23), the glossary's
// rule (ledger C1, `glossary-match.ts`) for callers that do not fold: an edge
// of the needle that is a Latin letter, a digit or a combining mark matches
// only where the text's neighbour at that edge continues no Latin word, and
// a Han edge needs no boundary, since Chinese writes no spaces. A raw
// substring read "as an ai" inside "as an aide", "i cannot" inside "taxi
// cannot" and "load" inside "download". Neighbours are read as whole
// characters (`code-points.ts`, ledger B22).
//
// A WORD STANDING AS A TOKEN OF ITS OWN is the stricter reading, for a word
// that is never a piece of an address, a path, a handle or a compound (the
// neutral pronoun floor, `translate-neutral-pronoun.ts`): `tokenStarts` also
// reads past the joiners those are built with.

/**
 How a needle may end where the text goes on: at a word boundary, or open to
 any continuation (a marker read as the start of a word, so "load" is found
 in "loaded" but not in "download").
 */
export type WordEnd = 'word' | 'open';

/**
 Every start of a needle in a text where each of its Latin edges stands at a
 word boundary.

 @param text - text searched

 @param needle - word or phrase looked for

 @param end - whether the needle's end needs a boundary too

 @returns Start offsets in ascending order, empty where the needle is absent
 or empty

 @example
 ```ts
 wordStarts({ text: 'an aide as an ai', needle: 'as an ai', end: 'word', },); // [8]
 ```
 */
export function wordStarts(
  {
    text,
    needle,
    end,
  }: {
    readonly text: string;
    readonly needle: string;
    readonly end: WordEnd;
  },
): readonly number[] {
  if (needle === '')
    return [];
  /**
   Whether the needle opens on a Latin word character, so its start needs a
   boundary.
   */
  const opensLatin = continuesLatinWord({
    character: codePointAt({
      text: needle,
      at: 0,
    },),
  },);
  /**
   Whether the needle closes on one, so a bounded end needs a boundary.
   */
  const closesLatin = (end === 'word') && continuesLatinWord({
    character: codePointBefore({
      text: needle,
      at: needle.length,
    },),
  },);
  /**
   Starts found so far; a linear cursor scan, since the text is unbounded.
   */
  const starts: number[] = [];
  for (let at = text.indexOf(needle,); at !== (-1); at = text.indexOf(
    needle,
    at + 1,
  )) {
    /**
     Whether the match runs on from a word before it.
     */
    const joinedBefore = opensLatin && continuesLatinWord({
      character: codePointBefore({
        text,
        at,
      },),
    },);
    /**
     Whether a word runs on after it.
     */
    const joinedAfter = closesLatin && continuesLatinWord({
      character: codePointAt({
        text,
        at: at + needle.length,
      },),
    },);
    if ((!joinedBefore) && (!joinedAfter))
      starts.push(at,);
  }
  return starts;
}

/**
 Characters that sit inside an address, a path, a handle or a compound, so a
 word beside one runs on into a longer token: `meta.ta`, `/ta/`, `@ta_cat`,
 `ta-da`, `https:`, and a query's `?`, `&`, `=`, `#`, `%` and `+`. Each is
 one UTF-16 unit, so a scan over them steps by one.

 NOT THE APOSTROPHE, which closes a possessive (`Ta's`), and not a dash,
 an ellipsis, an arrow, a quote, a bracket or an emphasis mark, none of which
 sits inside an address.
 */
const TOKEN_JOINERS: ReadonlySet<string> = new Set([
  '-',
  '_',
  '.',
  '/',
  '@',
  ':',
  '=',
  '#',
  '%',
  '&',
  '?',
  '+',
],);

/**
 Mark opening a handle: a word after it is the handle's name, so `@ta` is a
 token of its own and never the word `ta`.
 */
const MENTION_MARK = '@';

/**
 Whether a token runs on before an offset: past any joiners directly before
 it, a Latin word character waits, or a mention mark stands among them.

 @param text - text read

 @param at - offset the token would start at

 @returns Whether the token continues leftward into a longer one

 @example
 ```ts
 runsOnBefore({ text: 'meta.ta', at: 5, },); // true
 ```
 */
function runsOnBefore(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  for (let cursor = at; cursor > 0; cursor -= 1) {
    /**
     Character directly before the cursor, one unit while joiners last.
     */
    const unit = text.charAt(cursor - 1,);
    if (unit === MENTION_MARK)
      return true;
    if (!TOKEN_JOINERS.has(unit,))
      return continuesLatinWord({
        character: codePointBefore({
          text,
          at: cursor,
        },),
      },);
  }
  return false;
}

/**
 Whether a token runs on after an offset: past any joiners directly after
 it, a Latin word character waits.

 @param text - text read

 @param at - offset the token would end at

 @returns Whether the token continues rightward into a longer one

 @example
 ```ts
 runsOnAfter({ text: 'ta-da', at: 2, },); // true
 ```
 */
function runsOnAfter(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  for (let cursor = at; cursor < text.length; cursor += 1) {
    if (!TOKEN_JOINERS.has(text.charAt(cursor,),))
      return continuesLatinWord({
        character: codePointAt({
          text,
          at: cursor,
        },),
      },);
  }
  return false;
}

/**
 Every start of a needle in a text where it stands as a token of its own:
 the run of Latin word characters and {@link TOKEN_JOINERS} around it,
 trimmed of joiners at both ends, is the needle alone.

 STRICTER THAN {@link wordStarts} where a joiner sits between the needle and
 a word: `meta.ta`, `/ta/`, `@ta_cat` and `ta-da` are one token each, and so
 is a handle, `@ta`, while `猫/Ta`, `Ta?` and `—TA` leave the needle
 standing. As there, only a Latin edge of the needle is read.

 @param text - text searched

 @param needle - word looked for

 @returns Start offsets in ascending order, empty where the needle is absent
 or empty

 @example
 ```ts
 tokenStarts({ text: 'meta.ta or 猫/ta', needle: 'ta', },); // [13]
 ```
 */
export function tokenStarts(
  {
    text,
    needle,
  }: {
    readonly text: string;
    readonly needle: string;
  },
): readonly number[] {
  if (needle === '')
    return [];
  /**
   Whether the needle opens on a Latin word character, so its start is read.
   */
  const opensLatin = continuesLatinWord({
    character: codePointAt({
      text: needle,
      at: 0,
    },),
  },);
  /**
   Whether it closes on one, so its end is read.
   */
  const closesLatin = continuesLatinWord({
    character: codePointBefore({
      text: needle,
      at: needle.length,
    },),
  },);
  /**
   Starts found so far; a linear cursor scan, since the text is unbounded.
   Each joiner run is read at most from the occurrence on either side of it.
   */
  const starts: number[] = [];
  for (let at = text.indexOf(needle,); at !== (-1); at = text.indexOf(
    needle,
    at + 1,
  )) {
    /**
     Whether the token runs on into a word before the needle.
     */
    const joinedBefore = opensLatin && runsOnBefore({
      text,
      at,
    },);
    /**
     Whether it runs on into a word after.
     */
    const joinedAfter = closesLatin && runsOnAfter({
      text,
      at: at + needle.length,
    },);
    if ((!joinedBefore) && (!joinedAfter))
      starts.push(at,);
  }
  return starts;
}

/**
 Whether a text carries a word or phrase with its Latin edges at word
 boundaries.

 @param text - text searched

 @param needle - word or phrase looked for

 @param end - whether the needle's end needs a boundary too

 @returns Whether some occurrence stands apart

 @example
 ```ts
 carriesWord({ text: 'the cat served as an aide', needle: 'as an ai', end: 'word', },); // false
 ```
 */
export function carriesWord(
  {
    text,
    needle,
    end,
  }: {
    readonly text: string;
    readonly needle: string;
    readonly end: WordEnd;
  },
): boolean {
  /**
   Starts where the needle stands apart.
   */
  const starts = wordStarts({
    text,
    needle,
    end,
  },);
  return starts.length > 0;
}

//endregion Word bounds
