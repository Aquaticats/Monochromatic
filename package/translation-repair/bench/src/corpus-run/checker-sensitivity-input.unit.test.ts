/**
 Tests for the invented inputs of the checker sensitivity runner: what each
 sheet holds must be what its expectation says about it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ABSENT_ISSUE,
  ALL_FIXED_PATCHED_TEXT,
  ALL_FIXED_SOURCE_TEXT,
  CHECKER_SOURCE_TEXT,
  DEFECTIVE_TEXT,
  MEANING_ISSUE,
  MIXED_SHEET_PATCHED_TEXT,
  SINGLE_ISSUE_CASES,
  TENSE_ISSUE,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'checker sensitivity inputs',
  children: [
    it({
      name: 'NAMES the three accepted issues apart, tense then meaning then absent, in the order the sheets list them',
      fn: async () => {
        expect([
          TENSE_ISSUE,
          MEANING_ISSUE,
          ABSENT_ISSUE,
        ].map(function idOf(issue,) {
          return issue.issueId;
        },),).toEqual([
          'adjudicated/tense',
          'adjudicated/meaning',
          'adjudicated/absent',
        ],);
      },
    },),

    it({
      name: 'MAKES the untouched case the defective text itself and no other case that text',
      fn: async () => {
        expect(SINGLE_ISSUE_CASES.filter(function isDefective(check,): boolean {
          return check.patchedText === DEFECTIVE_TEXT;
        },).map(function labelOf(check,) {
          return check.label;
        },),).toEqual(['untouched',],);
      },
    },),

    it({
      name: 'GIVES the all-fixed sheet an original that adds the dog clause to the shared original, and a candidate that mentions the dog',
      fn: async () => {
        expect([
          ALL_FIXED_SOURCE_TEXT.startsWith(CHECKER_SOURCE_TEXT,),
          ALL_FIXED_SOURCE_TEXT.length > CHECKER_SOURCE_TEXT.length,
          ALL_FIXED_PATCHED_TEXT.includes('dog',),
          MIXED_SHEET_PATCHED_TEXT.includes('dog',),
        ],).toEqual([
          true,
          true,
          true,
          false,
        ],);
      },
    },),
  ],
},);
