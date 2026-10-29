/**
 Tests for the alternative text of an image that holds an angle autolink or a code span.

 The plain parse holds the alternative text of an image as a string: an angle autolink in it is its address alone, and a
 code span is its content as typed. The page must show the same, so the autolink is written as its address without the
 angle brackets MDX reads as a tag, and a code span is copied as it stands, since a backslash written inside one is a
 character of the code.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { encodeAsLiteralMdx, } from '../../dist/final/node/index.mjs';
import { encodedReading, } from './source-reading-same.test-fixture.ts';

await describe({
  name: 'source ships image alternative text',
  children: [
    it({
      name: 'WRITES AN ANGLE AUTOLINK IN THE ALTERNATIVE TEXT AS ITS ADDRESS ALONE, which the page shows as the source does',
      fn: async () => {
        expect(encodedReading({ source: '![<https://example.net/a>](u.png)', },),).toEqual({
          written: '![https://example.net/a](u.png)',
          kind: 'same',
        },);
        expect(encodedReading({ source: '![<a@b.net>](u.png)', },),).toEqual({
          written: '![a@b.net](u.png)',
          kind: 'same',
        },);
      },
    },),

    it({
      name: 'WRITES THE ADDRESS WITH A BACKSLASH BEFORE EACH CHARACTER MARKDOWN READS, and the text around it as any text',
      fn: async () => {
        expect(encodedReading({ source: '![x {y} <https://example.net/a_b>](u.png) {', },),).toEqual({
          written: String.raw`![x \{y\} https://example.net/a\_b](u.png) \{`,
          kind: 'same',
        },);
      },
    },),

    it({
      name: 'WRITES AN ANGLE AUTOLINK AS ITS ADDRESS INSIDE EMPHASIS IN THE ALTERNATIVE TEXT, and in an image reference',
      fn: async () => {
        expect(encodedReading({ source: '![*<https://example.net/a>*](u.png)', },),).toEqual({
          written: '![*https://example.net/a*](u.png)',
          kind: 'same',
        },);
        expect(encodedReading({ source: '![<https://example.net/a>][r]\n\n[r]: u.png\n', },),).toEqual({
          written: '![https://example.net/a][r]\n\n[r]: u.png\n',
          kind: 'same',
        },);
      },
    },),

    it({
      name: 'READS AN ALTERNATIVE TEXT OF SEVERAL LINES AS THE IMAGE HAS IT, the marks of a block quote and the indentation of '
        + 'a list item at the start of its lines among them',
      fn: async () => {
        expect(encodedReading({ source: '> ![a\n> b <https://example.net/a>](u.png) {\n', },),).toEqual({
          written: '> ![a\n> b https://example.net/a](u.png) \\{\n',
          kind: 'same',
        },);
        expect(encodedReading({ source: '- ![a\n  b <https://example.net/a>](u.png) {\n', },),).toEqual({
          written: '- ![a\n  b https://example.net/a](u.png) \\{\n',
          kind: 'same',
        },);
      },
    },),

    it({
      name: 'READS A LAST LINE THAT IS AN UNDERLINE OR A BREAK ON ITS OWN AS THE TEXT IT IS, since a closing bracket follows it '
        + 'in the image',
      fn: async () => {
        for (
          const [source, written,] of [
            ['![<https://example.net/a>\n===](u.png) {\n', '![https://example.net/a\n===](u.png) \\{\n',],
            ['- ![<https://example.net/a>\n  ---](u.png) {\n', '- ![https://example.net/a\n  ---](u.png) \\{\n',],
            ['![<https://example.net/a>\n***](u.png) {\n', '![https://example.net/a\n***](u.png) \\{\n',],
          ] as const
        )
          expect(encodedReading({ source, },),).toEqual({
            written,
            kind: 'same',
          },);
      },
    },),

    it({
      name: 'WRITES AN ANGLE AUTOLINK BESIDE A LINK IN THE ALTERNATIVE TEXT AS ITS ADDRESS, the link as it stands',
      fn: async () => {
        expect(encodedReading({ source: '![a [x](y) <https://example.net/a>](u.png) {\n', },),).toEqual({
          written: '![a [x](y) https://example.net/a](u.png) \\{\n',
          kind: 'same',
        },);
      },
    },),

    it({
      name: 'KEEPS THE OLD WRITING FOR A LINE OF A BLOCK QUOTE WITHOUT ITS MARK THAT READS AS AN UNDERLINE ON ITS OWN, a limit',
      fn: async () => {
        expect(encodeAsLiteralMdx({ text: '> ![<https://example.net/a>\n===\nb](u.png) {\n', },),)
          .toBe('> ![\\<https://example.net/a>\n===\nb](u.png) \\{\n',);
      },
    },),

    it({
      name: 'COPIES A CODE SPAN IN THE ALTERNATIVE TEXT AS IT STANDS, whatever it holds, since a backslash in it would show',
      fn: async () => {
        expect(encodedReading({ source: '![`<https://example.net/a>`](u.png)', },),).toEqual({
          written: '![`<https://example.net/a>`](u.png)',
          kind: 'same',
        },);
        expect(encodedReading({ source: '![`{x}` y {z}](u.png)', },),).toEqual({
          written: String.raw`![${ '`{x}`' } y \{z\}](u.png)`,
          kind: 'same',
        },);
      },
    },),
  ],
},);
