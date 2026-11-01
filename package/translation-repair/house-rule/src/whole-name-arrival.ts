import {
  codePointAt,
  codePointBefore,
  continuesLatinWord,
  isAsciiLetter,
  isAsciiLowerLetter,
  isLetterOrDigitOfAnyScript,
  isTagWhitespace,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type NameOccurrence,
  type NameProjection,
  nameOccurrences,
} from './name-projection.ts';
import { ORDINARY_CAPITALISED_WORDS, } from './ordinary-capitalised-words.ts';

//region Whole-name arrival
// WHETHER A DECLARED FORM THAT TURNS UP IN A REPLACEMENT STANDS AS A WHOLE
// NAME, for the survival guard's swap rule (`declared-name-survival.ts`).
//
// WHY CARRIAGE IS NOT ENOUGH. The guard asks whether a form is carried, and
// carries it wherever its letters sit at a word edge: after a camel edge
// (`XingLili`), beside a space (`Xing Lili`), even split (`Xing Li Li`, which
// the projection folds into the key of `Lili`). That is right for a form
// the base already held, and wrong for one that has to MEET a lost name: the
// replacement then wrote a name nobody declared, with the declared form
// inside it. XingZ60 slice 80 measured it: of 43 swaps the first swap rule
// passed over the stored artifacts, 19 wrote `Xing Lili` (11), `XingLili` (7)
// or `Xing Li Li` (1) where the front matter declares `XingZ60`, `Lili` and
// `Z60`, fusing a piece of the old name onto the alias.
//
// THREE TESTS, each on one occurrence, any occurrence sufficing:
//
//   - NOT FUSED ACROSS A JOINT. A form that runs on from or into a Latin
//     letter or digit directly, or across a hyphen or an underscore, is a
//     piece of a longer word (`XingLili`, `Xing-Lili`, `Lili2024`).
//   - SPELLED AS DECLARED. The text of the occurrence, Markdown escapes and
//     case aside, is the form: `Li Li` is not `Lili`.
//   - NO CAPITALISED WORD FUSED ON BESIDE IT across a space. A capitalised
//     Latin word directly beside the form reads as one name with it
//     (`Xing Lili`) unless it is part of some declared form, an ordinary
//     sentence opener, title or kin word (`When Lili`, `Dear Lili`), or a
//     word the base already held on that side of the lost name, which the
//     candidate then did not fuse.
//
// CONSERVATIVE IN THE DIRECTION OF THE FLOOR. Each test can only refuse an
// arrival, and a refused arrival is the plain rule of dropping nothing the
// base carried, so no wording is accepted that the first swap rule refused.

/**
 Words that may stand beside an arrival on each side without fusing with it,
 lower-cased.

 @example
 ```ts
 const excused: ExcusedNeighbours = { before: new Set([ 'when', ]), after: new Set(), };
 ```
 */
export type ExcusedNeighbours = {
  /**
   Words allowed directly before the form.
   */
  readonly before: ReadonlySet<string>;

  /**
   Words allowed directly after the form.
   */
  readonly after: ReadonlySet<string>;
};

/**
 Words standing directly beside occurrences of a name, lower-cased.

 @example
 ```ts
 const beside: BesideWords = { before: [ 'aoba', ], after: [], };
 ```
 */
export type BesideWords = {
  /**
   Capitalised Latin words directly before an occurrence.
   */
  readonly before: readonly string[];

  /**
   Capitalised Latin words directly after an occurrence.
   */
  readonly after: readonly string[];
};

/**
 Characters that join two words into one compound name when they stand
 directly between letters: a hyphen, a non-breaking hyphen and an underscore.
 */
const JOINING_MARKS: ReadonlySet<string> = new Set([
  '-',
  '‐',
  '‑',
  '_',
]);

/**
 The no-break space, written as an escape so no invisible character sits in
 the source.
 */
const NO_BREAK_SPACE = '\u{00A0}';

/**
 The backslash Markdown writes before a character it escapes.
 */
const ESCAPE = '\\';

/**
 Whether a character is a plain or a no-break space, the only separator
 through which two capitalised words read as one name.

 @param character - one UTF-16 unit

 @returns Whether it is a space

 @example
 ```ts
 const space = isSpace({ character: ' ', });
 ```
 */
