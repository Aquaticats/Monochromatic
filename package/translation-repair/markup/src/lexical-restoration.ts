import {
  continuesLatinWord,
  foldLatinWord,
  isLatinWordCharacter,
} from './latin-letters.ts';

//region Lexical restoration grade
// The retired first-pass restoration grader, kept as a cheap lower-bound
// signal alongside the bilingual judge. It measures how much of the
// distinctive vocabulary a deletion removed reappears in the repaired text;
// it under-credits terse-but-faithful re-translations, which is why the
// judge anchored on the Chinese source replaced it as the headline rate.

/**
 Minimum content-word length that counts as distinctive vocabulary;
 shorter words recur everywhere and prove nothing about restoration.
 */
const CONTENT_WORD_MIN_CHARS = 4;

/**
 Fraction of disappeared content words that must return for a seed to
 grade as restored under the lexical measure.
 */
export const RESTORATION_WORD_THRESHOLD: number = 1 / 2;

/**
 Distinct folded content words of one text, by linear scan over runs of Latin
 letters, digits and apostrophes; no regex needed.

 A run opens on a Latin letter, a digit or an apostrophe and goes on through
 them and combining marks, and each run is folded (`foldLatinWord`), so
 `château`, `cha\u{0302}teau` and `chateau` are one word whose length is its
 letters. The scan once took ASCII letters only and read a combining mark as a
 separator, which cut every accented word into fragments (ledger B18).

 @param text - text whose vocabulary is collected

 @returns Distinct words at least {@link CONTENT_WORD_MIN_CHARS} letters long

 @example
 ```ts
 contentWords({ text: 'The cat naps.', },);
 ```
 */
export function contentWords(
  { text, }: { readonly text: string; },
): ReadonlySet<string> {
  /**
   Distinct words collected by the scan.
   */
  const words = new Set<string>();

  /**
   Start of the run currently being scanned; -1 outside a run.
   */
  let runStart = -1;
  // One pass past the end, so the last run closes like any other.
  for (let index = 0; index <= text.length; index += 1) {
    /**
     Code unit under the cursor, empty past the end.
     */
    const character = text.charAt(index,);

    /**
     Whether this character opens a run or goes on with the open one.
     */
    const inRun = (runStart === (-1))
      ? (isLatinWordCharacter({ character, },) || (character === '\''))
      : (continuesLatinWord({ character, },) || (character === '\''));
    if (inRun) {
      if (runStart === (-1))
        runStart = index;
      continue;
    }
    if (runStart !== (-1)) {
      /**
       Closed run, folded.
       */
      const word = foldLatinWord({
        word: text.slice(
          runStart,
          index,
        ),
      },);
      if (word.length >= CONTENT_WORD_MIN_CHARS)
        words.add(word,);
      runStart = -1;
    }
  }

  return words;
}

/**
 Lexical restoration grade of one planted deletion.

 @example
 ```ts
 const grade: SeedRestoration = {
   measurable: true,
   disappearedWords: 8,
   returnedWords: 6,
   restored: true,
 };
 ```
 */
export type SeedRestoration = {
  /**
   Whether the deletion removed any distinctive vocabulary at all;
   a needle whose every word survives elsewhere cannot be graded.
   */
  readonly measurable: boolean;

  /**
   Content words the deletion removed from the seeded text.
   */
  readonly disappearedWords: number;

  /**
   Disappeared words present again in the repaired text.
   */
  readonly returnedWords: number;

  /**
   Whether returned reaches {@link RESTORATION_WORD_THRESHOLD} of
   disappeared; always false for unmeasurable seeds.
   */
  readonly restored: boolean;
};

/**
 Grades one planted deletion against the repaired text by vocabulary.
 Only vocabulary the deletion actually removed counts:
 a word surviving elsewhere in the seeded text proves nothing.

 @param needle - deleted sentence exactly as planted

 @param seededText - translation after planting, before repair

 @param repairedText - pipeline output under grading

 @returns Restoration grade as data

 @example
 ```ts
 const grade = measureSeedRestoration({ needle, seededText, repairedText, },);
 ```
 */
export function measureSeedRestoration(
  {
    needle,
    seededText,
    repairedText,
  }: {
    readonly needle: string;
    readonly seededText: string;
    readonly repairedText: string;
  },
): SeedRestoration {
  /**
   Vocabulary surviving in the seeded text.
   */
  const seededWords = contentWords({ text: seededText, },);

  /**
   Needle vocabulary the deletion actually removed.
   */
  const disappeared = [...contentWords({ text: needle, },),]
    .filter(function isGone(word,) {
      return !seededWords.has(word,);
    },);
  if (disappeared.length === 0) {
    return {
      measurable: false,
      disappearedWords: 0,
      returnedWords: 0,
      restored: false,
    };
  }

  /**
   Vocabulary of the repaired text.
   */
  const repairedWords = contentWords({ text: repairedText, },);

  /**
   Disappeared words the repair brought back.
   */
  const returned = disappeared.filter(function cameBack(word,) {
    return repairedWords.has(word,);
  },);

  return {
    measurable: true,
    disappearedWords: disappeared.length,
    returnedWords: returned.length,
    restored: (returned.length / disappeared.length) >= RESTORATION_WORD_THRESHOLD,
  };
}

//endregion Lexical restoration grade
