/**
 Tests for the page's definitions and for the readers that parse a slice with them (n159, X2 and X11). A reference or a footnote
 reference whose definition stands in another slice is plain text when the slice is parsed alone and a label when it is parsed
 with the page's definitions, which are collected the way the refinement gate collects them (`collectDefinitions`). The prose
 mask of a slice read with them says the label is no prose, so a rule that curls the reference curls the definition too, or
 neither.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pageDefinitionsOf, } from '@monochromatic-dev/translation-repair-shared-contract';
import { proseMaskReading, } from '../dist/final/node/index.mjs';
import { quoteFlags, } from './quote-flags.test-fixture.ts';

/**
 The definitions another slice of the page holds.
 */
const DEFINITIONS = '[dogs\' bowl]: https://x.example/d\n[b\'s]: https://x.example/b\n[^o\'k]: A note.';

/**
 A slice with a reference or a footnote reference, the flags of its marks read alone, and read with the page's definitions.
 */
type Row = {
  /**
   What the row is about.
   */
  readonly name: string;

  /**
   The slice.
   */
  readonly text: string;

  /**
   Flags of the marks when the slice is read alone.
   */
  readonly alone: string;

  /**
   Flags of the marks when the slice is read with the page's definitions.
   */
  readonly withDefinitions: string;
};

/**
 Slices whose label has a mark and whose definition stands in another slice.
 */
const LABELS: readonly Row[] = [
  {
    name: 'a full reference',
    text: 'See [the dogs][b\'s] and it\'s.',
    alone: 'PP',
    withDefinitions: 'NP',
  },
  {
    name: 'a shortcut reference',
    text: 'See [dogs\' bowl] and it\'s.',
    alone: 'PP',
    withDefinitions: 'NP',
  },
  {
    name: 'a footnote reference',
    text: 'See the note[^o\'k] and it\'s.',
    alone: 'PP',
    withDefinitions: 'NP',
  },
];

await describe({
  name: pageDefinitionsOf.name,
  children: [
    it({
      name: 'COLLECTS THE LINK AND FOOTNOTE DEFINITIONS OF A PAGE, one line per definition, as the refinement gate does',
      fn: async () => {
        expect(pageDefinitionsOf({
          pageText: 'See [a][x] and the note[^1].\n\n[x]: https://x.example/x\n\nThe cat naps.\n\n[^1]: A note.',
        },),).toBe('[x]: https://x.example/x\n[^1]: A note.',);
      },
    },),
    it({
      name: 'COLLECTS NOTHING FROM A PAGE THAT HOLDS NO DEFINITION',
      fn: async () => {
        expect(pageDefinitionsOf({ pageText: 'The cat naps.\n\nThe dogs\' bowl.', },),).toBe('',);
      },
    },),
    ...LABELS.map(function labelCase(row,) {
      return it({
        name: `READS A LABEL WHOSE DEFINITION IS IN ANOTHER SLICE AS NO PROSE ONCE THE PAGE'S DEFINITIONS ARE GIVEN: ${row.name}`,
        fn: async () => {
          expect({
            alone: quoteFlags({ text: row.text, },),
            withDefinitions: quoteFlags({
              text: row.text,
              definitions: DEFINITIONS,
            },),
          },).toEqual({
            alone: row.alone,
            withDefinitions: row.withDefinitions,
          },);
        },
      },);
    },),
    it({
      name: 'MAKES THE MASK OF A SLICE WITH DEFINITIONS AS LONG AS THE SLICE, so no unit of a definition is flagged beyond it',
      fn: async () => {
        /**
         The slice.
         */
        const text = 'See the note[^o\'k] and it\'s.';
        expect(proseMaskReading({
          text,
          definitions: DEFINITIONS,
        },).mask.length,).toBe(text.length,);
      },
    },),
    it({
      name: 'SAYS WHETHER THE GRAMMAR REFUSED THE TEXT: a brace that never closes is a refusal, a slice of code only is none',
      fn: async () => {
        expect({
          brace: proseMaskReading({ text: '{oops it\'s here', },).refused,
          plain: proseMaskReading({ text: 'It\'s here.', },).refused,
          code: proseMaskReading({ text: '`it\'s`', },).refused,
        },).toEqual({
          brace: true,
          plain: false,
          code: false,
        },);
      },
    },),
  ],
},);