function isSpace({ character, }: { readonly character: string; },): boolean {
  return (character === ' ') || (character === NO_BREAK_SPACE);
}

/**
 Whether a word is written with a capital first letter and small ASCII letters
 after it.

 @param word - run of ASCII letters

 @returns Whether it has that shape

 @example
 ```ts
 const shape = isCapitalised({ word: 'Xing', }); // true
 ```
 */
function isCapitalised({ word, }: { readonly word: string; },): boolean {
  if (word.length < 2)
    return false;
  if ((!isAsciiLetter({ character: word.charAt(0,), },)) || isAsciiLowerLetter({ character: word.charAt(0,), },))
    return false;
  for (let at = 1; at < word.length; at += 1) {
    if (!isAsciiLowerLetter({ character: word.charAt(at,), },))
      return false;
  }
  return true;
}

/**
 Whether a character is an ASCII letter, as a test a run can keep to.

 @param character - one UTF-16 unit

 @returns Whether it is `a` to `z` in either case

 @example
 ```ts
 const letter = isAsciiLetterCharacter('x',);
 ```
 */
function isAsciiLetterCharacter(character: string,): boolean {
  return isAsciiLetter({ character, },);
}

/**
 Whether a character is a space, as a test a run can keep to.

 @param character - one UTF-16 unit

 @returns Whether it is a plain or a no-break space

 @example
 ```ts
 const space = isSpaceCharacter(' ',);
 ```
 */
function isSpaceCharacter(character: string,): boolean {
  return isSpace({ character, },);
}

/**
 Offset where the run of kept characters that ends at an offset begins.

 @param text - composed text

 @param from - offset the run ends at

 @param keeps - test a character of the run passes

 @returns The offset of the run's first character, `from` when none is kept

 @example
 ```ts
 const begin = runStart({ text: 'ab  ', from: 4, keeps: isSpaceCharacter, },); // 2
 ```
 */
function runStart(
  {
    text,
    from,
    keeps,
  }: {
    readonly text: string;
    readonly from: number;
    readonly keeps: (character: string,) => boolean;
  },
): number {
  for (let cursor = from; cursor > 0; cursor -= 1) {
    if (!keeps(text.charAt(cursor - 1,),))
      return cursor;
  }
  return 0;
}

/**
 Offset just past the run of kept characters that begins at an offset.

 @param text - composed text

 @param from - offset the run begins at

 @param keeps - test a character of the run passes

 @returns The offset past the run's last character, `from` when none is kept

 @example
 ```ts
 const end = runEnd({ text: '  ab', from: 0, keeps: isSpaceCharacter, },); // 2
 ```
 */
function runEnd(
  {
    text,
    from,
    keeps,
  }: {
    readonly text: string;
    readonly from: number;
    readonly keeps: (character: string,) => boolean;
  },
): number {
  for (let cursor = from; cursor < text.length; cursor += 1) {
    if (!keeps(text.charAt(cursor,),))
      return cursor;
  }
  return text.length;
}

/**
 The capitalised ASCII word that ends just before an offset, a run of spaces
 between.

 @param text - composed text

 @param at - offset of the occurrence's first character

 @returns The word lower-cased, none when the text there is not a capitalised word

 @example
 ```ts
 const word = wordBefore({ text: 'When Lili', at: 5, },); // 'when'
 ```
 */
function wordBefore(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): readonly string[] {
  /**
   Offset just past the word, after the spaces are stepped back over.
   */
  const end = runStart({
    text,
    from: at,
    keeps: isSpaceCharacter,
  },);
  if (end === at)
    return [];

  /**
   The run of ASCII letters that ends there.
   */
  const word = text.slice(
    runStart({
      text,
      from: end,
      keeps: isAsciiLetterCharacter,
    },),
    end,
  );
  return isCapitalised({ word, },) ? [ word.toLowerCase(), ] : [];
}

/**
 The capitalised ASCII word that begins just after an offset, a run of spaces
 between.

 @param text - composed text

 @param at - offset just past the occurrence's last character

 @returns The word lower-cased, none when the text there is not a capitalised word

 @example
 ```ts
 const word = wordAfter({ text: 'Lili Xing', at: 4, },); // 'xing'
 ```
 */
