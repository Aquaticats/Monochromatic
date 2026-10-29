/**
 Tests for the text of a slice whose line endings are lone carriage returns, written as literal text for the strict MDX
 grammar.

 A LONE CARRIAGE RETURN IS A LINE ENDING TO THE GRAMMAR AND TO THE PLAIN PARSE, and to nothing the encoder reads lines
 with: its readers split on the line feed. The corpus reader folds Windows endings and leaves a lone carriage return
 alone (`slice-reading/src/line-endings.ts`), so these are the hardening cases for a text the corpus does not bring
 today: the encoder folds every lone carriage return to a line feed before it reads anything, so each text of the list
 is written as the same text with line feeds, and no page it makes holds an expression, an element or a module the
 original lacks, nor a second closing line after a flow the original closed.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parseMdxBody, } from '@monochromatic-dev/translation-repair-document-structure';

import {
  encodeAsLiteralMdx,
  leavesFlowOpen,
  readPageGrammar,
} from '../../dist/final/node/index.mjs';
import {
  liveConstructsOf,
  liveConstructsOfCarried,
} from './source-live-constructs.test-fixture.ts';
import {
  BEFORE,
  composePair,
  readPair,
} from './source-live-pairs.test-fixture.ts';

/**
 The byte order mark, written as an escape so no file holds the invisible character itself.
 */
const BOM = '\uFEFF';

/**
 Texts with line feeds whose lone carriage return form once leaked a live construct through the footnote separator, or
 swallowed the rest of the page, or was refused or shown as text; each is read as it stands and with every line feed
 written as a lone carriage return.
 */
const LINE_FEED_TEXTS: readonly (readonly [string, string])[] = [
  ['a footnote that is a tag alone, then a definition line', '[^1]: <b>\n[r]: u{b}\n'],
  ['a footnote, a tag alone on a line, then a definition line', '[^1]: x\n<a href="x">\n[r]: u{b}\n'],
  ['a footnote that is a division block', '[^a]: <div>\n\n    {a}\n\n    </div>\n'],
  [
    'a line of a byte order mark, then a footnote that is a tag alone and a definition',
    `x\n${BOM}\n[^1]: <b>\n[r]: u{c}\n`,
  ],
  ['a footnote, a self-closing tag alone, then a definition', '[^1]: x\n<br/>\n[r]: u{b}\n'],
  ['a footnote, a tag alone, then two definitions', '[^1]: x\n<i>\n[r]: u{b}\n[s]: v{c}\n'],
  [
    'two footnotes, each a tag alone before a definition',
    '[^1]: x\n<a>\n[r]: u{b}\n\n[^2]: y\n<b>\n[s]: v{c}\n',
  ],
  ['a footnote that is a tag, then an indented line and a definition', '[^1]: <b>\n\t{a}\n[r]: u{b}\n'],
  ['a footnote started by a tab, that is a tag, then a definition', '[^1]:\t<b>\n[r]: u{c}\n'],
  ['a table, then a footnote that is a tag and a definition', '| a |\n|---|\n| b |\n[^1]: <c>\n[r]: u{d}\n'],
  [
    'a table with a reference, a footnote, a tag alone, then a definition',
    '| a |\n|---|\n| b[^1] |\n\n[^1]: x\n<a href="x">\n[r]: u{d}\n',
  ],
  ['a heading, then a footnote, a tag alone and a definition', '# h\n[^1]: x\n<a href="x">\n[r]: u{d}\n'],
  ['a fence mark indented by a tab after a blank line', 'x\n\n\t```\n'],
  [
    'a definition holding a brace, then a fence mark indented four spaces and a fence',
    '[r]: u{b}\n    ```\n    {\n    ```\n',
  ],
  ['a footnote and a line of equals signs, then a definition', '[^1]: x\n===\n[r]: u{b}\n'],
  ['a footnote, a tag alone, then a definition indented two spaces', '[^1]: x\n<a>\n  [r]: u{c}\n'],
  ['a footnote, a tag alone, then a definition indented three spaces', '[^1]: x\n<a>\n   [r]: u{c}\n'],
  [
    'a footnote, a tag alone, a definition, a paragraph and a second definition',
    '[^1]: x\n<a>\n[r]: u{c}\nz\n[s]: v{d}\n',
  ],
  ['a footnote, a tag alone, then a definition with a title across lines', '[^1]: x\n<a>\n[r]: u\n"t\n{c}"\n'],
  ['a footnote that starts inside a paragraph, is a tag and ends before a definition', 'p\n[^1]: <b>\n[r]: u{c}\n'],
  ['a footnote with a tag across lines', '[^1]: x\n\n    <b\n    a="{c}">\n'],
  ['inline math across lines', '$x\n{y$ {z}\n'],
  ['a definition with an angle target after a footnote and a tag alone', '[^1]: x\n<a href="x">\n[r]: <u{b}> "t {"\n'],
  ['a fence of a footnote that never closes', '[^1]: x\n\n    ```\n    {a}\n'],
  ['a fence of a footnote that closes', '[^1]: x\n\n    ```\n    {a}\n    ```\n'],
  ['display math of a footnote that never closes', '[^1]: x\n\n    $$\n    {a}\n'],
  ['a fence closed by a mark indented four spaces', '```\n{a}\n    ```\n'],
  ['display math that closes', '$$\n{a}\n$$\n'],
  ['a fence that closes', '```\n{a}\n```\n'],
  ['a tilde fence that closes', '~~~\n{a}\n~~~\n'],
  ['display math that closes, then a brace', '$$\n{a}\n$$\n{b}\n'],
  ['a fence that closes, then a brace', '```\n{a}\n```\n{b}\n'],
  ['display math that never closes', '$$\n{a}\n'],
  ['a fence that never closes', '```\n{a}\n'],
  ['a block quote holding display math that closes', '> $$\n> {a}\n> $$\n{b}\n'],
  ['a list item holding a fence that closes', '- ```\n  {a}\n  ```\n{b}\n'],
  ['a comment that closes, then a brace', '<!--\n-->\n{a}\n'],
  ['a comment that never closes, then a definition line', 'x\n\n<!--\n[r]: u{b "t {"\n'],
  ['indented code after a paragraph', 'a\n\n    {x}\n'],
  ['a list item with a lazy line', '- a\n{b}\n'],
  ['a module line after a paragraph and a setext heading', 'x\n\nimport q\n===\nexport z\n'],
];

