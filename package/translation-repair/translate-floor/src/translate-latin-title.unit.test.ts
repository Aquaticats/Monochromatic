/**
 Guards class one hundred forty-one (2026-09-26): one page's second song
 credit shipped an English title inside the Chinese title marks. 《》 carry no meaning in English prose, where
 a song's title stands in quotation marks and a longer work's in italics, so
 a candidate that brackets a
 title with no Han in 《》 is refused before any judge reads it, unless the
 page it would replace writes the same bracketed title.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type SliceValidation,
  validateTranslatedSlice,
} from '../dist/final/node/index.mjs';

/**
 Original crediting a song by its Han title.
 */
const CREDIT = '<p style="text-align: end;">—— 猫猫《喵喵歌》</p>';

/**
 Original naming a poem by its own English title in title marks.
 */
const ENGLISH_TITLE = '她最爱的诗是《Cats Are Liquid》。';

/**
 Verdict on a candidate no floor refuses, read against a page.
 */
const VALID_ON_PAGE: SliceValidation = {
  kind: 'valid',
  pageGrammar: 'strict',
};

/**
 Verdict on a candidate no floor refuses, with no page behind it.
 */
const VALID_WITHOUT_PAGE: SliceValidation = {
  kind: 'valid',
  pageGrammar: 'absent',
};

/**
 Verdict refusing an English title left in the Chinese title marks.

 @param title - title between the marks

 @returns Whole verdict, so a check reads every word of the finding

 @example
 ```ts
 bracketRefusal({ title: 'Meow Song', },);
 ```
 */
function bracketRefusal({ title, }: { readonly title: string; },): SliceValidation {
  return {
    kind: 'invalid',
    findings: [
      `Your translation sets the English title 《${title}》 in the Chinese title marks 《》, which mean nothing in `
        + 'English prose: an English page sets a work\'s title in quotation marks, so write '
        + `“${title}” (keeping any link on the title).`,
    ],
  };
}

await describe({
  name: 'an English title left in 《》 (class one hundred forty-one)',
  children: [
    it({
      name: 'REFUSES a translated title bracketed in 《》',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: CREDIT,
          candidateText: '<p style="text-align: end;">— Maomao 《Meow Song》</p>',
        },),).toEqual(bracketRefusal({ title: 'Meow Song', },),);
      },
    },),
    it({
      name: 'ACCEPTS the translated title in quotation marks',
      fn: async () => {
        // THE SIGNER CARRIES ITS MEANING, which the signer floor asks of a
        // handle the original writes in Han (ledger A17), so the title is the
        // only thing this case weighs.
        expect(validateTranslatedSlice({
          sourceText: CREDIT,
          candidateText: '<p style="text-align: end;">— Maomao (Kitty Kitty), “Meow Song”</p>',
        },),).toEqual(VALID_WITHOUT_PAGE,);
      },
    },),
    it({
      name: 'REFUSES the original\'s own English title kept in 《》',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: ENGLISH_TITLE,
          candidateText: 'Her favourite poem was 《Cats Are Liquid》.',
        },),).toEqual(bracketRefusal({ title: 'Cats Are Liquid', },),);
      },
    },),
    it({
      name: 'ACCEPTS a bracketed English title the page itself writes',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: ENGLISH_TITLE,
          candidateText: 'Her favourite poem was 《Cats Are Liquid》.',
          pageText: 'Her favourite poem is 《Cats Are Liquid》.',
        },),).toEqual(VALID_ON_PAGE,);
      },
    },),
    it({
      name: 'ACCEPTS a bracketed English title the page writes with the other apostrophe (ledger B24): the '
        + 'typography restoration makes the two one title',
      fn: async () => {
        /**
         Original naming a poem whose English title carries an apostrophe.
         */
        const sourceText = '她最爱的诗是《Don\'t Wake the Cat》。';
        expect(validateTranslatedSlice({
          sourceText,
          candidateText: 'Her favourite poem was 《Don\'t Wake the Cat》.',
          pageText: 'Her favourite poem is 《Don’t Wake the Cat》.',
        },),).toEqual(VALID_ON_PAGE,);
        expect(validateTranslatedSlice({
          sourceText,
          candidateText: 'Her favourite poem was 《Don’t Wake the Cat》.',
          pageText: 'Her favourite poem is 《Don\'t Wake the Cat》.',
        },),).toEqual(VALID_ON_PAGE,);
      },
    },),
    it({
      name: 'READS AN OPENING MARK THAT NEVER CLOSED AS NO TITLE (ledger B38): a stray 《 before a title leaves the '
        + 'title quoted alone, and a 《 with no 》 after it ends the reading',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '她的诗：《Purr，还有《Cats Are Liquid》；下一首《Meow',
          candidateText: 'Her poems: 《Purr, and 《Cats Are Liquid》; next, 《Meow',
        },),).toEqual(bracketRefusal({ title: 'Cats Are Liquid', },),);
      },
    },),
    it({
      name: 'ACCEPTS a bracketed title that keeps Han beside its Latin letters when its English stands beside it '
        + '(bare, the Han residue floor refuses it: ledger F-3)',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '她最爱的游戏是《喵萌DX》。',
          candidateText: 'Her favourite game was 《喵萌DX》 (Meowmeow DX).',
        },),).toEqual(VALID_WITHOUT_PAGE,);
      },
    },),
  ],
},);
