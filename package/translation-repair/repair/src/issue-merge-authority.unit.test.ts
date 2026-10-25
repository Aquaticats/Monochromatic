/**
 Rejected descriptions of a real defect must not become repair authority through merging.

 @module
 */
import { describe, expect, it, } from '@monochromatic-dev/module-test/ts';
import { tallyVotes, } from '@monochromatic-dev/translation-repair-adjudication';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { buildLicensedQuotes, } from '@monochromatic-dev/translation-repair-glossary';
import {
  type AdjudicationConfig,
  type AggregatedClaim,
  dedupeAcceptedIssues,
  deriveEditableEnvelopes,
  type PanelVoteState,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { buildEditorMessages, } from '../dist/final/node/index.mjs';

const targetText = 'The cat left. Her friend stayed.';
const correctedSpan = 'The cat left.';
const support = Array.from({ length: 6 }, (): PanelVoteState => 'supported');
const oppose = Array.from({ length: 6 }, (): PanelVoteState => 'unsupported');
const ambiguous = Array.from({ length: 6 }, (): PanelVoteState => 'ambiguous');

/** Each row contains the actual six panelists' votes on one member. */
function adjudicate({ rows, merge = true, config, }: {
  readonly rows: readonly (readonly PanelVoteState[])[];
  readonly merge?: boolean;
  readonly config?: AdjudicationConfig;
}): ReturnType<typeof tallyVotes> {
  const members: readonly AggregatedClaim[] = rows.map((_row, index) => ({
    claimId: `issue/${index}`,
    claim: {
      category: index === 0 ? 'accuracy/mistranslation' : 'accuracy/addition',
      severity: index === 0 ? 'minor' : 'critical',
      summary: index === 0 ? 'Correct the cat’s direction.' : `Delete the independently rejected detail ${index}.`,
      spans: [{
        side: 'target', nodeId: 'block/0', nodeHash: hashContent({ content: targetText }),
        startOffset: 0, endOffset: index === 0 ? correctedSpan.length : targetText.length,
        quotedText: index === 0 ? correctedSpan : targetText,
      }],
    },
  }));
  return tallyVotes({
    clusters: [{ clusterId: 'cluster/cat-direction', position: 0, members }],
    configuredPanelists: 6,
    ballots: Object.fromEntries(support.map((_vote, panelist) => [
      `panelist/${panelist}`,
      {
        verdicts: Object.fromEntries(members.map((member, index) => [member.claimId, { vote: rows[index]?.[panelist] ?? 'abstain' }])),
        mergeOpinions: { 'cluster/cat-direction': merge },
        findings: [],
      },
    ])),
    ...(config === undefined ? {} : { config }),
  });
}

await describe({
  name: 'member repair authority',
  children: [
    it({
      name: 'LEAVES empty adjudication empty without creating partition findings',
      fn: async () => {
        expect(tallyVotes({ clusters: [], ballots: {}, configuredPanelists: 6 })).toEqual({ issues: [], findings: [] });
      },
    }),
    it({
      name: 'PRESERVES the rejected 2:4 diagnosis separately from an accepted 6:0 diagnosis of the same event',
      fn: async () => {
        const { issues, findings } = adjudicate({ rows: [support, ['supported', 'supported', 'unsupported', 'unsupported', 'unsupported', 'unsupported']] });
        expect(issues.map(issue => issue.status)).toEqual(['accepted', 'rejected']);
        expect(issues.map(issue => issue.claims.map(member => member.claimId))).toEqual([['issue/0'], ['issue/1']]);
        expect(issues[0]?.severity).toBe('minor');
        expect(issues[1]?.severity).toBe('critical');
        expect(findings).toEqual([`issue-merge-partitioned (cluster/cat-direction: ${issues[0]?.issueId}=accepted, ${issues[1]?.issueId}=rejected)`]);
        // Reporting now sees one accepted and one rejected issue, not one all-accepted record.
        expect(issues.filter(issue => issue.status === 'accepted').length / issues.length).toBe(1 / 2);
        for (const issue of issues) {
          const ids = issue.claims.map(member => member.claimId);
          expect(Object.keys(issue.tallies)).toEqual(ids);
          expect(Object.keys(issue.readings ?? {})).toEqual(ids);
          expect(issue.issueId).toBe(`adjudicated/${hashContent({ content: JSON.stringify(ids.toSorted()) })}`);
        }
      },
    }),
    ...[
      { name: 'accepted and uncertain', rows: [support, ambiguous], statuses: ['accepted', 'needs-human'] },
      { name: 'uncertain and rejected', rows: [ambiguous, oppose], statuses: ['needs-human', 'rejected'] },
      { name: 'below-quorum source concern', rows: [support, ['source-defect', 'abstain', 'abstain', 'abstain', 'abstain', 'abstain']], statuses: ['accepted', 'needs-human'] },
    ].map(example => it({
      name: `KEEPS ${example.name} diagnoses separate`,
      fn: async () => {
        const result = adjudicate({ rows: example.rows as readonly (readonly PanelVoteState[])[] });
        expect(result.issues.map(issue => issue.status)).toEqual(example.statuses);
        expect(result.issues.flatMap(issue => issue.claims)).toHaveLength(2);
      },
    })),
    it({
      name: 'ORDERS partitions by their first member while retaining noncontiguous same-status members',
      fn: async () => {
        const { issues } = adjudicate({ rows: [oppose, support, oppose, ambiguous, support] });
        expect(issues.map(issue => issue.status)).toEqual(['rejected', 'accepted', 'needs-human']);
        expect(issues.map(issue => issue.claims.map(member => member.claimId))).toEqual([
          ['issue/0', 'issue/2'], ['issue/1', 'issue/4'], ['issue/3'],
        ]);
      },
    }),
    it({
      name: 'USES the actual configured threshold rather than re-deriving authority under defaults',
      fn: async () => {
        const result = adjudicate({
          rows: [support, ['supported', 'supported', 'supported', 'supported', 'unsupported', 'unsupported']],
          config: { decisionThreshold: 3 / 4, sourceDefectThreshold: 1 / 3 },
        });
        expect(result.issues.map(issue => issue.status)).toEqual(['accepted', 'needs-human']);
      },
    }),
    it({
      name: 'BLOCKS the entire panel-merged cluster when one member reaches the protective source-defect threshold',
      fn: async () => {
        const result = adjudicate({ rows: [support, ['supported', 'supported', 'supported', 'supported', 'source-defect', 'source-defect'], oppose] });
        expect(result.issues).toHaveLength(1);
        expect(result.issues[0]?.status).toBe('source-defect');
        expect(result.issues[0]?.claims).toHaveLength(3);
        expect(deriveEditableEnvelopes({ issues: result.issues, targetText }).envelopes).toHaveLength(0);
      },
    }),
    it({
      name: 'DOES NOT turn a below-threshold source-defect vote into a global block',
      fn: async () => {
        const result = adjudicate({ rows: [support, ['supported', 'supported', 'supported', 'supported', 'supported', 'source-defect']] });
        expect(result.issues).toHaveLength(1);
        expect(result.issues[0]?.status).toBe('accepted');
        expect(result.issues[0]?.claims).toHaveLength(2);
      },
    }),
    ...[support, oppose, ambiguous].map(votes => it({
      name: `MERGES uniformly ${votes[0]} members as before`,
      fn: async () => {
        const { issues } = adjudicate({ rows: [votes, votes] });
        expect(issues).toHaveLength(1);
        expect(issues[0]?.claims.map(member => member.claimId)).toEqual(['issue/0', 'issue/1']);
        expect(Object.keys(issues[0]?.readings ?? {})).toEqual(['issue/0', 'issue/1']);
      },
    })),
    it({
      name: 'PRESERVES separate member decisions when the panel does not merge',
      fn: async () => {
        const { issues } = adjudicate({ rows: [support, oppose, support], merge: false });
        expect(issues.map(issue => issue.status)).toEqual(['accepted', 'rejected', 'accepted']);
        expect(issues.map(issue => issue.claims.length)).toEqual([1, 1, 1]);
      },
    }),
    it({
      name: 'KEEPS rejected summaries, broad edit spans and deletion licenses out of actual editor input',
      fn: async () => {
        const raw = adjudicate({ rows: [support, oppose] });
        const { issues } = dedupeAcceptedIssues({ issues: raw.issues });
        const { envelopes } = deriveEditableEnvelopes({ issues, targetText });
        const plan = buildEditorMessages({ sourceText: '猫回来了。她的朋友留下来了。', targetText, issues, envelopes });
        const user = plan.messages.find(message => message.role === 'user')?.content ?? '';
        expect(user).toContain('Correct the cat’s direction.');
        expect(user).not.toContain('Delete the independently rejected detail');
        expect(envelopes.map(envelope => envelope.baseText)).toEqual([correctedSpan]);
        expect([...buildLicensedQuotes({ envelopes, issues }).values()]).toEqual([[correctedSpan]]);
        expect(issues.filter(issue => issue.status === 'rejected')).toHaveLength(1);
      },
    }),
  ],
});
