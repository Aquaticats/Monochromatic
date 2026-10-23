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
  autolinkAsText,
  autolinkWritten,
  escapedLabel,
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
      name: escapedLabel.name,
      children: [
        it({
          name: 'WRITES A BACKSLASH BEFORE EACH CHARACTER MARKDOWN READS AND LEAVES THE OTHERS, an ampersand among them',
          fn: async () => {
            expect(escapedLabel({ label: 'a_b*c&amp;{d}[e]<f> 猫', },),)
              .toBe(String.raw`a\_b\*c\&amp;\{d\}\[e\]\<f\> 猫`,);
          },
        },),
      ],
    },),

    describe({
      name: autolinkAsText.name,
      children: [
        it({
          name: 'TAKES THE ADDRESS FROM BETWEEN THE ANGLE BRACKETS AND WRITES NO BRACKETS',
          fn: async () => {
            expect(autolinkAsText({
              source: '见 <https://example.net/a_b{c}>',
              range: {
                start: 2,
                end: 30,
              },
            },),).toBe(String.raw`https://example.net/a\_b\{c\}`,);
          },
        },),

        it({
          name: 'WRITES AN EMAIL ADDRESS AS IT STANDS',
          fn: async () => {
            expect(autolinkAsText({
              source: '<tabby@example.net>',
              range: {
                start: 0,
                end: 19,
              },
            },),).toBe('tabby@example.net',);
          },
        },),
      ],
    },),

    describe({
      name: autolinkWritten.name,
      children: [
        it({
          name: 'WRITES A LINK WITH THE SAME TEXT AND TARGET OUTSIDE LINK TEXT AND THE ADDRESS ALONE INSIDE IT',
          fn: async () => {
            /**
             An angle autolink between two spaces.
             */
            const source = '见 <https://example.net/a> 与';
            /**
             Where the autolink stands.
             */
            const range = {
              start: 2,
              end: 25,
            };
            expect(autolinkWritten({
              source,
              url: 'https://example.net/a',
              range,
              insideLink: false,
            },),).toBe('[https://example.net/a](https://example.net/a)',);
            expect(autolinkWritten({
              source,
              url: 'https://example.net/a',
              range,
              insideLink: true,
            },),).toBe('https://example.net/a',);
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
