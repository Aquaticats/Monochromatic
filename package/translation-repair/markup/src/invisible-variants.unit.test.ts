/**
 Tests for folding invisible variants out of a model's text.

 WHAT THESE PIN: the non-breaking hyphen the reading found becomes a hyphen
 and is named; spaces and joiners fold to what a reader would type; visible
 typography the archive itself uses passes through untouched; and a text with
 nothing to fold comes back byte-identical with no finding.

 EVERY FIXTURE IS SPELLED AS AN ESCAPE. The first version of this file wrote
 the characters themselves, and the tool that wrote it dropped two of them,
 which is exactly the invisibility the fold exists to catch.

 Fixtures are a sentence about a bookshop cat, so there is no corpus text here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { foldInvisibleVariants, } from '../dist/final/node/index.mjs';

await describe({
  name: foldInvisibleVariants.name,
  children: [
    it({
      name: 'folds the non-breaking hyphen to a hyphen and names it with its count',
      fn: async () => {
        const folded = foldInvisibleVariants({ text: 'A non\u2011binary, part\u2011time shop cat.', },);

        expect(folded.text,).toBe('A non-binary, part-time shop cat.',);
        expect(folded.findings,).toStrictEqual(['invisible-variant-folded (U+2011 x2)',],);
      },
    },),

    it({
      name: 'folds the no-break spaces to spaces and drops the soft hyphen and the zero-width joiners',
      fn: async () => {
        const folded = foldInvisibleVariants({
          text: 'She\u00A0slept by\u202Fthe till\u00AD, every\u200B day\u2060 long\uFEFF.',
        },);

        expect(folded.text,).toBe('She slept by the till, every day long.',);
        expect(folded.findings,).toStrictEqual([
          'invisible-variant-folded (U+00A0 x1)',
          'invisible-variant-folded (U+202F x1)',
          'invisible-variant-folded (U+00AD x1)',
          'invisible-variant-folded (U+200B x1)',
          'invisible-variant-folded (U+2060 x1)',
          'invisible-variant-folded (U+FEFF x1)',
        ],);
      },
    },),

    it({
      name: 'passes typographic quotes, dashes, the ellipsis and the emoji joiner through untouched',
      fn: async () => {
        /**
         Text carrying only visible typography, which is the archive's own
         convention, plus a joined emoji.
         */
        const text = '\u201CShe\u2019s ours\u201D \u2014 the shop \u2013 said\u2026 \u{1F469}\u200D\u{1F4BB}';

        expect(foldInvisibleVariants({ text, },),).toStrictEqual({
          text,
          findings: [],
        },);
      },
    },),

    it({
      name: 'returns plain text byte-identical with no finding',
      fn: async () => {
        expect(foldInvisibleVariants({ text: 'A tabby who kept the shop.', },),).toStrictEqual({
          text: 'A tabby who kept the shop.',
          findings: [],
        },);
      },
    },),
  ],
},);