/**
 Texts with line feeds, the first of which is built from the byte order mark, that stay as they are in every form.
 */
const UNCHANGED_TEXTS: readonly (readonly [string, string])[] = [
  ['Chinese prose with a footnote', '她[^1]说\n\n[^1]: 注\n'],
  ['one line with no ending', '她说'],
  ['a heading', '## 标题\n'],
];

/**
 Writes the lone carriage return form of a text: every line feed becomes a lone carriage return.

 @param text - text with line feeds

 @returns Text with a lone carriage return for each line feed

 @example
 ```ts
 const old = withCarriageReturns({ text: 'a\nb\n', },); // 'a\rb\r'
 ```
 */
function withCarriageReturns({ text, }: { readonly text: string; },): string {
  return text.replaceAll(
    '\n',
    '\r',
  );
}

/**
 Reads the top-level block types of the page a pair of runs makes, which show a flow that swallowed the rest of the
 page as a block of its own.

 @param first - source text of the first run

 @returns Types of the root's children, in order

 @example
 ```ts
 const types = topLevelTypesOf({ first: '$$\n{a}\n$$\n', },); // ['heading', 'paragraph', 'blockquote', 'math', ...]
 ```
 */
function topLevelTypesOf({ first, }: { readonly first: string; },): readonly string[] {
  /**
   The page of a pair whose first run is the text and whose second run is a brace.
   */
  const { composed, } = composePair({
    pair: {
      first,
      middle: 'She slept on the counter.\n',
      second: '{\n',
    },
  },);
  return parseMdxBody({ body: composed.pageText, },).children.map(function typeOf({ type, },): string {
    return type;
  },);
}

