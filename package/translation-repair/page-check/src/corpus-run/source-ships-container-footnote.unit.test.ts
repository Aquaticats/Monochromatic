/**
 Tests that a pair of runs holding a code span across a footnote definition line inside a block quote or a list item
 ships its original and holds no live construct in it.

 THE STRICT GRAMMAR STARTS THE FOOTNOTE DEFINITION AT THE LINE, behind the mark of a quote, at the content column of a
 list item, or after up to three spaces on a lazy line, and the span the plain reading keeps across the line ends
 there. The page then holds what follows the colon as text of the definition: a tag in it must be written as text so
 that the page parses (a run that does not parse is dropped), and a brace in it must be written as text so that it is
 no expression. The composition reads the pair as the sampled and the full checks do (`readPair`), which reports a
 page that ships without an original though no page was refused for its depth, and a construct found in a run written
 as literal text. The text of every shape is in `source-live-container-footnotes.test-fixture.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { containerFootnoteTexts, } from './source-live-container-footnotes.test-fixture.ts';
import { readPair, } from './source-live-pairs.test-fixture.ts';
import type { RunPair, } from './source-live-sample.test-fixture.ts';

/**
 A blank line and a block the strict grammar refuses, which follow a run and make it fail alone, so that the
 composition writes the run as literal text.
 */
const REFUSED_BLOCK = '\n<b>\n';

/**
 English between the two runs.
 */
const MIDDLE = 'She slept on the counter.\n';

/**
 A run that parses alone, beside the first.
 */
const SECOND_RUN = 'ok\n';

/**
 Pairs of runs the sampled check found once the pool held these shapes, each with what it stands for: the lines of a
 paragraph in a quote cut at a footnote line that carries the marks, then at a lazy one, then at one that carries them
 again; a heading made of a code span across a footnote line; and a page with a line of invisible characters, which the
 package reads as blank and a parse of the raw page does not.
 */
const FOUND_PAIRS: readonly (readonly [string, RunPair])[] = [
  [
    'A PARAGRAPH IN A BLOCK QUOTE CUT AT A FOOTNOTE LINE WITH THE MARK, THEN AT A LAZY ONE, THEN AT ONE WITH THE MARK',
    {
      first: '> a `x\n>  [^1]: {c}` {d}\n> a `x\n  [^1]: {c}` {d}\n> a `x\n>  [^1]: <b>` {d}\n',
      middle: MIDDLE,
      second: SECOND_RUN,
    },
  ],
  [
    'A SETEXT HEADING WHOSE CODE SPAN RUNS ACROSS A FOOTNOTE LINE INDENTED TWO SPACES',
    {
      first: 'a `x\n  [^1]: <b>` {d}\nabc\n===\n',
      middle: MIDDLE,
      second: SECOND_RUN,
    },
  ],
  [
    'A SETEXT HEADING WHOSE CODE SPAN RUNS ACROSS A FOOTNOTE LINE AT THE FIRST COLUMN, A BRACE AFTER THE COLON',
    {
      first: 'a `x\n[^1]: {c}` {d}\nabc\n---\n\n<b>\n',
      middle: MIDDLE,
      second: SECOND_RUN,
    },
  ],
  [
    'AN ES MODULE LINE, A LINE OF A BYTE ORDER MARK ONLY, AND A BRACE, BESIDE A BRACE',
    {
      first: 'import a from "b"\n\uFEFF\n\\{\n',
      middle: MIDDLE,
      second: '{\n',
    },
  ],
  [
    'A BARE ADDRESS ENDED BY A TAG, BESIDE A BRACE, A LINE OF A BYTE ORDER MARK ONLY AND A DEFINITION WITH A BRACE',
    {
      first: 'www.example.net/a<b>\n',
      middle: MIDDLE,
      second: '}\n\uFEFF\n[r]: u{b "t {"\n',
    },
  ],
];

await describe({
  name: 'a code span across a footnote definition line inside a container, in a pair of runs',
  children: [
    ...containerFootnoteTexts({ body: '<b>', },).map(function tagCase([name, text,],) {
      return it({
        name: `WRITES ${name.toUpperCase()} AS LITERAL TEXT and keeps the original, so the page is not shipped without it`,
        fn: async () => {
          /**
           What the composition made of the pair.
           */
          const { step, finding, } = readPair({
            pair: {
              first: text,
              middle: MIDDLE,
              second: SECOND_RUN,
            },
          },);
          expect(finding.found,).toEqual([],);
          expect(step,).toBe('encoded-failing-runs',);
        },
      },);
    },),

    ...FOUND_PAIRS.map(function foundCase([name, pair,],) {
      return it({
        name: `SHIPS NO LIVE CONSTRUCT AND KEEPS EVERY ORIGINAL FOR ${name}`,
        fn: async () => {
          /**
           What the composition made of the pair.
           */
          const { finding, } = readPair({ pair, },);
          expect(finding.found,).toEqual([],);
        },
      },);
    },),

    ...containerFootnoteTexts({ body: '{c}', },).map(function braceCase([name, text,],) {
      return it({
        name: `LEAVES NO LIVE EXPRESSION IN THE LITERAL TEXT WRITTEN FOR ${name.toUpperCase()}, beside a block the `
          + 'grammar refuses',
        fn: async () => {
          /**
           What the composition made of the pair.
           */
          const { step, finding, } = readPair({
            pair: {
              first: `${text}${REFUSED_BLOCK}`,
              middle: MIDDLE,
              second: SECOND_RUN,
            },
          },);
          expect(finding.found,).toEqual([],);
          expect(step,).toBe('encoded-failing-runs',);
        },
      },);
    },),
  ],
},);
