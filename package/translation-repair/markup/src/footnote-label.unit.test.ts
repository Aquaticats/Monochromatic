/**
 Tests for the label of a GFM footnote (`micromark-extension-gfm-footnote@2.1.0`, `lib/syntax.js:354-381`): each rule
 of `labelInside` and `labelEscape`, as the predicate and as the scan for the closing bracket answer it.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  footnoteLabelEnd,
  isGfmFootnoteLabel,
} from '../dist/final/node/index.mjs';

/**
 The most characters after the caret that GFM reads in a label.
 */
const MOST_READ = 999;

/**
 One label and whether GFM reads it, with the rule the case pins.
 */
const LABEL_CASES: readonly (readonly [string, string, boolean])[] = [
  ['ONE CHARACTER', '1', true,],
  ['LETTERS AND DIGITS', 'note2', true,],
  ['NOTHING', '', false,],
  ['A SPACE', 'a b', false,],
  ['A TAB', 'a\tb', false,],
  ['A LINE FEED', 'a\nb', false,],
  ['A CARRIAGE RETURN', 'a\rb', false,],
  ['A NO-BREAK SPACE, which GFM does not call white space', 'a\u00A0b', true,],
  ['AN IDEOGRAPHIC SPACE, which GFM does not call white space', 'a\u3000b', true,],
  ['AN OPENING BRACKET', 'a[b', false,],
  ['AN ESCAPED OPENING BRACKET', String.raw`a\[b`, true,],
  ['AN ESCAPED BACKSLASH', String.raw`a\\b`, true,],
  ['AN ESCAPED CLOSING BRACKET', String.raw`a\]b`, true,],
  ['A BACKSLASH BEFORE AN ORDINARY CHARACTER', String.raw`a\b`, true,],
  ['A BACKSLASH LAST, which escapes the closing bracket', 'a\\', false,],
  ['A BACKSLASH BEFORE A SPACE', String.raw`a\ b`, false,],
  ['A CLOSING BRACKET WITH NO ESCAPE', 'a]b', false,],
  ['999 CHARACTERS, the most GFM reads', 'a'.repeat(MOST_READ,), true,],
  ['1000 CHARACTERS', 'a'.repeat(MOST_READ + 1,), false,],
  ['998 CHARACTERS AND AN ESCAPED BRACKET, which count 1000', `${'a'.repeat(MOST_READ - 1,)}\\[`, false,],
  ['997 CHARACTERS AND AN ESCAPED BRACKET, which count 999', `${'a'.repeat(MOST_READ - 2,)}\\[`, true,],
];

/**
 One text, where its label starts, and where the closing bracket stands (-1 for none).
 */
const END_CASES: readonly (readonly [string, string, number, number])[] = [
  ['THE FIRST CLOSING BRACKET', 'the [^1] cat [^2]', 6, 7,],
  ['THE CLOSING BRACKET AFTER AN ESCAPED ONE', String.raw`the [^a\]b] cat`, 6, 10,],
  ['NO CLOSING BRACKET', 'the [^1 cat', 6, -1,],
  ['A LABEL BROKEN BY A LINE FEED BEFORE ITS CLOSING BRACKET', 'the [^a\nb] cat', 6, -1,],
  ['A LABEL OF NOTHING', 'the [^] cat', 6, -1,],
  ['A LABEL BROKEN BY AN OPENING BRACKET', 'the [^a[b] cat', 6, -1,],
  ['A LABEL TOO LONG, without reading to the end of the text', `[^${'a'.repeat(MOST_READ + 1,)}]`, 2, -1,],
  ['THE LAST ALLOWED CHARACTER', `[^${'a'.repeat(MOST_READ,)}]`, 2, MOST_READ + 2,],
];

await describe({
  name: 'footnote label',
  children: [
    describe({
      name: isGfmFootnoteLabel.name,
      children: LABEL_CASES.map(function labelCase([name, label, expected,],) {
        return it({
          name: `CALLS ${name} ${expected ? 'A' : 'NOT A'} FOOTNOTE LABEL`,
          fn: async () => {
            expect(isGfmFootnoteLabel({ label, },),).toBe(expected,);
          },
        },);
      },),
    },),

    describe({
      name: footnoteLabelEnd.name,
      children: END_CASES.map(function endCase([name, text, from, expected,],) {
        return it({
          name: `FINDS ${name}`,
          fn: async () => {
            expect(footnoteLabelEnd({
              text,
              from,
            },),).toBe(expected,);
          },
        },);
      },),
    },),
  ],
},);
