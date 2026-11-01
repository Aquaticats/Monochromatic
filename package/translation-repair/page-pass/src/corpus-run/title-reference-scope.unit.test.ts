/**
 The part of a slice's page text a bracket or quote search for a title
 reads: the page's line rendering the original line that brackets the
 title, where that line opens with a footnote label or a tag the page keeps,
 the whole text otherwise. Cat-themed invention throughout; no corpus
 content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { unifyTitleReferences, } from '../../dist/final/node/index.mjs';
import {
  pair,
  textsOf,
} from './title-reference.test-fixture.ts';

/**
 Finding for a slice offering two quoted spans, neither the heading's.
 */
const AMBIGUOUS = 'title-reference-ambiguous (slice 1: 「午后猫语」 rendered by the heading of slice 0 as '
  + '"Afternoon Cat Murmurs", but the slice offers more than one span to read)';

/**
 Page after the pass over a heading and one slice referencing its title.

 @param source - original text of the referencing slice

 @param page - page text of the referencing slice

 @returns Rows and findings

 @example
 ```ts
 const unified = passOverSlice({ source: '见「午后猫语」篇', page: 'See “Cat Talk”', },);
 ```
 */
function passOverSlice(
  {
    source,
    page,
  }: {
    readonly source: string;
    readonly page: string;
  },
): ReturnType<typeof unifyTitleReferences> {
  return unifyTitleReferences({
    slices: [
      pair({
        sliceIndex: 0,
        source: '### 午后猫语',
        target: '',
      },),
      pair({
        sliceIndex: 1,
        source,
        target: '',
      },),
    ],
    replacements: [
      {
        sliceIndex: 0,
        replacementText: '### Afternoon Cat Murmurs',
      },
      {
        sliceIndex: 1,
        replacementText: page,
      },
    ],
  },);
}

await describe({
  name: `${unifyTitleReferences.name} over the span a search reads`,
  children: [
    it({
      name: 'READS THE WHOLE TEXT WHERE THE LINE OPENS WITH A FOOTNOTE REFERENCE, which is no definition',
      fn: async () => {
        /**
         Pass over a line opening with a reference to a note.
         */
        const unified = passOverSlice({
          source: '[^1]见「午后猫语」篇。',
          page: '[^1] See the section \u{201C}Afternoon Cat Talk\u{201D}.',
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe('[^1] See the section \u{201C}Afternoon Cat Murmurs\u{201D}.',);
      },
    },),
    it({
      name: 'READS THE WHOLE TEXT WHERE THE PAGE LOST THE DEFINITION\'S LABEL, and stands aside over two quoted titles',
      fn: async () => {
        /**
         Page whose second definition carries another label.
         */
        const page = '[^5]: From \u{201C}The Cat Sutra\u{201D}.\n\n[^7]: See the section \u{201C}Afternoon Cat Talk\u{201D}.';
        /**
         Pass over that page.
         */
        const unified = passOverSlice({
          source: '[^5]: 出自《猫经》。\n\n[^6]: 另见「午后猫语」篇末尾。',
          page,
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe(page,);
        expect(unified.findings,).toEqual([AMBIGUOUS,],);
      },
    },),
    it({
      name: 'READS THE WHOLE TEXT WHERE THE LINE OPENS WITH A LESS-THAN SIGN THAT OPENS NO TAG',
      fn: async () => {
        /**
         Pass over a line opening with an arrow.
         */
        const unified = passOverSlice({
          source: '<- 另见「午后猫语」篇',
          page: '<- See the section \u{201C}Afternoon Cat Talk\u{201D}',
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe('<- See the section \u{201C}Afternoon Cat Murmurs\u{201D}',);
      },
    },),
    it({
      name: 'READS THE WHOLE TEXT WHERE THE PAGE DROPS THE CREDIT\'S TAG, and stands aside over two quoted spans',
      fn: async () => {
        /**
         Page writing the credit with no tag of its own.
         */
        const page = '<summary>\u{201C}The cat said goodnight\u{201D}</summary>\n\n'
          + '—— Yunmao 【Dream】 \u{201C}Wu Hou Mao Yu\u{201D}';
        /**
         Pass over that page.
         */
        const unified = passOverSlice({
          source: '<summary>「猫说晚安」</summary>\n\n<p style="text-align: end;">—— 云猫【梦】《午后猫语》</p>',
          page,
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe(page,);
        expect(unified.findings,).toEqual([AMBIGUOUS,],);
      },
    },),
    it({
      name: 'ENDS THE SCOPE AT THE DEFINITION LINE\'S END where more definitions follow it',
      fn: async () => {
        /**
         Pass over a definitions tail whose first definition references the title.
         */
        const unified = passOverSlice({
          source: '[^6]: 另见「午后猫语」篇末尾。\n\n[^7]: 出自《猫经》。',
          page: '[^6]: See the section \u{201C}Afternoon Cat Talk\u{201D}.\n\n[^7]: From \u{201C}The Cat Sutra\u{201D}.',
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe(
          '[^6]: See the section \u{201C}Afternoon Cat Murmurs\u{201D}.\n\n[^7]: From \u{201C}The Cat Sutra\u{201D}.',
        );
      },
    },),
  ],
},);
