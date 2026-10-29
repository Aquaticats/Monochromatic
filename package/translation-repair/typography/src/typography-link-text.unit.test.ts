/**
 Tests that the text of a link whose label is itself an address is no prose (n159, X6). The label of `[https://x.example/a](...)`
 is the address a reader copies, so a typography rule that curls a mark in it shows an address that is not the address that
 opens. A label that holds words beside an address, and every other label, stays prose.

 Each case prints one letter per straight quote mark in order of appearance: `P` for a prose mark, `N` for one that is not.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { proseMaskReading, } from '../dist/final/node/index.mjs';
import { quoteFlags, } from './quote-flags.test-fixture.ts';

/**
 A text and the prose flags of its straight marks.
 */
type Row = {
  /**
   What the row is about.
   */
  readonly name: string;

  /**
   The text.
   */
  readonly text: string;

  /**
   The flags, in order of the marks.
   */
  readonly flags: string;
};

/**
 Labels that are an address, with the marks of the label and of the destination, then the mark in the words after the link.
 */
const ADDRESS_LABELS: readonly Row[] = [
  {
    name: 'a protocol address as the label',
    text: 'See [https://x.example/O\'Toole](https://x.example/O\'Toole) now, it\'s fine.',
    flags: 'NNP',
  },
  {
    name: 'an email address as the label',
    text: 'Mail [a\'b@c.example](mailto:a\'b@c.example) now, it\'s fine.',
    flags: 'NNP',
  },
  {
    name: 'a www address as the label',
    text: 'See [www.x.example/a\'b](https://x.example/a) now, it\'s fine.',
    flags: 'NP',
  },
];

/**
 Labels that are words, or words beside an address: prose, with the destination still no prose.
 */
const WORD_LABELS: readonly Row[] = [
  {
    name: 'words as the label',
    text: 'See [the dogs\' bowl](https://x.example/d\'q) now.',
    flags: 'PN',
  },
  {
    name: 'words beside an address as the label',
    text: 'See [the dogs\' page https://x.example/a](https://x.example/d\'q) now.',
    flags: 'PN',
  },
  {
    name: 'emphasis in the label',
    text: 'See [*the dogs\'*](https://x.example/d\'q) now.',
    flags: 'PN',
  },
];

await describe({
  name: proseMaskReading.name,
  children: [
    ...ADDRESS_LABELS.map(function addressCase(row,) {
      return it({
        name: `READS THE TEXT OF A LINK WHOSE LABEL IS AN ADDRESS AS NO PROSE: ${row.name}`,
        fn: async () => {
          expect(quoteFlags({ text: row.text, },),).toBe(row.flags,);
        },
      },);
    },),
    ...WORD_LABELS.map(function wordCase(row,) {
      return it({
        name: `STILL READS A LABEL OF WORDS AS PROSE: ${row.name}`,
        fn: async () => {
          expect(quoteFlags({ text: row.text, },),).toBe(row.flags,);
        },
      },);
    },),
  ],
},);
