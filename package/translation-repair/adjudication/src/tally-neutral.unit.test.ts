/**
 Tests for how the tally settles a claim the panel grades neutral, the
 severity that asserts no defect (ledger L5).
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type {
  AggregatedClaim,
  ClaimCluster,
  IssueSeverity,
  PanelBallot,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { tallyVotes, } from '../dist/final/node/index.mjs';

/**
 Invented omission claim at a chosen severity.
 */
function member(
  {
    suffix,
    severity,
  }: {
    readonly suffix: string;
    readonly severity: IssueSeverity;
  },
): AggregatedClaim {
  return {
    claimId: `issue/${suffix}`,
    claim: {
      category: 'accuracy/omission',
      severity,
      summary: `The ${suffix} detail about the cat is missing.`,
      spans: [
        {
          side: 'target',
          nodeId: 'block/1',
          nodeHash: hashContent({ content: 'The cat naps.', },),
          startOffset: 10,
          endOffset: 10,
          quotedText: '',
        },
      ],
    },
  };
}

/**
 Ballot supporting one claim, with an optional re-grade.
 */
function supporting(
  {
    claimId,
    severity,
  }: {
    readonly claimId: string;
    readonly severity?: IssueSeverity;
  },
): PanelBallot {
  return {
    verdicts: {
      [claimId]: {
        vote: 'supported',
        ...(severity === undefined ? {} : { severity, }),
      },
    },
    mergeOpinions: {},
    findings: [],
  };
}

/**
 Tally of one solo claim under three supporting ballots.
 */
function tallyThree(
  {
    claimMember,
    regrades,
  }: {
    readonly claimMember: AggregatedClaim;
    readonly regrades: readonly (IssueSeverity | 'silent')[];
  },
): ReturnType<typeof tallyVotes> {
  return tallyVotes({
    configuredPanelists: regrades.length,
    clusters: [
      {
        clusterId: `cluster/${claimMember.claimId}`,
        position: 10,
        members: [claimMember,],
      } satisfies ClaimCluster,
    ],
    ballots: Object.fromEntries(regrades.map(function toBallot(severity, index,) {
      return [
        `panelist-${String(index,)}`,
        supporting({
          claimId: claimMember.claimId,
          ...(severity === 'silent' ? {} : { severity, }),
        },),
      ];
    },),),
  },);
}

await describe({
  name: 'neutral severity at the tally',
  children: [
    it({
      name: 'HOLDS FOR A HUMAN a claim filed neutral that every panelist supports without re-grading, since neutral asserts no defect for the editor to fix',
      fn: async () => {
        /** Claim filed neutral. */
        const claimMember = member({ suffix: 'whisker', severity: 'neutral', },);
        /** Unanimous support, nobody re-grading. */
        const result = tallyThree({ claimMember, regrades: ['silent', 'silent', 'silent',], },);
        expect(result.issues[0]?.status,).toBe('needs-human',);
        expect(result.issues[0]?.severity,).toBe('neutral',);
        expect(result.findings,).toContain(`neutral-held-for-human (${claimMember.claimId})`,);
      },
    },),

    it({
      name: 'HOLDS FOR A HUMAN a claim filed minor that the supporting panel re-grades to neutral',
      fn: async () => {
        /** Claim filed minor. */
        const claimMember = member({ suffix: 'collar', severity: 'minor', },);
        /** Two of three supporters re-grade neutral. */
        const result = tallyThree({ claimMember, regrades: ['neutral', 'neutral', 'silent',], },);
        expect(result.issues[0]?.status,).toBe('needs-human',);
        expect(result.findings,).toContain(`neutral-held-for-human (${claimMember.claimId})`,);
      },
    },),

    it({
      name: 'ACCEPTS a claim filed neutral once a supporter re-grades it to a real defect, because the upper median then leaves neutral',
      fn: async () => {
        /** Claim filed neutral. */
        const claimMember = member({ suffix: 'saucer', severity: 'neutral', },);
        /** One supporter re-grades minor. */
        const result = tallyThree({ claimMember, regrades: ['minor', 'silent', 'silent',], },);
        expect(result.issues[0]?.status,).toBe('accepted',);
        expect(result.issues[0]?.severity,).toBe('minor',);
        expect(result.findings,).toStrictEqual([],);
      },
    },),

    it({
      name: 'NAMES NO HOLD for a claim filed neutral that the panel rejects, since only an acceptance was held',
      fn: async () => {
        /** Claim filed neutral. */
        const claimMember = member({ suffix: 'yarn', severity: 'neutral', },);
        /** Ballot refuting the claim. */
        const refuting: PanelBallot = {
          verdicts: { [claimMember.claimId]: { vote: 'unsupported', }, },
          mergeOpinions: {},
          findings: [],
        };
        /** Unanimous rejection. */
        const result = tallyVotes({
          configuredPanelists: 3,
          clusters: [
            {
              clusterId: `cluster/${claimMember.claimId}`,
              position: 10,
              members: [claimMember,],
            },
          ],
          ballots: { a: refuting, b: refuting, c: refuting, },
        },);
        expect(result.issues[0]?.status,).toBe('rejected',);
        expect(result.findings,).toStrictEqual([],);
      },
    },),

    it({
      name: 'SPLITS a same-defect merge whose neutral member is held from its accepted minor member, so the held claim reaches no editor through its partner',
      fn: async () => {
        /** Claim filed neutral. */
        const heldMember = member({ suffix: 'bell', severity: 'neutral', },);
        /** Claim filed minor. */
        const acceptedMember = member({ suffix: 'basket', severity: 'minor', },);
        /** Cluster holding both. */
        const clusterId = 'cluster/bell-basket';
        /** Ballot supporting both and calling them one defect. */
        const merging: PanelBallot = {
          verdicts: {
            [heldMember.claimId]: { vote: 'supported', },
            [acceptedMember.claimId]: { vote: 'supported', },
          },
          mergeOpinions: { [clusterId]: true, },
          findings: [],
        };
        /** Unanimous support and merge. */
        const result = tallyVotes({
          configuredPanelists: 3,
          clusters: [
            {
              clusterId,
              position: 10,
              members: [heldMember, acceptedMember,],
            },
          ],
          ballots: { a: merging, b: merging, c: merging, },
        },);
        expect(result.issues.map(function statusOf(issue,) {
          return issue.status;
        },),)
          .toStrictEqual(['needs-human', 'accepted',],);
        expect(result.findings,).toContain(`neutral-held-for-human (${heldMember.claimId})`,);
      },
    },),
  ],
},);
