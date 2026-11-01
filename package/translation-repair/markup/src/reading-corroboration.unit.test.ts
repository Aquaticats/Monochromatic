/**
 Tests for whether two models' readings of the same picture corroborate
 each other, purely from how much of their character trigrams overlap.

 WHAT THESE PIN is the three design decisions the module's own header
 explains and measures: trigrams rather than shared anchors or single
 characters, dividing by the smaller reading so one model transcribing more
 of a picture than another is not scored as disagreement, and cutting by
 code point so a reading carrying an emoji or another character outside the
 basic plane still cuts the same way for every reader. The boundary case
 pins the comparison's direction directly, since `overlap < threshold` and
 `overlap <= threshold` differ only at the one value a range assertion over
 ordinary fixtures could never land on by chance.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  characterTrigrams,
  CORROBORATION_TRIGRAM_SHARE,
  readingsCorroborate,
  trigramOverlap,
} from '../dist/final/node/index.mjs';

/**
 Reading too short to cut into any trigram: a torn collar tag with only two
 legible characters, standing in for whatever a model hands back when a
 picture carries almost nothing to transcribe.
 */
const TORN_TAG_READING = 'Mu';

/**
 Opening clause of {@link LONG_NAP_READING}, kept identical so every
 trigram it cuts into is guaranteed to reappear among that longer
 reading's own trigrams.
 */
const SHORT_NAP_READING = 'Tabby napped';

/**
 Fuller account of the same moment as {@link SHORT_NAP_READING}, sharing
 its opening verbatim and then carrying on past it: everything the short
 reading vouches for, this one carries too, plus more the short one never
 mentioned.
 */
const LONG_NAP_READING = 'Tabby napped on the sunny windowsill for three hours';

/**
 One model's reading of a picture showing a tabby cat asleep on a
 windowsill in the afternoon sun.
 */
const WINDOWSILL_READING =
  'Whiskers the tabby cat is curled up asleep on the windowsill, warmed by the afternoon sun, '
    + 'her tail draped over the edge.';

/**
 A second model's reading of that same picture: it agrees with
 {@link WINDOWSILL_READING} on every fact but reorders the clauses and
 swaps a few words, the way two independent readings of one picture do.
 */
const AGREEING_READING =
  'Asleep on the windowsill in the afternoon sun, a tabby cat named Whiskers lies curled up '
    + 'with her tail hanging over the edge.';

/**
 A reading of some other picture entirely, sharing almost no vocabulary
 with {@link WINDOWSILL_READING} beyond the handful of short words any two
 sentences in the same language are bound to carry.
 */
const DISAGREEING_READING =
  'Biscuit the black kitten chases a red laser dot across the kitchen floor, knocking a spoon '
    + 'off the counter at midnight.';

/**
 Builds two readings whose measured trigram overlap sits at a chosen share
 of the shorter one's trigrams, for pinning behaviour that depends on
 landing on an exact number rather than falling somewhere in a range.

 TWELVE PAIRWISE-DISTINCT LETTERS stand in for a full microchip scan. Since
 no letter repeats, every one of the ten trigrams that scan cuts into is
 distinct by construction, so the shared-trigram count a given share needs
 is exact arithmetic rather than something to count by eye. The partner
 reading carries a verbatim prefix of that scan and then diverges into
 digits, which share no letters with the alphabet the scan itself uses, so
 nothing past the shared prefix can coincidentally match.

 @param share - fraction of full reading's trigrams partner must reproduce

 @returns Full reading and partner sharing requested share of its trigrams

 @example
 ```ts
 const { full, partial, } = trigramSharePair({ share: CORROBORATION_TRIGRAM_SHARE, },);
 ```
 */
