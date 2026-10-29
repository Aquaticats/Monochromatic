/**
 The section headings a title reference is unified with: which heading
 titles the pass reads, the rendering it takes from each, and a title two
 headings share. Cat-themed invention throughout; no corpus content appears
 here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type SliceReplacement,
  unifyTitleReferences,
} from '../../dist/final/node/index.mjs';
import {
  pair,
  textsOf,
} from './title-reference.test-fixture.ts';

/**
 Original credit bracketing the title the headings carry.
 */
const CREDIT = pair({
  sliceIndex: 1,
  source: '—— 云猫《猫的午后》',
  target: '',
},);

/**
 Page after the pass over one heading and a credit bracketing its title.

 @param title - title the original heads and credits

 @param heading - page text of the heading

 @param credit - page text of the credit

 @returns Rows and findings

 @example
 ```ts
 const unified = passOverHeading({ title: '猫', heading: '### Cat', credit: '—— Yunmao “Kitty”', },);
 ```
 */
function passOverHeading(
  {
    title,
    heading,
    credit,
  }: {
    readonly title: string;
    readonly heading: string;
    readonly credit: string;
  },
): ReturnType<typeof unifyTitleReferences> {
  return unifyTitleReferences({
    slices: [
      pair({
        sliceIndex: 0,
        source: `### ${title}`,
        target: '',
      },),
      pair({
        sliceIndex: 1,
        source: `—— 云猫《${title}》`,
        target: '',
      },),
    ],
    replacements: [
      {
        sliceIndex: 0,
        replacementText: heading,
      },
      {
        sliceIndex: 1,
        replacementText: credit,
      },
    ],
  },);
}

/**
 Page after the pass over two headings titled alike and the credit between
 them.

 @param first - page text of the first heading

 @param second - page text of the second heading

 @param credit - page text of the credit

 @returns Rows and findings

 @example
 ```ts
 const unified = passOverTwoHeadings({ first: '### Cat', second: '### Cat', credit: '—— Yunmao “Kitty”', },);
 ```
 */
function passOverTwoHeadings(
  {
    first,
    second,
    credit,
  }: {
    readonly first: string;
    readonly second: string;
    readonly credit: string;
  },
): ReturnType<typeof unifyTitleReferences> {
  /**
   What the page writes per slice.
   */
  const replacements: readonly SliceReplacement[] = [
    {
      sliceIndex: 0,
      replacementText: first,
    },
    {
      sliceIndex: 1,
      replacementText: credit,
    },
    {
      sliceIndex: 2,
      replacementText: second,
    },
  ];
  return unifyTitleReferences({
    slices: [
      pair({
        sliceIndex: 0,
        source: '### 猫的午后',
        target: '',
      },),
      CREDIT,
      pair({
        sliceIndex: 2,
        source: '### 猫的午后',
        target: '',
      },),
    ],
    replacements,
  },);
}

