import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { pinyin, } from 'pinyin-pro';
import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  isAsciiLetter,
  isHanCharacter,
} from '@monochromatic-dev/translation-repair-markup/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  rewriteEverySlice,
  type TextRewrite,
} from './page-slice-rewrite.ts';
import {
  inProse,
  protectedRanges,
} from '@monochromatic-dev/translation-repair-glossary/ts';

import { applySpanRewrites, } from './span-rewrites.ts';
import { runEnd, } from './text-runs.ts';

//region Pinyin tone
// CLASS ONE HUNDRED THIRTY-SEVEN (hulicaijia20, 2026-09-25): the archive's
// translator note glossed 金刚烷胺 as "jīn gāng wǎn’àn" to show the pun on
// 晚安 (wǎn’ān), and 烷 reads wán; the page carried the note as the archive
// wrote it, since no lane rewrites a note it has no reason to doubt. Where a
// parenthesis pairs a Han run with its tone-marked pinyin, one syllable to a
// character, and a character has a single reading, a syllable that differs
// from that reading in tone alone is written with the character's tone. A
// character with more than one reading, a count that does not match, a
// syllable that differs in more than tone, and pinyin without tone marks all
// stand aside: each could be the writer's reading, not a slip.

/**
 Vowels pinyin marks for tone.
 */
const TONE_MARKED: ReadonlySet<string> = new Set('āáǎàēéěèīíǐìōóǒòūúǔùǖǘǚǜ',);

/**
 Characters that part syllables inside a pinyin run.
 */
const SYLLABLE_BREAKS: ReadonlySet<string> = new Set([
  ' ',
  '’',
  '\'',
],);

/**
 Commas that may part a Han run from its pinyin.
 */
const PAIR_COMMAS: ReadonlySet<string> = new Set([
  ',',
  '，',
],);

/**
 The four combining tone marks (macron, acute, caron, grave), so a syllable
 decomposed to its base letters loses its tone and keeps the diaeresis of ü.
 */
const TONE_MARKS: readonly string[] = [
  '\u0304',
  '\u0301',
  '\u030C',
  '\u0300',
];

/**
 One syllable of a pinyin run and where it stands in the text.
 */
type Syllable = {
  readonly start: number;
  readonly end: number;
  readonly text: string;
};

/**
 One syllable rewritten to its character's tone.
 */
type ToneRewrite = {
  readonly start: number;
  readonly end: number;
  readonly from: string;
  readonly to: string;
  readonly character: string;
};

/**
 Whether one character may stand inside a pinyin run.

 @param character - one UTF-16 unit

 @returns Whether it is a letter, a tone-marked vowel, ü or a syllable break

 @example
 ```ts
 inPinyin({ character: 'ǎ', },); // true
 ```
 */
function inPinyin(
  { character, }: { readonly character: string; },
): boolean {
  return isAsciiLetter({ character, },)
    || TONE_MARKED.has(character,)
    || (character === 'ü')
    || SYLLABLE_BREAKS.has(character,);
}

/**
 Whether one character is anything but a tone-marked vowel.

 @param character - one UTF-16 unit

 @returns Whether it carries no tone mark

 @example
 ```ts
 unmarked({ character: 'a', },); // true
 ```
 */
function unmarked(
  { character, }: { readonly character: string; },
): boolean {
  return !TONE_MARKED.has(character,);
}

/**
 Whether one character is a space.

 @param character - one UTF-16 unit

 @returns Whether it is U+0020

 @example
 ```ts
 isSpace({ character: ' ', },); // true
 ```
 */
function isSpace(
  { character, }: { readonly character: string; },
): boolean {
  return character === ' ';
}

/**
 A syllable with its tone marks taken off, in lower case.

 @param syllable - pinyin syllable

 @returns Bare letters

 @example
 ```ts
 toneless({ syllable: 'Wǎn', },); // 'wan'
 ```
 */
function toneless(
  { syllable, }: { readonly syllable: string; },
): string {
  return TONE_MARKS.reduce(
    function unmark(
      bare,
      mark,
    ): string {
      return bare.replaceAll(
        mark,
        '',
      );
    },
    syllable.normalize('NFD',),
  )
    .normalize('NFC',)
    .toLowerCase();
}

/**
 The syllables of a pinyin run, parted at spaces and apostrophes.

 @param text - text under scan

 @param start - run's first offset

 @param end - run's exclusive end

 @returns Syllables in order

 @example
 ```ts
 syllablesOf({ text: '(鱼, yú’guàn)', start: 5, end: 12, },);
 ```
 */
function syllablesOf(
  {
    text,
    start,
    end,
  }: {
    readonly text: string;
    readonly start: number;
    readonly end: number;
  },
): readonly Syllable[] {
  /**
   Syllables read so far.
   */
  const syllables: Syllable[] = [];
  for (let at = start; at < end;) {
    if (SYLLABLE_BREAKS.has(text.charAt(at,),)) {
      at += 1;
      continue;
    }
    /**
     Where this syllable starts.
     */
    const from = at;
    while ((at < end) && (!SYLLABLE_BREAKS.has(text.charAt(at,),)))
      at += 1;
    syllables.push({
      start: from,
      end: at,
      text: text.slice(
        from,
        at,
      ),
    },);
  }
  return syllables;
}

/**
 The rewrite one syllable takes, or none where it stands as written.

 @param syllable - syllable under the check

 @param character - Han character it reads

 @returns Rewrite in a one-item list, or an empty list

 @example
 ```ts
 toneRewrite({ syllable: { start: 0, end: 3, text: 'wǎn' }, character: '烷', },);
 ```
 */
