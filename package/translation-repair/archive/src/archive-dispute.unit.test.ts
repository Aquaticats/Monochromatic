/**
 Guards class one hundred seven (owner answer 2026-09-24, "Not eligible;
 fall back to the repair text"): CuspariaKLSY10 slice 3's archive rendering
 named a suicide method the original never states, the repair lane's
 adjudicators accepted the `accuracy/addition` claims against it and removed
 the detail, the translate slate backed nobody, and the archive stood because
 an eligible standing keeps its single round. The reading here names such a
 slice a dispute and hands the repair lane's text over as the archive's
 stand-in. Cat-themed invention throughout; no corpus content appears here.

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
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  archiveDisputeNote,
  archiveDisputeNotesOf,
  archiveDisputesOf,
  describeArchiveDispute,
  logArchiveDisputes,
} from '../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

/**
 Claim the critics filed against the archive rendering.
 */
function claimOf(
  {
    category,
    summary,
    severity = 'major',
  }: {
    readonly category: IssueClaim['category'];
    readonly summary: string;
    readonly severity?: IssueClaim['severity'];
  },
): IssueClaim {
  return {
    category,
    severity,
    summary,
    spans: [],
  };
}

/**
 Adjudicated issue over one claim.
 */
function issueOf(
  {
    status,
    claim,
  }: {
    readonly status: AdjudicatedIssue['status'];
    readonly claim: IssueClaim;
  },
): AdjudicatedIssue {
  return {
    issueId: `adjudicated/${claim.summary}`,
    status,
    severity: claim.severity,
    claims: [{ claimId: `issue/${claim.summary}`, claim, },],
    tallies: {},
  };
}

/**
 Claim that the archive invented how the cat got onto the roof.
 */
const INVENTED = claimOf({
  category: 'accuracy/addition',
  summary: 'The translation adds that the cat climbed the drainpipe, which the original never states.',
},);

/**
 Claim that the archive rendered the wrong colour.
 */
const WRONG_COLOUR = claimOf({
  category: 'accuracy/mistranslation',
  summary: 'The translation says the cat was grey where the original says black.',
},);

/**
 Minor claim that the archive rendered a near-synonym.
 */
const NEAR_SYNONYM = claimOf({
  category: 'accuracy/mistranslation',
  summary: 'The translation says the cat dozed where the original says it napped.',
  severity: 'minor',
},);

/**
 Minor claim that the archive invented the time of day.
 */
const INVENTED_HOUR = claimOf({
  category: 'accuracy/addition',
  summary: 'The translation adds that it was noon, which the original never states.',
  severity: 'minor',
},);

/**
 Major claim outside accuracy, about the rendering's grammar.
 */
const UNGRAMMATICAL = claimOf({
  category: 'fluency/grammar',
  summary: 'The translation\'s second sentence has no verb.',
},);

/**
 Repair lane's text for the disputed slice, the invented detail removed.
 */
const REPAIRED = 'The cat sat on the roof.';