await describe({
  name: `${unifyTitleReferences.name} over the headings a reference points at`,
  children: [
    it({
      name: 'TAKES A HEADING RENDERED WITH ITS HAN GLOSS AS THE ENGLISH ALONE, so the reference carries no gloss',
      fn: async () => {
        /**
         Pass over a heading that glosses its own title.
         */
        const unified = passOverHeading({
          title: '午后猫语',
          heading: '### Afternoon Cat Murmurs (午后猫语)',
          credit: '—— Yunmao \u{201C}Cat Talk\u{201D}',
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe('—— Yunmao \u{201C}Afternoon Cat Murmurs\u{201D}',);
        expect(unified.findings,).toEqual([
          'title-reference-unified (slice 1: "Cat Talk" to "Afternoon Cat Murmurs"; 「午后猫语」 rendered by the heading of '
            + 'slice 0 as "Afternoon Cat Murmurs")',
        ],);
      },
    },),
    it({
      name: 'READS NO HEADING WHOSE ORIGINAL TITLE CARRIES A LATIN LETTER: the pass reads Han titles only',
      fn: async () => {
        /**
         Pass over a heading titled partly in Latin letters.
         */
        const unified = passOverHeading({
          title: 'Nyan猫',
          heading: '### Nyan Cat',
          credit: '—— Yunmao \u{201C}Nyan Kitty\u{201D}',
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe('—— Yunmao \u{201C}Nyan Kitty\u{201D}',);
        expect(unified.findings,).toEqual([],);
      },
    },),
    it({
      name: 'READS NO HEADING THE PAGE RENDERS AS ITS HAN GLOSS ALONE, which leaves no English to take',
      fn: async () => {
        /**
         Pass over a heading written as a bare gloss.
         */
        const unified = passOverHeading({
          title: '午后猫语',
          heading: '### (午后猫语)',
          credit: '—— Yunmao \u{201C}Cat Talk\u{201D}',
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe('—— Yunmao \u{201C}Cat Talk\u{201D}',);
        expect(unified.findings,).toEqual([],);
      },
    },),
    it({
      name: 'READS NO HEADING THE PAGE LEAVES IN HAN, which leaves no English to take',
      fn: async () => {
        /**
         Pass over a heading the page keeps as the original writes it.
         */
        const unified = passOverHeading({
          title: '午后猫语',
          heading: '### 午后猫语',
          credit: '—— Yunmao \u{201C}Cat Talk\u{201D}',
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe('—— Yunmao \u{201C}Cat Talk\u{201D}',);
        expect(unified.findings,).toEqual([],);
      },
    },),
    it({
      name: 'READS NO HEADING THAT SHOWS A READER NOTHING, which leaves no English to take: under a heading of a '
        + 'zero-width space, a Hangul filler, both among spaces, either before the title\'s gloss, or a filler in an '
        + 'HTML heading, the credit keeps the words its writer gave it and no row is rewritten',
      fn: async () => {
        /**
         Credit as its writer left it.
         */
        const credit = '—— Yunmao \u{201C}Cat Talk\u{201D}';
        /**
         Headings whose rendering shows a reader nothing, in both shapes the
         pass reads a heading from.
         */
        const headings = [
          '### \u{200B}',
          '### \u{3164}',
          '###  \u{200B}\u{3164} ',
          '### \u{200B} (午后猫语)',
          '<h3>\u{3164}</h3>',
        ];
        expect(headings.map(function passOver(heading,): ReturnType<typeof unifyTitleReferences> {
          return passOverHeading({
            title: '午后猫语',
            heading,
            credit,
          },);
        },),).toEqual(headings.map(function untouched(heading,): ReturnType<typeof unifyTitleReferences> {
          return {
            replacements: [
              {
                sliceIndex: 0,
                replacementText: heading,
              },
              {
                sliceIndex: 1,
                replacementText: credit,
              },
            ],
            restored: [],
            findings: [],
          };
        },),);
      },
    },),
    it({
      name: 'LEAVES A REFERENCE TO A TITLE TWO HEADINGS RENDER APART, since which rendering it takes cannot be read',
      fn: async () => {
        /**
         Pass over two headings rendering one title two ways.
         */
        const unified = passOverTwoHeadings({
          first: '### The Cat\u{2019}s Afternoon',
          second: '### An Afternoon with the Cat',
          credit: '—— Yunmao \u{201C}Cat Afternoon\u{201D}',
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe('—— Yunmao \u{201C}Cat Afternoon\u{201D}',);
        expect(unified.findings,).toEqual([],);
      },
    },),
    it({
      name: 'UNIFIES A REFERENCE TO A TITLE TWO HEADINGS RENDER ALIKE (quality call, T8 batch 10), apostrophe style '
        + 'aside, and names both headings',
      fn: async () => {
        /**
         Pass over two headings rendering one title alike but for the apostrophe.
         */
        const unified = passOverTwoHeadings({
          first: '### The Cat\u{2019}s Afternoon',
          second: '### The Cat\'s Afternoon',
          credit: '—— Yunmao \u{201C}Cat Afternoon\u{201D}',
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe('—— Yunmao \u{201C}The Cat\u{2019}s Afternoon\u{201D}',);
        expect(unified.findings,).toEqual([
          'title-reference-unified (slice 1: "Cat Afternoon" to "The Cat\u{2019}s Afternoon"; 「猫的午后」 rendered by the '
            + 'headings of slices 0, 2 as "The Cat\u{2019}s Afternoon")',
        ],);
      },
    },),
  ],
},);
