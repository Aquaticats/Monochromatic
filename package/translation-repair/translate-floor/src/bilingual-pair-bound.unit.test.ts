/**
 Guards the second arm of class eighty (2026-09-22). The
 sheet clause of `bilingual-line-clause.ts` reached every wording and the
 judges still chose the candidate that rendered the film quote's Chinese line
 a second time in English, calling the candidate carrying the English alone
 "a dropped line ... ineligible" under the line criterion, and one judge
 reasoning that the Chinese line "is a literal translation of the sentiment,
 which is distinct from the actual English quote". A judge without the film
 cannot see that the two lines are one, so the question is settled where the
 evidence is: the EXISTING TRANSLATION, a human's, carried the pair as one
 line, and a rendering carrying more lines than that on a slice with such a
 pair has invented one. Without page text, or where the page itself keeps the
 two lines apart (an introduction then a quoted poem, on ten other entries),
 nothing is refused and the judges decide.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { compareLineCounts, } from '../dist/final/node/index.mjs';

/**
 Original quoting a film line in Chinese with its English beside it, the
 attribution likewise, then a closing line.
 */
const BILINGUAL = [
  '> 愿每只猫都能找到属于它的阳光。',
  '>',
  '> May every cat find a sunbeam that’s all its own.',
  '>',
  '> 出自《猫的世界》',
  '>',
  '> From *The Cat Show*',
  '',
  '好了，猫，睡吧。',
].join('\n',);

/**
 Page as the human translator left it: each pair once.
 */
const PAGE = [
  '> May every cat find a sunbeam that’s all its own.',
  '>',
  '> From *The Cat Show*',
  '',
  'All right, cat, sleep now.',
].join('\n',);

/**
 Rendering that wrote the Chinese quote line again in English above the
 film's own line.
 */
const DOUBLED = [
  '> May every cat find its own sunshine.',
  '>',
  '> May every cat find a sunbeam that’s all its own.',
  '>',
  '> From *The Cat Show*',
  '',
  'All right, cat, sleep now.',
].join('\n',);

/**
 Original whose Chinese line introduces an English poem line of different
 content: a pair by adjacency, not by meaning.
 */
const INTRODUCED = [
  '就用一句小诗来收尾吧。',
  'The moon is a saucer of milk for the cat.',
  '',
  '好了，猫，睡吧。',
].join('\n',);

/**
 Original whose pair, a refrain, stands twice with a line between.
 */
const REFRAIN = [
  '晚安，猫。',
  'Good night, cat.',
  '',
  '窗边有一只猫。',
  '',
  '晚安，猫。',
  'Good night, cat.',
].join('\n',);

/**
 Page carrying the refrain's English once in each place.
 */
const REFRAIN_PAGE = [
  'Good night, cat.',
  '',
  'A cat sits at the window.',
  '',
  'Good night, cat.',
].join('\n',);

/**
 Finding for a rendering whose block holding the refrain carries two lines
 where the page's carries one.
 */
const REFRAIN_WIDENED = 'This slice is LINE-STRUCTURED and the ORIGINAL gives the line `Good night, cat.` twice, once in '
  + 'Chinese and once in English directly beside it; that pair is ONE line whose English is already its rendering, '
  + 'and the EXISTING TRANSLATION carries the block holding it as 1 line. Yours carries 2. Drop the second rendering '
  + 'of the pair (the Chinese line, or a second English wording of it), keeping the wording you chose elsewhere.';

