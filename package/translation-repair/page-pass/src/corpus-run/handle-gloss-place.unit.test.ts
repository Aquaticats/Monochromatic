/**
 Guards class eighty-eight (XingZ624, 2026-09-23): the house rule puts a
 romanised handle's literal meaning in parentheses at its first appearance
 on the page and the romanisation alone after that, but every slice is
 written on its own, so the page carried the bare handle in the section
 heading and the gloss on the signature below it, and one song credit bare
 with the gloss on the next. The page decides once, here, where every
 appearance is in view: the first appearance carries the gloss the bench
 wrote anywhere, the later ones drop it. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { placeHandleGlosses, } from '../../dist/final/node/index.mjs';
import { pair, } from './title-reference.test-fixture.ts';

/**
 A section the archive never translated, headed and signed by one handle.
 */
const HEADED = pair({
  sliceIndex: 0,
  source: '### 其十：锦猫\n\n它醒了。\n\n<p style="text-align: end;">——锦猫, 2025 年 2 月 10 日</p>',
  target: '',
},);

/**
 Two song credits the archive never translated, signed by a handle no
 heading names.
 */
const CREDITED: readonly ChunkPair[] = [
  pair({
    sliceIndex: 1,
    source: '> 歌词一\n\n<p style="text-align: end;">—— 雨猫 【妄想】《歌一》</p>',
    target: '',
  },),
  pair({
    sliceIndex: 2,
    source: '> 歌词二\n\n<p style="text-align: end;">—— 雨猫 【妄想】《歌二》</p>',
    target: '',
  },),
];

/**
 A section the archive carries, whose signer the archive knows.
 */
const CARRIED = pair({
  sliceIndex: 3,
  source: '### 其三：猫猫\n\n它睡了。\n\n<p style="text-align: end;">——猫猫, 2024 年 12 月 17 日</p>',
  target: '### Maomao\n\nIt sleeps.\n\n<p style="text-align: end;">——Maomao, December 17, 2024</p>',
},);

/**
 Replacement text per slice index of a result.

 @param replacements - what the page would write

 @returns Text by slice index

 @example
 ```ts
 const text = bySlice({ replacements, },);
 ```
 */
function bySlice(
  { replacements, }: {
    readonly replacements: readonly {
      readonly sliceIndex: number;
      readonly replacementText: string;
    }[];
  },
): ReadonlyMap<number, string> {
  return new Map(replacements.map(function toEntry(replacement,): readonly [
    number,
    string,
  ] {
    return [
      replacement.sliceIndex,
      replacement.replacementText,
    ];
  },),);
}