await describe({
  name: 'a lone carriage return as the line ending of a source text',
  children: [
    describe({
      name: 'what the encoder writes',
      children: [
        ...LINE_FEED_TEXTS.map(function sameAsLineFeeds([name, text,],) {
          return it({
            name: `WRITES THE TEXT WITH LONE CARRIAGE RETURNS AS IT WRITES THE TEXT WITH LINE FEEDS FOR ${
              name.toUpperCase()
            }`,
            fn: async () => {
              expect(encodeAsLiteralMdx({ text: withCarriageReturns({ text, },), },),)
                .toBe(encodeAsLiteralMdx({ text, },),);
            },
          },);
        },),

        ...UNCHANGED_TEXTS.map(function stays([name, text,],) {
          return it({
            name: `LEAVES ${name.toUpperCase()} AS IT IS in line feed form and in lone carriage return form`,
            fn: async () => {
              expect(encodeAsLiteralMdx({ text, },),).toBe(text,);
              expect(encodeAsLiteralMdx({ text: withCarriageReturns({ text, },), },),).toBe(text,);
            },
          },);
        },),

        it({
          name: 'FOLDS ONLY THE CARRIAGE RETURNS THAT ARE NOT PART OF A WINDOWS LINE ENDING: a text mixing the three '
            + 'endings keeps its CRLF and gets a line feed for each lone carriage return',
          fn: async () => {
            expect(encodeAsLiteralMdx({ text: 'a\r\nb\rc\n{d}', },),).toBe('a\r\nb\nc\n\\{d\\}',);
          },
        },),

        it({
          name: 'KEEPS A TEXT OF WINDOWS LINE ENDINGS AS IT WAS, a closing fence among them',
          fn: async () => {
            expect(encodeAsLiteralMdx({ text: '```\r\n{a}\r\n', },),).toBe('```\r\n{a}\r\n```',);
          },
        },),

        it({
          name: 'READS A FLOW CLOSED BY LONE CARRIAGE RETURNS AS CLOSED: display math and a fence get no second '
            + 'closing line, and the text leaves no flow open',
          fn: async () => {
            for (const text of ['$$\r{a}\r$$\r', '```\r{a}\r```\r', '~~~\r{a}\r~~~\r',]) {
              expect(encodeAsLiteralMdx({ text, },),).toBe(text.replaceAll(
                '\r',
                '\n',
              ),);
              expect(leavesFlowOpen({ text, },),).toBe(false,);
            }
          },
        },),

        it({
          name: 'READS A FLOW LEFT OPEN BY LONE CARRIAGE RETURNS AS OPEN, and closes it with a line feed',
          fn: async () => {
            expect(encodeAsLiteralMdx({ text: '$$\r{a}\r', },),).toBe('$$\n{a}\n$$',);
            expect(leavesFlowOpen({ text: '```\r{a}\r', },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'what the page holds',
      children: [
        ...LINE_FEED_TEXTS.map(function holdsNoCode([name, text,],) {
          return it({
            name: `PARSES AND HOLDS NO EXPRESSION, ELEMENT OR MODULE FOR ${name.toUpperCase()}, written with lone `
              + 'carriage returns',
            fn: async () => {
              /**
               The text with lone carriage returns, as the encoder writes it.
               */
              const written = encodeAsLiteralMdx({ text: withCarriageReturns({ text, },), },);
              expect(readPageGrammar({ pageText: `${BEFORE}${written}\n`, },),).toEqual({ kind: 'parses', },);
              expect(liveConstructsOfCarried({ carried: written, },),).toEqual([],);
            },
          },);
        },),

        it({
          name: 'MAKES THE SAME BLOCKS OF A PAGE WHOSE FIRST RUN CLOSES ITS FLOW WITH LONE CARRIAGE RETURNS AS OF ONE '
            + 'WITH LINE FEEDS: no second flow swallows the English and the second run',
          fn: async () => {
            for (const text of ['$$\n{a}\n$$\n', '```\n{a}\n```\n', '~~~\n{a}\n~~~\n',]) {
              /**
               The blocks of the page with line feeds, which end in the block of the flow and no second one.
               */
              const lineFeeds = topLevelTypesOf({ first: text, },);
              expect(topLevelTypesOf({ first: withCarriageReturns({ text, },), },),).toEqual(lineFeeds,);
              expect(lineFeeds.filter(function isFlow(type,): boolean {
                return (type === 'math') || (type === 'code');
              },).length,).toBe(1,);
            }
          },
        },),

        it({
          name: 'SHIPS NO LIVE CONSTRUCT FOR A PAIR OF RUNS WRITTEN WITH LONE CARRIAGE RETURNS, a run first, second '
            + 'and twice, so the composition of the page reads them as it reads line feeds',
          fn: async () => {
            for (const [, text,] of LINE_FEED_TEXTS) {
              /**
               The text with lone carriage returns.
               */
              const carriageReturns = withCarriageReturns({ text, },);
              for (
                const pair of [
                  {
                    first: carriageReturns,
                    middle: 'She slept on the counter.\n',
                    second: '{\n',
                  },
                  {
                    first: '{\n',
                    middle: 'She slept on the counter.\n',
                    second: carriageReturns,
                  },
                  {
                    first: carriageReturns,
                    middle: 'She slept on the counter.\n',
                    second: carriageReturns,
                  },
                ]
              ) {
                expect(readPair({ pair, },).finding.found,).toEqual([],);
              }
            }
          },
        },),

        it({
          name: 'WRITES THE FIRST RUN AS LITERAL TEXT WHEN IT HOLDS A FENCE MARK INDENTED BY A TAB AND LONE CARRIAGE '
            + 'RETURNS, the way it does with line feeds: the mark opens a fence in the grammar that swallows the '
            + 'English and closes at the fence of the second run, which is written as literal text for the comment '
            + 'it never closes and would then leave its brace an expression',
          fn: async () => {
            for (
              const [first, second,] of [
                ['x\n\n\t```\n', '```\n{a}\n```\n\n<!--\n',],
                ['x\r\r\t```\r', '```\r{a}\r```\r\r<!--\r',],
                ['x\r\r    ```\r', '```\r{a}\r```\r\r<!--\r',],
              ] as const
            ) {
              expect(readPair({
                pair: {
                  first,
                  middle: 'She slept on the counter.\n',
                  second,
                },
              },).finding.found,).toEqual([],);
            }
          },
        },),

        it({
          name: 'HOLDS NO EXPRESSION FOR THE PAGE OF A FOOTNOTE THAT IS A TAG ALONE AND A DEFINITION LINE, in lone '
            + 'carriage return form, read whole and not only in the slice',
          fn: async () => {
            /**
             The page of the text after the note.
             */
            const pageText = `${BEFORE}${encodeAsLiteralMdx({ text: '[^1]: <b>\r[r]: u{b}\r', },)}\n`;
            expect(liveConstructsOf({ pageText, },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
