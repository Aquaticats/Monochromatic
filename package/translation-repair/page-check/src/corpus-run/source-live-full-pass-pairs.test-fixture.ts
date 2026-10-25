/**
 Pairs of runs the full pass over the 40000 pairs of each of the two seeds found, after the sampled check passed on the
 widened pool, each with the shape it stands for. The sample keeps one in four of the draws and met none of them; the
 full pass met fifteen that shipped a live construct or dropped an original, and one that threw a depth error out of
 the reading of a footnote part.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import type { RunPair, } from './source-live-sample.test-fixture.ts';

/**
 A line of open brackets longer than the nesting bound, which the parser refuses a text for.
 */
const BRACKETS_PAST_BOUND = 300;

/**
 The pairs, each named by its shape, in the order the two seeds' passes listed them. The comment above each says which
 draw of which seed it is, so the full pass can be run again to see it.
 */
export const FULL_PASS_PAIRS: readonly (readonly [
  string,
  RunPair,
])[] = [
  // Seed 31415926, draw 4679.
  [
    'A FOOTNOTE AND A TAG ON LINES THAT END IN A LONE CARRIAGE RETURN, THEN A QUOTE WITH A CODE SPAN ACROSS A FOOTNOTE LINE',
    {
      first: '[^1]: <b>\r<Foo />\r> a `x\n   [^1]: <b>` {d}\n',
      middle: '--> She slept.\n',
      second: '<div>\n[https://example.net/a{ b](u)\n</div> \uFEFF\nimport q\n=== \uFEFF- ```\n',
    },
  ],
  // Seed 31415926, draw 4940.
  [
    'A FOOTNOTE AND A TAG ALONE, THEN A QUOTE WITH A CODE SPAN ACROSS A FOOTNOTE LINE, BESIDE A LABEL THAT IS A TAG',
    {
      first: '[^1]: <b>\r<q>\r> a `x\n>  [^1]: {c}` {d}\n',
      middle: 'She slept on the counter.\n',
      second: '[^<b>]: x\n\n[^<b>] {c}     ```\n    {\n    ``` <Foo />\n',
    },
  ],
  // Seed 31415926, draw 5109.
  [
    'A FOOTNOTE, BESIDE A QUOTE WITH A FOOTNOTE, A TAG AND A DEFINITION AND A FOOTNOTE WITH INDENTED CODE AND A DEFINITION',
    {
      first: '[^1]: 注\n',
      middle: 'import x from "y"\n',
      second: '> [^1]: x\n> <a href="x">\n> [r]: u{b}\n[^1]: x\n\n        c\n[r]: u{b}\n',
    },
  ],
  // Seed 31415926, draw 5482.
  [
    'A FOOTNOTE AND A DEFINITION WITH A BRACE, BESIDE A LIST ITEM WITH A CODE SPAN ACROSS A FOOTNOTE LINE AND A TAG',
    {
      first: '[^1]: <b>\n[r]: u{b}\nwww.example.net/a<b>\nimport data from the old system\n===\n',
      middle: 'import x from "y"\n',
      second: '- a `x\n [^1]: <b>` {d}\n<a href="x">\n`x\n    ```\n    y`\n',
    },
  ],
  // Seed 31415926, draw 9633.
  [
    'A QUOTE WHOSE CODE SPAN RUNS ACROSS A FOOTNOTE LINE AND WHOSE LAZY LINE HOLDS A FENCE MARK INDENTED FOUR SPACES',
    {
      first: '[<https://example.net/a>](u)\n> a `x\n>  [^1]: <b>` {d}\nabc\n    ```\n',
      middle: '--> She slept.\n',
      second: '- a `x\n   [^1]: <b>` {d}\r\r```\r{a}\r```\r\n',
    },
  ],
  // Seed 31415926, draw 11130.
  [
    'A QUOTE WITH A FOOTNOTE, A TAG AND A DEFINITION, THEN A FOOTNOTE WITH INDENTED CODE AND A DEFINITION',
    {
      first: '> [^1]: x\n> <a href="x">\n> [r]: u{b}\r[^1]: x\n\n        c\n[r]: u{b}\r[a {b][]\n',
      middle: '--> She slept.\n',
      second: '[r]: u{b}\n',
    },
  ],
  // Seed 31415926, draw 15625.
  [
    'A FOOTNOTE AND A TAG WITH AN ATTRIBUTE ON LINES THAT END IN A LONE CARRIAGE RETURN, THEN A QUOTE WITH A CODE SPAN',
    {
      first: '[^note]: <b>\r<a href="x">\r> a `x\n>  [^1]: {c}` {d}\n',
      middle: '[^1]: A note.\n',
      second: '$$\n    ```\n    {\n    ```\n',
    },
  ],
  // Seed 31415926, draw 20344.
  [
    'A FOOTNOTE, A DEFINITION, A SETEXT UNDERLINE AND A TAG WITH AN ATTRIBUTE, ON LINES THAT END IN A LONE CARRIAGE RETURN',
    {
      first: '[^1]: <b>\n[r]: u{b}\rexport to PDF\n---\r[^1]: x\r<a href="x">\r[r]: u{b}\r\n',
      middle: '[^1]: A note.\n',
      second: 'a_b*c\n',
    },
  ],
  // Seed 27182818, draw 7504.
  [
    'A FOOTNOTE, A TAG WITH AN ATTRIBUTE AND A DEFINITION WITH A BRACE',
    {
      first: '[^1]: x\n<a href="x">\n[r]: u{b}\n',
      middle: '[^1]: A note.\n',
      second: '[^1]: <b>\n[r]: u{b "t {"\n\uFEFF\nimport q\n===\nwww.example.net/a<b>\n',
    },
  ],
  // Seed 27182818, draw 21529.
  [
    'A FOOTNOTE AND A DEFINITION WITH A TITLE, BESIDE A FENCE WITH A LONG LINE OF BRACKETS IN ITS INFO STRING',
    {
      first: '-->\n[^1]: <b>\n[r]: u{b "t {"\n',
      middle: '[^1]: A note.\n',
      second: `~~~js {x} ${'['.repeat(BRACKETS_PAST_BOUND,)} <br>\n`,
    },
  ],
  // Seed 27182818, draw 23243.
  [
    'A LINE OF BRACKETS PAST THE BOUND, BESIDE A QUOTE WITH A FOOTNOTE, A TAG AND A DEFINITION',
    {
      first: `> \r\rhttps://example.net/a_b<c>\r\r${'['.repeat(BRACKETS_PAST_BOUND,)}\n`,
      middle: 'She slept on the counter.\n',
      second: '[^1]: <b>\n> [^1]: x\n> <a href="x">\n> [r]: u{b}\n[^1]: x\n\n        c\n[r]: u{b}\n',
    },
  ],
  // Seed 27182818, draw 27242.
  [
    'A QUOTE WITH A CODE SPAN ACROSS A FOOTNOTE LINE, A TAG ALONE AND A CODE SPAN WITH A BRACE, ON LINES THAT END IN A LONE CARRIAGE RETURN',
    {
      first: '> a `x\n>  [^1]: <b>` {d}\r<q>\r`{`\n',
      middle: '[^1]: A note.\n',
      second: '\\<\n',
    },
  ],
  // Seed 27182818, draw 30012.
  [
    'A QUOTE WITH A FOOTNOTE, A LONE CARRIAGE RETURN AND AN OPEN TAG, THEN A QUOTE WITH A CODE SPAN ACROSS A FOOTNOTE LINE',
    {
      first: '> [^1]: 注\n[b]: u\r</\r[^1]: x\n\n        c\n[r]: u{b}\n',
      middle: '--> She slept.\n',
      second: '> a `x\n   [^1]: <b>` {d}\r\rhttps://example.net/b{c\r\r</>\n',
    },
  ],
  // Seed 27182818, draw 33130.
  [
    'A QUOTE WITH A FOOTNOTE LINE AND A FENCE MARK INDENTED FOUR SPACES, ON LINES THAT END IN A LONE CARRIAGE RETURN',
    {
      first: '<!--\r\r\uFEFF```\n{\n```\n',
      middle: '--> She slept.\n',
      second: '> a `x\n>  [^1]: {c}` {d}\r> a\n>     ```\r    {y}\n',
    },
  ],
  // Seed 27182818, draw 35417.
  [
    'A DISPLAY OF MATH, THEN A QUOTE WITH A FOOTNOTE, A TAG AND A DEFINITION, AND A FOOTNOTE WITH INDENTED CODE',
    {
      first: '$a < b$\n',
      middle: '--> She slept.\n',
      second: '$a \\< b$\r> [^1]: x\n> <a href="x">\n> [r]: u{b}\r[^1]: x\n\n        c\n[r]: u{b}\n',
    },
  ],
  // Seed 27182818, draw 11096.
  [
    'A FENCE THAT LONE CARRIAGE RETURNS OPEN AND CLOSE, THEN A LINE OF BRACKETS PAST THE BOUND AFTER A FOOTNOTE LINE',
    {
      first: '<https://example.net/a>\r- a `x\n [^1]: <b>` {d}\n',
      middle: '[^1]: A note.\n',
      second: `~~~\r{a}\r~~~\r\n> a \`x\n>  [^1]: {c}\` {d}\n${'['.repeat(BRACKETS_PAST_BOUND,)}\n`,
    },
  ],
];
