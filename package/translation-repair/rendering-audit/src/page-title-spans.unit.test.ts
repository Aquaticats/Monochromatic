/**
 Tests for the titles a page repeats that the archive leaves unpaired
 (ledger H16): which marked spans count as titles, which repeat, and which
 the page-name glossary already carries.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { repeatedTitleSpans, } from '../dist/final/node/index.mjs';

/**
 A title of twenty-five Han characters, one past the longest read as a title.
 */
const LONG_TITLE = '猫'.repeat(25,);

/**
 Titles repeated on an original, against an archive that renders no heading.

 @param sourceText - original page

 @returns Each repeated title with its count

 @example
 ```ts
 const spans = spansOf({ sourceText: '《猫》《猫》', },);
 ```
 */
function spansOf({ sourceText, }: { readonly sourceText: string; },): readonly unknown[] {
  return repeatedTitleSpans({
    sourceText,
    targetText: 'The kitten sang all afternoon.',
  },);
}

await describe({
  name: repeatedTitleSpans.name,
  children: [
    it({
      name: 'COUNTS A TITLE IN EVERY MARKED PLACE: a heading, 《》 and 「」 repeating the heading',
      fn: async () => {
        expect(spansOf({
          sourceText: '## 猫之歌\n\n小猫唱了《猫之歌》。\n\n她说：「猫之歌」是她最爱的歌。\n',
        },),).toEqual([{ source: '猫之歌', occurrences: 3, },],);
      },
    },),
    it({
      name: 'READS AN HTML HEADING, and leaves one that holds a nested tag',
      fn: async () => {
        expect({
          plain: spansOf({ sourceText: '<h3 align="center">猫之歌</h3>\n\n《猫之歌》很好听。\n', },),
          // Twice, so reading the nested heading whole would list it.
          nested: spansOf({ sourceText: '<h3><b>猫之歌</b></h3>\n\n<h3><b>猫之歌</b></h3>\n', },),
        },).toEqual({
          plain: [{ source: '猫之歌', occurrences: 2, },],
          nested: [],
        },);
      },
    },),
    it({
      name: 'READS 【】 AS A TITLE MARK',
      fn: async () => {
        expect(spansOf({ sourceText: '【猫之歌】\n\n再唱一遍【猫之歌】。\n', },),)
          .toEqual([{ source: '猫之歌', occurrences: 2, },],);
      },
    },),
    it({
      name: 'READS AN OPENING MARK THAT NEVER CLOSED AS NO TITLE (ledger B38), counting only the title after it, '
        + 'though the same stray mark stands before it twice',
      fn: async () => {
        expect(spansOf({ sourceText: '小猫唱了《喵《猫之歌》。\n\n再唱一遍《喵《猫之歌》。\n', },),)
          .toEqual([{ source: '猫之歌', occurrences: 2, },],);
      },
    },),
    it({
      name: 'LEAVES A TITLE WRITTEN ONCE, since nothing can render it two ways',
      fn: async () => {
        expect(spansOf({ sourceText: '小猫唱了《猫之歌》。\n', },),).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES 「」 THAT REPEATS NO TITLE, since corner brackets mostly mark quotations',
      fn: async () => {
        expect(spansOf({ sourceText: '她说：「喵呜」。她又说：「喵呜」。\n', },),).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES A TITLE WITHOUT HAN, and one longer than a title',
      fn: async () => {
        expect({
          latin: spansOf({ sourceText: '《Cat Song》与《Cat Song》\n', },),
          long: spansOf({ sourceText: `《${LONG_TITLE}》与《${LONG_TITLE}》\n`, },),
        },).toEqual({
          latin: [],
          long: [],
        },);
      },
    },),
    it({
      name: 'READS A LINKED TITLE BY ITS TEXT',
      fn: async () => {
        expect(spansOf({ sourceText: '《[猫之歌](https://example.invalid/song)》与《猫之歌》\n', },),)
          .toEqual([{ source: '猫之歌', occurrences: 2, },],);
      },
    },),
    it({
      name: 'COUNTS ONLY WHAT THE PAGE SHOWS: a title inside a comment is not a place it stands',
      fn: async () => {
        expect(spansOf({ sourceText: '<!-- 《猫之歌》 -->\n\n小猫唱了《猫之歌》。\n', },),).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES MARKERS WITH NO SPACE AFTER THEM, which CommonMark renders as text',
      fn: async () => {
        expect(spansOf({ sourceText: '##猫之歌\n\n小猫唱了《猫之歌》。\n', },),).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES AN <h7> TAG, which HTML has no heading level for',
      fn: async () => {
        expect(spansOf({ sourceText: '<h7>猫之歌</h7>\n\n<h7>猫之歌</h7>\n', },),).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES A LINE OF SEVEN MARKERS, which is no heading',
      fn: async () => {
        expect(spansOf({ sourceText: '####### 猫之歌\n\n小猫唱了《猫之歌》。\n', },),).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES A TITLE THE ARCHIVE PAIRS, which the page-name glossary already carries',
      fn: async () => {
        expect(repeatedTitleSpans({
          sourceText: '## 猫之歌\n\n小猫唱了《猫之歌》。\n',
          targetText: '## Song of the Cat\n\nThe kitten sang it.\n',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'LISTS TITLES IN ORDER OF FIRST APPEARANCE',
      fn: async () => {
        expect(spansOf({ sourceText: '《鱼之梦》《猫之歌》《猫之歌》《鱼之梦》\n', },),).toEqual([
          { source: '鱼之梦', occurrences: 2, },
          { source: '猫之歌', occurrences: 2, },
        ],);
      },
    },),
    it({
      name: 'LISTS BY PLACE ACROSS MARKERS: a 《》 title before a later heading comes first',
      fn: async () => {
        expect(spansOf({ sourceText: '小猫读了《鱼之梦》。\n\n## 猫之歌\n\n《猫之歌》之后又是《鱼之梦》。\n', },),).toEqual([
          { source: '鱼之梦', occurrences: 2, },
          { source: '猫之歌', occurrences: 2, },
        ],);
      },
    },),
  ],
},);
