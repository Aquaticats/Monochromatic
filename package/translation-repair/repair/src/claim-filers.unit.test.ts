/**
 Guards the claim-author record (owner, 2026-09-24: "we're not logging enough
 then. Refine how we log."). A CuspariaKLSY10 critic claim inverted the
 original's cause and effect, was accepted, and the artifact's adjudicated
 issue named no author: the reader had to join `sliceCritics[].claimAttributions`
 by claim id to learn which critic filed it. Every resolved claim is now
 logged with its filers, every adjudicated issue with its filers, and the
 issue carries the filers beside its claims. Cat-themed invention throughout;
 no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ClaimAttribution, } from '@monochromatic-dev/translation-repair-adjudication';
import type {
  AdjudicatedIssue,
  IssueClaim,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  attachClaimFilers,
  claimFilersOf,
  describeClaimFiling,
  describeIssueFiling,
  panelClause,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Claim one critic filed.
 */
const LONE_CLAIM_ID = 'issue/whisker';

/**
 Claim two critics filed.
 */
const SHARED_CLAIM_ID = 'issue/paw';

/**
 Claim nobody filed, as an issue rebuilt from an older record carries.
 */
const ORPHAN_CLAIM_ID = 'issue/tail';

/**
 Attribution as the critic phase records it.
 */
const ATTRIBUTIONS: readonly ClaimAttribution[] = [
  {
    claimId: SHARED_CLAIM_ID,
    proposers: [
      { modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, emissionCount: 1, },
      { modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, emissionCount: 2, },
    ],
  },
  {
    claimId: LONE_CLAIM_ID,
    proposers: [{ modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, emissionCount: 1, },],
  },
];

/**
 Claim body the lines quote.
 */
const OMISSION_CLAIM: IssueClaim = {
  category: 'accuracy/omission',
  severity: 'major',
  summary: 'Translation omits why the cat left the windowsill.',
  spans: [],
};

/**
 Issue the panel accepted over the lone claim and the orphan.
 */
const ISSUE: AdjudicatedIssue = {
  issueId: 'adjudicated/whisker-tail',
  status: 'accepted',
  severity: 'major',
  claims: [
    { claimId: LONE_CLAIM_ID, claim: OMISSION_CLAIM, },
    { claimId: ORPHAN_CLAIM_ID, claim: { ...OMISSION_CLAIM, summary: 'The cat is called a kitten.', }, },
  ],
  tallies: {},
};

/**
 The same issue with the panel's reading of the lone claim, one ballot
 finding it supported and one unsupported, and no reading of the orphan claim.
 */
const SPLIT_PANEL_ISSUE: AdjudicatedIssue = {
  ...ISSUE,
  readings: {
    [LONE_CLAIM_ID]: {
      ballots: [
        { panelistId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, vote: 'supported', weight: 1, },
        { panelistId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, vote: 'unsupported', weight: 1, },
      ],
      configuredPanelists: 2,
      tally: { supported: 1, unsupported: 1, ambiguous: 0, sourceDefect: 0, abstain: 0, },
    },
  },
};

/**
 The same issue with a reading of the lone claim in which no seated panelist
 was heard.
 */
const SILENT_PANEL_ISSUE: AdjudicatedIssue = {
  ...ISSUE,
  readings: {
    [LONE_CLAIM_ID]: {
      ballots: [],
      configuredPanelists: 2,
      tally: { supported: 0, unsupported: 0, ambiguous: 0, sourceDefect: 0, abstain: 0, },
    },
  },
};

await describe({
  name: 'claim filers (owner, 2026-09-24)',
  children: [
    it({
      name: 'NAMES every filer of a claim, sorted, from the attribution',
      fn: async () => {
        expect(claimFilersOf({ attributions: ATTRIBUTIONS, },),).toEqual({
          [SHARED_CLAIM_ID]: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
          [LONE_CLAIM_ID]: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
        },);
      },
    },),
    it({
      name: 'LOGS a resolved claim with its filers, category, severity and summary',
      fn: async () => {
        expect(describeClaimFiling({
          sliceIndex: 3,
          claimId: LONE_CLAIM_ID,
          claim: OMISSION_CLAIM,
          filers: claimFilersOf({ attributions: ATTRIBUTIONS, },),
        },),).toBe(
          `chunk 3: claim ${LONE_CLAIM_ID} filed by ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}: accuracy/omission major: Translation omits why the cat left the windowsill.`,
        );
      },
    },),
    it({
      name: 'LOGS an adjudicated issue with its status and every claim\'s filers, naming nobody for an unattributed claim',
      fn: async () => {
        expect(describeIssueFiling({
          sliceIndex: 3,
          issue: ISSUE,
          filers: claimFilersOf({ attributions: ATTRIBUTIONS, },),
        },),).toBe(
          `chunk 3: issue adjudicated/whisker-tail accepted major: ${LONE_CLAIM_ID} filed by ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}, panel not on record: accuracy/omission major: Translation omits why the cat left the windowsill.; ${ORPHAN_CLAIM_ID} filed by nobody on record, panel not on record: accuracy/omission major: The cat is called a kitten.`,
        );
      },
    },),
    it({
      name: 'NAMES THE PANEL behind each claim, every voter with its vote (hulicaijia26, 2026-09-26: the owner asked who '
        + 'accepted a translator note as an addition, and the log line named its filer alone)',
      fn: async () => {
        expect(describeIssueFiling({
          sliceIndex: 3,
          issue: SPLIT_PANEL_ISSUE,
          filers: claimFilersOf({ attributions: ATTRIBUTIONS, },),
        },),).toBe(
          `chunk 3: issue adjudicated/whisker-tail accepted major: ${LONE_CLAIM_ID} filed by ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}, panel ${SEAT_HYPER_OPENROUTER_VISION_EDITOR} supported, ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} unsupported: accuracy/omission major: Translation omits why the cat left the windowsill.; ${ORPHAN_CLAIM_ID} filed by nobody on record, panel not on record: accuracy/omission major: The cat is called a kitten.`,
        );
      },
    },),
    it({
      name: 'ATTACHES the filers to the issue beside its claims, only for claims the record names',
      fn: async () => {
        /**
         Issues with the filers attached.
         */
        const attached = attachClaimFilers({
          issues: [ISSUE,],
          filers: claimFilersOf({ attributions: ATTRIBUTIONS, },),
        },);
        expect(attached,).toEqual([
          {
            ...ISSUE,
            filedBy: { [LONE_CLAIM_ID]: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,], },
          },
        ],);
      },
    },),

    it({
      name: 'LEAVES an issue as it stands where none of its claims is on record',
      fn: async () => {
        expect(attachClaimFilers({
          issues: [ISSUE,],
          filers: {},
        },),).toEqual([ISSUE,],);
      },
    },),

    it({
      name: 'NAMES each panelist with its vote where the panel cast ballots on a claim, and says not on '
        + 'record where it cast none or the issue carries no reading for the claim',
      fn: async () => {
        expect(panelClause({
          claimId: LONE_CLAIM_ID,
          issue: SPLIT_PANEL_ISSUE,
        },),).toBe(
          `${SEAT_HYPER_OPENROUTER_VISION_EDITOR} supported, ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} unsupported`,
        );
        expect(panelClause({
          claimId: LONE_CLAIM_ID,
          issue: SILENT_PANEL_ISSUE,
        },),).toBe('not on record',);
        expect(panelClause({
          claimId: ORPHAN_CLAIM_ID,
          issue: SPLIT_PANEL_ISSUE,
        },),).toBe('not on record',);
      },
    },),
  ],
},);
