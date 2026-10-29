/**
 Tests that the typography restoration touches only what the page parse shows as prose (n156, W1). A straight quote mark
 inside a link destination or title, a bare address, inline code of any backtick count, a fenced code block of any kind,
 math, an MDX expression, a tag or any attribute of it, an HTML comment or front matter stays as written; the same
 mark in prose beside each of those still curls on a curly page. An indented line is prose and an angle autolink is a text the
 MDX grammar refuses, which has no prose. Each protected input is the one the n155 review ran
 (`s224/probes/p-w-typo-masks.mjs`), so the table shows what the reader of the page would have seen changed.

 The ellipsis follows the same mask, and a text the parse refuses has no prose at all.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { restoreTypography, } from '../dist/final/node/index.mjs';

/**
 A page that writes curly apostrophes and curly double quotes.
 */
const CURLY_PAGE = 'The cat\u{2019}s \u{201C}bowl\u{201D} was full. The dogs\u{2019} bowl too.';

/**
 A page that writes the ellipsis as one character.
 */
const ELLIPSIS_PAGE = 'The cat\u{2019}s bowl\u{2026} was full.';

/**
 A text and the text the restoration writes for it on a curly page.
 */
type Row = {
  /**
   What the row is about.
   */
  readonly name: string;

  /**
   The replacement the editor wrote.
   */
  readonly input: string;
};

/**
 Inputs whose every straight mark stands outside prose, so the restoration writes them back as they came.
 */
const PROTECTED: readonly Row[] = [
  { name: 'a link destination with a mark between letters', input: 'See [Peter](https://en.wikipedia.org/wiki/Peter_O\'Toole) now.', },
  { name: 'a link destination ending in a mark', input: 'See [dogs](https://x.example/dogs\') now.', },
  { name: 'a link title in double marks', input: 'See [the dogs](https://x.example/d "The dogs") now.', },
  { name: 'a link title in single marks', input: 'See [the dogs](https://x.example/d \'The dogs\') now.', },
  { name: 'an image title', input: 'See ![the dogs](https://x.example/d.png "The dogs") now.', },
  { name: 'a reference definition with a title', input: 'See [a][1] now.\n\n[1]: https://x.example/d "The dogs"\n', },
  { name: 'a reference definition address', input: 'See [a][1] now.\n\n[1]: https://x.example/O\'Toole\n', },
  { name: 'a bare address', input: 'Go to https://x.example/O\'Toole now.', },
  { name: 'an angle destination', input: '[a](<https://x.example/it\'s me>)', },
  { name: 'inline code in one backtick', input: 'Run `it\'s` now.', },
  { name: 'inline code in two backticks holding a backtick', input: 'Run ``a`b it\'s`` now.', },
  { name: 'inline code in two backticks', input: 'Run ``it\'s`` now.', },
  { name: 'a backtick fence', input: 'Run:\n\n```js\nconsole.log("hi", \'it\'s\');\n```\n\nDone.', },
  { name: 'a tilde fence', input: 'Run:\n\n~~~js\nconsole.log("hi");\n~~~\n\nDone.', },
  { name: 'a four-backtick fence around a three-backtick one', input: 'Run:\n\n````md\n```js\nlog("hi");\n```\n````\n\nDone.', },
  { name: 'a tag with double-quoted attributes', input: 'See <a href="x" title="The dogs">link</a> now.', },
  { name: 'a tag with single-quoted attributes', input: 'See <a href=\'x\' title=\'dogs\'>link</a> now.', },
  { name: 'a tag with an array expression and an attribute', input: 'See <Photo photos={["a", "b"]} alt="d" /> now.', },
  { name: 'a tag with a greater-than inside an expression', input: 'See <Photo ok={a > b} alt="d" title="x" /> now.', },
  { name: 'a text expression in double marks', input: 'Value {"a"} here.', },
  { name: 'a text expression in single marks', input: 'Value {\'a\'} here.', },
  { name: 'a text expression holding a function', input: 'Value {items.map((x) => "a" + "b")} here.', },
  { name: 'an expression comment', input: 'Note {/* it\'s here */} ok.', },
  { name: 'inline math', input: 'Let $f\'(x)$ be the slope.', },
  { name: 'block math', input: 'Let\n\n$$\nf\'(x) = "a"\n$$\n\nbe it.', },
  { name: 'an HTML comment', input: 'Note <!-- it\'s "here" --> ok.', },
  { name: 'front matter', input: '---\ntitle: it\'s\n---\n\nBody.', },
];

/**
 Inputs whose marks stand in prose, with the text written for them: the controls that show the restoration still curls.
 */
