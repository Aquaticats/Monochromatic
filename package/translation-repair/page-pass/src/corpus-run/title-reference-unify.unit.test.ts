/**
 Guards class one hundred (2026-09-23): a section title the original writes
 once as a heading and again in brackets (a linked credit, a footnote's
 「title」篇, a song credit's 《title》) reached one page in two English
 renderings each, because each slice is judged alone. The
 page decides once, where every heading is in view: a reference to a
 heading takes the heading's rendering. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { unifyTitleReferences, } from '../../dist/final/node/index.mjs';
import {
  pair,
  textsOf,
} from './title-reference.test-fixture.ts';

/**
 Two headings the archive never rendered, referenced by a link, a
 footnote, a bracketed credit and a glossed credit.
 */
const SLICES: readonly ChunkPair[] = [
  pair({
    sliceIndex: 0,
    source: '<h3 align = "center">窗边猫</h3>',
    target: '',
  },),
  pair({
    sliceIndex: 1,
    source: '——出自《[窗边猫](https://example.test/cat)》',
    target: '',
  },),
  pair({
    sliceIndex: 2,
    source: '### 午后猫语',
    target: '',
  },),
  pair({
    sliceIndex: 3,
    source: '[^6]: 另见「午后猫语」篇末尾。',
    target: '',
  },),
  pair({
    sliceIndex: 4,
    source: '—— 云猫《午后猫语》',
    target: '',
  },),
  pair({
    sliceIndex: 5,
    source: '—— 云猫【梦】《午后猫语》',
    target: '',
  },),
  pair({
    sliceIndex: 6,
    source: '[^5]: 出自《猫经》。\n\n[^6]: 另见「午后猫语」篇末尾。',
    target: '',
  },),
];

/**
 Renderings of the two headings.
 */
const HEADINGS = [
  {
    sliceIndex: 0,
    replacementText: '<h3 align = "center">Cat by the Window</h3>',
  },
  {
    sliceIndex: 2,
    replacementText: '### Afternoon Cat Murmurs',
  },
] as const;

