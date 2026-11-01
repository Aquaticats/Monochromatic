/**
 Tests for the panel ballot wire format and its fail-closed resolution.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isPanelBallotWire,
  resolvePanelBallot,
  usablePanelBallotFor,
} from '../dist/final/node/index.mjs';

/**
 Claim ids in prompt numbering order for resolution tests.
 */
const CLAIM_IDS = [
  'issue/whisker',
  'issue/paw',
  'issue/tail',
] as const;

/**
 Cluster ids in prompt numbering order for resolution tests.
 */
const CLUSTER_IDS = [
  'cluster/nap',
  'cluster/chase',
] as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isPanelBallotWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts complete and minimal ballots',
          fn: async () => {
            expect(isPanelBallotWire({
              verdicts: [
                {
                  claim: 1,
                  vote: 'supported',
                  severity: 'major',
                },
              ],
              groups: [
                {
                  group: 1,
                  sameDefect: true,
                },
              ],
            },),).toBe(true,);
            expect(isPanelBallotWire({ verdicts: [], },),).toBe(true,);
          },
        },),

        it({
          name: 'rejects malformed ballots',
          fn: async () => {
            expect(isPanelBallotWire({},),).toBe(false,);
            expect(isPanelBallotWire({ verdicts: [{ claim: 'one', vote: 'supported', },], },),)
              .toBe(false,);
            expect(isPanelBallotWire({ verdicts: [{ claim: 1.5, vote: 'supported', },], },),)
              .toBe(false,);
            expect(isPanelBallotWire({ verdicts: [{ claim: 1, },], },),).toBe(false,);
            expect(isPanelBallotWire({
              verdicts: [],
              groups: [{ group: 1, },],
            },),).toBe(false,);
            expect(isPanelBallotWire('a string',),).toBe(false,);
            // A REASON THAT IS NOT TEXT is malformed; a missing one is resolved
            // per verdict instead (owner, 2026-09-27, "Reason before vote").
            expect(isPanelBallotWire({ verdicts: [{ claim: 1, reason: 7, vote: 'supported', },], },),)
              .toBe(false,);
            expect(isPanelBallotWire({ verdicts: [{ claim: 1, reason: 'Paws match.', vote: 'supported', },], },),)
              .toBe(true,);
            // A verdict array element that is not itself a record carries none of
            // the required fields.
            expect(isPanelBallotWire({ verdicts: ['a string, not a verdict record',], },),)
              .toBe(false,);
            // A groups array element that is not itself a record carries neither
            // the group number nor the same-defect flag.
            expect(isPanelBallotWire({ verdicts: [], groups: ['a string, not a group record',], },),)
              .toBe(false,);
            // A group reference must itself be a number, not a numeral string.
            expect(isPanelBallotWire({ verdicts: [], groups: [{ group: '1', sameDefect: true, },], },),)
              .toBe(false,);
            // A group reference must be a whole number: the prompt sheet numbers
            // groups one by one, never fractionally.
            expect(isPanelBallotWire({ verdicts: [], groups: [{ group: 1.5, sameDefect: true, },], },),)
              .toBe(false,);
            // The groups field itself must be an array when present.
            expect(isPanelBallotWire({ verdicts: [], groups: 'not an array', },),)
              .toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: usablePanelBallotFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'refuses a value that is not a wire ballot at all',
          fn: async () => {
            expect(usablePanelBallotFor({ claimCount: 3, },)('not a ballot',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: resolvePanelBallot.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'resolves verdicts and group opinions through the index maps',
          fn: async () => {
            /** Complete well-formed ballot. */
            const ballot = resolvePanelBallot({
              wire: {
                verdicts: [
                  {
                    claim: 1,
                    reason: 'The whiskers are counted wrong.',
                    vote: 'supported',
                    severity: 'critical',
                  },
                  {
                    claim: 2,
                    reason: 'The paw is where the original puts it.',
                    vote: 'unsupported',
                  },
                  {
                    claim: 3,
                    reason: 'The tail is not in either quote.',
                    vote: 'abstain',
                  },
                ],
                groups: [
                  {
                    group: 2,
                    sameDefect: false,
                  },
                ],
              },
              claimIds: CLAIM_IDS,
              clusterIds: CLUSTER_IDS,
            },);
            expect(ballot.verdicts['issue/whisker'],).toEqual({
              vote: 'supported',
              severity: 'critical',
              reason: 'The whiskers are counted wrong.',
            },);
            expect(ballot.verdicts['issue/paw'],).toEqual({
              vote: 'unsupported',
              reason: 'The paw is where the original puts it.',
            },);
            expect(ballot.verdicts['issue/tail'],).toEqual({
              vote: 'abstain',
              reason: 'The tail is not in either quote.',
            },);
            expect(ballot.mergeOpinions['cluster/chase'],).toBe(false,);
            expect(ballot.findings,).toHaveLength(0,);
          },
        },),

        it({
          name: 'KEEPS THE VOTE OF A VERDICT GIVEN WITHOUT A REASON and records the gap (owner, 2026-09-27, '
            + '"Reason before vote"): the reason explains the vote rather than qualifying it, and a provider '
            + 'that does not hold the reply to its schema can drop it, so losing the ballot over it would '
            + 'turn an audit gap into a lost voice',
          fn: async () => {
            /** Ballot with one reason missing and one blank. */
            const ballot = resolvePanelBallot({
              wire: {
                verdicts: [
                  {
                    claim: 1,
                    reason: 'The whiskers are counted wrong.',
                    vote: 'supported',
                  },
                  {
                    claim: 2,
                    vote: 'unsupported',
                  },
                  {
                    claim: 3,
                    reason: '   ',
                    vote: 'abstain',
                  },
                ],
              },
              claimIds: CLAIM_IDS,
              clusterIds: CLUSTER_IDS,
            },);
            expect(ballot.verdicts['issue/paw'],).toEqual({ vote: 'unsupported', },);
            expect(ballot.verdicts['issue/tail'],).toEqual({ vote: 'abstain', },);
            expect(ballot.findings,).toStrictEqual([
              'missing-reason (2)',
              'missing-reason (3)',
            ],);
          },
        },),

        it({
          name: 'RECORDS THE SAME GAP for a reason of invisible characters alone, which trim() keeps and no reader '
            + 'sees, so "Reason before vote" is not met by a zero-width space (ledger B40)',
          fn: async () => {
            /** Ballot whose first reason shows nothing, the others stated. */
            const ballot = resolvePanelBallot({
              wire: {
                verdicts: [
                  {
                    claim: 1,
                    reason: '\u{200B}\u{2060}',
                    vote: 'supported',
                  },
                  {
                    claim: 2,
                    reason: 'The paw is where the original puts it.',
                    vote: 'unsupported',
                  },
                  {
                    claim: 3,
                    reason: 'The tail is not in either quote.',
                    vote: 'abstain',
                  },
                ],
              },
              claimIds: CLAIM_IDS,
              clusterIds: CLUSTER_IDS,
            },);
            expect(ballot.verdicts['issue/whisker'],).toEqual({ vote: 'supported', },);
            expect(ballot.findings,).toStrictEqual(['missing-reason (1)',],);
          },
        },),

        it({
          name: 'records out-of-range references as findings',
          fn: async () => {
            /** Ballot pointing at claims and groups beyond the sheet. */
            const ballot = resolvePanelBallot({
              wire: {
                verdicts: [
                  {
                    claim: 0,
                    vote: 'supported',
                  },
                  {
                    claim: 9,
                    vote: 'supported',
                  },
                ],
                groups: [
                  {
                    group: 7,
                    sameDefect: true,
                  },
                ],
              },
              claimIds: CLAIM_IDS,
              clusterIds: CLUSTER_IDS,
            },);
            expect(Object.keys(ballot.verdicts,),).toHaveLength(0,);
            expect(ballot.findings,).toContain('verdict-index-out-of-range (0)',);
            expect(ballot.findings,).toContain('verdict-index-out-of-range (9)',);
            expect(ballot.findings,).toContain('group-index-out-of-range (7)',);
          },
        },),

        it({
          name: 'keeps the first verdict on duplicates and records the repeat',
          fn: async () => {
            /** Ballot voting claim one twice. */
            const ballot = resolvePanelBallot({
              wire: {
                verdicts: [
                  {
                    claim: 1,
                    vote: 'supported',
                  },
                  {
                    claim: 1,
                    vote: 'unsupported',
                  },
                  {
                    claim: 2,
                    vote: 'ambiguous',
                  },
                  {
                    claim: 3,
                    vote: 'abstain',
                  },
                ],
              },
              claimIds: CLAIM_IDS,
              clusterIds: CLUSTER_IDS,
            },);
            expect(ballot.verdicts['issue/whisker']?.vote,).toBe('supported',);
            expect(ballot.findings,).toContain('duplicate-verdict (1)',);
          },
        },),

        it({
          name: 'records unknown votes and keeps votes with dropped bad re-grades',
          fn: async () => {
            /** Ballot with one invented vote state and one invented severity. */
            const ballot = resolvePanelBallot({
              wire: {
                verdicts: [
                  {
                    claim: 1,
                    vote: 'maybe',
                  },
                  {
                    claim: 2,
                    vote: 'supported',
                    severity: 'catastrophic',
                  },
                  {
                    claim: 3,
                    vote: 'source-defect',
                  },
                ],
              },
              claimIds: CLAIM_IDS,
              clusterIds: CLUSTER_IDS,
            },);
            expect(ballot.verdicts['issue/whisker'],).toBe(undefined,);
            expect(ballot.findings,).toContain('unknown-vote (maybe)',);
            expect(ballot.verdicts['issue/paw'],).toEqual({ vote: 'supported', },);
            expect(ballot.findings,).toContain('unknown-regrade-severity (catastrophic)',);
            expect(ballot.verdicts['issue/tail']?.vote,).toBe('source-defect',);
          },
        },),

        it({
          name: 'records claims left without any verdict',
          fn: async () => {
            /** Ballot answering only the first claim. */
            const ballot = resolvePanelBallot({
              wire: {
                verdicts: [
                  {
                    claim: 1,
                    vote: 'supported',
                  },
                ],
              },
              claimIds: CLAIM_IDS,
              clusterIds: CLUSTER_IDS,
            },);
            expect(ballot.findings,).toContain('missing-verdict (2)',);
            expect(ballot.findings,).toContain('missing-verdict (3)',);
          },
        },),

        it({
          name: 'keeps the first group opinion on duplicates and records the repeat',
          fn: async () => {
            /** Ballot opining the first cluster twice. */
            const ballot = resolvePanelBallot({
              wire: {
                verdicts: [
                  {
                    claim: 1,
                    vote: 'supported',
                  },
                  {
                    claim: 2,
                    vote: 'supported',
                  },
                  {
                    claim: 3,
                    vote: 'supported',
                  },
                ],
                groups: [
                  {
                    group: 1,
                    sameDefect: true,
                  },
                  {
                    group: 1,
                    sameDefect: false,
                  },
                ],
              },
              claimIds: CLAIM_IDS,
              clusterIds: CLUSTER_IDS,
            },);
            expect(ballot.mergeOpinions['cluster/nap'],).toBe(true,);
            expect(ballot.findings,).toContain('duplicate-group-opinion (1)',);
          },
        },),
      ],
    },),
  ],
},);
