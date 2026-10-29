/**
 Tests for the lines the checker sensitivity runner prints.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { IssueResolutionTally, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  CHECKER_NOTE,
  sheetCheckLine,
  SINGLE_ISSUE_CASES,
  singleCheckLine,
} from '../../dist/final/node/index.mjs';

/**
 Tally whose counts all differ, so a count printed under the wrong name shows.
 */
const COUNTED_TALLY: IssueResolutionTally = {
  fixed: 2,
  notFixed: 1,
  worse: 0.5,
  resolved: true,
  regressed: false,
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: singleCheckLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS the label, the expectation, the checkers heard and every count under its own name',
          fn: async () => {
            expect(singleCheckLine({
              label: 'untouched',
              expectation: 'not-fixed',
              heard: 3,
              tally: COUNTED_TALLY,
            },),).toBe(
              'CHECKER untouched expected=not-fixed heard=3 fixed=2 notFixed=1 worse=0.5 resolved=true regressed=false',
            );
          },
        },),

        it({
          name: 'PRINTS the opposite flags as false and true',
          fn: async () => {
            expect(singleCheckLine({
              label: 'fixed-but-damaged',
              expectation: 'fixed-or-worse',
              heard: 0,
              tally: {
                fixed: 0,
                notFixed: 0,
                worse: 3,
                resolved: false,
                regressed: true,
              },
            },),).toBe(
              'CHECKER fixed-but-damaged expected=fixed-or-worse heard=0 fixed=0 notFixed=0 worse=3 '
                + 'resolved=false regressed=true',
            );
          },
        },),
      ],
    },),

    describe({
      name: sheetCheckLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS the sheet and issue as one label, with the counts and resolved but no heard or regressed',
          fn: async () => {
            expect(sheetCheckLine({
              sheet: 'mixed-sheet',
              issueId: 'adjudicated/tense',
              expectation: 'fixed',
              tally: COUNTED_TALLY,
            },),).toBe(
              'CHECKER mixed-sheet/adjudicated/tense expected=fixed fixed=2 notFixed=1 worse=0.5 resolved=true',
            );
          },
        },),
      ],
    },),

    describe({
      name: 'CHECKER_NOTE',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS the untouched case is the one that matters and why the mixed sheet follows it',
          fn: async () => {
            expect(CHECKER_NOTE,).toBe(
              'NOTE the untouched case is the one that matters: a majority calling an '
                + 'unrepaired text fixed would mean the 98.1 percent resolution rate '
                + 'measures the checkers rather than the repairs. The mixed sheet asks '
                + 'the same question under the shape that rate was measured on, since '
                + 'production passes every accepted issue of a chunk in one call.',
            );
          },
        },),
      ],
    },),

    describe({
      name: 'SINGLE_ISSUE_CASES',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS the genuinely fixed, untouched and fixed-but-damaged candidates in run order',
          fn: async () => {
            expect(SINGLE_ISSUE_CASES,).toEqual([
              {
                label: 'genuinely-fixed',
                patchedText: 'The cat sleeps on the windowsill, and she wakes when the sun moves.',
                expectation: 'fixed',
              },
              {
                label: 'untouched',
                patchedText: 'The cat is doing the sleeping on the windowsill, and she wakes when the sun moves.',
                expectation: 'not-fixed',
              },
              {
                label: 'fixed-but-damaged',
                patchedText: 'The cat sleeps on the windowsill.',
                expectation: 'fixed-or-worse',
              },
            ],);
          },
        },),
      ],
    },),
  ],
},);
