/**
 Guards class sixty-eight (XingZ616, 2026-09-19): the ten section headings
 `其一` to `其十` reached the page in seven ordinal styles ("One:", none,
 "IV:", "The Fifth:", "Part Six:", "IX:", "Ten:") because slices are judged
 one at a time and a parallel series has no one style. The page renders a
 numbered heading series in one style. Class eighty-nine (XingZ624,
 2026-09-23) took the archive's style wherever the archive headed the series,
 and XingZ6015 (2026-10-20) showed that dropped nine numbers the original
 gives: a bare archive numbers nothing. The style is now the archive's where
 the archive numbers the series in one style, else the house style "Part
 One:", whatever the lanes wrote. Cat-themed invention throughout; no corpus
 content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { unifyHeadingSeries, } from '../../dist/final/node/index.mjs';
import { pair, } from './title-reference.test-fixture.ts';

/**
 One slice carrying one heading of the series.

 @param sliceIndex - where the slice stands

 @param source - original heading

 @param target - archive heading, empty where the archive never headed the section

 @returns Prepared pair

 @example
 ```ts
 const slice = section({ sliceIndex: 0, source: '### 其一：橘猫', target: '### Ginger', },);
 ```
 */
function section(
  {
    sliceIndex,
    source,
    target,
  }: {
    readonly sliceIndex: number;
    readonly source: string;
    readonly target: string;
  },
): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: source.length,
      text: `${source}\n\n猫。`,
    },
    target: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: target.length,
      text: (target === '') ? 'Cat.' : `${target}\n\nCat.`,
    },
  };
}

/**
 Three numbered sections the archive never headed, so the renderings alone
 set the style.
 */
const SLICES: readonly ChunkPair[] = [
  section({
    sliceIndex: 0,
    source: '### 其一：橘猫',
    target: '',
  },),
  section({
    sliceIndex: 1,
    source: '### 其二：黑猫',
    target: '',
  },),
  section({
    sliceIndex: 2,
    source: '### 其三：白猫',
    target: '',
  },),
];

/**
 The same three sections as the archive heads them, without ordinals.
 */
const ARCHIVED: readonly ChunkPair[] = [
  section({
    sliceIndex: 0,
    source: '### 其一：橘猫',
    target: '### Ginger',
  },),
  section({
    sliceIndex: 1,
    source: '### 其二：黑猫',
    target: '### Sooty',
  },),
  section({
    sliceIndex: 2,
    source: '### 其三：白猫',
    target: '### Snowy',
  },),
];

/**
 The same three sections as an archive heads them with "Part" and a
 cardinal.
 */
const PARTED: readonly ChunkPair[] = [
  section({
    sliceIndex: 0,
    source: '### 其一：橘猫',
    target: '### Part One: Ginger',
  },),
  section({
    sliceIndex: 1,
    source: '### 其二：黑猫',
    target: '### Part Two: Sooty',
  },),
  section({
    sliceIndex: 2,
    source: '### 其三：白猫',
    target: '### Part Three: Snowy',
  },),
];

/**
 Replacement carrying one heading rendering.

 @param sliceIndex - slice replaced

 @param heading - rendered heading line

 @returns Replacement as the page would write it

 @example
 ```ts
 const row = rendered({ sliceIndex: 0, heading: '### One: Ginger', },);
 ```
 */
function rendered(
  {
    sliceIndex,
    heading,
  }: {
    readonly sliceIndex: number;
    readonly heading: string;
  },
): { readonly sliceIndex: number; readonly replacementText: string; } {
  return {
    sliceIndex,
    replacementText: `${heading}\n\nCat.`,
  };
}

/**
 Heading lines of the replacements, by slice order.

 @param replacements - what the page would write

 @returns First line of each replacement

 @example
 ```ts
 const lines = headingsOf({ replacements, },);
 ```
 */
function headingsOf(
  { replacements, }: { readonly replacements: readonly { readonly replacementText: string; }[]; },
): readonly string[] {
  return replacements.map(function firstLine(replacement,): string {
    return replacement.replacementText
      .split('\n',)[0] ?? '';
  },);
}

