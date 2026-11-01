/**
 Tests for resolution-check wire resolution and majority tallying.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { UNATTRIBUTED_TEXT, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  isResolutionReportWire,
  resolveResolutionChecks,
  tallyResolutionChecks,
} from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Checker whose ballots the resolution cases resolve.
 */
const CHECKER = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 Issue ids in prompt numbering order for resolution tests.
 */
const ISSUE_IDS = [
  'adjudicated/whisker',
  'adjudicated/paw',
] as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isResolutionReportWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts well-formed reports and rejects malformed ones',
          fn: async () => {
            expect(isResolutionReportWire({
              checks: [
                {
                  issue: 1,
                  verdict: 'fixed',
                },
              ],
            },),).toBe(true,);
            expect(isResolutionReportWire({ checks: [], },),).toBe(true,);
            expect(isResolutionReportWire({},),).toBe(false,);
            expect(isResolutionReportWire({ checks: [{ issue: 1.5, verdict: 'fixed', },], },),)
              .toBe(false,);
            expect(isResolutionReportWire({ checks: [{ issue: 1, },], },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: resolveResolutionChecks.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'resolves checks through the index map and records every irregularity whole, the issue the '
            + 'unknown verdict left unanswered named by id and checker with its cause',
          fn: async () => {
            /** Report with one good check, one bad index, one unknown verdict. */
            const ballot = resolveResolutionChecks({
              wire: {
                checks: [
                  {
                    issue: 1,
                    verdict: 'fixed',
                  },
                  {
                    issue: 9,
                    verdict: 'fixed',
                  },
                  {
                    issue: 2,
                    verdict: 'perfect',
                  },
                ],
              },
              issueIds: ISSUE_IDS,
              checkerModelId: CHECKER,
            },);
            expect(ballot,).toEqual({
              verdicts: { 'adjudicated/whisker': 'fixed', },
              findings: [
                'check-index-out-of-range (9)',
                'unknown-resolution-verdict (perfect)',
                `missing-check (adjudicated/paw, ${CHECKER}, unknown-verdict)`,
              ],
            },);
          },
        },),

        it({
          name: 'NAMES an issue the checker never mentioned by id and checker as unanswered',
          fn: async () => {
            /** Report answering the first issue alone. */
            const ballot = resolveResolutionChecks({
              wire: {
                checks: [
                  {
                    issue: 1,
                    verdict: 'not-fixed',
                  },
                ],
              },
              issueIds: ISSUE_IDS,
              checkerModelId: CHECKER,
            },);
            expect(ballot,).toEqual({
              verdicts: { 'adjudicated/whisker': 'not-fixed', },
              findings: [`missing-check (adjudicated/paw, ${CHECKER}, unanswered)`,],
            },);
          },
        },),

        it({
          name: 'NAMES an issue the checker answered with an unknown verdict and then with a known one as '
            + 'answered, keeping the known verdict and recording the unknown one alone',
          fn: async () => {
            /** Report giving issue two an unknown verdict, then a known one. */
            const ballot = resolveResolutionChecks({
              wire: {
                checks: [
                  {
                    issue: 1,
                    verdict: 'fixed',
                  },
                  {
                    issue: 2,
                    verdict: 'perfect',
                  },
                  {
                    issue: 2,
                    verdict: 'worse',
                  },
                ],
              },
              issueIds: ISSUE_IDS,
              checkerModelId: CHECKER,
            },);
            expect(ballot,).toEqual({
              verdicts: {
                'adjudicated/whisker': 'fixed',
                'adjudicated/paw': 'worse',
              },
              findings: ['unknown-resolution-verdict (perfect)',],
            },);
          },
        },),

        it({
          name: 'keeps the first check on duplicates and records the repeat',
          fn: async () => {
            /** Report answering issue one twice. */
            const ballot = resolveResolutionChecks({
              wire: {
                checks: [
                  {
                    issue: 1,
                    verdict: 'fixed',
                  },
                  {
                    issue: 1,
                    verdict: 'worse',
                  },
                  {
                    issue: 2,
                    verdict: 'not-fixed',
                  },
                ],
              },
              issueIds: ISSUE_IDS,
              checkerModelId: CHECKER,
            },);
            expect(ballot,).toEqual({
              verdicts: {
                'adjudicated/whisker': 'fixed',
                'adjudicated/paw': 'not-fixed',
              },
              findings: ['duplicate-check (1)',],
            },);
          },
        },),
      ],
    },),

    describe({
      name: tallyResolutionChecks.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'resolves on strict fixed majority and flags worse majorities',
          fn: async () => {
            /** Three checkers: two fixed one not-fixed on the first issue, worse-heavy on the second. */
            const tallies = tallyResolutionChecks({
              issueIds: ISSUE_IDS,
              authorship: UNATTRIBUTED_TEXT,
              ballots: {
                a: {
                  verdicts: {
                    'adjudicated/whisker': 'fixed',
                    'adjudicated/paw': 'worse',
                  },
                  findings: [],
                },
                b: {
                  verdicts: {
                    'adjudicated/whisker': 'fixed',
                    'adjudicated/paw': 'worse',
                  },
                  findings: [],
                },
                c: {
                  verdicts: {
                    'adjudicated/whisker': 'not-fixed',
                    'adjudicated/paw': 'fixed',
                  },
                  findings: [],
                },
              },
            },);
            expect(tallies['adjudicated/whisker']?.resolved,).toBe(true,);
            expect(tallies['adjudicated/whisker']?.regressed,).toBe(false,);
            expect(tallies['adjudicated/paw']?.resolved,).toBe(false,);
            expect(tallies['adjudicated/paw']?.regressed,).toBe(true,);
          },
        },),

        it({
          name: 'ties and silence resolve nothing',
          fn: async () => {
            /** One fixed against one not-fixed on the first issue; nobody answers the second. */
            const tallies = tallyResolutionChecks({
              issueIds: ISSUE_IDS,
              authorship: UNATTRIBUTED_TEXT,
              ballots: {
                a: {
                  verdicts: { 'adjudicated/whisker': 'fixed', },
                  findings: [],
                },
                b: {
                  verdicts: { 'adjudicated/whisker': 'not-fixed', },
                  findings: [],
                },
              },
            },);
            expect(tallies['adjudicated/whisker']?.resolved,).toBe(false,);
            expect(tallies['adjudicated/paw']?.resolved,).toBe(false,);
            expect(tallies['adjudicated/paw']?.regressed,).toBe(false,);
            expect(tallies['adjudicated/paw']?.fixed,).toBe(0,);
          },
        },),

        it({
          name: 'A LONE WORSE BALLOT REGRESSES NOTHING when the other checker cast no verdict on the issue: the '
            + 'flag reorders shippable candidates, so one judge alone never sets it (owner rule of 2026-10-25)',
          fn: async () => {
            /** The first issue judged worse by one checker while the other stays silent on it. */
            const tallies = tallyResolutionChecks({
              issueIds: ISSUE_IDS,
              authorship: UNATTRIBUTED_TEXT,
              ballots: {
                a: {
                  verdicts: { 'adjudicated/whisker': 'worse', },
                  findings: [],
                },
                b: {
                  verdicts: { 'adjudicated/paw': 'fixed', },
                  findings: [],
                },
              },
            },);
            expect(tallies['adjudicated/whisker']?.worse,).toBe(1,);
            expect(tallies['adjudicated/whisker']?.regressed,).toBe(false,);
          },
        },),

        it({
          name: 'REGRESSES on two worse ballots against one fixed, which is the positive control for the lone '
            + 'worse ballot: the assertion has to be reading the count of worse ballots and not the weights',
          fn: async () => {
            /** Two checkers judge the first issue worse and one judges it fixed. */
            const tallies = tallyResolutionChecks({
              issueIds: ISSUE_IDS,
              authorship: UNATTRIBUTED_TEXT,
              ballots: {
                a: {
                  verdicts: { 'adjudicated/whisker': 'worse', },
                  findings: [],
                },
                b: {
                  verdicts: { 'adjudicated/whisker': 'worse', },
                  findings: [],
                },
                c: {
                  verdicts: { 'adjudicated/whisker': 'fixed', },
                  findings: [],
                },
              },
            },);
            expect(tallies['adjudicated/whisker'],).toEqual({
              fixed: 1,
              notFixed: 0,
              worse: 2,
              resolved: false,
              regressed: true,
            },);
          },
        },),

        it({
          name: 'DOES NOT REGRESS when two worse ballots are level with two fixed ballots: worse has to strictly '
            + 'outweigh the rest as well as come from two judges',
          fn: async () => {
            /** Two checkers judge the first issue worse and two judge it fixed. */
            const tallies = tallyResolutionChecks({
              issueIds: ISSUE_IDS,
              authorship: UNATTRIBUTED_TEXT,
              ballots: {
                a: {
                  verdicts: { 'adjudicated/whisker': 'worse', },
                  findings: [],
                },
                b: {
                  verdicts: { 'adjudicated/whisker': 'worse', },
                  findings: [],
                },
                c: {
                  verdicts: { 'adjudicated/whisker': 'fixed', },
                  findings: [],
                },
                d: {
                  verdicts: { 'adjudicated/whisker': 'fixed', },
                  findings: [],
                },
              },
            },);
            expect(tallies['adjudicated/whisker'],).toEqual({
              fixed: 2,
              notFixed: 0,
              worse: 2,
              resolved: false,
              regressed: false,
            },);
          },
        },),

        it({
          name: 'ONE CAST BALLOT RESOLVES NOTHING, while two agreeing ballots do: one model never decides '
            + 'that a defect is gone (759 of 8,788 readings over 403 run directories resolved on one '
            + 'ballot, 756 inside a selected patch; XingZ6014 slice 87 shipped one)',
          fn: async () => {
            /** The first issue heard by one checker alone; the second by two agreeing ones. */
            const tallies = tallyResolutionChecks({
              issueIds: ISSUE_IDS,
              authorship: UNATTRIBUTED_TEXT,
              ballots: {
                a: {
                  verdicts: {
                    'adjudicated/whisker': 'fixed',
                    'adjudicated/paw': 'fixed',
                  },
                  findings: [],
                },
                b: {
                  verdicts: { 'adjudicated/paw': 'fixed', },
                  findings: [],
                },
              },
            },);
            expect(tallies['adjudicated/whisker']?.fixed,).toBe(1,);
            expect(tallies['adjudicated/whisker']?.resolved,).toBe(false,);
            expect(tallies['adjudicated/paw']?.resolved,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