const PROSE: readonly (Row & { readonly expected: string; })[] = [
  { name: 'a sentence', input: 'The dog\'s bowl.', expected: 'The dog\u{2019}s bowl.', },
  { name: 'a footnote definition', input: 'Note[^1].\n\n[^1]: The dogs\' bowl "x".', expected: 'Note[^1].\n\n[^1]: The dogs\u{2019} bowl \u{201C}x.\u{201D}', },
  { name: 'a heading', input: '## The dog\'s bowl', expected: '## The dog\u{2019}s bowl', },
  { name: 'a table', input: '| a | b |\n|---|---|\n| it\'s | "x" |', expected: '| a | b |\n|---|---|\n| it\u{2019}s | \u{201C}x\u{201D} |', },
  { name: 'a link label', input: 'See [Peter\'s page](https://x.example/p) now.', expected: 'See [Peter\u{2019}s page](https://x.example/p) now.', },
  { name: 'a list item', input: '- it\'s here\n- and there', expected: '- it\u{2019}s here\n- and there', },
  { name: 'a quotation', input: '> it\'s here\n> and there', expected: '> it\u{2019}s here\n> and there', },
  { name: 'emphasis', input: 'A *it\'s* nap.', expected: 'A *it\u{2019}s* nap.', },
  { name: 'the text of an element', input: '<Note>It\'s here</Note> and there.', expected: '<Note>It\u{2019}s here</Note> and there.', },
  {
    name: 'an indented line, which the MDX grammar reads as a paragraph and not as code',
    input: 'Run:\n\n    console.log("hi");\n\nDone.',
    expected: 'Run:\n\n    console.log(\u{201C}hi\u{201D});\n\nDone.',
  },
];

/**
 Restores a replacement on a curly page.

 @param replacement - text the editor wrote

 @returns Restored replacement

 @example
 ```ts
 curled({ replacement: "didn't", },);
 ```
 */
function curled({ replacement, }: { readonly replacement: string; },): string {
  return restoreTypography({
    replacement,
    replaced: '',
    convention: CURLY_PAGE,
  },);
}

await describe({
  name: 'restoreTypography reads only prose (n156, W1)',
  children: [
    ...PROTECTED.map(function protectedCase(row,) {
      return it({
        name: `LEAVES EVERY MARK OF ${row.name} AS WRITTEN on a curly page, since it is no prose text`,
        fn: async () => {
          expect(curled({ replacement: row.input, },),).toBe(row.input,);
        },
      },);
    },),
    ...PROTECTED.map(function besideCase(row,) {
      return it({
        name: `STILL CURLS THE PROSE BESIDE ${row.name}, a paragraph after it that holds an apostrophe`,
        fn: async () => {
          expect(curled({ replacement: `${row.input}\n\nIt's here.`, },),).toBe(`${row.input}\n\nIt\u{2019}s here.`,);
        },
      },);
    },),
    ...PROSE.map(function proseCase(row,) {
      return it({
        name: `STILL CURLS THE MARKS OF ${row.name}`,
        fn: async () => {
          expect(curled({ replacement: row.input, },),).toBe(row.expected,);
        },
      },);
    },),
    it({
      name: 'LEAVES A TEXT THE PAGE PARSE REFUSES AS WRITTEN, an unclosed brace, a tag that never closes and an angle autolink '
        + '(MDX reads `<https:` as a tag), since no parse shows what is prose there',
      fn: async () => {
        expect({
          brace: curled({ replacement: '{oops it\'s here', },),
          tag: curled({ replacement: '<b>it\'s here', },),
          angle: curled({ replacement: 'See <https://x.example/O\'Toole> now, it\'s here.', },),
        },).toEqual({
          brace: '{oops it\'s here',
          tag: '<b>it\'s here',
          angle: 'See <https://x.example/O\'Toole> now, it\'s here.',
        },);
      },
    },),
    it({
      name: 'WRITES THE ELLIPSIS ONLY IN PROSE: three dots in an address, in a fence and in inline code stay, while the three dots '
        + 'of a sentence become one character',
      fn: async () => {
        expect({
          address: restoreTypography({
            replacement: 'See [a](https://x.example/a...b) and wait...',
            replaced: '',
            convention: ELLIPSIS_PAGE,
          },),
          fence: restoreTypography({
            replacement: 'Run:\n\n~~~js\nlog(...args);\n~~~\n\nWait...',
            replaced: '',
            convention: ELLIPSIS_PAGE,
          },),
          code: restoreTypography({
            replacement: 'Run ``f(...)`` and wait...',
            replaced: '',
            convention: ELLIPSIS_PAGE,
          },),
        },).toEqual({
          address: 'See [a](https://x.example/a...b) and wait\u{2026}',
          fence: 'Run:\n\n~~~js\nlog(...args);\n~~~\n\nWait\u{2026}',
          code: 'Run ``f(...)`` and wait\u{2026}',
        },);
      },
    },),
  ],
},);