/**
 Headings the bare archive's series ships for one set of lane renderings.

 @param headings - rendered heading per slice, in slice order

 @returns First line of each shipped replacement

 @example
 ```ts
 const shipped = shippedFor({ headings: ['### Ginger', '### Sooty', '### Snowy',], },);
 ```
 */
function shippedFor({ headings, }: { readonly headings: readonly string[]; },): readonly string[] {
  return headingsOf({ replacements: unifyHeadingSeries({
    slices: ARCHIVED,
    replacements: headings.map(function row(
      heading,
      sliceIndex,
    ) {
      return rendered({
        sliceIndex,
        heading,
      },);
    },),
  },).replacements, },);
}

await describe({
  name: 'a numbered heading series takes one ordinal style (class sixty-eight, XingZ616)',
  children: [
    it({
      name: 'RENDERS every heading in the house style where the archive never headed the series, whatever '
        + 'style the renderings tied or leaned to',
      fn: async () => {
        const unified = unifyHeadingSeries({
          slices: SLICES,
          replacements: [
            rendered({
              sliceIndex: 0,
              heading: '### One: Ginger',
            },),
            rendered({
              sliceIndex: 1,
              heading: '### Sooty',
            },),
            rendered({
              sliceIndex: 2,
              heading: '### III: Snowy',
            },),
          ],
        },);
        expect(headingsOf({ replacements: unified.replacements, },),).toEqual([
          '### Part One: Ginger',
          '### Part Two: Sooty',
          '### Part Three: Snowy',
        ],);
        expect(unified.findings.length,).toBe(3,);
      },
    },),
    it({
      name: 'REACHES a heading the page carries from the archive alone by writing a replacement for it',
      fn: async () => {
        const unified = unifyHeadingSeries({
          slices: [
            SLICES[0] as ChunkPair,
            ARCHIVED[1] as ChunkPair,
            SLICES[2] as ChunkPair,
          ],
          replacements: [
            rendered({
              sliceIndex: 0,
              heading: '### The First: Ginger',
            },),
            rendered({
              sliceIndex: 2,
              heading: '### The Third: Snowy',
            },),
          ],
        },);
        expect(unified.replacements
          .find(function second(replacement,): boolean {
            return replacement.sliceIndex === 1;
          },)?.replacementText,).toBe('### Part Two: Sooty\n\nCat.',);
      },
    },),
    it({
      name: 'TAKES THE ARCHIVE\'S STYLE where the archive numbers the series (class eighty-nine, XingZ624, '
        + '2026-09-23): an archive with "Part" and a cardinal writes that on every heading, and an archive '
        + 'without ordinals numbers nothing, so the house style stands (XingZ6015, 2026-10-20)',
      fn: async () => {
        const bare = unifyHeadingSeries({
          slices: ARCHIVED,
          replacements: [
            rendered({
              sliceIndex: 0,
              heading: '### One: Ginger',
            },),
            rendered({
              sliceIndex: 1,
              heading: '### Part Two: Sooty',
            },),
            rendered({
              sliceIndex: 2,
              heading: '### Part Three: Snowy',
            },),
          ],
        },);
        expect(headingsOf({ replacements: bare.replacements, },),).toEqual([
          '### Part One: Ginger',
          '### Part Two: Sooty',
          '### Part Three: Snowy',
        ],);
        expect(bare.findings[0],).toContain('the house style',);
        const parted = unifyHeadingSeries({
          slices: PARTED,
          replacements: [
            rendered({
              sliceIndex: 0,
              heading: '### Ginger',
            },),
            rendered({
              sliceIndex: 1,
              heading: '### Sooty',
            },),
            rendered({
              sliceIndex: 2,
              heading: '### III: Snowy',
            },),
          ],
        },);
        expect(headingsOf({ replacements: parted.replacements, },),).toEqual([
          '### Part One: Ginger',
          '### Part Two: Sooty',
          '### Part Three: Snowy',
        ],);
        expect(parted.findings.length,).toBe(3,);
        expect(parted.findings[0],).toContain('the archive\'s own style',);
      },
    },),
    it({
      name: 'READS A LEADER WORD THE TABLE NEVER LISTED (class ninety, XingZ625, 2026-09-23): "Poem Ten:" '
        + 'escaped the series and shipped beside nine bare headings; any word before a number word is its '
        + 'leader, so the house style replaces it and an archive that numbers with "Song" writes it on every '
        + 'heading',
      fn: async () => {
        const dropped = unifyHeadingSeries({
          slices: ARCHIVED,
          replacements: [
            rendered({
              sliceIndex: 0,
              heading: '### Poem One: Ginger (Orange Cat)',
            },),
            rendered({
              sliceIndex: 1,
              heading: '### Sooty',
            },),
            rendered({
              sliceIndex: 2,
              heading: '### Snowy',
            },),
          ],
        },);
        expect(headingsOf({ replacements: dropped.replacements, },),).toEqual([
          '### Part One: Ginger (Orange Cat)',
          '### Part Two: Sooty',
          '### Part Three: Snowy',
        ],);
        expect(dropped.findings.length,).toBe(3,);
        const sung = unifyHeadingSeries({
          slices: [
            section({
              sliceIndex: 0,
              source: '### 其一：橘猫',
              target: '### Song One: Ginger',
            },),
            section({
              sliceIndex: 1,
              source: '### 其二：黑猫',
              target: '### Song Two: Sooty',
            },),
            section({
              sliceIndex: 2,
              source: '### 其三：白猫',
              target: '### Song Three: Snowy',
            },),
          ],
          replacements: [
            rendered({
              sliceIndex: 0,
              heading: '### Song One: Ginger',
            },),
            rendered({
              sliceIndex: 1,
              heading: '### Song Two: Sooty',
            },),
            rendered({
              sliceIndex: 2,
              heading: '### Snowy',
            },),
          ],
        },);
        expect(headingsOf({ replacements: sung.replacements, },),).toEqual([
          '### Song One: Ginger',
          '### Song Two: Sooty',
          '### Song Three: Snowy',
        ],);
      },
    },),
    it({
      name: 'READS THE NUMBER BEFORE THE FIRST COLON where a title carries an ASCII colon and a fullwidth one after it',
      fn: async () => {
        const unified = unifyHeadingSeries({
          slices: SLICES,
          replacements: [
            rendered({
              sliceIndex: 0,
              heading: '### One: Ginger',
            },),
            rendered({
              sliceIndex: 1,
              heading: '### Two: Sooty：the black one',
            },),
            rendered({
              sliceIndex: 2,
              heading: '### Three: Snowy',
            },),
          ],
        },);
        expect(headingsOf({ replacements: unified.replacements, },),).toEqual([
          '### Part One: Ginger',
          '### Part Two: Sooty：the black one',
          '### Part Three: Snowy',
        ],);
        expect(unified.findings.length,).toBe(3,);
      },
    },),
    it({
      name: 'LEAVES a page with fewer than two numbered headings alone',
      fn: async () => {
        const unified = unifyHeadingSeries({
          slices: [SLICES[0] as ChunkPair,],
          replacements: [rendered({
            sliceIndex: 0,
            heading: '### IX: Ginger',
          },),],
        },);
        expect(unified.findings,).toEqual([],);
        expect(unified.restored,).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES A SLICE WHOSE PAGE DROPPED ONE OF ITS NUMBERED HEADINGS AS THE PAGE HAS IT: pairing the page\'s '
        + 'headings with the original\'s numbers by position gave the third section the second\'s number '
        + '(ledger B84)',
      fn: async () => {
        /**
         What the page writes for the two-section slice, its first heading dropped.
         */
        const dropped = {
          sliceIndex: 1,
          replacementText: 'Cat.\n\n### Part Three: Snowy\n\nCat.',
        };
        const unified = unifyHeadingSeries({
          slices: [
            PARTED[0] as ChunkPair,
            pair({
              sliceIndex: 1,
              source: '### 其二：黑猫\n\n猫。\n\n### 其三：白猫\n\n猫。',
              target: '### Part Two: Sooty\n\nCat.\n\n### Part Three: Snowy\n\nCat.',
            },),
          ],
          replacements: [dropped,],
        },);
        expect(unified,).toEqual({
          replacements: [dropped,],
          restored: [],
          findings: [],
        },);
      },
    },),
    it({
      name: 'PASSES OVER HEADINGS THE ORIGINAL NEVER NUMBERED: a slice with no numbered heading, a heading that '
        + 'opens with 其 but carries no colon, and an unnumbered heading beside a numbered one in the same slice '
        + '(T8, twentieth batch)',
      fn: async () => {
        const unified = unifyHeadingSeries({
          slices: [
            pair({
              sliceIndex: 0,
              source: '### 序\n\n猫。',
              target: '### Preface\n\nCat.',
            },),
            pair({
              sliceIndex: 1,
              source: '### 其他\n\n猫。\n\n### 其一：橘猫\n\n猫。',
              target: '### Others\n\nCat.\n\n### One: Ginger\n\nCat.',
            },),
            pair({
              sliceIndex: 2,
              source: '### 其二：黑猫\n\n猫。',
              target: '### Two: Sooty\n\nCat.',
            },),
          ],
          replacements: [
            {
              sliceIndex: 1,
              replacementText: '### Others\n\nCat.\n\n### I: Ginger\n\nCat.',
            },
            {
              sliceIndex: 2,
              replacementText: '### II: Sooty\n\nCat.',
            },
          ],
        },);
        expect(unified.replacements,).toEqual([
          {
            sliceIndex: 1,
            replacementText: '### Others\n\nCat.\n\n### One: Ginger\n\nCat.',
          },
          {
            sliceIndex: 2,
            replacementText: '### Two: Sooty\n\nCat.',
          },
        ],);
      },
    },),
    it({
      name: 'TAKES THE HOUSE STYLE WHERE THE ARCHIVE HEADS THE SERIES IN TWO STYLES, and counts a rendered '
        + 'prefix that is no ordinal as no number at all, so its words stay in the title (T8, twentieth batch)',
      fn: async () => {
        const unified = unifyHeadingSeries({
          slices: [
            section({
              sliceIndex: 0,
              source: '### 其一：橘猫',
              target: '### One: Ginger',
            },),
            PARTED[1] as ChunkPair,
            PARTED[2] as ChunkPair,
          ],
          replacements: [
            rendered({
              sliceIndex: 0,
              heading: '### Kitten: Ginger',
            },),
            rendered({
              sliceIndex: 1,
              heading: '### Sooty',
            },),
            rendered({
              sliceIndex: 2,
              heading: '### III: Snowy',
            },),
          ],
        },);
        expect(headingsOf({ replacements: unified.replacements, },),).toEqual([
          '### Part One: Kitten: Ginger',
          '### Part Two: Sooty',
          '### Part Three: Snowy',
        ],);
        expect(unified.findings[0],).toContain('the house style',);
      },
    },),
    it({
      name: 'READS THE ARCHIVE\'S STYLE ONLY WHERE ITS HEADINGS PAIR WITH THE ORIGINAL\'S: an archive slice that '
        + 'opens with an interlude heading of its own once had that heading read as the series\' third, so the '
        + 'archive seemed to use two styles and its own "Part" style was lost (ledger B84)',
      fn: async () => {
        const unified = unifyHeadingSeries({
          slices: [
            PARTED[0] as ChunkPair,
            PARTED[1] as ChunkPair,
            pair({
              sliceIndex: 2,
              source: '### 其三：白猫\n\n猫。',
              target: '### Interlude\n\nCat.\n\n### Part Three: Snowy\n\nCat.',
            },),
          ],
          replacements: [
            rendered({
              sliceIndex: 0,
              heading: '### Ginger',
            },),
            rendered({
              sliceIndex: 1,
              heading: '### Sooty',
            },),
            rendered({
              sliceIndex: 2,
              heading: '### Snowy',
            },),
          ],
        },);
        expect(headingsOf({ replacements: unified.replacements, },),).toEqual([
          '### Part One: Ginger',
          '### Part Two: Sooty',
          '### Part Three: Snowy',
        ],);
      },
    },),
    it({
      name: 'KEEPS A NUMBER ON EVERY HEADING WHERE THE ARCHIVE HEADS THE SERIES BY NAME ALONE (XingZ6015, '
        + '2026-10-20): the lanes rendered "Part One:" to "Part Four:" and the page dropped all of them to '
        + 'match the archive, nine numbers lost; the series takes the house style "Part One:"',
      fn: async () => {
        /**
         Four sections: the archive heads the first three by name alone and never headed the fourth.
         */
        const slices = [
          ARCHIVED[0] as ChunkPair,
          ARCHIVED[1] as ChunkPair,
          ARCHIVED[2] as ChunkPair,
          section({
            sliceIndex: 3,
            source: '### 其四：花猫',
            target: '',
          },),
        ];
        const unified = unifyHeadingSeries({
          slices,
          replacements: [
            rendered({
              sliceIndex: 0,
              heading: '### Part One: Ginger',
            },),
            rendered({
              sliceIndex: 1,
              heading: '### Part Two: Sooty',
            },),
            rendered({
              sliceIndex: 2,
              heading: '### Snowy',
            },),
            rendered({
              sliceIndex: 3,
              heading: '### The Fourth: Patches',
            },),
          ],
        },);
        expect(headingsOf({ replacements: unified.replacements, },),).toEqual([
          '### Part One: Ginger',
          '### Part Two: Sooty',
          '### Part Three: Snowy',
          '### Part Four: Patches',
        ],);
      },
    },),
    it({
      name: 'GIVES THE SAME STYLE WHATEVER THE LANES WROTE: one original and one bare archive, three different '
        + 'sets of renderings (all bare, a "Roman" majority, a "Part" majority), one shipped series',
      fn: async () => {
        /**
         The series every run must ship.
         */
        const expected = [
          '### Part One: Ginger',
          '### Part Two: Sooty',
          '### Part Three: Snowy',
        ];
        expect(shippedFor({ headings: ['### Ginger', '### Sooty', '### Snowy',], },),).toEqual(expected,);
        expect(shippedFor({ headings: ['### I: Ginger', '### II: Sooty', '### Snowy',], },),).toEqual(expected,);
        expect(shippedFor({ headings: ['### Part One: Ginger', '### Part Two: Sooty', '### Part Three: Snowy',], },),)
          .toEqual(expected,);
      },
    },),
    it({
      name: 'TAKES THE HOUSE STYLE where the archive never headed the series or headed it in two styles, and '
        + 'names the house style in the finding',
      fn: async () => {
        /**
         Renderings that would, by plurality, have dropped every number.
         */
        const bare = [
          rendered({
            sliceIndex: 0,
            heading: '### Ginger',
          },),
          rendered({
            sliceIndex: 1,
            heading: '### Sooty',
          },),
          rendered({
            sliceIndex: 2,
            heading: '### Part 3: Snowy',
          },),
        ];
        const silent = unifyHeadingSeries({
          slices: SLICES,
          replacements: bare,
        },);
        expect(headingsOf({ replacements: silent.replacements, },),).toEqual([
          '### Part One: Ginger',
          '### Part Two: Sooty',
          '### Part Three: Snowy',
        ],);
        expect(silent.findings[0],).toContain('the house style',);
        const mixed = unifyHeadingSeries({
          slices: [
            section({
              sliceIndex: 0,
              source: '### 其一：橘猫',
              target: '### One: Ginger',
            },),
            PARTED[1] as ChunkPair,
            PARTED[2] as ChunkPair,
          ],
          replacements: bare,
        },);
        expect(headingsOf({ replacements: mixed.replacements, },),).toEqual([
          '### Part One: Ginger',
          '### Part Two: Sooty',
          '### Part Three: Snowy',
        ],);
      },
    },),
  ],
},);