function trigramSharePair({ share, }: { readonly share: number; },): {
  readonly full: string;
  readonly partial: string;
} {
  /**
   Full scan: twelve pairwise-distinct letters, so every trigram it cuts
   into is distinct too.
   */
  const full = 'abcdefghijkl';

  /**
   That scan's own trigrams, whose count anchors how many the partner must
   reproduce.
   */
  const fullGrams = characterTrigrams({ text: full, },);

  /**
   How many of the full scan's trigrams the partner must carry to land on
   the requested share.
   */
  const sharedCount = fullGrams.size * share;

  /**
   Prefix of the full scan cutting into exactly `sharedCount` trigrams: a
   run of `n` characters yields `n - 2` three-character windows.
   */
  const sharedPrefix = full.slice(0, sharedCount + 2,);

  return {
    full,
    partial: `${sharedPrefix}12345678`,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: characterTrigrams.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CUTS OVERLAPPING WINDOWS INTO A SET, COLLAPSING A REPEATED TRIGRAM INTO ONE ENTRY '
            + 'rather than counting every window it came from, since two windows made of the same '
            + 'three characters carry no more evidence of a match than one',
          fn: async () => {
            /**
             A long purr, spelled with a run of repeated letters so one of its
             three-character windows recurs.
             */
            const grams = characterTrigrams({ text: 'purrrr', },);

            expect(grams.size,).toBe(3,);
            expect(grams.has('pur',),).toBe(true,);
            expect(grams.has('urr',),).toBe(true,);
            expect(grams.has('rrr',),).toBe(true,);
          },
        },),

        it({
          name: 'COLLAPSES WHITESPACE BEFORE CUTTING GRAMS, SO A NEWLINE WHERE ONE READING BROKE ITS '
            + 'line and a plain space where the other did not still cut into the same trigrams: two '
            + 'models lay a transcription out differently, and layout is not a fact about the picture',
          fn: async () => {
            /**
             Reading laid out on one line, the way a model that runs its
             transcription together would return it.
             */
            const onOneLine = characterTrigrams({ text: 'Tabby purrs loudly', },);

            /**
             The same words, laid out as a model that preserves line breaks
             would return them: a newline sits where `onOneLine`
             has a plain space.
             */
            const acrossLines = characterTrigrams({ text: 'Tabby\npurrs\nloudly', },);

            expect(onOneLine.size,).toBe(acrossLines.size,);
            for (const gram of onOneLine)
              expect(acrossLines.has(gram,),).toBe(true,);
          },
        },),

        it({
          name: 'RETURNS AN EMPTY SET FOR A READING SHORTER THAN THE WIDTH OF ONE TRIGRAM, rather '
            + 'than a shorter partial gram, since a two-character reading carries no three-character '
            + 'window at all',
          fn: async () => {
            /**
             Two characters, one short of the minimum a single trigram needs.
             */
            const grams = characterTrigrams({ text: TORN_TAG_READING, },);

            expect(grams.size,).toBe(0,);
          },
        },),

        it({
          name: 'CUTS BY CODE POINT RATHER THAN UTF-16 UNIT, SO A CAT EMOJI OPENING A READING STAYS '
            + 'whole inside its trigram instead of splintering into the lone surrogate halves that '
            + 'indexing by UTF-16 unit would produce',
          fn: async () => {
            /**
             A model's reply that led with a cat emoji before naming what the
             picture shows. The emoji occupies two UTF-16 units but one code
             point, so cutting by the wrong unit would visibly change what
             this produces.
             */
            const reading = '\u{1F408} nap';

            /**
             UTF-16 length of the reading: six units for five code points,
             confirming the fixture actually differs between the two ways of
             counting length, which is what lets it catch a UTF-16 regression.
             */
            const utf16Length = reading.length;

            expect(utf16Length,).toBe(6,);

            /**
             Trigrams cut from that reading. Correct code-point cutting keeps
             the emoji whole inside the first window rather than splitting its
             surrogate pair across two malformed ones.
             */
            const grams = characterTrigrams({ text: reading, },);

            expect(grams.size,).toBe(3,);
            expect(grams.has('\u{1F408} n',),).toBe(true,);
            expect(grams.has(' na',),).toBe(true,);
            expect(grams.has('nap',),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: trigramOverlap.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'DIVIDES BY THE SMALLER SIDE, SO A SHORT READING FULLY CONTAINED IN A LONGER ONE '
            + 'scores a perfect overlap instead of being penalised for the extra content the longer '
            + 'reading carries: one model transcribing more of a picture than another is not disagreement',
          fn: async () => {
            /**
             Overlap of a reading against a longer one that opens with that
             exact reading verbatim before continuing past it.
             */
            const overlap = trigramOverlap({ left: SHORT_NAP_READING, right: LONG_NAP_READING, },);

            expect(overlap,).toBe(1,);
          },
        },),

        it({
          name: 'RETURNS ZERO OVERLAP WHEN EITHER SIDE CARRIES NO TRIGRAM, since a share computed '
            + 'against zero trigrams has nothing to divide by and a reading that short cannot vouch '
            + 'for anything',
          fn: async () => {
            expect(trigramOverlap({ left: TORN_TAG_READING, right: LONG_NAP_READING, },),).toBe(0,);

            expect(trigramOverlap({ left: LONG_NAP_READING, right: TORN_TAG_READING, },),).toBe(0,);
          },
        },),

        it({
          name: 'RETURNS A PERFECT OVERLAP FOR TWO IDENTICAL READINGS, the ceiling every other '
            + 'measurement in this file is read relative to',
          fn: async () => {
            expect(trigramOverlap({ left: LONG_NAP_READING, right: LONG_NAP_READING, },),).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: readingsCorroborate.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES THE MEASURED OVERLAP BESIDE ITS VERDICT WHETHER THE READINGS AGREE OR NOT, '
            + 'so a run can be read for how close a corroboration or a disagreement sat to the '
            + 'threshold rather than only which side of it landed',
          fn: async () => {
            /**
             Verdict over a reading fully contained in a longer one, whose
             overlap sits at the ceiling.
             */
            const corroborated = readingsCorroborate({ left: SHORT_NAP_READING, right: LONG_NAP_READING, },);

            expect(corroborated.kind,).toBe('corroborated',);
            expect(corroborated.overlap,).toBe(1,);

            /**
             Verdict over a reading too short to carry any trigram, whose
             overlap sits at the floor.
             */
            const disagreeing = readingsCorroborate({ left: TORN_TAG_READING, right: LONG_NAP_READING, },);

            expect(disagreeing.kind,).toBe('disagree',);
            expect(disagreeing.overlap,).toBe(0,);
          },
        },),

        it({
          name: 'SEPARATES TWO UNRELATED CAT-THEMED PASSAGES AS DISAGREEING, which is what a reading '
            + 'of the wrong picture looks like: prose in the same language still shares its short '
            + 'common words, but not enough of them to clear the threshold',
          fn: async () => {
            /**
             Verdict comparing readings of two different pictures.
             */
            const verdict = readingsCorroborate({ left: WINDOWSILL_READING, right: DISAGREEING_READING, },);

            expect(verdict.kind,).toBe('disagree',);
            expect(verdict.overlap,).toBeLessThan(CORROBORATION_TRIGRAM_SHARE,);
          },
        },),

        it({
          name: 'CORROBORATES TWO PARAPHRASES OF THE SAME CAT-THEMED PASSAGE, which is what two '
            + 'models independently reading the same picture look like: the clauses reorder and a '
            + 'few words change, but the names and the scene they describe do not',
          fn: async () => {
            /**
             Verdict comparing two independently worded readings of the same
             picture.
             */
            const verdict = readingsCorroborate({ left: WINDOWSILL_READING, right: AGREEING_READING, },);

            expect(verdict.kind,).toBe('corroborated',);
            expect(verdict.overlap,).toBeGreaterThan(CORROBORATION_TRIGRAM_SHARE,);
          },
        },),

        it({
          name: 'CORROBORATES A READING SITTING EXACTLY ON THE THRESHOLD, since the module compares '
            + 'with a strict less-than for disagreement and leaves equality on the corroborating '
            + 'side, the one boundary value where getting the comparison direction wrong would not '
            + 'show up as a range error',
          fn: async () => {
            /**
             Full and partial microchip-scan readings whose measured overlap
             sits at the corroboration threshold, built once here so both
             assertions of this case measure the same pair.
             */
            const { full, partial, } = trigramSharePair({ share: CORROBORATION_TRIGRAM_SHARE, },);

            /**
             Measured directly rather than assumed, so this test fails loudly
             if the fixture ever stops landing exactly on the threshold.
             */
            const overlap = trigramOverlap({ left: full, right: partial, },);

            expect(overlap,).toBe(CORROBORATION_TRIGRAM_SHARE,);

            /**
             Verdict over that same pair, which is what the boundary claim is
             actually about.
             */
            const verdict = readingsCorroborate({ left: full, right: partial, },);

            expect(verdict.kind,).toBe('corroborated',);
            expect(verdict.overlap,).toBe(CORROBORATION_TRIGRAM_SHARE,);
          },
        },),
      ],
    },),
  ],
},);
