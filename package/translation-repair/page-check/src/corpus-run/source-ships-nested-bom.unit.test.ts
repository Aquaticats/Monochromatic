/**
 Tests that a block nested in a footnote definition that opens with a byte order mark is shown as the source shows it.

 The nested reading parses the characters of a footnote's blocks as a text of their own, and a parser drops a byte order
 mark that opens its input: every offset of the parse then stands one character off. On the page the mark is an ordinary
 character in the middle of the text, so the nested text is given to the parse with a letter in its place.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { describe, } from '@monochromatic-dev/test-harness-core/ts';

import { readerEqualityCases, } from './source-reader-equality.test-fixture.ts';

/**
 Texts a reader must see as the source shows them: what the case is, and the text.
 */
const CASES: readonly (readonly [string, string])[] = [
  [
    'A FENCE THAT OPENS WITH A BYTE ORDER MARK UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE, AFTER A BLANK LINE',
    '> [^1]: x\n>\n>     \uFEFF```\n>     {b}\n',
  ],
  [
    'A FENCE THAT OPENS WITH A BYTE ORDER MARK UNDER A FOOTNOTE DEFINITION IN A LIST ITEM, AFTER A BLANK LINE',
    '- [^1]: x\n\n      \uFEFF```\n      {b}\n',
  ],
  [
    'A FENCE THAT OPENS WITH A BYTE ORDER MARK UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE IN A BLOCK QUOTE',
    '> > [^1]: x\n> >\n> >     \uFEFF```\n> >     {b}\n',
  ],
  [
    'A CODE SPAN THAT OPENS WITH A BYTE ORDER MARK UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE, AFTER A BLANK LINE',
    '> [^1]: x\n>\n>     \uFEFF`y {b}` z\n',
  ],
  [
    'A FENCE THAT OPENS WITH A BYTE ORDER MARK UNDER A FOOTNOTE DEFINITION AT THE TOP, AFTER A BLANK LINE',
    '[^1]: x\n\n    \uFEFF```\n    {b}\n',
  ],
  [
    'A FOOTNOTE DEFINITION IN A BLOCK QUOTE WHOSE BODY OPENS WITH A BYTE ORDER MARK',
    '> [^1]: \uFEFFa {b}\n',
  ],
];

await describe({
  name: 'a block nested in a footnote definition that opens with a byte order mark',
  children: readerEqualityCases({ cases: CASES, },),
},);
