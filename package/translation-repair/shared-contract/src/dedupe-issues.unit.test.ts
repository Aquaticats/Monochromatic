/**
 Tests for merging accepted issues that name one defect several times.

 The measurement behind this is that 13.4% of accepted issues are exact-place
 duplicates, and the human grader independently marked 14% of a 50-item draw
 as duplicates. The cost is not the arithmetic: it is the editor repairing one
 defect twice and cutting two overlapping envelopes for it.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type AdjudicatedIssue,
  dedupeAcceptedIssues,
} from '../dist/final/node/index.mjs';

/**
 Builds an adjudicated issue over one span.

 @param issueId - issue identity

 @param claimId - single member claim identity

 @param category - claimed category

 @param startOffset - span start, which distinguishes places

 @param status - adjudication status

 @returns Issue

 @example
 ```ts
 const issue = issueAt({ issueId: 'i/1', claimId: 'c/1', },);
 ```
 */
function issueAt(
  {
    issueId,
    claimId,
    category = 'accuracy/mistranslation',
    startOffset = 4,
    status = 'accepted',
  }: {
    readonly issueId: string;
    readonly claimId: string;
    readonly category?: string;
    readonly startOffset?: number;
    readonly status?: string;
  },
): AdjudicatedIssue {
  return {
    issueId,
    status,
    severity: 'major',
    tallies: {},
    claims: [
      {
        claimId,
        claim: {
          category,
          severity: 'major',
          spans: [
            {
              side: 'target',
              nodeId: 'node/nap',
              nodeHash: 'hash/nap',
              startOffset,
            },
          ],
        },
      },
    ],
  } as unknown as AdjudicatedIssue;
}

await describe({
  name: dedupeAcceptedIssues.name,
  children: [
    it({
      name: 'MERGES two accepted issues naming the same category over the same '
        + 'span, and keeps every member claim id, because those ids are how '
        + 'attribution answers whether a duplicate came from one critic '
        + 'repeating itself or several critics agreeing',
      fn: async () => {
        /**
         One defect accepted twice.
         */
        const outcome = dedupeAcceptedIssues({
          issues: [
            issueAt({ issueId: 'issue/first', claimId: 'claim/one', },),
            issueAt({ issueId: 'issue/second', claimId: 'claim/two', },),
          ],
        },);

        expect(outcome.issues,).toHaveLength(1,);
        expect(outcome.issues[0]?.issueId,).toBe('issue/first',);
        expect(
          outcome.issues[0]?.claims.map(function toId(member,) {
            return member.claimId;
          },),
        ).toStrictEqual(['claim/one', 'claim/two',],);
        expect(outcome.findings,).toHaveLength(1,);
      },
    },),

    it({
      name: 'keeps two accepted issues apart when they claim DIFFERENT '
        + 'categories over the same span, since a shared span is narrow but is '
        + 'not proof that two complaints are one defect',
      fn: async () => {
        /**
         Two different complaints about one sentence.
         */
        const outcome = dedupeAcceptedIssues({
          issues: [
            issueAt({ issueId: 'issue/first', claimId: 'claim/one', },),
            issueAt({
              issueId: 'issue/second',
              claimId: 'claim/two',
              category: 'fluency/awkward-phrasing',
            },),
          ],
        },);

        expect(outcome.issues,).toHaveLength(2,);
        expect(outcome.findings,).toHaveLength(0,);
      },
    },),

    it({
      name: 'keeps two accepted issues apart when the same category lands on '
        + 'DIFFERENT spans, which is two defects of one kind rather than one '
        + 'defect counted twice',
      fn: async () => {
        /**
         Same complaint, two places.
         */
        const outcome = dedupeAcceptedIssues({
          issues: [
            issueAt({ issueId: 'issue/first', claimId: 'claim/one', startOffset: 4, },),
            issueAt({ issueId: 'issue/second', claimId: 'claim/two', startOffset: 91, },),
          ],
        },);

        expect(outcome.issues,).toHaveLength(2,);
      },
    },),

    it({
      name: 'leaves REJECTED issues alone even when they duplicate each other, '
        + 'because a rejected duplicate costs no repair budget and collapsing '
        + 'rejections would change what the precision denominator counts',
      fn: async () => {
        /**
         Two identical rejections.
         */
        const outcome = dedupeAcceptedIssues({
          issues: [
            issueAt({ issueId: 'issue/first', claimId: 'claim/one', status: 'rejected', },),
            issueAt({ issueId: 'issue/second', claimId: 'claim/two', status: 'rejected', },),
          ],
        },);

        expect(outcome.issues,).toHaveLength(2,);
        expect(outcome.findings,).toHaveLength(0,);
      },
    },),

    it({
      name: 'does not double-count a claim id already held by the survivor, so '
        + 'merging cannot inflate the very count attribution reads',
      fn: async () => {
        /**
         Duplicates that share a member claim.
         */
        const outcome = dedupeAcceptedIssues({
          issues: [
            issueAt({ issueId: 'issue/first', claimId: 'claim/shared', },),
            issueAt({ issueId: 'issue/second', claimId: 'claim/shared', },),
          ],
        },);

        expect(outcome.issues[0]?.claims,).toHaveLength(1,);
      },
    },),

    it({
      name: 'preserves input order and returns everything when nothing '
        + 'duplicates, so an ordinary chunk passes through untouched',
      fn: async () => {
        /**
         Three distinct defects.
         */
        const outcome = dedupeAcceptedIssues({
          issues: [
            issueAt({ issueId: 'issue/a', claimId: 'claim/a', startOffset: 1, },),
            issueAt({ issueId: 'issue/b', claimId: 'claim/b', startOffset: 2, },),
            issueAt({ issueId: 'issue/c', claimId: 'claim/c', startOffset: 3, },),
          ],
        },);

        expect(
          outcome.issues.map(function toId(issue,) {
            return issue.issueId;
          },),
        ).toStrictEqual(['issue/a', 'issue/b', 'issue/c',],);
        expect(outcome.findings,).toHaveLength(0,);
      },
    },),
  ],
},);
