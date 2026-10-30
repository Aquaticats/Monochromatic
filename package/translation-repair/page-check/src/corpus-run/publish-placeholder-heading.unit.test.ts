/**
 Tests for the report of a page heading made of placeholder marks only where the original heading has words (owner,
 2026-10-29, "Report it as a defect (Recommended)", ledger B1052): the page ships, no wording changes, and the run never
 stops.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { placeholderHeadingDefects, } from '../../dist/final/node/index.mjs';

/**
 A heading holding one ideographic space, written by its code point so that no invisible character sits in this file.
 */
const IDEOGRAPHIC_SPACE_HEADING = `### ${String.fromCodePoint(0x30_00,)}`;

await describe({
  name: '',
  children: [
    describe({
      name: placeholderHeadingDefects.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A PAGE HEADING OF PLACEHOLDER MARKS ONLY WHERE THE ORIGINAL HEADING HAS WORDS: underscores, '
            + 'dashes, blanks, an empty heading, an ellipsis and the number of a series with a placeholder after '
            + 'it (XingZ6022: `### Part Nine: __` over `### 其九：空白`), saying the page heading and nothing it quotes '
            + '(owner, 2026-10-29, "Report it as a defect")',
          fn: async () => {
            for (const heading of [
              '### __',
              '### ——',
              '### ',
              '###',
              '### - - -',
              '### …',
              '### ＿＿',
              IDEOGRAPHIC_SPACE_HEADING,
              '### Part Nine: __',
              '### Part Nine: ——',
            ]) {
              expect(placeholderHeadingDefects({
                entryId: 'WhiskerCat',
                sourceText: '### 其九：空白\n\n正文。\n',
                pageText: `${heading}\n\nBody.\n`,
              },),).toStrictEqual([{
                check: 'placeholder-heading',
                message: 'entry WhiskerCat ships a heading of placeholder marks only where the original heading '
                  + 'has words, at page heading 1',
              },],);
            }
          },
        },),
        it({
          name: 'SAYS NOTHING for a heading with words, a number with a title after it, a number alone, and a '
            + 'heading the original also leaves without words',
          fn: async () => {
            for (const heading of [
              '### Part Nine: Blank',
              '### Part Nine:',
              '### Blank',
              '### 9',
              '### 其九：空白',
            ]) {
              expect(placeholderHeadingDefects({
                entryId: 'WhiskerCat',
                sourceText: '### 其九：空白\n\n正文。\n',
                pageText: `${heading}\n\nBody.\n`,
              },),).toStrictEqual([],);
            }
            for (const original of [
              '### __',
              '### ——',
              '### ',
              '###',
              '### Part Nine: __',
            ]) {
              expect(placeholderHeadingDefects({
                entryId: 'WhiskerCat',
                sourceText: `${original}\n\n正文。\n`,
                pageText: '### __\n\nBody.\n',
              },),).toStrictEqual([],);
            }
          },
        },),
        it({
          name: 'PAIRS HEADINGS BY POSITION where the original and the page hold as many, and names every page '
            + 'heading that has marks only where its original has words, not one the original also leaves blank',
          fn: async () => {
            expect(placeholderHeadingDefects({
              entryId: 'WhiskerCat',
              sourceText: '### 其一：猫\n\n正文。\n\n### 其二：狗\n\n正文。\n\n### 其三：——\n\n正文。\n\n### 其四：鸟\n\n正文。\n',
              pageText: '### Part One: Cat\n\nBody.\n\n### Part Two: __\n\nBody.\n\n### Part Three: __\n\nBody.\n\n'
                + '### Part Four: ——\n\nBody.\n',
            },),).toStrictEqual([{
              check: 'placeholder-heading',
              message: 'entry WhiskerCat ships 2 headings of placeholder marks only where the original headings '
                + 'have words, at page heading 2, page heading 4',
            },],);
          },
        },),
        it({
          name: 'COMPARES THE COUNTS where the original and the page hold different numbers of headings, since '
            + 'nothing is paired then: more placeholder headings on the page than in the original is a defect, '
            + 'as many or fewer is not',
          fn: async () => {
            expect(placeholderHeadingDefects({
              entryId: 'WhiskerCat',
              sourceText: '## 简介\n\n正文。\n\n## 生平\n\n正文。\n',
              pageText: '## Description\n\nBody.\n\n## __\n\nBody.\n\n## Life\n\nBody.\n',
            },),).toStrictEqual([{
              check: 'placeholder-heading',
              message: 'entry WhiskerCat ships 1 heading of placeholder marks only against 0 in the original, whose '
                + '2 headings are not paired with the page\'s 3, at page heading 2',
            },],);
            expect(placeholderHeadingDefects({
              entryId: 'WhiskerCat',
              sourceText: '## __\n\n正文。\n\n## 生平\n\n正文。\n',
              pageText: '## Description\n\nBody.\n\n## __\n\nBody.\n\n## Life\n\nBody.\n',
            },),).toStrictEqual([],);
            expect(placeholderHeadingDefects({
              entryId: 'WhiskerCat',
              sourceText: '## 简介\n\n正文。\n',
              pageText: '## Description\n\nBody.\n\n## __\n\nBody.\n\n## __\n\nBody.\n',
            },),).toStrictEqual([{
              check: 'placeholder-heading',
              message: 'entry WhiskerCat ships 2 headings of placeholder marks only against 0 in the original, whose '
                + '1 heading is not paired with the page\'s 3, at page heading 2, page heading 3',
            },],);
          },
        },),
        it({
          name: 'SAYS NOTHING for a page and an original with no heading, and reads front matter as no heading',
          fn: async () => {
            expect(placeholderHeadingDefects({
              entryId: 'WhiskerCat',
              sourceText: '一只虎斑猫。\n',
              pageText: 'A tabby.\n',
            },),).toStrictEqual([],);
            expect(placeholderHeadingDefects({
              entryId: 'WhiskerCat',
              sourceText: '---\nname: 猫\n---\n\n## 简介\n\n正文。\n',
              pageText: '---\nname: Cat\n---\n\n## Description\n\nBody.\n',
            },),).toStrictEqual([],);
          },
        },),
      ],
    },),
  ],
},);
