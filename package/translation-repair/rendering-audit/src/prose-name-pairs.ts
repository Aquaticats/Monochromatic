import {
  hanRunsOf,
  type ProseNamePair,
  settledPairs,
  stretchPairs,
} from './prose-name-han.ts';
import {
  letterReading,
  spellingIndex,
} from './prose-name-spelling.ts';

//region Prose name pairs
// A HAN NAME THE ARCHIVE SPELLS INSIDE A SENTENCE (the zheermao16 pass, slice
// 13, 2026-10-20). The page-name glossary pairs link texts, signatures and
// headings, so a cited author's handle written in a sentence had no archive
// rendering on any sheet: the archive's "Han Lianyi" for 寒涟漪 read as a
// handle with none, and a polish regrouped it and glossed it.
//
// THE PAIRING GOES FROM THE ARCHIVE'S LATIN WORDS TO THE ORIGINAL'S HAN, not
// the other way. A sentence is one unbroken run of Han, so the name cannot be
// cut out of it by looking at the Han alone; but a run of capitalised Latin
// words has a reading (its letters, accents and capitals folded away), and a
// stretch of Han characters has one too (their pinyin syllables joined). A name
// is paired when the two readings are the same letters, which is what the
// house rule's own grouping ignores ("Han Lianyi", "Hanlian Yi" and "Han Lian
// Yi" are one reading).
//
// WHAT KEEPS AN ENGLISH WORD OUT: only capitalised words are read; a single
// word the archive also writes in lower case is a word and no name (the 她可
// of "she can" reads "take"); a spelling inside a link address or code is no
// spelling; a name is at least two syllables and four letters; a stretch that
// is only a part of a longer name's pairing is dropped; and a spelling that
// more than one Han text reads as is left out, since the pairing could not say
// which text the archive spelled.

/**
 Most words one spelling may hold; a longer run of capitalised words is a
 title or a heading, not a name.

/**
 Han names of an original that the archive spells in Latin letters inside its
 sentences, paired with the spelling, by their reading letters.

 @param sourceText - original document or slice

 @param targetText - archive document or slice

 @returns One pair per name and spelling, in the order the original first
 writes them; none when either text has nothing to pair

 @example
 ```ts
 proseNamePairs({ sourceText: '雪团子来了。', targetText: 'Xue Tuanzi came.', },); // [{ source: '雪团子', rendering: 'Xue Tuanzi' }]
 ```
 */
export function proseNamePairs(
  {
    sourceText,
    targetText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
  },
): readonly ProseNamePair[] {
  /**
   The archive's spellings by reading.
   */
  const index = spellingIndex({ text: targetText, },);
  if (index.size === 0)
    return [];
  return settledPairs({
    found: hanRunsOf({ text: sourceText, },)
      .flatMap(function pairsOf(run,): readonly ProseNamePair[] {
        return stretchPairs({
          run,
          index,
        },);
      },),
  },);
}

/**
 The spellings a text gives a name's reading other than the archive's, when
 it does not write the archive's.

 @param text - candidate document or slice

 @param rendering - the archive's spelling of the name

 @returns Each other spelling of the same letters, in order; none when the
 text writes the archive's spelling as it stands or does not write the name

 @example
 ```ts
 respelledRendering({ text: 'Xuetuan Zi came.', rendering: 'Xue Tuanzi', },); // ['Xuetuan Zi']
 ```
 */
export function respelledRendering(
  {
    text,
    rendering,
  }: {
    readonly text: string;
    readonly rendering: string;
  },
): readonly string[] {
  /**
   The text's spellings by reading.
   */
  const index = spellingIndex({ text, },);
  /**
   Spellings of the name in the text, as written.
   */
  const spellings = index.get(letterReading({ text: rendering, },),) ?? [];
  return spellings.includes(rendering,) ? [] : spellings;
}

//endregion Prose name pairs