await describe({
  name: 'a romanised handle carries its literal meaning at its first appearance alone (class eighty-eight, XingZ624)',
  children: [
    it({
      name: 'MOVES the gloss to the first appearance: the heading takes the signature\'s gloss, the signature '
        + 'drops it; the first song credit takes the second\'s gloss, the second drops it',
      fn: async () => {
        const placed = placeHandleGlosses({
          slices: [
            HEADED,
            ...CREDITED,
          ],
          replacements: [
            {
              sliceIndex: 0,
              replacementText: '### Ten: Jinmao\n\nIt woke.\n\n'
                + '<p style="text-align: end;">——Jinmao (Brocade Cat), February 10, 2025</p>',
            },
            {
              sliceIndex: 1,
              replacementText: '> Lyrics one\n\n<p style="text-align: end;">—— Yumao [Delusion] “Song One”</p>',
            },
            {
              sliceIndex: 2,
              replacementText: '> Lyrics two\n\n'
                + '<p style="text-align: end;">—— Yumao (Rain Cat) [Delusion] “Song Two”</p>',
            },
          ],
        },);
        /**
         Page text per slice after the pass.
         */
        const text = bySlice({ replacements: placed.replacements, },);
        expect(text.get(0,),).toBe(
          '### Ten: Jinmao (Brocade Cat)\n\nIt woke.\n\n'
            + '<p style="text-align: end;">——Jinmao, February 10, 2025</p>',
        );
        expect(text.get(1,),).toBe(
          '> Lyrics one\n\n<p style="text-align: end;">—— Yumao (Rain Cat) [Delusion] “Song One”</p>',
        );
        expect(text.get(2,),).toBe(
          '> Lyrics two\n\n<p style="text-align: end;">—— Yumao [Delusion] “Song Two”</p>',
        );
        expect(placed.findings.length,).toBe(4,);
        expect(placed.findings[0],).toContain('handle-gloss-placed (slice 0:',);
      },
    },),
    it({
      name: 'READS a rendering whose last letter carries a combining accent as another word, so the gloss never '
        + 'lands between a letter and its accent; a cased test read the mark as no letter (ledger B18)',
      fn: async () => {
        // The pre-letter pass read `Yumao` plus a combining acute in the lyrics
        // as the handle's first appearance and wrote the gloss between the `o`
        // and its accent, which then sat on the closing parenthesis.
        const placed = placeHandleGlosses({
          slices: CREDITED,
          replacements: [
            {
              sliceIndex: 1,
              replacementText: '> Lyrics one, sung by Yumao\u{0301} the cat\n\n'
                + '<p style="text-align: end;">—— Yumao [Delusion] “Song One”</p>',
            },
            {
              sliceIndex: 2,
              replacementText: '> Lyrics two\n\n'
                + '<p style="text-align: end;">—— Yumao (Rain Cat) [Delusion] “Song Two”</p>',
            },
          ],
        },);
        /**
         Page text per slice after the pass.
         */
        const text = bySlice({ replacements: placed.replacements, },);
        expect(text.get(1,),).toBe(
          '> Lyrics one, sung by Yumao\u{0301} the cat\n\n'
            + '<p style="text-align: end;">—— Yumao (Rain Cat) [Delusion] “Song One”</p>',
        );
        expect(text.get(2,),).toBe(
          '> Lyrics two\n\n<p style="text-align: end;">—— Yumao [Delusion] “Song Two”</p>',
        );
      },
    },),
    it({
      name: 'LEAVES a page whose gloss already stands at the first appearance alone, and one the bench never '
        + 'glossed',
      fn: async () => {
        const glossed = placeHandleGlosses({
          slices: [HEADED,],
          replacements: [{
            sliceIndex: 0,
            replacementText: '### Ten: Jinmao (Brocade Cat)\n\nIt woke.\n\n'
              + '<p style="text-align: end;">——Jinmao, February 10, 2025</p>',
          },],
        },);
        expect(glossed.findings,).toEqual([],);
        expect(glossed.restored,).toEqual([],);
        const bare = placeHandleGlosses({
          slices: [HEADED,],
          replacements: [{
            sliceIndex: 0,
            replacementText: '### Ten: Jinmao\n\nIt woke.\n\n'
              + '<p style="text-align: end;">——Jinmao, February 10, 2025</p>',
          },],
        },);
        expect(bare.findings,).toEqual([],);
        expect(bare.restored,).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES a signer the archive renders alone: the house rule glosses handles the archive never '
        + 'rendered, and a rendering inside a longer word is no appearance',
      fn: async () => {
        const placed = placeHandleGlosses({
          slices: [CARRIED,],
          replacements: [{
            sliceIndex: 3,
            replacementText: '### Maomao\n\nMaomaos sleep.\n\n'
              + '<p style="text-align: end;">——Maomao (Cat Cat), December 17, 2024</p>',
          },],
        },);
        expect(placed.findings,).toEqual([],);
        expect(placed.restored,).toEqual([],);
      },
    },),
    it({
      name: 'NEVER READS link text in parentheses as a gloss, and leaves that appearance as it stands (ledger A6: '
        + 'the first close parenthesis ended the "gloss" inside a link, which the pass then moved)',
      fn: async () => {
        const placed = placeHandleGlosses({
          slices: [HEADED,],
          replacements: [{
            sliceIndex: 0,
            replacementText: '### Ten: Jinmao\n\nJinmao ([her song](https://example.test/one)) woke.\n\n'
              + '<p style="text-align: end;">——Jinmao (Brocade Cat), February 10, 2025</p>',
          },],
        },);
        // The prose appearance keeps its link; the heading, the first
        // appearance, takes the signature's gloss, and the signature drops it.
        expect(bySlice({ replacements: placed.replacements, },).get(0,),).toBe(
          '### Ten: Jinmao (Brocade Cat)\n\nJinmao ([her song](https://example.test/one)) woke.\n\n'
            + '<p style="text-align: end;">——Jinmao, February 10, 2025</p>',
        );
      },
    },),
    it({
      name: 'REACHES a fixed point: a second run over its own output changes nothing (ledger A6)',
      fn: async () => {
        /**
         Slices of the page.
         */
        const slices = [
          HEADED,
          ...CREDITED,
        ];
        const first = placeHandleGlosses({
          slices,
          replacements: [
            {
              sliceIndex: 0,
              replacementText: '### Ten: Jinmao\n\nIt woke.\n\n'
                + '<p style="text-align: end;">——Jinmao (Brocade Cat), February 10, 2025</p>',
            },
            {
              sliceIndex: 1,
              replacementText: '> Lyrics one\n\n<p style="text-align: end;">—— Yumao ([Delusion](https://example.test/one)) '
                + '“Song One”</p>',
            },
            {
              sliceIndex: 2,
              replacementText: '> Lyrics two\n\n'
                + '<p style="text-align: end;">—— Yumao (Rain Cat) [Delusion] “Song Two”</p>',
            },
          ],
        },);
        const second = placeHandleGlosses({
          slices,
          replacements: first.replacements,
        },);
        expect(second.findings,).toEqual([],);
        expect(second.restored,).toEqual([],);
      },
    },),

    it({
      name: 'LEAVES an appearance whose parenthesis is no gloss as its writer left it, writing no gloss before '
        + 'that parenthesis and stripping none from the signature under it: a heading whose parenthesis only a '
        + 'later line closes, one whose parenthesis is empty, and one whose parenthesis never closes in its '
        + 'slice while another slice carries the gloss',
      fn: async () => {
        /**
         Signature carrying the handle's gloss, well formed.
         */
        const glossedSignature = '<p style="text-align: end;">——Jinmao (Brocade Cat), February 10, 2025</p>';
        /**
         Rows of one slice whose heading opens a parenthesis its own line
         never closes, and of one whose heading carries an empty one.
         */
        const pages = [
          '### Ten: Jinmao (Brocade Cat',
          '### Ten: Jinmao ()',
        ].map(function pageUnder(heading,) {
          return [{
            sliceIndex: 0,
            replacementText: `${heading}\n\nIt woke.\n\n${glossedSignature}`,
          },];
        },);
        expect(pages.map(function placedOver(replacements,) {
          return placeHandleGlosses({
            slices: [HEADED,],
            replacements,
          },);
        },),).toEqual(pages.map(function untouched(replacements,) {
          return {
            replacements,
            restored: [],
            findings: [],
          };
        },),);

        /**
         Rows of two slices: the first ends inside its heading's
         parenthesis, the second signs with the gloss.
         */
        const acrossSlices = [
          {
            sliceIndex: 0,
            replacementText: '### Ten: Jinmao (Brocade Cat\n\nIt woke.',
          },
          {
            sliceIndex: 1,
            replacementText: `It woke again.\n\n${glossedSignature}`,
          },
        ];
        expect(placeHandleGlosses({
          slices: [
            HEADED,
            pair({
              sliceIndex: 1,
              source: '它又醒了。\n\n<p style="text-align: end;">——锦猫, 2025 年 2 月 10 日</p>',
              target: '',
            },),
          ],
          replacements: acrossSlices,
        },),).toEqual({
          replacements: acrossSlices,
          restored: [],
          findings: [],
        },);
      },
    },),

    it({
      name: 'LEAVES an appearance whose parenthesis shows a reader nothing as its writer left it, as it leaves an '
        + 'empty one: a heading whose parenthesis holds a space, a zero-width space, a Hangul filler, or both '
        + 'among spaces takes no gloss of that nothing, and the signature under it keeps its own',
      fn: async () => {
        /**
         Rows of one slice whose heading carries a parenthesis of the given filling.
         */
        const pages = [
          ' ',
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
        ].map(function pageUnder(filling,) {
          return [{
            sliceIndex: 0,
            replacementText: `### Ten: Jinmao (${filling})\n\nIt woke.\n\n`
              + '<p style="text-align: end;">——Jinmao (Brocade Cat), February 10, 2025</p>',
          },];
        },);
        expect(pages.map(function placedOver(replacements,) {
          return placeHandleGlosses({
            slices: [HEADED,],
            replacements,
          },);
        },),).toEqual(pages.map(function untouched(replacements,) {
          return {
            replacements,
            restored: [],
            findings: [],
          };
        },),);
      },
    },),

    it({
      name: 'READS a handle standing inside a longer word as no appearance: a heading writing it after another '
        + 'letter takes no gloss, and the signature, the handle\'s one appearance, keeps its own',
      fn: async () => {
        /**
         Rows of one slice whose heading runs a letter into the handle.
         */
        const replacements = [{
          sliceIndex: 0,
          replacementText: '### Ten: xJinmao\n\nIt woke.\n\n'
            + '<p style="text-align: end;">——Jinmao (Brocade Cat), February 10, 2025</p>',
        },];
        expect(placeHandleGlosses({
          slices: [HEADED,],
          replacements,
        },),).toEqual({
          replacements,
          restored: [],
          findings: [],
        },);
      },
    },),
  ],
},);
