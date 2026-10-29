/**
 Emission deduplication must retain every member's known decision evidence.

 @module
 */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  type AdjudicatedIssue,
  dedupeAcceptedIssues,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { tallyVotes, } from '../dist/final/node/index.mjs';

/** Real tally records for separate accepted claims sharing an emission key. */
function issues(): readonly AdjudicatedIssue[] {
  const members = ['first', 'second'].map(id => ({
    claimId: `claim/${id}`,
    claim: {
      category: 'accuracy/mistranslation' as const, severity: 'major' as const, summary: `The ${id} reading reverses the cat’s direction.`,
      spans: [{ side: 'target' as const, nodeId: 'block/0', nodeHash: 'cat-hash', startOffset: 0, endOffset: 12, quotedText: 'The cat left' }],
    },
  }));
  return tallyVotes({
    clusters: members.map(member => ({ clusterId: `cluster/${member.claimId}`, position: 0, members: [member] })),
    configuredPanelists: 3,
    ballots: Object.fromEntries(['a', 'b', 'c'].map(id => [id, {
      verdicts: Object.fromEntries(members.map(member => [member.claimId, { vote: 'supported' as const }])),
      mergeOpinions: {}, findings: [],
    }])),
  }).issues;
}

/** Legacy absence is unknown evidence, not an empty set of ballots. */
function withoutReadings(issue: AdjudicatedIssue): AdjudicatedIssue {
  return { issueId: issue.issueId, status: issue.status, severity: issue.severity, claims: issue.claims, tallies: issue.tallies };
}

await describe({
  name: 'deduplicated claim evidence',
  children: [
    it({
      name: 'RETAINS new member tallies and readings while preserving the existing representative identity',
      fn: async () => {
        const original = issues();
        const { issues: [merged] } = dedupeAcceptedIssues({ issues: original });
        expect(merged?.issueId).toBe(original[0]?.issueId);
        expect(merged?.severity).toBe(original[0]?.severity);
        expect(merged?.claims.map(member => member.claimId)).toEqual(['claim/first', 'claim/second']);
        expect(Object.keys(merged?.tallies ?? {})).toEqual(['claim/first', 'claim/second']);
        expect(Object.keys(merged?.readings ?? {})).toEqual(['claim/first', 'claim/second']);
        expect(merged?.readings?.['claim/second']).toEqual(original[1]?.readings?.['claim/second']);
      },
    }),
    it({
      name: 'KEEPS identical repeated claim evidence once',
      fn: async () => {
        const [original] = issues();
        if (original === undefined) throw new Error('missing fixture');
        const outcome = dedupeAcceptedIssues({ issues: [original, { ...original, issueId: 'another-id' }] });
        expect(outcome.issues[0]?.claims).toHaveLength(1);
        expect(outcome.issues[0]?.readings).toEqual(original.readings);
      },
    }),
    ...['tally', 'reading'].map(kind => it({
      name: `REJECTS a repeated claim with conflicting ${kind} evidence`,
      fn: async () => {
        const [original] = issues();
        const tally = original?.tallies['claim/first'];
        const reading = original?.readings?.['claim/first'];
        if ((original === undefined) || (tally === undefined) || (reading === undefined)) throw new Error('missing fixture');
        const conflicting = kind === 'tally'
          ? { ...original, tallies: { 'claim/first': { ...tally, supported: 4 } } }
          : { ...original, readings: { 'claim/first': { ...reading, configuredPanelists: 4 } } };
        expect(() => dedupeAcceptedIssues({ issues: [original, conflicting] })).toThrow('claim/first');
      },
    })),
    ...['first', 'second', 'neither'].map(known => it({
      name: `PRESERVES missing readings when only ${known} input has them`,
      fn: async () => {
        const original = issues();
        const inputs = original.map((issue, index) => (index === 0 ? known === 'first' : known === 'second') ? issue : withoutReadings(issue));
        const { issues: [merged] } = dedupeAcceptedIssues({ issues: inputs });
        if (known === 'neither') expect(merged?.readings).toBeUndefined();
        else expect(Object.keys(merged?.readings ?? {})).toEqual([`claim/${known}`]);
      },
    })),
    it({
      name: 'RETAINS known evidence when a repeated legacy member has no reading',
      fn: async () => {
        const [original] = issues();
        if (original === undefined) throw new Error('missing fixture');
        const { issues: [merged] } = dedupeAcceptedIssues({ issues: [withoutReadings(original), original] });
        expect(merged?.readings).toEqual(original.readings);
      },
    }),
  ],
});
