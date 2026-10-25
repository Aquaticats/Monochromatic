/**
 Tests that the code a cut span keeps still ends with a closed comment's last character.

 The marks of a block quote between the code of a cut span and the opener that cuts it are written after the closing
 delimiter, and the code keeps what stood before them. A closed comment at the end of that code ends with a character
 that is also the mark of a block quote, which the code keeps.

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
    'A CODE SPAN WITH A CLOSED COMMENT BEFORE AN OPENER THAT NEVER CLOSES',
    'a `x\n<!-- a --> <!-- never` z {d}\n',
  ],
  [
    'A CODE SPAN WITH A CLOSED COMMENT BEFORE AN OPENER, THE COMMENT INDENTED',
    'a `x\n  <!-- a --> <!-- never` z {d}\n',
  ],
  [
    'INLINE MATH WITH A CLOSED COMMENT BEFORE AN OPENER THAT NEVER CLOSES',
    'a $x\n<!-- a --> <!-- never$ z {d}\n',
  ],
  [
    'A CODE SPAN WITH A CLOSED COMMENT BEFORE AN OPENER IN A BLOCK QUOTE',
    '> a `x\n> <!-- a --> <!-- never` z {d}\n',
  ],
  [
    'A CODE SPAN WITH A CLOSED COMMENT BEFORE AN OPENER IN A LIST ITEM',
    '- a `x\n  <!-- a --> <!-- never` z {d}\n',
  ],
  [
    'A CODE SPAN WITH A MARK OF A BLOCK QUOTE BEFORE AN OPENER, THE CODE ENDING WITH AN ANGLE BRACKET OF ITS OWN',
    '> a `x>\n> <!-- never` z {d}\n',
  ],
];

await describe({
  name: 'the code a cut span keeps before an opener',
  children: readerEqualityCases({ cases: CASES, },),
},);