await describe({
  name: 'the existing translation bounds a bilingual pair to one line (class eighty)',
  children: [
    it({
      name: 'REFUSES a rendering carrying more lines than the page where the page carries the pair once',
      fn: async () => {
        const found = compareLineCounts({
          lineStructured: true,
          sourceText: BILINGUAL,
          candidateText: DOUBLED,
          pageText: PAGE,
        },);
        expect(found.length,).toBe(1,);
        expect(found[0],).toContain('once in Chinese and once in English',);
        expect(found[0],).toContain('EXISTING TRANSLATION',);
      },
    },),
    it({
      name: 'ACCEPTS the rendering carrying each pair once, as the page does',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: BILINGUAL,
          candidateText: PAGE,
          pageText: PAGE,
        },).length,).toBe(0,);
      },
    },),
    it({
      name: 'STAYS SILENT without page text, where the judges decide on the sheet clause',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: BILINGUAL,
          candidateText: DOUBLED,
        },).length,).toBe(0,);
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: BILINGUAL,
          candidateText: DOUBLED,
          pageText: '',
        },).length,).toBe(0,);
      },
    },),
    it({
      name: 'STAYS SILENT where the page itself keeps the adjacent lines apart (an introduction then a poem line)',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: INTRODUCED,
          candidateText: 'Let a short poem close this.\nThe moon is a saucer of milk for the cat.\n\nAll right, cat, sleep now.',
          pageText: 'To end, a short poem.\nThe moon is a saucer of milk for the cat.\n\nAll right, cat, sleep now.',
        },).length,).toBe(0,);
      },
    },),
    it({
      name: 'REFUSES the doubled quote even where the page splits another line of the slice (one run: the '
        + 'archive writes the closing line as two, so the whole-slice count never matched and the bound slept)',
      fn: async () => {
        const found = compareLineCounts({
          lineStructured: true,
          sourceText: BILINGUAL,
          candidateText: DOUBLED,
          pageText: [
            '> May every cat find a sunbeam that’s all its own.',
            '>',
            '> From *The Cat Show*',
            '',
            'All right, cat.',
            'Sleep now.',
          ].join('\n',),
        },);
        expect(found.length,).toBe(1,);
        expect(found[0],).toContain('once in Chinese and once in English',);
      },
    },),
    it({
      name: 'STAYS SILENT where the rendering reworded both English lines of the quote, since the block holding a pair '
        + 'can no longer be found; reworded on one line only, the other pair still names the block',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: BILINGUAL,
          candidateText: DOUBLED
            .replace('May every cat find a sunbeam', 'Let each cat find a sunbeam',)
            .replace('From *The Cat Show*', 'Taken from *The Cat Show*',),
          pageText: PAGE,
        },).length,).toBe(0,);
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: BILINGUAL,
          candidateText: DOUBLED.replace('May every cat find a sunbeam', 'Let each cat find a sunbeam',),
          pageText: PAGE,
        },).length,).toBe(1,);
      },
    },),
    it({
      name: 'REFUSES a doubled pair in a paragraph, whose block ends at a blank line before it and a quote after it, '
        + 'and ACCEPTS the pair carried once',
      fn: async () => {
        /** Original whose pair is a paragraph between a closing line and a quoted farewell. */
        const sourceText = ['好了。', '', '猫睡在窗台上。', 'The cat sleeps on the sill.', '> 晚安。',].join('\n',);
        /** Page carrying the pair once. */
        const pageText = ['All right.', '', 'The cat sleeps on the sill.', '> Good night.',].join('\n',);
        expect(compareLineCounts({
          lineStructured: true,
          sourceText,
          candidateText: ['All right.', '', 'The cat dozes on the sill.', 'The cat sleeps on the sill.', '> Good night.',]
            .join('\n',),
          pageText,
        },),).toEqual([
          'This slice is LINE-STRUCTURED and the ORIGINAL gives the line `The cat sleeps on the sill.` twice, once in '
            + 'Chinese and once in English directly beside it; that pair is ONE line whose English is already its '
            + 'rendering, and the EXISTING TRANSLATION carries the block holding it as 1 line. Yours carries 2. Drop '
            + 'the second rendering of the pair (the Chinese line, or a second English wording of it), keeping the '
            + 'wording you chose elsewhere.',
        ],);
        expect(compareLineCounts({
          lineStructured: true,
          sourceText,
          candidateText: pageText,
          pageText,
        },),).toEqual([],);
      },
    },),
    it({
      name: 'STAYS SILENT where the page reworded both English lines of the quote, or keeps the Chinese lines too, '
        + 'since then the page does not show the pair carried once',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: BILINGUAL,
          candidateText: DOUBLED,
          pageText: PAGE
            .replace('May every cat find a sunbeam that’s all its own.', 'Let every cat find its very own sunbeam.',)
            .replace('From *The Cat Show*', 'Taken from *The Cat Show*',),
        },),).toEqual([],);
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: BILINGUAL,
          candidateText: DOUBLED,
          pageText: ['愿每只猫都能找到属于它的阳光。', '', '出自《猫的世界》', '', PAGE,].join('\n',),
        },),).toEqual([],);
      },
    },),
    it({
      name: 'STAYS SILENT on a slice with no pair however long the rendering runs, the shortfall check\'s own blind spot kept',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: '猫醒了。\n太阳很暖。',
          candidateText: 'The cat wakes,\nstretching.\nThe sun is warm.',
          pageText: 'The cat wakes.\nThe sun is warm.',
        },).length,).toBe(0,);
      },
    },),
    it({
      name: 'REFUSES a rendering that widens the SECOND block of a refrain the page carries in two places',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: REFRAIN,
          candidateText: [
            'Good night, cat.',
            '',
            'A cat sits at the window.',
            '',
            'Sleep well, cat.',
            'Good night, cat.',
          ].join('\n',),
          pageText: REFRAIN_PAGE,
        },),).toEqual([REFRAIN_WIDENED,],);
      },
    },),
    it({
      name: 'REFUSES a rendering that widens the FIRST block of a refrain the page carries in two places',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: REFRAIN,
          candidateText: [
            'Sleep well, cat.',
            'Good night, cat.',
            '',
            'A cat sits at the window.',
            '',
            'Good night, cat.',
          ].join('\n',),
          pageText: REFRAIN_PAGE,
        },),).toEqual([REFRAIN_WIDENED,],);
      },
    },),
    it({
      name: 'ACCEPTS a refrain carried in both places as the page carries it',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: REFRAIN,
          candidateText: REFRAIN_PAGE,
          pageText: REFRAIN_PAGE,
        },),).toEqual([],);
      },
    },),
    it({
      name: 'STAYS SILENT ON THE PAIR where the rendering carries the wording on more lines than the page does, '
        + 'leaving the repeat to the repeated-line check',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: '晚安，猫。\nGood night, cat.\n\n窗边有一只猫。',
          candidateText: [
            'Sleep well, cat.',
            'Good night, cat.',
            '',
            'A cat sits at the window.',
            'Good night, cat.',
          ].join('\n',),
          pageText: 'Good night, cat.\n\nA cat sits at the window.',
        },),).toEqual([
          'This slice is LINE-STRUCTURED and your rendering repeats the line `Good night, cat.` 2 times where the '
            + 'ORIGINAL repeats no line more than once. A Chinese line and its own English beside it are one line to '
            + 'render, not two. Drop the repeat, keeping the wording you chose.',
        ],);
      },
    },),
    it({
      name: 'REFUSES a widened block through the second of two pairs sharing one English wording, '
        + 'where the page kept the first pair\'s Han line only',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: '晚安，猫。\nGood night, cat.\n\n窗边有一只猫。\n\n今夜很静。\nGood night, cat.',
          candidateText: 'A cat sits at the window.\n\nSleep well, cat.\nGood night, cat.',
          pageText: '晚安，猫。\n\nGood night, cat.\n\nA cat sits at the window.',
        },),).toEqual([REFRAIN_WIDENED,],);
      },
    },),
    it({
      name: 'REFUSES a widened block through the second of two pairs sharing one Han line with two English wordings, '
        + 'where the page kept neither English wording but the first',
      fn: async () => {
        expect(compareLineCounts({
          lineStructured: true,
          sourceText: '晚安，猫。\nGood night, cat.\n\n窗边有一只猫。\n\n晚安，猫。\nSleep well, cat.',
          candidateText: 'A cat sits at the window.\n\nSay good night.\nSleep well, cat.',
          pageText: 'Good night, cat.\n\nA cat sits at the window.\n\nSleep well, cat.',
        },),).toEqual([
          REFRAIN_WIDENED.replace('Good night, cat.', 'Sleep well, cat.',),
        ],);
      },
    },),
  ],
},);
