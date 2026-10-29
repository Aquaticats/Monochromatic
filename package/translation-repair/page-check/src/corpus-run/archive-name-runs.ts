import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  isTitleCase,
  onHeadingLine,
} from './archive-casing-restore.ts';
import {
  inProse,
  protectedRanges,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  continuesLatinWord,
  type LatinWord,
  latinWordSpans,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Archive name runs
// The scans class one hundred thirty-six reads a page with (`archive-name-
// casing.ts`): runs of title-case words, whether a run opens a sentence, and
// where a phrase stands in prose in any casing. Each is one index scan.

/**
 Fewest words a run needs before it reads as a multi-word name.
 */
const MIN_WORDS = 2;

/**
 Characters that may stand before a run it still reads as mid-sentence,
 besides a letter or a digit.
 */
const MID_SENTENCE_BEFORE: ReadonlySet<string> = new Set([
  ',',
  ';',
],);

/**
 One run of title-case words and where it stands.
 */
export type NameRun = {
  readonly phrase: string;
  readonly start: number;
};

/**
 Whether a run starting at one offset stands mid-sentence: the nearest
 character before it past spaces is a letter (accented or not), a digit, a
 combining mark, a comma or a semicolon.

 @param text - text under scan

 @param at - run's first offset

 @returns Whether the run does not open a sentence, a line or the text

 @example
 ```ts
 midSentence({ text: 'on the Maowu Street', at: 7, },); // true
 ```
 */
export function midSentence(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  /**
   Nearest character before the run past spaces, empty at the text's start.
   */
  const last = text
    .slice(
    0,
    at,
    )
    .trimEnd()
    .at(-1,)
    ?? '';
  return continuesLatinWord({ character: last, },) || MID_SENTENCE_BEFORE.has(last,);
}

/**
 Every run of two or more title-case words joined by single spaces.

 @param text - text under scan

 @returns Runs in order

 @example
 ```ts
 titleRuns({ text: 'on Maowu Fish Street today', },); // [{ phrase: 'Maowu Fish Street', start: 3 }]
 ```
 */
export function titleRuns(
  { text, }: { readonly text: string; },
): readonly NameRun[] {
  /**
   Runs of title-case words, each a list of its words; a word in any other
   casing closes the run before it.
   */
  const groups: readonly (readonly LatinWord[])[] = latinWordSpans({ text, },)
    .reduce<LatinWord[][]>(
    function group(
      built,
      word,
    ): LatinWord[][] {
      if (!isTitleCase({ word: word.word, },)) {
        built.push([],);
        return built;
      }
        /**
         Run the word may extend.
         */
      const open = built.at(-1,);
        /**
         Last word of that run.
         */
      const previous = open?.at(-1,);
        /**
         Whether one space alone stands between the two words.
         */
      const joined = (previous !== undefined)
        && (text.slice(
          previous.start
            + previous.word
              .length,
          word.start,
        ) === ' ');
      if (joined)
        open?.push(word,);
      else
        built.push([word,],);
      return built;
    },
    [],
    );
  return groups
    .filter(function longEnough(words,): boolean {
      return words.length >= MIN_WORDS;
    },)
    .map(function toRun(words,): NameRun {
      /**
       First word of the run.
       */
      const first = nonNullishOrThrow(words[0],);
      /**
       Last word of the run.
       */
      const last = nonNullishOrThrow(words.at(-1,),);
      return {
        phrase: text.slice(
          first.start,
          last.start
            + last.word
              .length,
        ),
        start: first.start,
      };
    },);
}

/**
 Lower case read one code point at a time, keeping as written any code point
 whose lower case has another UTF-16 length, so every offset in the result
 is the same offset in the text.

 WHY NOT `text.toLowerCase()`: one İ (U+0130) lowers to two units, after
 which no offset lines up, and the scan that read the whole text past
 found no occurrence anywhere on the page (ledger B121). A code point kept
 as written still matches itself, since the phrase is lowered the same way.

 @param text - text to lower

 @returns Text of the same length, lowered wherever lowering keeps the length

 @example
 ```ts
 lengthKeepingLowerCase({ text: 'İpek on Maowu Street', },); // 'İpek on maowu street'
 ```
 */
function lengthKeepingLowerCase({ text, }: { readonly text: string; },): string {
  return Array.from(
    text,
    function lowered(character,): string {
      /**
       This code point's lower case, which may be longer than it.
       */
      const lower = character.toLowerCase();
      return (lower.length === character.length) ? lower : character;
    },
  )
    .join('',);
}

/**
 Every prose occurrence of a phrase in any casing, bounded by non-letters and
 off heading lines.

 @param text - text under scan

 @param phrase - phrase to look for

 @returns Offsets where the phrase stands in any casing

 @example
 ```ts
 phraseOccurrences({ text: 'on maowu street', phrase: 'Maowu Street', },); // [3]
 ```
 */
export function phraseOccurrences(
  {
    text,
    phrase,
  }: {
    readonly text: string;
    readonly phrase: string;
  },
): readonly number[] {
  /**
   The text in lower case, offset for offset.
   */
  const lowered = lengthKeepingLowerCase({ text, },);
  /**
   The phrase lowered the same way.
   */
  const needle = lengthKeepingLowerCase({ text: phrase, },);
  /**
   The text's non-prose ranges.
   */
  const ranges = protectedRanges({ text, },);
  /**
   Offsets found so far.
   */
  const found: number[] = [];
  for (let at = lowered.indexOf(needle,); at !== (-1); at = lowered.indexOf(
    needle,
    at + 1,
  )) {
    /**
     Whether a letter, digit or combining mark touches the match on either
     side: a mark after it would sit on the match's last letter, making it
     another word.
     */
    const glued = continuesLatinWord({ character: text.charAt(at - 1,), },)
      || continuesLatinWord({ character: text.charAt(at + needle.length,), },);
    if (glued || onHeadingLine({
      text,
      at,
    },)
      || (!inProse({
        ranges,
        start: at,
        end: at + needle.length,
      },)))
      continue;
    found.push(at,);
  }
  return found;
}
//endregion Archive name runs
