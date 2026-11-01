import {
  inProse,
  protectedRanges,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  foldLatinWord,
  type LatinWord,
  latinWordSpans,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { visibleText, } from './page-visible-text.ts';

//region Latin spellings
// The Latin side of the pairing: the capitalised words an archive spells names with, by their reading letters.
// `prose-name-pairs.ts` states the whole pairing and why it reads this way.

/**
 Most words one spelling may hold; a longer run of capitalised words is a
 title or a heading, not a name.
 */
const MAX_WORDS = 6;

/**
 Fewest letters a reading may have; a shorter one is a word of the language.
 */
const MIN_LETTERS = 4;

/**
 What a text shows a reader and the Latin words in its prose.
 */
type ProseWords = {
  readonly shown: string;
  readonly words: readonly LatinWord[];
};

/**
 Whether a Latin word is written as a name is: an initial capital and some
 lower case after it, so a lower-case word, a bare capital and a word in
 capitals alone are none.

 @param word - run of Latin letters

 @returns Whether the word is capitalised as a name

 @example
 ```ts
 isNameWord({ word: 'Tuanzi', },); // true
 ```
 */
export function isNameWord({ word, }: { readonly word: string; },): boolean {
  /**
   The word's first character.
   */
  const first = word.charAt(0,);
  /**
   Everything after it.
   */
  const rest = word.slice(1,);
  return (first !== first.toLowerCase()) && (rest !== rest.toUpperCase());
}

/**
 Whether a Latin word opens in lower case, as a word of the language does.

 @param word - run of Latin letters

 @returns Whether its first letter is lower case

 @example
 ```ts
 opensLower({ word: 'take', },); // true
 ```
 */
function opensLower({ word, }: { readonly word: string; },): boolean {
  /**
   The word's first character.
   */
  const first = word.charAt(0,);
  return first === first.toLowerCase();
}

/**
 The reading of Latin text: its letters alone, accents and capitals folded
 away, spaces dropped.

 @param text - words as written

 @returns Folded letters joined

 @example
 ```ts
 letterReading({ text: 'Xue Tuanzi', },); // 'xuetuanzi'
 ```
 */
export function letterReading({ text, }: { readonly text: string; },): string {
  return latinWordSpans({ text, },)
    .map(function folded({ word, },): string {
      return foldLatinWord({ word, },);
    },)
    .join('',);
}

/**
 What a text shows a reader, and the Latin words of it that stand in prose.

 @param text - whole document or slice

 @returns The shown text and its prose words with where each stands

 @example
 ```ts
 const { words, } = proseWords({ text: 'Xue Tuanzi wrote.', },);
 ```
 */
export function proseWords({ text, }: { readonly text: string; },): ProseWords {
  /**
   What the text shows a reader.
   */
  const shown = visibleText({ text, },);
  /**
   Ranges of it that are no prose: addresses, code and the like.
   */
  const ranges = protectedRanges({ text: shown, },);
  return {
    shown,
    words: latinWordSpans({ text: shown, },)
      .filter(function inText({
        word,
        start,
      },): boolean {
        return inProse({
          ranges,
          start,
          end: start + word.length,
        },);
      },),
  };
}

/**
 Offset just past a Latin word.

 @param word - word and where it starts

 @returns Offset of the first unit after it

 @example
 ```ts
 endOf({ word: 'Xue', start: 4, },); // 7
 ```
 */
function endOf(
  {
    word,
    start,
  }: LatinWord,
): number {
  return start + word.length;
}

/**
 Whether a name word continues the run its neighbour before it is in: that
 neighbour is a name word too and one space alone stands between them.

 @param shown - text the words were read from

 @param previous - word before this one

 @param word - word under test

 @returns Whether the word extends the neighbour's run

 @example
 ```ts
 continuesRun({ shown: 'Xue Tuanzi', previous: { word: 'Xue', start: 0, }, word: { word: 'Tuanzi', start: 4, }, },); // true
 ```
 */
function continuesRun(
  {
    shown,
    previous,
    word,
  }: {
    readonly shown: string;
    readonly previous: LatinWord;
    readonly word: LatinWord;
  },
): boolean {
  if (!isNameWord({ word: previous.word, },))
    return false;
  return shown.slice(
    endOf(previous,),
    word.start,
  ) === ' ';
}

/**
 Runs of capitalised words a single space apart, in order; a word that is
 no name's closes the run before it.

 @param shown - text the words were read from

 @param words - prose words of that text, in order

 @returns The words of each run

 @example
 ```ts
 const runs = nameRuns({ shown: 'Xue Tuanzi came.', words, },);
 ```
 */
function nameRuns(
  {
    shown,
    words,
  }: {
    readonly shown: string;
    readonly words: readonly LatinWord[];
  },
): readonly (readonly string[])[] {
  /**
   Runs gathered; the last is the one a word may extend.
   */
  const runs: string[][] = [];
  for (const [at, word,] of words.entries()) {
    if (!isNameWord({ word: word.word, },))
      continue;
    /**
     Run the word extends, when it continues its neighbour's.
     */
    const open = runs.at(-1,);
    /**
     Word before this one, none at the start.
     */
    const previous = words[at - 1];
    if (open === undefined) {
      runs.push([word.word,],);
      continue;
    }
    if (previous === undefined) {
      runs.push([word.word,],);
      continue;
    }
    if (continuesRun({
      shown,
      previous,
      word,
    },))
      open.push(word.word,);
    else
      runs.push([word.word,],);
  }
  return runs;
}

/**
 Every stretch of one to `MAX_WORDS` neighbouring words of a run, as the
 words written.

 @param run - capitalised words a space apart

 @returns Each stretch's words, longest first at each start

 @example
 ```ts
 stretchesOf({ run: ['Xue', 'Tuanzi',], },); // [['Xue', 'Tuanzi'], ['Xue'], ['Tuanzi']]
 ```
 */
function stretchesOf({ run, }: { readonly run: readonly string[]; },): readonly (readonly string[])[] {
  /**
   Stretches found, in order of start.
   */
  const stretches: (readonly string[])[] = [];
  for (let from = 0; from < run.length; from += 1) {
    for (
      let to = Math.min(
        run.length,
        from + MAX_WORDS,
      );
      to > from;
      to -= 1
    ) {
      stretches.push(run.slice(
        from,
        to,
      ),);
    }
  }
  return stretches;
}

/**
 The spellings a text gives, by their readings.

 @param text - archive or candidate document or slice

 @returns Each reading with the spellings written for it, in order of appearance

 @example
 ```ts
 const index = spellingIndex({ text: 'Xue Tuanzi came.', },);
 ```
 */
export function spellingIndex({ text, }: { readonly text: string; },): ReadonlyMap<string, readonly string[]> {
  /**
   The text's prose words.
   */
  const prose = proseWords({ text, },);
  /**
   Words the text also writes in lower case, which are words of the language.
   */
  const ordinary = new Set(prose.words
    .filter(opensLower,)
    .map(function folded({ word, },): string {
      return foldLatinWord({ word, },);
    },),);
  /**
   Spellings by reading.
   */
  const index = new Map<string, string[]>();
  for (
    const run of nameRuns({
      shown: prose.shown,
      words: prose.words,
    },)
  ) {
    for (const stretch of stretchesOf({ run, },)) {
      /**
       The stretch as written.
       */
      const spelling = stretch.join(' ',);
      /**
       The stretch's reading.
       */
      const reading = letterReading({ text: spelling, },);
      if (reading.length < MIN_LETTERS)
        continue;
      if ((stretch.length === 1) && ordinary.has(reading,))
        continue;
      /**
       Spellings already written for the reading.
       */
      const spellings = index.get(reading,) ?? [];
      if (!spellings.includes(spelling,))
        spellings.push(spelling,);
      index.set(
        reading,
        spellings,
      );
    }
  }
  return index;
}

//endregion Latin spellings
