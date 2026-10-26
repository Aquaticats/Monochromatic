/**
 Tests that a code span across a footnote definition line indented four spaces or more leaves no live construct and no
 refused page.

 The strict grammar disables indented code, so a footnote definition may start at any indent there, where CommonMark
 reads a line indented four spaces as a lazy line of the paragraph above it. The code span the original pairs across
 the two lines is cut at the definition on the page, and what the encoder left in it unescaped was then read as syntax.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { readText, } from './source-live-pairs.test-fixture.ts';

/**
 Texts with a code span across a footnote definition line: what the case is, and the text.
 */
const SPAN_CASES: readonly (readonly [string, string])[] = [
  [
    'A TAG AFTER THE LABEL, INDENTED FOUR SPACES AT THE TOP',
    'a `x\n    [^1]: <b>` {d}\n',
  ],
  [
    'A BRACE AFTER THE LABEL, INDENTED FOUR SPACES AT THE TOP',
    'a `x\n    [^1]: {c}` {d}\n',
  ],
  [
    'A TAG AFTER THE LABEL, INDENTED FOUR SPACES ON A LAZY LINE OF A BLOCK QUOTE',
    '> a `x\n    [^1]: <b>` {d}\n',
  ],
  [
    'A BRACE AFTER THE LABEL, INDENTED FOUR SPACES ON A LAZY LINE OF A BLOCK QUOTE',
    '> a `x\n    [^1]: {c}` {d}\n',
  ],
  [
    'A BRACE AFTER THE LABEL, INDENTED SIX SPACES AT THE TOP',
    'a `x\n      [^1]: {c}` {d}\n',
  ],
  [
    'A BRACE AFTER THE LABEL, INDENTED BY A TAB AT THE TOP',
    'a `x\n\t[^1]: {c}` {d}\n',
  ],
  [
    'A BRACE AFTER THE LABEL, INDENTED FOUR SPACES AFTER THE MARKS OF A BLOCK QUOTE',
    '> a `x\n>     [^1]: {c}` {d}\n',
  ],
  [
    'A BRACE AFTER THE LABEL, INDENTED FOUR SPACES UNDER THE TEXT OF A LIST ITEM',
    '- a `x\n      [^1]: {c}` {d}\n',
  ],
];

await describe({
  name: 'a code span across a footnote definition line indented four spaces or more',
  children: SPAN_CASES.map(function spanCase([name, text,],) {
    return it({
      name: `PARSES WITH NO LIVE CONSTRUCT FOR ${name}`,
      fn: async () => {
        /**
         What the page grammar says of the text once written, and what it holds.
         */
        const { parses, finding, } = readText({ text, },);
        expect(parses,).toBe(true,);
        expect(finding.found,).toEqual([],);
      },
    },);
  },),
},);
