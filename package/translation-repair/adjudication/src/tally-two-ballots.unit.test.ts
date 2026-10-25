/**
 Tests for the panel tally's floor of two answering panelists, which replaced the minimum ballot weight of three.

 THE OWNER'S RULINGS OF 2026-10-25, quoted where the rule is stated
 (`doc/design-commitments-votes-repairs-and-run-offs.md`): "floor of 3 - don't set a floor.", and on a claim only one
 judge answers, "Two ballots decide (Recommended)". The case that made the weight floor cost a page was the corpus pass
 XingZ6021, slice 67 to 69: two panelists supported every claim and the electorate weight of 2 sat under the floor of
 3, so a unanimous panel decided nothing and an invented quotation shipped.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type {
  AggregatedClaim,
  ClaimCluster,
  IssueSeverity,
  PanelBallot,
  PanelVoteState,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { tallyVotes, } from '../dist/final/node/index.mjs';

/**
 Invented member claim with a chosen id suffix and severity.
 */
function member(
  {
    suffix,
    severity = 'major',
  }: {
    readonly suffix: string;
    readonly severity?: IssueSeverity;
  },
): AggregatedClaim {
  return {
    claimId: `issue/${suffix}`,
    claim: {
      category: 'accuracy/addition',
      severity,
      summary: `The ${suffix} paragraph about the cat is not in the source.`,
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
 Single-member cluster around one claim.
 */
function soloCluster(
  { claimMember, }: { readonly claimMember: AggregatedClaim; },
): ClaimCluster {
  return {
    clusterId: `two-ballots-cluster/${claimMember.claimId}`,
    position: 10,
    members: [claimMember,],
  };
}

/**
 Ballot voting one state on every listed claim.
 */
function uniformBallot(
  {
    claimIds,
    vote,
  }: {
    readonly claimIds: readonly string[];
    readonly vote: PanelVoteState;
  },
): PanelBallot {
  return {
    verdicts: Object.fromEntries(claimIds.map(function toVerdict(claimId,) {
      return [claimId, { vote, },];
    },),),
    mergeOpinions: {},
    findings: [],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: tallyVotes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a claim two panelists support and nobody else answered, the corpus pass XingZ6021 shape, at '
            + 'the default weight of one each',
          fn: async () => {
            /** Claim under vote. */
            const claimMember = member({ suffix: 'quotation', severity: 'critical', },);
            /** Ids voted on. */
            const claimIds = [claimMember.claimId,];
            /** Two panelists answer; six more seats never replied. */
            const result = tallyVotes({
              configuredPanelists: 8,
              clusters: [soloCluster({ claimMember, },),],
              ballots: {
                haiku: uniformBallot({ claimIds, vote: 'supported', },),
                deepseek: uniformBallot({ claimIds, vote: 'supported', },),
              },
            },);
            expect(result.issues[0]?.status,).toBe('accepted',);
            expect(result.issues[0]?.severity,).toBe('critical',);
          },
        },),

        it({
          name: 'REJECTS a claim two panelists call unsupported when a third ballot abstains',
          fn: async () => {
            /** Claim under vote. */
            const claimMember = member({ suffix: 'dismissed', },);
            /** Ids voted on. */
            const claimIds = [claimMember.claimId,];
            /** Two answer unsupported; the third ballot names no claim. */
            const result = tallyVotes({
              configuredPanelists: 3,
              clusters: [soloCluster({ claimMember, },),],
              ballots: {
                a: uniformBallot({ claimIds, vote: 'unsupported', },),
                b: uniformBallot({ claimIds, vote: 'unsupported', },),
                quiet: uniformBallot({ claimIds: [], vote: 'supported', },),
              },
            },);
            expect(result.issues[0]?.status,).toBe('rejected',);
          },
        },),

        it({
          name: 'KEEPS the strict majority of the weight that answered: two panelists split one to one still land '
            + 'needs-human',
          fn: async () => {
            /** Claim under vote. */
            const claimMember = member({ suffix: 'split', },);
            /** Ids voted on. */
            const claimIds = [claimMember.claimId,];
            /** One supporter against one dissenter. */
            const result = tallyVotes({
              configuredPanelists: 4,
              clusters: [soloCluster({ claimMember, },),],
              ballots: {
                a: uniformBallot({ claimIds, vote: 'supported', },),
                b: uniformBallot({ claimIds, vote: 'unsupported', },),
              },
            },);
            expect(result.issues[0]?.status,).toBe('needs-human',);
          },
        },),

        it({
          name: 'LEAVES a claim one panelist alone answers needs-human, since one judge alone never decides, '
            + 'whatever weight it carries',
          fn: async () => {
            /** Claim under vote. */
            const claimMember = member({ suffix: 'lonely', },);
            /** Ids voted on. */
            const claimIds = [claimMember.claimId,];
            /** One heavyweight supporter, one abstaining ballot. */
            const result = tallyVotes({
              configuredPanelists: 3,
              clusters: [soloCluster({ claimMember, },),],
              ballots: {
                heavy: uniformBallot({ claimIds, vote: 'supported', },),
                quiet: uniformBallot({ claimIds: [], vote: 'supported', },),
              },
              config: {
                decisionThreshold: 1 / 2,
                sourceDefectThreshold: 1 / 3,
                weights: { heavy: 5, },
              },
            },);
            expect(result.issues[0]?.status,).toBe('needs-human',);
          },
        },),

        it({
          name: 'COUNTS an ambiguous or source-defect answer as an answering panelist, as the electorate always did',
          fn: async () => {
            /** Claim under vote. */
            const claimMember = member({ suffix: 'defect', },);
            /** Ids voted on. */
            const claimIds = [claimMember.claimId,];
            /** One supporter and one source-defect vote: half the electorate blocks. */
            const result = tallyVotes({
              configuredPanelists: 3,
              clusters: [soloCluster({ claimMember, },),],
              ballots: {
                a: uniformBallot({ claimIds, vote: 'supported', },),
                b: uniformBallot({ claimIds, vote: 'source-defect', },),
              },
            },);
            expect(result.issues[0]?.status,).toBe('source-defect',);
          },
        },),
      ],
    },),
  ],
},);
