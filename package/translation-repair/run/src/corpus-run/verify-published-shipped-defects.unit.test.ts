/**
 Tests for the lines `verify-published` prints for the defects a page ships with that its agreement with the artifact
 cannot show (B1231), without quoting the grammar's refusal, which names a place in corpus text: a page the grammar refuses, an archive kept where the grammar refused a wording, and a body
 without naturalness approval.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { shippedDefectLinesOf, } from '../../dist/final/node/index.mjs';

await describe({
  name: shippedDefectLinesOf.name,
  children: [
    it({
      name: 'SAYS NOTHING for a page that parses, whose composition parses, and a body every slice of which has approval',
      fn: async () => {
        expect(shippedDefectLinesOf({
          entryId: 'CatEntry1',
          naturalness: [],
          pageGrammar: { kind: 'parses', },
          composedGrammar: { kind: 'parses', },
          missing: [],
        },),).toEqual([],);
      },
    },),

    it({
      name: 'NAMES page-unparseable for a page the strict grammar refuses, with the refusal and what it costs, where the '
        + 'agreement check printed agreed for it',
      fn: async () => {
        expect(shippedDefectLinesOf({
          entryId: 'CatEntry1',
          naturalness: [],
          pageGrammar: { kind: 'refused', refusal: 'at 3:12 (mdx-jsx)', },
          composedGrammar: { kind: 'refused', refusal: 'at 3:12 (mdx-jsx)', },
          missing: [],
        },),).toEqual([
          'CatEntry1: page-unparseable, the strict grammar refuses the page on disk; the publisher ships such a page '
            + 'with this defect, and the site build will not take it',
        ],);
      },
    },),

    it({
      name: 'EXPLAINS THE MISSING SLICES as page-grammar-archive-kept when the page parses and the artifact\'s wordings '
        + 'composed as one page are refused, since the publisher keeps the archive at the slices that break it',
      fn: async () => {
        expect(shippedDefectLinesOf({
          entryId: 'CatEntry1',
          naturalness: [],
          pageGrammar: { kind: 'parses', },
          composedGrammar: { kind: 'refused', refusal: 'at 9:2 (mdx-jsx)', },
          missing: [4, 7,],
        },),).toEqual([
          'CatEntry1: page-grammar-archive-kept, the strict grammar refuses the artifact\'s wordings composed as one page, '
            + 'so the publisher keeps the archive at the slices that break it; the slices named MISSING, [4, 7], are '
            + 'among them',
        ],);
      },
    },),

    it({
      name: 'LEAVES THE MISSING SLICES UNEXPLAINED when the composition parses, since then the page lost them for '
        + 'another reason that the missing lines already report',
      fn: async () => {
        expect(shippedDefectLinesOf({
          entryId: 'CatEntry1',
          naturalness: [],
          pageGrammar: { kind: 'parses', },
          composedGrammar: { kind: 'parses', },
          missing: [4,],
        },),).toEqual([],);
      },
    },),

    it({
      name: 'NAMES final-naturalness with the slice the defect names, so a body shipped without approval is not '
        + 'invisible to the report',
      fn: async () => {
        expect(shippedDefectLinesOf({
          entryId: 'CatEntry1',
          naturalness: [{
            check: 'final-naturalness',
            message: 'entry CatEntry1 ships slice 5 without the approval of the naturalness review',
          },],
          pageGrammar: { kind: 'parses', },
          composedGrammar: { kind: 'parses', },
          missing: [],
        },),).toEqual([
          'CatEntry1: final-naturalness, entry CatEntry1 ships slice 5 without the approval of the naturalness review',
        ],);
      },
    },),
  ],
},);
