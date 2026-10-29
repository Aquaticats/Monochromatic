/**
 Tests for the content-survival check: what counts as a specific, and what
 does not.

 WHY IT EXISTS: a slice rewritten into generic prose keeps the document's
 length and structure and loses what it was about, so every other instrument
 in this package reads clean on it. Measured on real output in
 `doc/audit/the-damage-no-instrument-was-catching.md`.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  contentSurvivalFindings,
  distinctiveWords,
  measureContentSurvival,
} from '../dist/final/node/index.mjs';

/**
 An archive passage carrying specifics: long words used once each.
 */
const ARCHIVE = 'She kept her collection of tortoiseshell buttons and lavender ribbons '
  + 'inside a battered biscuit tin beneath the windowsill.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: distinctiveWords.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES long words the archive uses rarely',
          fn: async () => {
            const words = distinctiveWords({ archiveText: ARCHIVE, },);
            expect(words,).toContain('tortoiseshell',);
            expect(words,).toContain('lavender',);
            expect(words,).toContain('windowsill',);
          },
        },),
        it({
          name: 'REFUSES words shorter than the six-letter floor',
          fn: async () => {
            const words = distinctiveWords({ archiveText: ARCHIVE, },);
            expect(words,).not.toContain('tin',);
            expect(words,).not.toContain('she',);
            expect(words,).not.toContain('kept',);
          },
        },),
        it({
          name: 'REFUSES a long word the archive leans on, which is register not specifics',
          fn: async () => {
            const words = distinctiveWords({
              archiveText: 'The kitten remembers. The tabby remembers. The old cat remembers too.',
            },);
            expect(words,).not.toContain('remembers',);
          },
        },),
        it({
          name: 'RETURNS nothing for text with no long rare words',
          fn: async () => {
            expect(distinctiveWords({ archiveText: 'the cat sat on the mat', },).length,).toBe(0,);
          },
        },),
        it({
          name: 'HOLDS BOTH BOUNDARIES: six letters counts and five does not; two uses count and three do not',
          fn: async () => {
            /** Six- and five-letter words once each, and two long words used twice and thrice. */
            const words = distinctiveWords({
              archiveText: 'purred tabby. biscuit biscuit. whisker whisker whisker.',
            },);
            expect(words,).toContain('purred',);
            expect(words,).not.toContain('tabby',);
            expect(words,).toContain('biscuit',);
            expect(words,).not.toContain('whisker',);
          },
        },),
      ],
    },),

    describe({
      name: measureContentSurvival.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS every specific as kept when the document preserves them',
          fn: async () => {
            const survival = measureContentSurvival({
              archiveText: ARCHIVE,
              shippedText: `Truly, ${ARCHIVE}`,
            },);
            expect(survival.lost,).toBe(0,);
            expect(survival.kept,).toBe(survival.distinctive,);
          },
        },),
        it({
          name: 'COUNTS specifics as lost when a detail is rewritten into generic prose',
          fn: async () => {
            const survival = measureContentSurvival({
              archiveText: ARCHIVE,
              // The shape of the real damage: same register, no specifics, similar length.
              shippedText: 'She kept a number of small treasured objects tucked away somewhere safe.',
            },);
            expect(survival.distinctive,).toBeGreaterThan(0,);
            expect(survival.lost,).toBe(survival.distinctive,);
            expect(survival.kept,).toBe(0,);
          },
        },),
        it({
          name: 'IGNORES punctuation and case, which are not content',
          fn: async () => {
            const survival = measureContentSurvival({
              archiveText: 'The lavender ribbons.',
              shippedText: '(LAVENDER, ribbons!)',
            },);
            expect(survival.lost,).toBe(0,);
          },
        },),
        it({
          name: 'COUNTS an accented word as one specific whose length is its letters, composed or combining, and keeps it '
            + 'where the document writes it without the accent (ledger B18)',
          fn: async () => {
            // ASCII letters alone cut `château` into `ch` and `teau` and `Émilie`
            // into `milie`, all too short to count, so losing either went unseen;
            // counted by UTF-16 unit, a combining accent made a five-letter word
            // six long.
            for (const archiveText of ['Mittens napped in the château with Émilie and Émile.', 'Mittens napped in the cha\u{0302}teau with E\u{0301}milie and E\u{0301}mile.',]) {
              expect([
                measureContentSurvival({
                  archiveText,
                  shippedText: 'Mittens napped in the house.',
                },),
                measureContentSurvival({
                  archiveText,
                  shippedText: 'Mittens napped in the chateau with Emilie.',
                },),
              ].map(function counts({ distinctive, lost, },): readonly number[] {
                return [distinctive, lost,];
              },),).toEqual([[4, 2,], [4, 0,],],);
            }
          },
        },),
      ],
    },),

    describe({
      name: contentSurvivalFindings.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the counts, and NO wording',
          fn: async () => {
            const findings = contentSurvivalFindings({
              archiveText: ARCHIVE,
              shippedText: 'She kept a number of small treasured objects tucked away somewhere safe.',
            },);
            expect(findings.length,).toBe(1,);
            expect(findings[0],).toContain('content-survival',);
            expect(findings[0],).toContain('distinctive archive words kept',);
            // The whole point of rendering counts: corpus wording must not travel.
            expect(findings[0],).not.toContain('tortoiseshell',);
          },
        },),
        it({
          name: 'RETURNS nothing when the archive carries no specifics to lose',
          fn: async () => {
            expect(
              contentSurvivalFindings({
                archiveText: 'the cat sat on the mat',
                shippedText: 'a cat sat',
              },).length,
            ).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
