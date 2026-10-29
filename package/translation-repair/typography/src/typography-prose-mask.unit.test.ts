/**
 Tests for the prose mask every typography reader shares (n156, W1): a unit of a text is prose only where the page
 parse shows prose text. A quote mark in a link destination or title, a bare address, code of any kind, math, an
 expression, a tag or its attributes, a comment or front matter is not prose; a mark beside each of those is.

 Each case prints one letter per straight quote mark in order of appearance: `P` for a prose mark, `N` for one that is
 not, so a case reads as the marks a reader would see changed or left alone.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { proseMaskReading, } from '../dist/final/node/index.mjs';
import { quoteFlags, } from './quote-flags.test-fixture.ts';

/**
 Source files whose rules read the mask, each named by the family package that holds it and its path under that
 package's `src`.
 */
const READERS: readonly string[] = [
  'page-pass/restore-typography.ts',
  'page-pass/restore-ellipsis.ts',
  'typography/quote-normalize.ts',
  'translate-floor/translate-guillemets.ts',
  'page-check/corpus-run/quote-style-unify.ts',
];

await describe({
  name: 'proseMaskReading (n156, W1)',
  children: [
    it({
      name: 'READS A LINK LABEL AS PROSE AND ITS DESTINATION AND TITLE AS NOT, whichever mark quotes the title, and an image '
        + 'title and a reference definition the same, with the prose mark beside each still prose',
      fn: async () => {
        expect({
          destination: quoteFlags({ text: 'See [Peter\'s page](https://en.wikipedia.org/wiki/Peter_O\'Toole) now.', },),
          doubleTitle: quoteFlags({ text: 'See [the dogs](https://x.example/d "The dogs") and it\'s here.', },),
          singleTitle: quoteFlags({ text: 'See [the dogs](https://x.example/d \'The dogs\') and it\'s here.', },),
          imageTitle: quoteFlags({ text: 'See ![the dogs](https://x.example/d.png "The dogs") and it\'s here.', },),
          definition: quoteFlags({
            text: 'See [a][1] and it\'s here.\n\n[1]: https://x.example/O\'Toole "The dogs"\n',
          },),
          angleDestination: quoteFlags({ text: 'See [dogs](<https://x.example/it\'s me>) and it\'s here.', },),
        },).toEqual({
          destination: 'PN',
          doubleTitle: 'NNP',
          singleTitle: 'NNP',
          imageTitle: 'NNP',
          definition: 'PNNN',
          angleDestination: 'NP',
        },);
      },
    },),
    it({
      name: 'READS A BARE ADDRESS AND A WWW ADDRESS AS NOT PROSE, with the prose mark after each prose',
      fn: async () => {
        expect({
          bare: quoteFlags({ text: 'Go to https://x.example/O\'Toole now, it\'s fine.', },),
          www: quoteFlags({ text: 'Visit www.example.com/it\'s today, it\'s fine.', },),
        },).toEqual({
          bare: 'NP',
          www: 'NP',
        },);
      },
    },),
    it({
      name: 'READS AN ANGLE AUTOLINK AS A TEXT THE GRAMMAR REFUSES, since MDX reads `<https:` as the start of a tag: no mark of it '
        + 'is prose, the one after the address included',
      fn: async () => {
        expect(quoteFlags({ text: 'See <https://x.example/O\'Toole> and it\'s here.', },),).toBe('NN',);
      },
    },),
    it({
      name: 'READS INLINE CODE OF ANY BACKTICK COUNT AS NOT PROSE, a double-backtick span holding a backtick included',
      fn: async () => {
        expect({
          one: quoteFlags({ text: 'Run `it\'s` now, it\'s fine.', },),
          two: quoteFlags({ text: 'Run ``it\'s`` now, it\'s fine.', },),
          three: quoteFlags({ text: 'Run ```it\'s``` now, it\'s fine.', },),
          inner: quoteFlags({ text: 'Run ``a`b it\'s`` now, it\'s fine.', },),
        },).toEqual({
          one: 'NP',
          two: 'NP',
          three: 'NP',
          inner: 'NP',
        },);
      },
    },),
    it({
      name: 'READS A FENCED CODE BLOCK OF ANY KIND AS NOT PROSE: a backtick fence, a tilde fence and a four-backtick fence '
        + 'around a three-backtick one, with the prose after each prose',
      fn: async () => {
        expect({
          backtick: quoteFlags({ text: 'Run:\n\n```js\nlog("hi", \'it\');\n```\n\nIt\'s done.', },),
          tilde: quoteFlags({ text: 'Run:\n\n~~~js\nlog("hi");\n~~~\n\nIt\'s done.', },),
          fourBacktick: quoteFlags({ text: 'Run:\n\n````md\n```js\nlog("hi");\n```\n````\n\nIt\'s done.', },),
        },).toEqual({
          backtick: 'NNNNP',
          tilde: 'NNP',
          fourBacktick: 'NNP',
        },);
      },
    },),
    it({
      name: 'READS AN INDENTED LINE AS PROSE, since the MDX grammar the site compiles has no indented code (its parse holds a '
        + 'paragraph there), so its marks are text a reader sees and are curled like any other',
      fn: async () => {
        expect(quoteFlags({ text: 'Run:\n\n    log("hi");\n\nIt\'s done.', },),).toBe('PPP',);
      },
    },),
    it({
      name: 'READS INLINE AND BLOCK MATH AS NOT PROSE',
      fn: async () => {
        expect({
          inline: quoteFlags({ text: 'Let $f\'(x)$ be the slope, it\'s fine.', },),
          block: quoteFlags({ text: 'Let\n\n$$\nf\'(x) = "a"\n$$\n\nIt\'s done.', },),
        },).toEqual({
          inline: 'NP',
          block: 'NNNP',
        },);
      },
    },),
    it({
      name: 'READS AN EXPRESSION, A TAG AND EVERY ATTRIBUTE OF IT AS NOT PROSE, an attribute after a greater-than inside an '
        + 'expression included, while text inside the element is prose',
      fn: async () => {
        expect({
          flowExpression: quoteFlags({ text: 'Value {"a"} here and it\'s fine.', },),
          singleExpression: quoteFlags({ text: 'Value {\'a\'} here and it\'s fine.', },),
          template: quoteFlags({ text: 'Value {items.map((x) => "a" + "b")} here and it\'s fine.', },),
          greaterThan: quoteFlags({ text: 'See <Photo ok={a > b} alt="d" title="x" /> and it\'s here.', },),
          arrayExpression: quoteFlags({ text: 'See <Photo photos={["a", "b"]} alt="d" /> and it\'s here.', },),
          attributes: quoteFlags({ text: '<a href="x" title="The dogs">it\'s</a>', },),
          children: quoteFlags({ text: '<Note>It\'s here</Note> and it\'s there.', },),
        },).toEqual({
          flowExpression: 'NNP',
          singleExpression: 'NNP',
          template: 'NNNNP',
          greaterThan: 'NNNNP',
          arrayExpression: 'NNNNNNP',
          attributes: 'NNNNP',
          children: 'PP',
        },);
      },
    },),
    it({
      name: 'READS AN HTML COMMENT, A LONE CONTAINER TAG LINE AND FRONT MATTER AS NOT PROSE, with the prose beside each prose',
      fn: async () => {
        expect({
          comment: quoteFlags({ text: 'Note <!-- it\'s "here" --> and it\'s ok.', },),
          container: quoteFlags({ text: '<Note title="it\'s">\n\nIt\'s here.\n\n</Note>', },),
          frontMatter: quoteFlags({ text: '---\ntitle: it\'s\n---\n\nBody it\'s.', },),
        },).toEqual({
          comment: 'NNNP',
          container: 'NNNP',
          frontMatter: 'NP',
        },);
      },
    },),
    it({
      name: 'READS HEADINGS, TABLE CELLS, BLOCKQUOTES, LIST ITEMS, EMPHASIS AND FOOTNOTE TEXT AS PROSE',
      fn: async () => {
        expect({
          heading: quoteFlags({ text: '## It\'s here', },),
          table: quoteFlags({ text: '| it\'s | b |\n|---|---|\n| "x" | y |', },),
          blockquote: quoteFlags({ text: '> it\'s here\n> and "there"', },),
          list: quoteFlags({ text: '- it\'s here\n- "there"', },),
          emphasis: quoteFlags({ text: 'A *it\'s* and **"there"**.', },),
          footnote: quoteFlags({ text: 'Note[^1].\n\n[^1]: It\'s here.', },),
        },).toEqual({
          heading: 'P',
          table: 'PPP',
          blockquote: 'PPP',
          list: 'PPP',
          emphasis: 'PPP',
          footnote: 'P',
        },);
      },
    },),
    it({
      name: 'READS A TEXT THE PAGE PARSE REFUSES AS NOT PROSE AT ALL, since no parse shows what is prose there: an unclosed '
        + 'brace and a tag that never closes',
      fn: async () => {
        expect({
          brace: quoteFlags({ text: '{oops it\'s', },),
          tag: quoteFlags({ text: '<b>it\'s', },),
        },).toEqual({
          brace: 'N',
          tag: 'N',
        },);
      },
    },),
    it({
      name: 'HOLDS ONE FLAG PER UTF-16 UNIT, so a handle in astral letters keeps the mask aligned with the text',
      fn: async () => {
        /**
         Text with letters outside the Basic Multilingual Plane.
         */
        const text = '𝓠𝓾𝓪𝓷\'s nap and `𝓠\'s`.';
        /**
         Mask of the text.
         */
        const { mask, } = proseMaskReading({ text, },);
        expect({
          length: mask.length === text.length,
          flags: quoteFlags({ text, },),
        },).toEqual({
          length: true,
          flags: 'PN',
        },);
      },
    },),
    it({
      name: 'IS THE ONE SCANNER OF PROSE: every rule that reads quotes, ellipses or guillemets takes its mask from '
        + '`proseMaskReading`, and none keeps a scanner of its own for code or tags',
      fn: async () => {
        /**
         Whether each reader's source calls the shared mask.
         */
        const calls = await Promise.all(READERS.map(async function callsMask(reader,): Promise<string> {
          /**
           Source of the reader.
           */
          const [directory = '', ...underSrc] = reader.split('/',);
          const source = await readFile(
            new URL(
              `../../${directory}/src/${underSrc.join('/',)}`,
              import.meta.url,
            ),
            'utf8',
          );
          return `${reader}:${source.includes('proseMaskReading({',)}`;
        },),);
        expect(calls,).toEqual(READERS.map(function expected(reader,): string {
          return `${reader}:true`;
        },),);
      },
    },),
  ],
},);