await describe({
  name: archiveDisputesOf.name,
  children: [
    it({
      name: 'NAMES A DISPUTE where an accepted accuracy/addition claim stands against the archive, handing the repair text over as the stand-in',
      fn: async () => {
        const disputes = archiveDisputesOf({
          chunks: [
            {
              sliceIndex: 3,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [],
              issues: [
                issueOf({ status: 'accepted', claim: INVENTED, },),
                issueOf({ status: 'accepted', claim: WRONG_COLOUR, },),
                issueOf({ status: 'rejected', claim: INVENTED, },),
              ],
            },
          ],
        },);
        expect([...disputes.keys(),],).toEqual([3,],);
        expect(disputes.get(3,)?.standIn,).toBe(REPAIRED,);
        // Class one hundred seventy-six: the accepted major mistranslation
        // disputes the archive beside the addition.
        expect(disputes.get(3,)?.acceptedClaims,).toEqual([
          `accuracy/addition major: ${INVENTED.summary}`,
          `accuracy/mistranslation major: ${WRONG_COLOUR.summary}`,
        ],);
      },
    },),
    it({
      name: 'NAMES A DISPUTE where an accepted accuracy claim of major severity or worse stands alone, and where an accepted addition is minor (class one hundred seventy-six, owner answer 2026-09-27: "Major+ accuracy")',
      fn: async () => {
        // THE FAILURE THIS CLOSES. TianqiChen66610 slice 13: the repair lane's
        // adjudicators accepted major mistranslation claims against the
        // archive's gloss of a character the performer was remembered as, the
        // gate tied 2 to 2, and the archive shipped because only additions
        // disputed it.
        const disputes = archiveDisputesOf({
          chunks: [
            {
              sliceIndex: 5,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [],
              issues: [issueOf({ status: 'accepted', claim: WRONG_COLOUR, },),],
            },
            {
              sliceIndex: 6,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [],
              issues: [issueOf({ status: 'accepted', claim: INVENTED_HOUR, },),],
            },
          ],
        },);
        expect([...disputes.keys(),],).toEqual([
          5,
          6,
        ],);
        expect(disputes.get(5,)?.acceptedClaims,).toEqual([`accuracy/mistranslation major: ${WRONG_COLOUR.summary}`,],);
        expect(disputes.get(6,)?.acceptedClaims,).toEqual([`accuracy/addition minor: ${INVENTED_HOUR.summary}`,],);
      },
    },),
    it({
      name: 'STANDS ASIDE where the addition claim was rejected or left to a human, where only a minor mistranslation or a claim outside accuracy was accepted, and where nothing was filed',
      fn: async () => {
        const disputes = archiveDisputesOf({
          chunks: [
            {
              sliceIndex: 0,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [],
              issues: [issueOf({ status: 'rejected', claim: INVENTED, },),],
            },
            {
              sliceIndex: 1,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [],
              issues: [issueOf({ status: 'needs-human', claim: INVENTED, },),],
            },
            {
              sliceIndex: 2,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [],
              issues: [
                issueOf({ status: 'accepted', claim: NEAR_SYNONYM, },),
                issueOf({ status: 'accepted', claim: UNGRAMMATICAL, },),
              ],
            },
            {
              sliceIndex: 4,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [],
              issues: [],
            },
          ],
        },);
        expect(disputes.size,).toBe(0,);
      },
    },),
    it({
      name: 'DESCRIBES the dispute as a finding naming the slice and the accepted claims',
      fn: async () => {
        expect(describeArchiveDispute({
          dispute: {
            sliceIndex: 3,
            standIn: REPAIRED,
            standInEligible: true,
            acceptedClaims: [
              `accuracy/addition major: ${INVENTED.summary}`,
              `accuracy/mistranslation major: ${WRONG_COLOUR.summary}`,
            ],
          },
        },),).toBe(
          'translate-archive-disputed (slice 3): the repair lane\'s adjudicators accepted 2 disputing claim(s) '
            + '(accuracy/addition at any severity, any other accuracy claim the panel settled at major or worse) against the archive '
            + 'rendering, so the repair lane\'s text stands in for it (classes one hundred seven and one hundred '
            + 'seventy-six)',
        );
      },
    },),
    it({
      name: 'WRITES THE SHEET NOTE naming every accepted claim and saying a detail they name is not page content in any wording (class one hundred eight, CuspariaKLSY11 slice 3, 2026-09-24)',
      fn: async () => {
        // THE FAILURE THIS CLOSES. CuspariaKLSY11: the repair lane softened
        // the archive's invented method to "She took medication that night",
        // the consolidation dropped it, and the gate kept the stand-in 2 to 1
        // as "dropped page content ... which the Chinese does not contradict".
        const note = archiveDisputeNote({
          dispute: {
            sliceIndex: 3,
            standIn: REPAIRED,
            standInEligible: true,
            acceptedClaims: [
              `accuracy/addition critical: ${INVENTED.summary}`,
              'accuracy/addition major: The translation adds an unverified detail about the roof.',
            ],
          },
        },);
        expect(note.startsWith('ARCHIVE RENDERING DISPUTED',),).toBe(true,);
        expect(note,).toContain(`(1) accuracy/addition critical: ${INVENTED.summary}`,);
        expect(note,).toContain('(2) accuracy/addition major: The translation adds an unverified detail about the roof.',);
        expect(note,).toContain('not page content and not the page\'s apparatus',);
        expect(note,).toContain('in the archive\'s wording or any softer one',);
        expect(note,).toContain('has dropped nothing',);
        expect(note,).toContain('carries an accepted addition',);
        // No mistranslation was accepted, so the note says nothing of one.
        expect(note,).not.toContain('mistranslated',);
      },
    },),
    it({
      name: 'WRITES THE SHEET NOTE for an accepted mistranslation, saying the archive\'s reading is not the page\'s authority (class one hundred seventy-six)',
      fn: async () => {
        const note = archiveDisputeNote({
          dispute: {
            sliceIndex: 5,
            standIn: REPAIRED,
            standInEligible: true,
            acceptedClaims: [`accuracy/mistranslation major: ${WRONG_COLOUR.summary}`,],
          },
        },);
        expect(note.startsWith('ARCHIVE RENDERING DISPUTED',),).toBe(true,);
        expect(note,).toContain(`(1) accuracy/mistranslation major: ${WRONG_COLOUR.summary}`,);
        expect(note,).toContain('departs from the ORIGINAL',);
        // LEDGER S20: a non-translation claim is a reading too.
        expect(note,).toContain(
          'mistranslated, omitted, left untranslated or no translation of the ORIGINAL at all is not the page\'s '
            + 'authority',
        );
        // No addition was accepted, so the addition rule stays off the sheet.
        expect(note,).not.toContain('carries an accepted addition',);
      },
    },),
    it({
      name: 'LOGS EVERY DISPUTE ONCE as a warning, in slice order, and nothing where none was read (ledger T8, '
        + 'eighteenth batch)',
      fn: async () => {
        /**
         Two disputed slices, one the repair fixed and one it did not.
         */
        const disputes = archiveDisputesOf({
          chunks: [
            {
              sliceIndex: 2,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [`adjudicated/${INVENTED.summary}`,],
              issues: [issueOf({ status: 'accepted', claim: INVENTED, },),],
            },
            {
              sliceIndex: 5,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [],
              issues: [issueOf({ status: 'accepted', claim: WRONG_COLOUR, },),],
            },
          ],
        },);
        /**
         Lines logged, each behind its level.
         */
        const lines: string[] = [];
        logArchiveDisputes({
          disputes,
          l: levelCapturingLogger({ lines, },),
        },);
        // Both slices are disputes, the case's premise.
        expect([...disputes.keys(),],).toEqual([
          2,
          5,
        ],);
        expect(lines,).toEqual([...disputes.values(),].map(function warned(dispute,): string {
          return `warn ${describeArchiveDispute({ dispute, },)}`;
        },),);
        /**
         Lines logged for no dispute.
         */
        const silent: string[] = [];
        logArchiveDisputes({
          disputes: new Map(),
          l: levelCapturingLogger({ lines: silent, },),
        },);
        expect(silent,).toEqual([],);
      },
    },),
    it({
      name: 'KEYS A SHEET NOTE TO EVERY DISPUTED SLICE and to no other, each naming that slice\'s claims (ledger T8, '
        + 'eighteenth batch: the contest reads these, and no case gave it a dispute)',
      fn: async () => {
        /**
         Notes over a disputed slice, an undisputed one, and a second
         disputed one.
         */
        const notes = archiveDisputeNotesOf({
          chunks: [
            {
              sliceIndex: 1,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [],
              issues: [issueOf({ status: 'accepted', claim: INVENTED, },),],
            },
            {
              sliceIndex: 2,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [],
              issues: [issueOf({ status: 'accepted', claim: NEAR_SYNONYM, },),],
            },
            {
              sliceIndex: 4,
              repairedText: REPAIRED,
              changed: true,
              resolvedIssueIds: [],
              issues: [issueOf({ status: 'accepted', claim: WRONG_COLOUR, },),],
            },
          ],
        },);
        expect([...notes.keys(),],).toEqual([
          1,
          4,
        ],);
        expect({
          first: (notes.get(1,) ?? '').includes(`(1) accuracy/addition major: ${INVENTED.summary}`,),
          firstOther: (notes.get(1,) ?? '').includes(WRONG_COLOUR.summary,),
          second: (notes.get(4,) ?? '').includes(`(1) accuracy/mistranslation major: ${WRONG_COLOUR.summary}`,),
          secondOther: (notes.get(4,) ?? '').includes(INVENTED.summary,),
        },).toEqual({
          first: true,
          firstOther: false,
          second: true,
          secondOther: false,
        },);
      },
    },),
  ],
},);