await describe({
  name: 'a reference to a section heading (class one hundred)',
  children: [
    it({
      name: 'UNIFIES a linked, a quoted, a bracketed, a glossed and a footnote-tail reference to the heading\'s rendering',
      fn: async () => {
        /**
         Pass over a page whose references each rendered the title afresh.
         */
        const unified = unifyTitleReferences({
          slices: SLICES,
          replacements: [
            ...HEADINGS,
            {
              sliceIndex: 1,
              replacementText: '——From [The Window Cat](https://example.test/cat)',
            },
            {
              sliceIndex: 3,
              replacementText: '[^6]: See the end of the section “Afternoon Cat Talk”.',
            },
            {
              sliceIndex: 4,
              replacementText: '—— Yunmao 《Cat Talk》',
            },
            {
              sliceIndex: 5,
              replacementText: '—— Yunmao, from “Dream,” Afternoon Cat Talk (午后猫语)',
            },
            {
              sliceIndex: 6,
              replacementText: '[^5]: From “The Cat Sutra”.\n\n[^6]: See the section “Afternoon Cat Talk”.',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          '<h3 align = "center">Cat by the Window</h3>',
          '### Afternoon Cat Murmurs',
          '——From [Cat by the Window](https://example.test/cat)',
          '[^6]: See the end of the section “Afternoon Cat Murmurs”.',
          '—— Yunmao 《Afternoon Cat Murmurs》',
          '—— Yunmao, from “Dream,” Afternoon Cat Murmurs (午后猫语)',
          '[^5]: From “The Cat Sutra”.\n\n[^6]: See the section “Afternoon Cat Murmurs”.',
        ],);
        expect(unified.restored.length,).toBe(5,);
        expect(unified.findings.join('\n',),).toContain(
          'title-reference-unified (slice 3: "Afternoon Cat Talk" to "Afternoon Cat Murmurs"',
        );
      },
    },),
    it({
      name: 'READS AN OPENING MARK ON THE PAGE THAT NEVER CLOSED AS NO TITLE (ledger B38), rewriting the bracketed '
        + 'title after it and keeping the words before it',
      fn: async () => {
        /**
         Pass over a credit whose page carries a stray opening mark.
         */
        const unified = unifyTitleReferences({
          slices: SLICES,
          replacements: [
            ...HEADINGS,
            {
              sliceIndex: 4,
              replacementText: '—— Yunmao 《Purr 《Cat Talk》',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          '<h3 align = "center">Cat by the Window</h3>',
          '### Afternoon Cat Murmurs',
          '—— Yunmao 《Purr 《Afternoon Cat Murmurs》',
        ],);
      },
    },),
    it({
      name: 'SCOPES THE SEARCH TO THE LINE THAT BRACKETS THE TITLE (ledger B23), not an earlier footnote naming it bare',
      fn: async () => {
        /**
         Pass over a slice whose first footnote names the title bare and whose
         second references it in brackets.
         */
        const unified = unifyTitleReferences({
          slices: [
            pair({
              sliceIndex: 2,
              source: '### 午后猫语',
              target: '',
            },),
            pair({
              sliceIndex: 7,
              source: '[^5]: 午后猫语是一首歌。\n\n[^6]: 另见「午后猫语」篇末尾。',
              target: '',
            },),
          ],
          replacements: [
            HEADINGS[1],
            {
              sliceIndex: 7,
              replacementText: '[^5]: Afternoon Cat Talk is a song.\n\n[^6]: See the end of the section “Afternoon Cat Talk”.',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          '### Afternoon Cat Murmurs',
          '[^5]: Afternoon Cat Talk is a song.\n\n[^6]: See the end of the section “Afternoon Cat Murmurs”.',
        ],);
      },
    },),
    it({
      name: 'REWRITES A QUOTED RENDERING THAT ONLY ENDS WITH THE HEADING\'S (ledger B23): a longer title is another '
        + 'rendering, not the same one',
      fn: async () => {
        /**
         Pass over a footnote quoting a longer rendering of the heading.
         */
        const unified = unifyTitleReferences({
          slices: [
            pair({
              sliceIndex: 2,
              source: '### 午后猫语',
              target: '',
            },),
            pair({
              sliceIndex: 3,
              source: '[^6]: 另见「午后猫语」篇末尾。',
              target: '',
            },),
          ],
          replacements: [
            {
              sliceIndex: 2,
              replacementText: '### Cat Murmurs',
            },
            {
              sliceIndex: 3,
              replacementText: '[^6]: See the end of the section “Evening Cat Murmurs”.',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          '### Cat Murmurs',
          '[^6]: See the end of the section “Cat Murmurs”.',
        ],);
      },
    },),
    it({
      name: 'LEAVES A GLOSSED RUN WHOSE LEAD-IN WORDS END IN THE HEADING\'S RENDERING, since the run before a gloss '
        + 'cannot be told from the words ahead of it',
      fn: async () => {
        /**
         Pass over a credit whose glossed run says more than the title.
         */
        const unified = unifyTitleReferences({
          slices: [
            pair({
              sliceIndex: 2,
              source: '### 午后猫语',
              target: '',
            },),
            pair({
              sliceIndex: 5,
              source: '—— 云猫【梦】《午后猫语》',
              target: '',
            },),
          ],
          replacements: [
            HEADINGS[1],
            {
              sliceIndex: 5,
              replacementText: '—— Yunmao, from the Afternoon Cat Murmurs (午后猫语)',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          '### Afternoon Cat Murmurs',
          '—— Yunmao, from the Afternoon Cat Murmurs (午后猫语)',
        ],);
      },
    },),
    it({
      name: 'REWRITES A GLOSSED RUN WHOSE WORD ONLY RUNS INTO THE HEADING\'S RENDERING: "WildCat Murmurs" does not end '
        + 'in "Cat Murmurs" as words',
      fn: async () => {
        /**
         Pass over a credit whose glossed run glues a word to the title.
         */
        const unified = unifyTitleReferences({
          slices: [
            pair({
              sliceIndex: 2,
              source: '### 午后猫语',
              target: '',
            },),
            pair({
              sliceIndex: 5,
              source: '—— 云猫【梦】《午后猫语》',
              target: '',
            },),
          ],
          replacements: [
            {
              sliceIndex: 2,
              replacementText: '### Cat Murmurs',
            },
            {
              sliceIndex: 5,
              replacementText: '—— Yunmao, WildCat Murmurs (午后猫语)',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          '### Cat Murmurs',
          '—— Yunmao, Cat Murmurs (午后猫语)',
        ],);
      },
    },),
    it({
      name: 'LEAVES A SLICE THAT NAMES THE TITLE WITHOUT BRACKETS, whatever its page quotes: a bare mention is words, '
        + 'not a reference',
      fn: async () => {
        /**
         Pass over a slice that says the title's words without marking them.
         */
        const unified = unifyTitleReferences({
          slices: [
            pair({
              sliceIndex: 2,
              source: '### 午后猫语',
              target: '',
            },),
            pair({
              sliceIndex: 8,
              source: '她说午后猫语很好听。',
              target: '',
            },),
          ],
          replacements: [
            HEADINGS[1],
            {
              sliceIndex: 8,
              replacementText: 'She said “Afternoon Cat Talk” sounds lovely.',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          '### Afternoon Cat Murmurs',
          'She said “Afternoon Cat Talk” sounds lovely.',
        ],);
      },
    },),
    it({
      name: 'LEAVES a reference already rendered as the heading, one the archive alone carries, and one it cannot place',
      fn: async () => {
        /**
         Pass over a page whose link already matches, whose footnote is the
         archive's own, and whose credit quotes two things.
         */
        const unified = unifyTitleReferences({
          slices: [
            ...SLICES.slice(
              0,
              3,
            ),
            pair({
              sliceIndex: 3,
              source: '[^6]: 另见「午后猫语」篇末尾。',
              target: '[^6]: See the section “Afternoon Cat Talk”.',
            },),
            ...SLICES.slice(
              4,
              6,
            ),
          ],
          replacements: [
            ...HEADINGS,
            {
              sliceIndex: 1,
              replacementText: '——From [Cat by the Window](https://example.test/cat)',
            },
            {
              sliceIndex: 4,
              replacementText: '—— Yunmao, “Cat Talk,” from “Dream”',
            },
          ],
        },);
        expect(unified.restored,).toEqual([],);
        expect(unified.findings.join('\n',),).toContain('title-reference-ambiguous (slice 4',);
        expect(unified.findings.length,).toBe(1,);
      },
    },),
    it({
      name: 'READS A REFERENCE APART FROM THE HEADING ONLY IN APOSTROPHE STYLE AS THE HEADING\'S (ledger B24): '
        + 'neither ambiguous beside a second quoted title nor rewritten, since the typography restoration makes them one',
      fn: async () => {
        /**
         Credit quoting the heading with the other apostrophe beside a second title.
         */
        const credit = '—— Yunmao “The Cat\'s Afternoon”, from “The Cat Sutra”';
        /**
         Pass over a curly heading and a credit writing it straight.
         */
        const beside = unifyTitleReferences({
          slices: [
            pair({
              sliceIndex: 0,
              source: '### 猫的午后',
              target: '',
            },),
            pair({
              sliceIndex: 1,
              source: '—— 云猫《猫的午后》，出自《猫经》',
              target: '',
            },),
          ],
          replacements: [
            {
              sliceIndex: 0,
              replacementText: '### The Cat’s Afternoon',
            },
            {
              sliceIndex: 1,
              replacementText: credit,
            },
          ],
        },);
        expect(beside.replacements[1]?.replacementText,).toBe(credit,);
        expect(beside.findings,).toEqual([],);
        /**
         Credit quoting a straight heading with a curly apostrophe.
         */
        const curly = '—— Yunmao “The Cat’s Afternoon”';
        /**
         Pass over a straight heading and a credit writing it curly.
         */
        const alone = unifyTitleReferences({
          slices: [
            pair({
              sliceIndex: 0,
              source: '### 猫的午后',
              target: '',
            },),
            pair({
              sliceIndex: 1,
              source: '—— 云猫《猫的午后》',
              target: '',
            },),
          ],
          replacements: [
            {
              sliceIndex: 0,
              replacementText: '### The Cat\'s Afternoon',
            },
            {
              sliceIndex: 1,
              replacementText: curly,
            },
          ],
        },);
        expect(alone.replacements[1]?.replacementText,).toBe(curly,);
        expect(alone.findings,).toEqual([],);
      },
    },),
    it({
      name: 'SETTLES SEVERAL QUOTED SPANS ONLY BY ONE CARRYING THE HEADING AS WORDS (ledger B23, found under B24): '
        + '"Catnip Days" does not carry "Cat", so the credit is ambiguous rather than rewritten',
      fn: async () => {
        /**
         Credit quoting two titles, neither of them the heading.
         */
        const credit = '—— Yunmao “Catnip Days”, from “Dog Days”';
        /**
         Pass over a heading and that credit.
         */
        const unified = unifyTitleReferences({
          slices: [
            pair({
              sliceIndex: 0,
              source: '### 猫',
              target: '',
            },),
            pair({
              sliceIndex: 1,
              source: '—— 云猫《猫》，出自《狗日》',
              target: '',
            },),
          ],
          replacements: [
            {
              sliceIndex: 0,
              replacementText: '### Cat',
            },
            {
              sliceIndex: 1,
              replacementText: credit,
            },
          ],
        },);
        expect(unified.replacements[1]?.replacementText,).toBe(credit,);
        expect(unified.findings.join('\n',),).toContain('title-reference-ambiguous (slice 1',);
      },
    },),
    it({
      name: 'READS past a tag attribute\'s quotes and past a second quoted span that already carries the heading',
      fn: async () => {
        /**
         Pass over a credit slice that also carries a tag with quoted
         attributes, and over one that quotes the heading beside another
         quoted title.
         */
        const unified = unifyTitleReferences({
          slices: [
            ...SLICES.slice(
              0,
              3,
            ),
            pair({
              sliceIndex: 3,
              source: '—— 云猫《午后猫语》\n\n<Ring text="☿☿" size="1rem"/>',
              target: '',
            },),
            pair({
              sliceIndex: 4,
              source: '—— 云猫《午后猫语》，出自《猫经》',
              target: '',
            },),
          ],
          replacements: [
            ...HEADINGS,
            {
              sliceIndex: 3,
              replacementText: '—— Yunmao "Cat Talk"\n\n<Ring text="☿☿" size="1rem"/>',
            },
            {
              sliceIndex: 4,
              replacementText: '—— Yunmao “Afternoon Cat Murmurs”, from “The Cat Sutra”',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          '<h3 align = "center">Cat by the Window</h3>',
          '### Afternoon Cat Murmurs',
          '—— Yunmao "Afternoon Cat Murmurs"\n\n<Ring text="☿☿" size="1rem"/>',
          '—— Yunmao “Afternoon Cat Murmurs”, from “The Cat Sutra”',
        ],);
        expect(unified.findings.length,).toBe(1,);
      },
    },),
    it({
      // CLASS ONE HUNDRED TWENTY (2026-09-24). The credit rendered
      // the title bare, the only quoted span left was the next heading's
      // `align = "center"`, spaced around its equals sign, and the pass wrote
      // the title into the attribute.
      name: 'LEAVES a tag attribute spaced around its equals sign where the credit renders the title bare',
      fn: async () => {
        /**
         Pass over a slice whose credit carries no marks and whose heading
         tag spaces its attribute.
         */
        const unified = unifyTitleReferences({
          slices: [
            pair({
              sliceIndex: 0,
              source: '<h3 align = "center">七彩猫</h3>',
              target: '',
            },),
            pair({
              sliceIndex: 1,
              source: '—— 云猫【梦】《七彩猫》\n\n<h3 align = "center">午后猫语</h3>',
              target: '',
            },),
          ],
          replacements: [
            {
              sliceIndex: 0,
              replacementText: '<h3 align = "center">Rainbow Cat</h3>',
            },
            {
              sliceIndex: 1,
              replacementText: '—— Yunmao 【Dream】 Rainbow Cat\n\n<h3 align = "center">Afternoon Cat Murmurs</h3>',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          '<h3 align = "center">Rainbow Cat</h3>',
          '—— Yunmao 【Dream】 Rainbow Cat\n\n<h3 align = "center">Afternoon Cat Murmurs</h3>',
        ],);
        expect(unified.restored,).toEqual([],);
      },
    },),
    it({
      // CLASS ONE HUNDRED FORTY-SIX (2026-09-26). The second song
      // credit rendered its title in pinyin inside a slice whose summary
      // quotes a line, so the quote search found two spans, neither the
      // heading's, and stood aside; the credit line the original writes in
      // its own tag was one line of the page.
      name: 'READS the credit on the page\'s line opening with the same tag where the slice quotes elsewhere too',
      fn: async () => {
        /**
         Pass over a slice whose summary quotes the cat and whose tagged
         credit renders the title afresh.
         */
        const unified = unifyTitleReferences({
          slices: [
            ...SLICES.slice(
              0,
              3,
            ),
            pair({
              sliceIndex: 3,
              source: '<summary>「猫说晚安」</summary>\n\n<p style="text-align: end;">—— 云猫【梦】《午后猫语》</p>',
              target: '',
            },),
          ],
          replacements: [
            ...HEADINGS,
            {
              sliceIndex: 3,
              replacementText: '<summary>“The cat said goodnight”</summary>\n\n'
                + '<p style="text-align: end;">—— Yunmao 【Dream】 “Wu Hou Mao Yu”</p>',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },)[2],).toBe(
          '<summary>“The cat said goodnight”</summary>\n\n'
            + '<p style="text-align: end;">—— Yunmao 【Dream】 “Afternoon Cat Murmurs”</p>',
        );
        expect(unified.findings.join('\n',),).not.toContain('title-reference-ambiguous',);
      },
    },),
  ],
},);
