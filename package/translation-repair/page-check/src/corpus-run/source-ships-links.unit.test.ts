/**
 Tests for writing a link in Markdown's own syntax where the original had an angle autolink or a bare address.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  explicitLink,
  linkInsteadOfAutolink,
} from '../../dist/final/node/index.mjs';

/**
 The code point of a no-break space, which Markdown reads as white space.
 */
const NO_BREAK_SPACE = 0xA0;

await describe({
  name: 'source ships links',
  children: [
    describe({
      name: explicitLink.name,
      children: [
        it({
          name: 'WRITES THE CHARACTERS MARKDOWN READS WITH A BACKSLASH IN THE TEXT AND AS A PERCENT ESCAPE IN THE '
            + 'TARGET',
          fn: async () => {
            expect(explicitLink({
              label: 'a_b*c[d]{e}',
              target: 'https://example.net/a b(c){d}',
            },),).toBe(String.raw`[a\_b\*c\[d\]\{e\}](https://example.net/a%20b%28c%29%7Bd%7D)`,);
          },
        },),

        it({
          name: 'WRITES A CHARACTER BEYOND ASCII THAT IS NO LETTER AS THE ESCAPE OF ITS BYTES and a letter as itself',
          fn: async () => {
            expect(explicitLink({
              label: '猫',
              target: `https://example.net/猫${ String.fromCodePoint(NO_BREAK_SPACE,) }x`,
            },),).toBe('[猫](https://example.net/猫%C2%A0x)',);
          },
        },),
      ],
    },),

    describe({
      name: linkInsteadOfAutolink.name,
      children: [
        it({
          name: 'TAKES THE TEXT FROM BETWEEN THE ANGLE BRACKETS AND THE TARGET FROM THE PARSE',
          fn: async () => {
            expect(linkInsteadOfAutolink({
              source: '见 <tabby@example.net>',
              url: 'mailto:tabby@example.net',
              range: {
                start: 2,
                end: 21,
              },
            },),).toBe('[tabby@example.net](mailto:tabby@example.net)',);
          },
        },),
      ],
    },),
  ],
},);
