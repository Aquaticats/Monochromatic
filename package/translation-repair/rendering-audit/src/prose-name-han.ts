import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { pinyin, } from 'pinyin-pro';

import {
  inProse,
  protectedRanges,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  foldLatinWord,
  isHanCharacter,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { visibleText, } from './page-visible-text.ts';
import { letterReading, } from './prose-name-spelling.ts';

//region Han readings
// The Han side of the pairing: the runs of an original and the Latin spellings their pinyin reads as.
// `prose-name-pairs.ts` states the whole pairing and why it reads this way.

/**
 Fewest Han characters a name may have; one syllable reads as many words.
 */
const MIN_CHARACTERS = 2;

/**
 Most letters a Han stretch's reading is grown to, which bounds the scan from
 each character.
 */
const MAX_LETTERS = 40;

/**
 A Han name and the Latin spelling the archive gives it.

 @example
 ```ts
 const pair: ProseNamePair = { source: '雪团子', rendering: 'Xue Tuanzi', };
 ```
 */
export type ProseNamePair = {
  /**
   Name as the original writes it.
   */
  readonly source: string;

  /**
   Name as the archive spells it.
   */
  readonly rendering: string;
};

/**
 The whole character that starts at an offset, one code point.

 @param text - text under scan

 @param at - offset of the character's first UTF-16 unit, inside the text

 @returns Character, one or two units long

 @example
 ```ts
 characterAt({ text: '雪团', at: 1, },); // '团'
 ```
 */
function characterAt(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): string {
  /**
   The code point at the offset.
   */
  const point = text.codePointAt(at,);
  return String.fromCodePoint(nonNullishOrThrow(point,),);
}

/**
 Whether the character at an offset is Han.

 @param text - text under scan

 @param at - offset of the character's first UTF-16 unit, inside the text

 @returns Whether it is a Han character

 @example
 ```ts
 isHanAt({ text: '雪团', at: 1, },); // true
 ```
 */
function isHanAt(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  return isHanCharacter({
    character: characterAt({
      text,
      at,
    },),
  },);
}

/**
 How many UTF-16 units the character at an offset takes.

 @param text - text under scan

 @param at - offset of the character's first UTF-16 unit, inside the text

 @returns One or two

 @example
 ```ts
 widthAt({ text: '雪团', at: 1, },); // 1
 ```
 */
function widthAt(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): number {
  /**
   The character at the offset.
   */
  const character = characterAt({
    text,
    at,
  },);
  return character.length;
}

/**
 Distinct runs of Han characters a text shows as prose, in order of
 appearance.

 @param text - original document or slice

 @returns Each run once

 @example
 ```ts
 const runs = hanRunsOf({ text: '在她离世后', },);
 ```
 */
export function hanRunsOf({ text, }: { readonly text: string; },): readonly string[] {
  /**
   What the text shows, and the ranges that are no prose.
   */
  const shown = visibleText({ text, },);
  /**
   Ranges of the shown text that are no prose.
   */
  const ranges = protectedRanges({ text: shown, },);
  /**
   Runs found, a set so a run repeated is read once.
   */
  const runs = new Set<string>();
  // BY CODE POINT, deliberately: an ideograph beyond the first plane is two
  // UTF-16 units, and a run is cut where the next code point is no Han.
  for (let at = 0; at < shown.length;) {
    /**
     Where the run being read starts.
     */
    const start = at;
    while ((at < shown.length) && isHanAt({
      text: shown,
      at,
    },)) {
      at += widthAt({
        text: shown,
        at,
      },);
    }
    if (at === start) {
      at += widthAt({
        text: shown,
        at,
      },);
      continue;
    }
    if (inProse({
      ranges,
      start,
      end: at,
    },))
      runs.add(shown.slice(
        start,
        at,
      ),);
  }
  return [...runs,];
}

/**
 Pairs found for the Han stretches of one run.

 @param run - run of Han characters

 @param index - spellings by reading

 @returns Each stretch whose reading has a spelling, with the spelling

 @example
 ```ts
 const found = stretchPairs({ run: '雪团子', index, },);
 ```
 */
export function stretchPairs(
  {
    run,
    index,
  }: {
    readonly run: string;
    readonly index: ReadonlyMap<string, readonly string[]>;
  },
): readonly ProseNamePair[] {
  // BY CODE POINT, one syllable for each, as the reading of the run is.
  /**
   The run's characters.
   */
  const characters = Array.from(run,);
  /**
   One syllable per character, accents and capitals folded away.
   */
  const syllables = pinyin(
    run,
    {
      toneType: 'none',
      type: 'array',
    },
  )
    .map(function folded(syllable,): string {
      return foldLatinWord({ word: syllable, },);
    },);
  /**
   Pairs found, in order of start and then of length.
   */
  const found: ProseNamePair[] = [];
  for (let from = 0; from < syllables.length; from += 1) {
    /**
     Reading of the stretch so far.
     */
    const reading: string[] = [];
    for (let to = from; to < syllables.length; to += 1) {
      reading.push(nonNullishOrThrow(syllables[to],),);
      /**
       Letters of the stretch so far.
       */
      const letters = reading.join('',);
      if (letters.length > MAX_LETTERS)
        break;
      if (reading.length < MIN_CHARACTERS)
        continue;
      for (const rendering of index.get(letters,) ?? []) {
        found.push({
          source: characters.slice(
            from,
            to + 1,
          )
            .join('',),
          rendering,
        },);
      }
    }
  }
  return found;
}

/**
 Whether a text holds another.

 @param whole - text that may hold it

 @param part - text looked for

 @returns Whether `part` stands in `whole`

 @example
 ```ts
 holds({ whole: 'Xue Tuanzi', part: 'Tuanzi', },); // true
 ```
 */
function holds(
  {
    whole,
    part,
  }: {
    readonly whole: string;
    readonly part: string;
  },
): boolean {
  return whole.includes(part,);
}

/**
 Pairs left once a pair that is only a part of another is dropped, and a
 spelling that more than one Han text reads as is left out.

 @param found - pairs of every stretch, in order

 @returns Distinct pairs in the order found

 @example
 ```ts
 const pairs = settledPairs({ found, },);
 ```
 */
export function settledPairs({ found, }: { readonly found: readonly ProseNamePair[]; },): readonly ProseNamePair[] {
  /**
   Each pair once.
   */
  const distinct = found.filter(function first(
    pair,
    at,
  ): boolean {
    return found.findIndex(function same(other,): boolean {
      return (other.source === pair.source) && (other.rendering === pair.rendering);
    },) === at;
  },);
  /**
   Pairs that are not part of a longer name's spelling, in the name's Han and its Latin.
   */
  const whole = distinct.filter(function notPart(pair,): boolean {
    return !distinct.some(function coversPair(other,): boolean {
      return (other !== pair)
        && holds({
          whole: other.source,
          part: pair.source,
        },)
        && holds({
          whole: other.rendering,
          part: pair.rendering,
        },);
    },);
  },);
  /**
   The pairs of each reading.
   */
  const byReading = Map.groupBy(
    whole,
    function letterReadingPair(pair,): string {
      return letterReading({ text: pair.rendering, },);
    },
  );
  return whole.filter(function unambiguous(pair,): boolean {
    /**
     Pairs that spell this reading.
     */
    const sharing = byReading.get(letterReading({ text: pair.rendering, },),) ?? [];
    return new Set(sharing.map(function sourceOf(other,): string {
      return other.source;
    },),).size === 1;
  },);
}

//endregion Han readings
