import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Assembly adjacent repetition
// THE DOCUMENT-SCALE CHECK CANNOT SEE THE DAMAGE IT WAS BUILT FOR, which is why
// this file exists beside it rather than inside it.
//
// `findIntroducedRepetitions` requires a phrase to carry two words of at least
// five letters before it will report anything. That gate is load-bearing at
// DOCUMENT scale, where any two distant sentences may share ordinary phrasing,
// and removing it took a measured reading back to mostly noise.
//
// Its motivating example does not clear it. The duplication `lintong` shipped
// across slices 2 and 3 is six words of twenty-three characters whose word
// lengths are 4,3,3,2,3,3, so it carries no content word at all and the shipped
// check reports nothing on the defect it was written to catch.
//
// ADJACENCY IS THE SPECIFICITY. Two consecutive slices shipping the same
// wording, which the archive did not repeat, is a much narrower claim than two
// distant passages agreeing, so it needs no content gate to stay quiet.
// Measured over every settled artifact carrying a delivery ledger, this fires
// once in twenty-two lane readings, and that once is the documented damage.
// `doc/audit/an-archive-rebuilt-from-the-ledger-is-not-the-archive.md` records
// the measurement.

import {
  countPhrases,
  holdsPhrase,
  whitespaceTokensOf,
} from './assembly-repetition.ts';

/**
 Shortest repeat worth reporting, in words.

 Matches {@link MIN_PHRASE_WORDS} in the document-scale check deliberately:
 adjacency relaxes the CONTENT requirement, not the length one, because three
 shared words between neighbouring passages is still ordinary English.
 */
const MIN_ADJACENT_WORDS = 4;

/**
 Longest repeat looked for, in words.

 A repeat longer than this is reported at this length, which is enough to name
 it; the finding carries counts rather than wording, so nothing is gained by
 growing the window further.
 */
const MAX_ADJACENT_WORDS = 12;

/**
 Archive occurrences at which a repeat stops being ours.

 An archive that already says a phrase twice is an author who repeats, and a
 document preserving that is faithful rather than damaged. Measured on
 `lintong`, where a phrase the archive states twice was reported as introduced
 by a reader that had rebuilt the archive from sliced text and lost one of the
 two occurrences.
 */
const ARCHIVE_REPEAT_FLOOR = 2;

/**
 One slice's shipped wording, in document order.

 @example
 ```ts
 const slice: AdjacentSliceText = { sliceIndex: 3, text: 'the tabby waited', };
 ```
 */
export type AdjacentSliceText = {
  /**
   Global slice index, as `prepareDocumentPair` stamped it.
   */
  readonly sliceIndex: number;

  /**
   Wording this slice contributed to the assembled document, whether it is a
   replacement the lane wrote or the incumbent it kept.
   */
  readonly text: string;
};

/**
 Wording two neighbouring slices both shipped, which the archive did not
 repeat.

 CARRIES NO WORDING. The slice pair and the measurements locate the finding
 well enough to act on, and a findings list travels into logs and artifacts
 where corpus text does not belong.

 @example
 ```ts
 const found: AdjacentRepetition = {
   earlierSliceIndex: 2,
   laterSliceIndex: 3,
   words: 6,
   characters: 23,
   archiveOccurrences: 1,
 };
 ```
 */
export type AdjacentRepetition = {
  /**
   Earlier slice of the neighbouring pair.
   */
  readonly earlierSliceIndex: number;

  /**
   Later slice of the neighbouring pair.
   */
  readonly laterSliceIndex: number;

  /**
   Repeat length in words.
   */
  readonly words: number;

  /**
   Repeat length in characters, spaces between words included.
   */
  readonly characters: number;

  /**
   Times the archive stated this wording, always below
   {@link ARCHIVE_REPEAT_FLOOR}.
   */
  readonly archiveOccurrences: number;
};

/**
 Two slices that sit next to each other in the assembled document.

 @example
 ```ts
 const pair: NeighbouringPair = { earlier, later, };
 ```
 */
type NeighbouringPair = {
  /**
   Slice that appears first.
   */
  readonly earlier: AdjacentSliceText;

  /**
   Slice that follows it directly.
   */
  readonly later: AdjacentSliceText;
};

/**
 Pairs each slice with the one after it, in document order.

 @param slices - shipped slice wordings in document order

 @returns Neighbouring pairs, one fewer than the slices given

 @example
 ```ts
 const pairs = neighbouringPairs({ slices, },);
 ```
 */
function neighbouringPairs(
  {
    slices,
  }: {
    readonly slices: readonly AdjacentSliceText[];
  },
): readonly NeighbouringPair[] {
  return slices
    .flatMap(function withNext(
      earlier,
      at,
    ) {
      /**
       Slice following this one, absent at the last position.
       */
      const later = slices[at + 1];
      return (later === undefined)
        ? []
        : [
          {
            earlier,
            later,
          },
        ];
    },);
}

/**
 One kept repeat, held with the wording that produced it.

 The wording is needed only to drop shorter matches contained in one already
 kept, and it never leaves this file.

 @example
 ```ts
 const kept: KeptRepeat = { phrase: 'by the garden gate again', found, };
 ```
 */