function wordAfter(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): readonly string[] {
  /**
   Offset where the word begins, after the spaces are stepped over.
   */
  const begin = runEnd({
    text,
    from: at,
    keeps: isSpaceCharacter,
  },);
  if (begin === at)
    return [];

  /**
   The run of ASCII letters that begins there.
   */
  const word = text.slice(
    begin,
    runEnd({
      text,
      from: begin,
      keeps: isAsciiLetterCharacter,
    },),
  );
  return isCapitalised({ word, },) ? [ word.toLowerCase(), ] : [];
}

/**
 Whether a character joins a name to the letters beyond it: a Latin letter
 or digit, or a joining mark with one behind it.

 @param text - composed text

 @param at - offset of the character beside the occurrence, going outward

 @param step - one past the occurrence's end, or minus one before its start

 @returns Whether the occurrence runs on across that edge

 @example
 ```ts
 const fused = runsOnAcross({ text: 'XingLili', at: 3, step: -1, },);
 ```
 */
function runsOnAcross(
  {
    text,
    at,
    step,
  }: {
    readonly text: string;
    readonly at: number;
    readonly step: 1 | -1;
  },
): boolean {
  /**
   The character directly beside the occurrence.
   */
  const near = (step === (-1))
    ? codePointBefore({
      text,
      at,
    },)
    : codePointAt({
      text,
      at,
    },);
  if (continuesLatinWord({ character: near, },))
    return true;
  if (!JOINING_MARKS.has(near,))
    return false;

  /**
   Offset of the mark itself.
   */
  const mark = (step === (-1)) ? at - near.length : at + near.length;

  /**
   Offset of the character past the mark, stepping over a Markdown backslash
   that escapes the mark.
   */
  const past = ((step === (-1)) && (text.charAt(mark - 1,) === ESCAPE))
    ? mark - 1
    : mark;

  /**
   The character past the mark.
   */
  const beyond = (step === (-1))
    ? codePointBefore({
      text,
      at: past,
    },)
    : codePointAt({
      text,
      at: past,
    },);
  return continuesLatinWord({ character: beyond, },);
}

/**
 Whether a character is a letter or a digit of any script.

 @param character - one whole character

 @returns Whether it belongs to a name

 @example
 ```ts
 const kept = isNameCharacter('猫',);
 ```
 */
function isNameCharacter(character: string,): boolean {
  return isLetterOrDigitOfAnyScript({ character, },);
}

/**
 A text as it is compared for the spelling of a form: composed, Markdown
 escapes removed, lower-cased, each run of white space one space, and the
 punctuation at either end left off.

 THE ENDS ARE LEFT OFF because an occurrence is found by its letters and digits,
 so it starts and ends at one, while a declared form can end in a bracket
 (`Huasheng(Peanut)`) that the projection drops.

 @param text - form or the stretch of text that holds it

 @returns The comparable spelling

 @example
 ```ts
 const spelling = spellingOf({ text: 'Wooly_Cat', }); // 'wooly_cat'
 ```
 */
function spellingOf({ text, }: { readonly text: string; },): string {
  /**
   Characters kept, each run of white space as one space.
   */
  const kept: string[] = [];
  for (const character of text.normalize('NFC',)
    .replaceAll(
    ESCAPE,
    '',
    )
    .toLowerCase()) {
    if (!isTagWhitespace({ character, },)) {
      kept.push(character,);
      continue;
    }
    if (kept.at(-1,) !== ' ')
      kept.push(' ',);
  }

  return kept.slice(
    kept.findIndex(isNameCharacter,),
    kept.findLastIndex(isNameCharacter,) + 1,
  )
    .join('',);
}

/**
 Words standing directly beside occurrences of a name.

 @param projection - text projected by the name projection

 @param key - name projected the same way

 @returns Capitalised Latin words before and after each occurrence, lower-cased

 @example
 ```ts
 const beside = besideWordsOf({ projection, key: 'mittens', });
 ```
 */
export function besideWordsOf(
  {
    projection,
    key,
  }: {
    readonly projection: NameProjection;
    readonly key: string;
  },
): BesideWords {
  /**
   Occurrences of the name.
   */
  const places: readonly NameOccurrence[] = nameOccurrences({
    projection,
    key,
  },);
  return {
    before: places.flatMap(function beforeOne({ start, },): readonly string[] {
      return wordBefore({
        text: projection.composed,
        at: start,
      },);
    },),
    after: places.flatMap(function afterOne({ end, },): readonly string[] {
      return wordAfter({
        text: projection.composed,
        at: end,
      },);
    },),
  };
}

