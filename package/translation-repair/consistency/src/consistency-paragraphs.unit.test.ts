/**
 Tests for where the paragraphs of an English are: the offsets a paragraph or the text of a setext heading holds, a line break
 inside one and a line break between it and the block that follows, how many quotations enclose a line break, and an English
 the grammar refuses, which has no paragraphs to tell.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  insideParagraphAt,
  quoteDepthAt,
} from '../dist/final/node/index.mjs';

/**
 Whether the first line break of an English lies inside a paragraph.

 @param english - English

 @returns Whether the paragraph holds the break

 @example
 ```ts
 const inside = breakInside({ english: 'Fill the\nbowl.', },); // true
 ```
 */
function breakInside({ english, }: { readonly english: string; },): boolean {
  return insideParagraphAt({ english, index: english.indexOf('\n',), },);
}

/**
 How many quotations enclose the first line break of an English.

 @param english - English

 @returns Number of quotations around the block that holds the break

 @example
 ```ts
 const depth = depthOfBreak({ english: '> Fill the\n> bowl.', },); // 1
 ```
 */
function depthOfBreak({ english, }: { readonly english: string; },): number {
  return quoteDepthAt({ english, index: english.indexOf('\n',), },);
}

/**
 The suite of the question whether a break lies inside a paragraph or a setext heading.
 */
const insideSuite = describe({
  name: insideParagraphAt.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'HOLDS A SOFT BREAK, A HARD BREAK AND A BREAK INSIDE A LIST ITEM OR A QUOTATION IN THE PARAGRAPH THEY STAND IN',
      fn: async () => {
        expect([
          breakInside({ english: 'Fill the\nbowl.', },),
          breakInside({ english: 'Fill the  \nbowl.', },),
          breakInside({ english: 'Fill the\\\nbowl.', },),
          breakInside({ english: '- Fill the\n  bowl.', },),
          breakInside({ english: '> Fill the\n> bowl.', },),
        ],).toEqual([true, true, true, true, true,],);
      },
    },),
    it({
      name: 'HOLDS A BREAK INSIDE THE TEXT OF A SETEXT HEADING, which spans lines as a paragraph does, and not the break before '
        + 'its underline or a break inside a heading of one line',
      fn: async () => {
        expect([
          breakInside({ english: 'Fill the\nbowl.\n===', },),
          breakInside({ english: 'Fill the\nbowl.\n---', },),
          breakInside({ english: 'Fill the  \nbowl.\n===', },),
          breakInside({ english: '> Fill the\n> bowl.\n> ===', },),
          insideParagraphAt({ english: 'Fill the\nbowl.\n===', index: 14, },),
          breakInside({ english: '# Fill the bowl.\nThen nap.', },),
          breakInside({ english: 'Fill the bowl.\n# Then nap', },),
        ],).toEqual([true, true, true, true, false, false, false,],);
      },
    },),
    it({
      name: 'DOES NOT HOLD A BREAK BETWEEN A PARAGRAPH AND THE BLOCK AFTER IT, a list item, a heading, a quotation, a table or '
        + 'another paragraph, and none before the first paragraph',
      fn: async () => {
        expect([
          breakInside({ english: 'Fill the bowl.\n- Then nap.', },),
          breakInside({ english: 'Fill the bowl.\n# Then nap', },),
          breakInside({ english: 'Fill the bowl.\n> Then nap.', },),
          breakInside({ english: 'Fill the bowl.\n\nThen nap.', },),
          breakInside({ english: '| Fill | bowl |\n| --- | --- |\n| cat | nap |', },),
          breakInside({ english: '\nFill the bowl.', },),
        ],).toEqual([false, false, false, false, false, false,],);
      },
    },),
    it({
      name: 'TAKES EVERY OFFSET OF AN ENGLISH THE GRAMMAR REFUSES FOR INSIDE A PARAGRAPH, a stray brace being what it refuses',
      fn: async () => {
        expect([
          breakInside({ english: 'Fill the {\nbowl.', },),
          breakInside({ english: 'Fill the bowl {\n\nThen nap.', },),
        ],).toEqual([true, true,],);
      },
    },),
  ],
},);

/**
 The suite of the question how many quotations enclose a break.
 */
const depthSuite = describe({
  name: quoteDepthAt.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'COUNTS THE QUOTATIONS AROUND THE PARAGRAPH OR THE SETEXT HEADING THAT HOLDS THE BREAK, none for a block outside '
        + 'every quotation, one for each nesting in a list item as in a quotation',
      fn: async () => {
        expect([
          depthOfBreak({ english: 'Fill the\nbowl.', },),
          depthOfBreak({ english: '- Fill the\n  bowl.', },),
          depthOfBreak({ english: '> Fill the\n> bowl.', },),
          depthOfBreak({ english: '> > Fill the\n> > bowl.', },),
          depthOfBreak({ english: '>> Fill the\n>> bowl.', },),
          depthOfBreak({ english: '> - Fill the\n>   bowl.', },),
          depthOfBreak({ english: '- > Fill the\n  > bowl.', },),
          depthOfBreak({ english: '> Fill the\n> bowl.\n> ===', },),
          depthOfBreak({ english: '> > - > Fill the\n> >   > bowl.', },),
        ],).toEqual([0, 0, 1, 2, 2, 1, 1, 1, 3,],);
      },
    },),
    it({
      name: 'COUNTS NO QUOTATION FOR A BREAK THAT NO PARAGRAPH HOLDS, and takes an English the grammar refuses for as deep as '
        + 'a quotation can be, so that nothing of its line starts is read as text',
      fn: async () => {
        expect([
          depthOfBreak({ english: '> Fill the bowl.\n\n> Then nap.', },),
          depthOfBreak({ english: 'Fill the bowl.\n> Then nap.', },),
          depthOfBreak({ english: '> Fill the {\n> bowl.', },),
          depthOfBreak({ english: 'Fill the {\nbowl.', },),
        ],).toEqual([0, 0, Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY,],);
      },
    },),
  ],
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    insideSuite,
    depthSuite,
  ],
},);