function toneRewrite(
  {
    syllable,
    character,
  }: {
    readonly syllable: Syllable;
    readonly character: string;
  },
): readonly ToneRewrite[] {
  /**
   Every reading the character has.
   */
  const readings = pinyin(
    character,
    {
      multiple: true,
      type: 'array',
    },
  );
  /**
   The character's reading where it has only one.
   */
  const [reading,] = readings;
  if ((readings.length !== 1) || (reading === undefined))
    return [];
  /**
   The syllable in lower case.
   */
  const lowered = syllable.text
    .toLowerCase();
  if ((lowered === reading) || (toneless({ syllable: lowered, },) !== toneless({ syllable: reading, },)))
    return [];
  /**
   Whether the writer capitalised the syllable.
   */
  const capitalised = lowered !== syllable.text;
  return [{
    start: syllable.start,
    end: syllable.end,
    from: syllable.text,
    to: capitalised ? `${reading.charAt(0,)
      .toUpperCase()}${reading.slice(1,)}` : reading,
    character,
  },];
}

/**
 The tone rewrites one parenthesis opening at an offset asks for.

 @param text - text under scan

 @param open - offset of the opening parenthesis

 @returns Rewrites for its pinyin, empty where it is not a Han and pinyin pair

 @example
 ```ts
 pairRewrites({ text: '(线, xiǎn)', open: 0, },);
 ```
 */
function pairRewrites(
  {
    text,
    open,
  }: {
    readonly text: string;
    readonly open: number;
  },
): readonly ToneRewrite[] {
  /**
   Where the Han run starts, just inside the parenthesis.
   */
  const hanStart = open + 1;
  /**
   Where the Han run ends.
   */
  const hanEnd = runEnd({
    text,
    from: hanStart,
    keeps: isHanCharacter,
  },);
  if ((hanEnd === hanStart) || (!PAIR_COMMAS.has(text.charAt(hanEnd,),)))
    return [];
  /**
   Where the pinyin run starts, past the comma and its spaces.
   */
  const pinyinStart = runEnd({
    text,
    from: hanEnd + 1,
    keeps: isSpace,
  },);
  /**
   Where the pinyin run ends.
   */
  const pinyinEnd = runEnd({
    text,
    from: pinyinStart,
    keeps: inPinyin,
  },);
  /**
   The pinyin run's syllables.
   */
  const syllables = syllablesOf({
    text,
    start: pinyinStart,
    end: pinyinEnd,
  },);
  /**
   Whether the run carries a tone mark, the sign it is written as pinyin.
   */
  const marked = runEnd({
    text,
    from: pinyinStart,
    keeps: unmarked,
  },) < pinyinEnd;
  /**
   The Han run's characters, whole. BY CODE POINT, NOT BY UTF-16 UNIT (ledger
   B21): pinyin-pro 3.29.3 reads Extension B ideographs too, each two units,
   so a count of units would pair a syllable with half a character.
   */
  const characters = Array.from(text.slice(
    hanStart,
    hanEnd,
  ),);
  if ((text.charAt(pinyinEnd,) !== ')') || (!marked)
    || (syllables.length !== characters.length))
    return [];
  return syllables.flatMap(function rewriteFor(
    syllable,
    index,
  ): readonly ToneRewrite[] {
    return toneRewrite({
      syllable,
      // The count check just matched the syllables to the characters.
      character: nonNullishOrThrow(characters[index],),
    },);
  },);
}

/**
 Writes every pinyin syllable of a Han and pinyin pair in its character's
 tone where the character has one reading and the syllable differs from it
 in tone alone.

 @param text - text under scan

 @returns Rewritten text and each change as "from" to "to" for its character

 @example
 ```ts
 correctPinyinTones({ text: '(线, xiǎn)', },).text; // '(线, xiàn)'
 ```
 */
export function correctPinyinTones(
  { text, }: { readonly text: string; },
): TextRewrite {
  /**
   The text's non-prose ranges.
   */
  const ranges = protectedRanges({ text, },);
  /**
   Every rewrite, in order.
   */
  const rewrites: ToneRewrite[] = [];
  for (let open = text.indexOf('(',); open !== (-1); open = text.indexOf(
    '(',
    open + 1,
  )) {
    /**
     Rewrites this parenthesis asks for, in prose only.
     */
    const asked = pairRewrites({
      text,
      open,
    },)
      .filter(function prose(rewrite,): boolean {
        return inProse({
          ranges,
          start: rewrite.start,
          end: rewrite.end,
        },);
      },);
    rewrites.push(...asked,);
  }
  /**
   Text with the rewrites applied, and the rewrites that were.
   */
  const rebuilt = applySpanRewrites({
    text,
    rewrites,
  },);
  return {
    text: rebuilt.text,
    changed: rebuilt.applied
      .map(function describe(rewrite,): string {
        return `"${rewrite.from}" to "${rewrite.to}" for ${rewrite.character}`;
      },),
  };
}

/**
 Corrects pinyin tones on every slice the page carries, the slices no lane
 replaced included.

 @param slices - prepared pairs, whose archive text stands where no row replaces it

 @param replacements - what the page would write per slice

 @param archiveOriginalSpans - spans sealed as the English original

 @returns Replacements with the tones corrected, the rewritten rows alone,
 and one finding per slice changed

 @example
 ```ts
 const page = correctPinyinPage({ slices, replacements, archiveOriginalSpans: [], },);
 ```
 */
export function correctPinyinPage(
  {
    slices,
    replacements,
    archiveOriginalSpans,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
    readonly archiveOriginalSpans: readonly ArchiveOriginalSpan[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  return rewriteEverySlice({
    slices,
    replacements,
    archiveOriginalSpans,
    rewrite: correctPinyinTones,
    findingName: 'pinyin-tone-corrected',
  },);
}
//endregion Pinyin tone
