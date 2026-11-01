/**
 Guards the owner's answer of 2026-09-27 ("Adjudicated", seventeenth addendum
 of `doc/decision/translation-repair-ineligible-standing.md`): the
 major-or-worse dispute rule reads the severity the panel settled on the
 issue, not the severity a critic filed on one claim. On XingZ6014 slice 66
 a claim filed major disputed the archive though the panel settled the issue
 minor, and slice 19 went undisputed though the panel settled its issue
 major.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  AdjudicatedIssue,
  IssueClaim,
  IssueSeverity,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { archiveDisputesOf, } from '../dist/final/node/index.mjs';

/**
 Repair lane's text for every slice here.
 */
const REPAIRED = 'The cat sat on the windowsill.';

/**
 Accepted issue whose one claim the critic filed at one severity and the
 panel settled at another.

 @param category - category the critic filed

 @param summary - what the claim says

 @param filed - severity the critic filed

 @param settled - severity the panel settled on the issue

 @returns Accepted adjudicated issue

 @example
 ```ts
 const issue = acceptedIssue({ category: 'accuracy/mistranslation', summary, filed: 'major', settled: 'minor', },);
 ```
 */
function acceptedIssue(
  {
    category,
    summary,
    filed,
    settled,
  }: {
    readonly category: IssueClaim['category'];
    readonly summary: string;
    readonly filed: IssueSeverity;
    readonly settled: IssueSeverity;
  },
): AdjudicatedIssue {
  return {
    issueId: `adjudicated/${summary}`,
    status: 'accepted',
    severity: settled,
    claims: [{
      claimId: `issue/${summary}`,
      claim: {
        category,
        severity: filed,
        summary,
        spans: [],
      },
    },],
    tallies: {},
  };
}

/**
 Mistranslation the critic called major and the panel settled minor.
 */
const DOWNGRADED = 'The translation says the cat was grey where the original says silver.';

/**
 Mistranslation the critic called minor and the panel settled major.
 */
const UPGRADED = 'The translation says the cat slept where the original says it hid.';

/**
 Addition the panel settled minor.
 */
const MINOR_ADDITION = 'The translation adds that the cat purred, which the original never states.';

await describe({
  name: 'the dispute rule reads the panel\'s severity (owner, 2026-09-27)',
  children: [
    it({
      name: 'STANDS ASIDE for a mistranslation the critic filed major and the panel settled minor',
      fn: async () => {
        const disputes = archiveDisputesOf({
          chunks: [{
            sliceIndex: 1,
            repairedText: REPAIRED,
            changed: true,
            resolvedIssueIds: [],
            issues: [acceptedIssue({
              category: 'accuracy/mistranslation',
              summary: DOWNGRADED,
              filed: 'major',
              settled: 'minor',
            },),],
          },],
        },);
        expect(disputes.size,).toBe(0,);
      },
    },),
    it({
      name: 'NAMES A DISPUTE for a mistranslation the critic filed minor and the panel settled major, printing the '
        + 'panel\'s severity beside the claim',
      fn: async () => {
        const disputes = archiveDisputesOf({
          chunks: [{
            sliceIndex: 2,
            repairedText: REPAIRED,
            changed: true,
            resolvedIssueIds: [],
            issues: [acceptedIssue({
              category: 'accuracy/mistranslation',
              summary: UPGRADED,
              filed: 'minor',
              settled: 'major',
            },),],
          },],
        },);
        expect(disputes.get(2,)?.acceptedClaims,).toEqual([`accuracy/mistranslation major: ${UPGRADED}`,],);
      },
    },),
    it({
      name: 'KEEPS an accepted addition disputing at any severity the panel settles',
      fn: async () => {
        const disputes = archiveDisputesOf({
          chunks: [{
            sliceIndex: 3,
            repairedText: REPAIRED,
            changed: true,
            resolvedIssueIds: [],
            issues: [acceptedIssue({
              category: 'accuracy/addition',
              summary: MINOR_ADDITION,
              filed: 'major',
              settled: 'minor',
            },),],
          },],
        },);
        expect(disputes.get(3,)?.acceptedClaims,).toEqual([`accuracy/addition minor: ${MINOR_ADDITION}`,],);
      },
    },),
  ],
},);