type KeptRepeat = {
  /**
   Repeated wording, normalised to single spaces.
   */
  readonly phrase: string;

  /**
   What is reported for it.
   */
  readonly found: AdjacentRepetition;
};

/**
 Names wording both slices of one pair carry, longest first.

 MAXIMAL MATCHES ONLY, for the reason {@link findIntroducedRepetitions} gives:
 a shared eight-word passage also shares as five four-word ones, and reporting
 every one buries the finding in its own substrings.

 @param earlier - earlier slice of the pair

 @param later - later slice of the pair

 @param archiveWords - archive document as words, for the faithfulness test

 @returns Repeats this pair introduced, longest first

 @example
 ```ts
 const found = repeatsInPair({ earlier, later, archiveWords, },);
 ```
 */
function repeatsInPair(
  {
    earlier,
    later,
    archiveWords,
  }: {
    readonly earlier: AdjacentSliceText;
    readonly later: AdjacentSliceText;
    readonly archiveWords: readonly string[];
  },
): readonly AdjacentRepetition[] {
  /**
   Earlier slice as words, which the candidate windows are cut from.
   */
  const earlierWords = whitespaceTokensOf({ text: earlier.text, },);

  /**
   Later slice as words, for membership tests at each length.
   */
  const laterWords = whitespaceTokensOf({ text: later.text, },);

  /**
   Repeats kept so far, with the wording that produced each, so a shorter
   match contained in one already kept can be dropped.
   */
  const kept: KeptRepeat[] = [];

  // Longest first, so a maximal match is always seen before its substrings.
  for (
    let length = MAX_ADJACENT_WORDS;
    length >= MIN_ADJACENT_WORDS;
    length -= 1
  ) {
    /**
     Every phrase of this length the later slice carries.
     */
    const laterPhrases = countPhrases({
      words: laterWords,
      length,
    },);

    /**
     Every phrase of this length the archive carries, with its count.
     */
    const archivePhrases = countPhrases({
      words: archiveWords,
      length,
    },);

    for (const [phrase, ] of countPhrases({
      words: earlierWords,
      length,
    },)) {
      if (!laterPhrases.has(phrase,))
        continue;

      /**
       Times the archive already stated this wording.
       */
      const archiveOccurrences = archivePhrases.get(phrase,) ?? 0;
      if (archiveOccurrences >= ARCHIVE_REPEAT_FLOOR)
        continue;
      if (kept
        .some(function contains(seen,): boolean {
          return holdsPhrase({
            longer: seen.phrase,
            phrase,
          },);
        },))
        continue;
      kept.push({
        phrase,
        found: {
          earlierSliceIndex: earlier.sliceIndex,
          laterSliceIndex: later.sliceIndex,
          words: length,
          characters: phrase.length,
          archiveOccurrences,
        },
      },);
    }
  }
  return kept
    .map(function toFound(entry,): AdjacentRepetition {
      return entry.found;
    },);
}

/**
 Names wording neighbouring slices both shipped that the archive did not
 repeat.

 @param archiveText - translation as it stood before the pipeline ran

 @param shippedSlices - shipped slice wordings in document order

 @returns Adjacent repetitions, in document order and longest first per pair

 @example
 ```ts
 const found = findAdjacentRepetitions({ archiveText, shippedSlices, },);
 ```
 */
export function findAdjacentRepetitions(
  {
    archiveText,
    shippedSlices,
  }: {
    readonly archiveText: string;
    readonly shippedSlices: readonly AdjacentSliceText[];
  },
): readonly AdjacentRepetition[] {
  /**
   Archive as words, cut once rather than per pair.
   */
  const archiveWords = whitespaceTokensOf({ text: archiveText, },);
  return neighbouringPairs({ slices: shippedSlices, },)
    .flatMap(function inPair(pair,): readonly AdjacentRepetition[] {
      return repeatsInPair({
        earlier: pair.earlier,
        later: pair.later,
        archiveWords,
      },);
    },);
}

/**
 Renders adjacent repetitions as assembly findings.

 NAMES NO WORDING, for the reason {@link AdjacentRepetition} gives.

 @param archiveText - translation as it stood before the pipeline ran

 @param shippedSlices - shipped slice wordings in document order

 @returns One finding per adjacent repetition

 @example
 ```ts
 const findings = adjacentRepetitionFindings({ archiveText, shippedSlices, },);
 ```
 */
export function adjacentRepetitionFindings(
  {
    archiveText,
    shippedSlices,
  }: {
    readonly archiveText: string;
    readonly shippedSlices: readonly AdjacentSliceText[];
  },
): readonly string[] {
  return findAdjacentRepetitions({
    archiveText,
    shippedSlices,
  },)
    .map(function toFinding(found,): string {
      return `adjacent-repetition (slices ${String(found.earlierSliceIndex,)} and ${
        String(found.laterSliceIndex,)
      }, ${String(found.words,)} ${
        wordForCount({
          count: found.words,
          one: 'word',
          many: 'words',
        },)
      }, ${String(found.characters,)} ${
        wordForCount({
          count: found.characters,
          one: 'character',
          many: 'characters',
        },)
      }, archive ${
        String(found.archiveOccurrences,)
      })`;
    },);
}

//endregion Assembly adjacent repetition
