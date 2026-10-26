/**
 Tests that a comment opener that starts a block of raw HTML to a reader of the source, in a block quote or a list item or
 after text that holds another opener, is written so that the page shows the source.

 An opener of a comment that never closes, first on a line after the marks of its containers and at most three spaces, is
 the start of a block of raw HTML that runs to the end of the container (to the end of the text at the top), so a backtick
 or a dollar sign before it pairs with nothing after it, and a list marker, a heading mark or a brace in the lines of the
 block is text. The page carries the opener as a character reference and has no raw HTML, so the encoder cuts the
 paragraph before the opener and writes the lines of the block as the text of one. It once knew only the first opener of a
 text, and only when nothing but spaces stood before it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { describe, } from '@monochromatic-dev/module-test/ts';

import { readerEqualityCases, } from './source-reader-equality.test-fixture.ts';

/**
 Texts a reader must see as the source shows them: what the case is, and the text.
 */
const CASES: readonly (readonly [string, string])[] = [
  [
    'AN OPENER FIRST ON A LINE OF A BLOCK QUOTE AFTER A PARAGRAPH THAT HOLDS A BACKTICK',
    '> a `x\n> <!-- never` z {d}\n',
  ],
  [
    'AN OPENER FIRST ON A LINE OF A BLOCK QUOTE AFTER A PARAGRAPH THAT HOLDS A DOLLAR SIGN',
    '> a $x\n> <!-- never$ z {d}\n',
  ],
  [
    'AN OPENER FIRST ON A LINE OF A BLOCK QUOTE, THEN A LIST MARKER',
    '> a\n> <!-- c\n> - j\n> ',
  ],
  [
    'AN OPENER FIRST ON A LINE OF A BLOCK QUOTE, THEN A HEADING MARK AND A BRACE',
    '> a\n> <!-- c\n> # h\n> {b}\n',
  ],
  [
    'AN OPENER STARTING A BLOCK QUOTE, THEN A LIST MARKER',
    '> <!-- c\n> - j\n',
  ],
  [
    'AN OPENER STARTING A SECOND BLOCK QUOTE, THEN A LIST MARKER',
    '> a\n\n> <!-- c\n> - j\n',
  ],
  [
    'AN OPENER FIRST ON A LINE OF A BLOCK QUOTE IN A BLOCK QUOTE',
    '> > a `x\n> > <!-- never` z {d}\n',
  ],
  [
    'AN OPENER FIRST ON A LINE OF A LIST ITEM IN A BLOCK QUOTE',
    '> - a `x\n>   <!-- never` z {d}\n',
  ],
  [
    'AN OPENER FIRST ON A LINE AFTER A LINE THAT HOLDS ANOTHER OPENER AFTER TEXT, THEN A LIST MARKER',
    'x <!-- 1\n\n<!-- 2\n- j\n',
  ],
  [
    'AN OPENER FIRST ON A LINE AFTER A LINE THAT HOLDS ANOTHER OPENER AFTER TEXT, THEN A BACKTICK',
    'x <!-- 1\na `y\n<!-- 2` z {d}\n',
  ],
  [
    'AN OPENER FIRST ON A LINE OF A LIST ITEM AFTER A PARAGRAPH THAT HOLDS A BACKTICK',
    '- a `x\n  <!-- never` z {d}\n',
  ],
  [
    'AN OPENER FIRST ON A LINE AT THE TOP AFTER A PARAGRAPH THAT HOLDS A BACKTICK',
    'a `x\n<!-- never` z {d}\n',
  ],
  [
    'AN OPENER AFTER TEXT ON ITS LINE IN A BLOCK QUOTE, TEXT TO BOTH READERS, THEN A LIST MARKER',
    '> a <!-- c\n> - j\n',
  ],
  [
    'AN OPENER FIRST ON A LINE OF A BLOCK QUOTE, THEN A LINE WITHOUT THE MARK AND A PARAGRAPH',
    '> a\n> <!-- c\n\ntail {z}\n',
  ],
];

await describe({
  name: 'a comment opener that starts a block of raw HTML in a container',
  children: readerEqualityCases({ cases: CASES, },),
},);