/**
 Lower-cased words of every declared form, which may stand beside any form
 without fusing with it.

 @param forms - declared forms of anyone

 @returns Their words, split at anything that is no letter or digit

 @example
 ```ts
 const words = declaredWordsOf({ forms: [ 'Wooly Blossom', ], },); // wooly, blossom
 ```
 */
export function declaredWordsOf(
  { forms, }: { readonly forms: readonly string[]; },
): ReadonlySet<string> {
  return new Set(forms.flatMap(function wordsOf(form,): readonly string[] {
    /**
     Words found, each closed at the first character that is no letter or digit.
     */
    const words: string[] = [];

    /**
     Characters of the word being read.
     */
    const current: string[] = [];
    for (const character of form.toLowerCase()) {
      if (isLetterOrDigitOfAnyScript({ character, },)) {
        current.push(character,);
        continue;
      }
      if (current.length > 0)
        words.push(current.splice(0,)
          .join('',),);
    }
    if (current.length > 0)
      words.push(current.join('',),);
    return words;
  },),);
}

/**
 Whether a declared form stands as a whole name somewhere in a text.

 @param projection - candidate text projected by the name projection

 @param form - the declared form as the front matter writes it

 @param key - the form projected the same way

 @param excused - words allowed directly beside the form

 @returns Whether some occurrence is not fused, is spelled as declared, and has
 only excused words beside it

 @example
 ```ts
 const whole = arrivesAsWholeName({ projection, form: 'Lili', key: 'lili', excused, },);
 ```
 */
export function arrivesAsWholeName(
  {
    projection,
    form,
    key,
    excused,
  }: {
    readonly projection: NameProjection;
    readonly form: string;
    readonly key: string;
    readonly excused: ExcusedNeighbours;
  },
): boolean {
  /**
   Text the offsets index.
   */
  const text = projection.composed;

  /**
   The form as it must be spelled.
   */
  const declared = spellingOf({ text: form, },);

  /**
   Whether the form begins in Latin letters, so a word before it can fuse.
   */
  const latinFirst = continuesLatinWord({ character: codePointAt({
    text: form,
    at: 0,
  },), },);

  /**
   Whether the form ends in Latin letters, so a word after it can fuse.
   */
  const latinLast = continuesLatinWord({ character: codePointBefore({
    text: form,
    at: form.length,
  },), },);
  return nameOccurrences({
    projection,
    key,
  },)
    .some(function standsWhole({
      start,
      end,
    },): boolean {
      if (runsOnAcross({
        text,
        at: start,
        step: -1,
      },) || runsOnAcross({
        text,
        at: end,
        step: 1,
      },))
        return false;
      if (spellingOf({ text: text.slice(
        start,
        end,
      ), },) !== declared)
        return false;
      /**
       Words directly beside the occurrence.
       */
      const beside = {
        before: latinFirst ? wordBefore({
          text,
          at: start,
        },) : [],
        after: latinLast ? wordAfter({
          text,
          at: end,
        },) : [],
      };
      return beside.before
        .every(function excusedBefore(word,): boolean {
          return excused.before
            .has(word,);
        },)
        && beside.after
          .every(function excusedAfter(word,): boolean {
            return excused.after
              .has(word,);
          },);
    },);
}

/**
 Builds the words allowed beside an arrival on each side.

 @param declaredWords - words of every declared form

 @param base - words the base held beside the lost forms

 @returns Declared words, ordinary capitalised words and the base's own, per side

 @example
 ```ts
 const excused = excusedNeighboursOf({ declaredWords, base, },);
 ```
 */
export function excusedNeighboursOf(
  {
    declaredWords,
    base,
  }: {
    readonly declaredWords: ReadonlySet<string>;
    readonly base: BesideWords;
  },
): ExcusedNeighbours {
  return {
    before: new Set([
      ...declaredWords,
      ...ORDINARY_CAPITALISED_WORDS,
      ...base.before,
    ],),
    after: new Set([
      ...declaredWords,
      ...ORDINARY_CAPITALISED_WORDS,
      ...base.after,
    ],),
  };
}

//endregion Whole-name arrival
