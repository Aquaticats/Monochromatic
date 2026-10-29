/**
 Tests for the deterministic half of the insertion gate.

 `doc/decision/translation-repair-absence-verdict.md` requires two independent
 readings before anything is written into a page: the coverage roster must
 call a passage absent, AND the page must be measurably too short to hold it.
 These cover the second, which consults no model and so can be pinned exactly.

 WHAT WOULD GO WRONG WITHOUT THEM. A gate that reported every page as short
 would wave through every insertion the roster proposed, and a gate that
 reported none as short would silently disable the lane; both look identical
 from outside, since both produce a run that raises nothing. The cases here
 exercise both directions on the same fixture so neither degenerate answer can
 pass.

 Fixtures are cat-themed invention. The corpus is unlicensed and this file is
 committed, so no passage from it appears here; what is borrowed is the SHAPE,
 a Chinese source beside an English rendering some multiple of its size.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  admitWithinShortfall,
  CORPUS_EXPANSION,
  expectedTranslationPoints,
  pageShortfall,
} from '../dist/final/node/index.mjs';

/**
 Chinese source standing in for a page, long enough that expansion arithmetic
 lands on numbers worth asserting about.
 */
const SOURCE = '橘猫在窗台上睡了整个下午，阳光把它的毛烤得暖烘烘的。';

/**
 What an ordinarily complete rendering of that source would occupy.
 */
const WHOLE = SOURCE.length * CORPUS_EXPANSION;

/**
 Two passages of equal size, each small enough that a page missing everything
 has room for both.
 */
const PASSAGES = [
  {
    position: 1,
    sourceText: '第一只猫。',
  },
  {
    position: 2,
    sourceText: '第二只猫。',
  },
] as const;

await describe({
  name: 'coverage corroboration',
  children: [
    it({
      name: 'SCALES a source by the measured corpus median, so the expectation is the corpus '
        + 'reading rather than a number chosen to make a page pass',
      fn: async () => {
        // Every character in the fixture is BMP, so code points and UTF-16
        // units agree and the expectation can be written against `length`.
        expect(expectedTranslationPoints({ sourceText: SOURCE, },),)
          .toBeCloseTo(
          WHOLE,
          6,
          );
      },
    },),

    it({
      name: 'COUNTS code points rather than UTF-16 units, since counting units inflates a source '
        + 'holding rare characters and admits passages the page has no room for',
      fn: async () => {
        // A cat face outside the BMP occupies two UTF-16 units and one code
        // point.
        expect(expectedTranslationPoints({ sourceText: '🐱', },),)
          .toBeCloseTo(
          CORPUS_EXPANSION,
          6,
          );
      },
    },),

    it({
      name: 'IGNORES surrounding whitespace, which is content on neither side',
      fn: async () => {
        expect(expectedTranslationPoints({ sourceText: `\n\n  ${SOURCE}  \n`, },),)
          .toBeCloseTo(
          expectedTranslationPoints({ sourceText: SOURCE, },),
          6,
          );
      },
    },),

    it({
      name: 'REPORTS a page missing most of itself as short',
      fn: async () => {
        expect(pageShortfall({
          sourceText: SOURCE,
          targetText: 'The cat slept.',
        },),)
          .toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'REPORTS a page of ordinary length as not short, which is the direction that stops '
        + 'the gate waving through every insertion the roster proposes',
      fn: async () => {
        expect(pageShortfall({
          sourceText: SOURCE,
          targetText: 'x'.repeat(Math.ceil(WHOLE,) + 1,),
        },),)
          .toBe(0,);
      },
    },),

    it({
      name: 'FLOORS at zero rather than going negative, since a translation running long is not '
        + 'evidence of anything and would otherwise subtract from a budget',
      fn: async () => {
        expect(pageShortfall({
          sourceText: SOURCE,
          targetText: 'x'.repeat(SOURCE.length * 10,),
        },),)
          .toBe(0,);
      },
    },),

    it({
      name: 'SIZES the gap to what is missing, so a page holding almost nothing reads as shorter '
        + 'than one holding half',
      fn: async () => {
        expect(pageShortfall({
          sourceText: SOURCE,
          targetText: 'x',
        },),)
          .toBeGreaterThan(pageShortfall({
            sourceText: SOURCE,
            targetText: 'x'.repeat(Math.floor(WHOLE / 2,),),
          },),);
      },
    },),

    it({
      name: 'ADMITS nothing into a page of ordinary length, however many passages the roster '
        + 'voted absent on',
      fn: async () => {
        expect(admitWithinShortfall({
          sourceText: SOURCE,
          targetText: 'x'.repeat(Math.ceil(WHOLE,) + 1,),
          passages: PASSAGES,
        },),)
          .toStrictEqual([],);
      },
    },),

    it({
      name: 'ADMITS both when the page has room for both',
      fn: async () => {
        expect(admitWithinShortfall({
          sourceText: SOURCE,
          targetText: '',
          passages: PASSAGES,
        },),)
          .toStrictEqual([
            1,
            2,
          ],);
      },
    },),

    it({
      name: 'STOPS at what the page is actually missing, so a page short by room for one passage '
        + 'never takes two',
      fn: async () => {
        /**
         What admitting one of these passages is expected to add.
         */
        const one = expectedTranslationPoints({ sourceText: PASSAGES[0].sourceText, },);

        /**
         Half again of that, so the budget holds one passage and a sliver.
         */
        const ROOM_FOR_ONE = 1.5;

        expect(admitWithinShortfall({
          sourceText: SOURCE,
          targetText: 'x'.repeat(Math.floor(WHOLE - (one * ROOM_FOR_ONE),),),
          passages: PASSAGES,
        },),)
          .toStrictEqual([
            1,
          ],);
      },
    },),

    it({
      name: 'SKIPS a passage too large and keeps spending on smaller ones, since giving up at '
        + 'the first unaffordable candidate would let one large one veto every later one',
      fn: async () => {
        expect(admitWithinShortfall({
          sourceText: SOURCE,
          targetText: '',
          passages: [
            {
              position: 0,
              // TWICE the page's own source, so its rendering cannot fit a
              // budget that is only the page's whole expected size. At exactly
              // the source length it FITS, which is correct and was what this
              // fixture first got wrong.
              sourceText: '猫'.repeat(SOURCE.length * 2,),
            },
            {
              position: 1,
              sourceText: '猫',
            },
          ],
        },),)
          .toStrictEqual([
            1,
          ],);
      },
    },),

    it({
      name: 'ADMITS nothing from an empty candidate list, rather than reading the budget as a '
        + 'licence to write',
      fn: async () => {
        expect(admitWithinShortfall({
          sourceText: SOURCE,
          targetText: '',
          passages: [],
        },),)
          .toStrictEqual([],);
      },
    },),
  ],
},);
