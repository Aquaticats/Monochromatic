/**
 Tests for the claims the repair panel accepted about a slice, as the polish
 gate's sheet is given them (CuspariaKLSY21 slice 0, 2026-10-21): a minor
 mistranslation claim never reached the dispute note, so the gate that
 re-softened the line was shown none. Cat-themed invention throughout; no
 corpus content appears here.

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
  acceptedRepairClaimNotesOf,
  acceptedRepairClaimsNote,
  acceptedRepairClaimsOf,
  archiveDisputesOf,
  repairClaimEvidenceOf,
} from '../dist/final/node/index.mjs';

/**
 Issue over the claims, settled at the severity the panel gave it.

 @param status - what the panel settled
 @param severity - severity the panel settled
 @param claims - claims filed under the issue, as category and summary
 */
function issueOf(
  {
    status,
    severity,
    claims,
  }: {
    readonly status: AdjudicatedIssue['status'];
    readonly severity: AdjudicatedIssue['severity'];
    readonly claims: readonly (readonly [IssueClaim['category'], string])[];
  },
): AdjudicatedIssue {
  return {
    issueId: `adjudicated/${claims.map(function summaryOf([, summary,],): string {
      return summary;
    },).join('|',)}`,
    status,
    severity,
    claims: claims.map(function toMember([category, summary,],) {
      return {
        claimId: `issue/${summary}`,
        claim: {
          category,
          severity,
          summary,
          spans: [],
        },
      };
    },),
    tallies: {},
  };
}

/**
 Minor mistranslation the panel accepted: the hunger line softened.
 */
const SOFTENED = issueOf({
  status: 'accepted',
  severity: 'minor',
  claims: [
    ['accuracy/mistranslation', '"cannot have a good meal" softens the cat\'s hunger into a poor dinner.',],
    ['accuracy/mistranslation', '"cannot have a good meal" softens the cat\'s hunger into a poor dinner.',],
  ],
});

/**
 Major addition the panel accepted: it disputes the archive.
 */
const INVENTED = issueOf({
  status: 'accepted',
  severity: 'major',
  claims: [['accuracy/addition', 'The translation adds that the cat climbed the drainpipe.',],],
});

/**
 Claim the panel rejected.
 */
const REJECTED = issueOf({
  status: 'rejected',
  severity: 'minor',
  claims: [['style/awkward-phrasing', 'The second sentence reads stiffly.',],],
});

/**
 Claim the panel left to a human.
 */
const UNSETTLED = issueOf({
  status: 'needs-human',
  severity: 'minor',
  claims: [['fluency/inconsistency', 'The tense shifts once.',],],
});

await describe({
  name: '',
  children: [
    describe({
      name: acceptedRepairClaimsOf.name,
      children: [
        it({
          name: 'LISTS ONLY ACCEPTED CLAIMS at the severity the panel settled, none twice, whatever that severity',
          fn: async () => {
            expect(acceptedRepairClaimsOf({ issues: [SOFTENED, REJECTED, UNSETTLED, INVENTED,], },),).toStrictEqual([
              'accuracy/mistranslation minor: "cannot have a good meal" softens the cat\'s hunger into a poor dinner.',
              'accuracy/addition major: The translation adds that the cat climbed the drainpipe.',
            ],);
          },
        },),
        it({
          name: 'LISTS NOTHING when the panel accepted nothing',
          fn: async () => {
            expect(acceptedRepairClaimsOf({ issues: [REJECTED, UNSETTLED,], },),).toStrictEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: acceptedRepairClaimsNote.name,
      children: [
        it({
          name: 'NUMBERS THE CLAIMS, calls them claims to check against the ORIGINAL, and says a softer rewording puts '
            + 'the fault back',
          fn: async () => {
            /**
             Note over two claims.
             */
            const note = acceptedRepairClaimsNote({ claims: ['a/b minor: first', 'a/c major: second',], },);
            expect(note,).toContain('REPAIR PANEL CLAIMS ACCEPTED: the repair lane\'s adjudicators accepted 2 claim(s)',);
            expect(note,).toContain('(1) a/b minor: first; (2) a/c major: second.',);
            expect(note,).toContain('They are claims, not facts: check each against the ORIGINAL.',);
            expect(note,).toContain('softer, vaguer or less specific than the ORIGINAL puts the same fault back',);
          },
        },),
      ],
    },),
    describe({
      name: acceptedRepairClaimNotesOf.name,
      children: [
        it({
          name: 'WRITES A NOTE for a slice no dispute covers (the minor claim), and none for a slice the panel accepted '
            + 'nothing on',
          fn: async () => {
            /**
             Chunks the repair lane settled.
             */
            const chunks = [
              {
                sliceIndex: 0,
                repairedText: 'She often told me she could not get enough to eat.',
                changed: true,
                issues: [SOFTENED,],
                resolvedIssueIds: [SOFTENED.issueId,],
              },
              {
                sliceIndex: 1,
                repairedText: 'The cat sat.',
                changed: false,
                issues: [REJECTED,],
                resolvedIssueIds: [],
              },
            ];
            /**
             Notes keyed by slice.
             */
            const notes = acceptedRepairClaimNotesOf({
              chunks,
              disputes: archiveDisputesOf({ chunks, },),
            },);
            expect([...notes.keys(),],).toStrictEqual([0,],);
            expect(notes.get(0,),).toContain('(1) accuracy/mistranslation minor: "cannot have a good meal" softens',);
            expect(notes.get(0,),).toContain('accepted 1 claim(s)',);
          },
        },),
        it({
          name: 'LEAVES OUT A CLAIM THE DISPUTE NOTE ALREADY CARRIES, so no sheet names a claim twice, and writes none '
            + 'when the dispute carries them all',
          fn: async () => {
            /**
             One disputed slice with a further minor claim, and one the dispute covers whole.
             */
            const chunks = [
              {
                sliceIndex: 2,
                repairedText: 'The cat sat on the roof.',
                changed: true,
                issues: [INVENTED, SOFTENED,],
                resolvedIssueIds: [INVENTED.issueId,],
              },
              {
                sliceIndex: 3,
                repairedText: 'The cat sat on the roof.',
                changed: true,
                issues: [INVENTED,],
                resolvedIssueIds: [INVENTED.issueId,],
              },
            ];
            /**
             Notes keyed by slice.
             */
            const notes = acceptedRepairClaimNotesOf({
              chunks,
              disputes: archiveDisputesOf({ chunks, },),
            },);
            expect([...notes.keys(),],).toStrictEqual([2,],);
            expect(notes.get(2,),).toContain('accepted 1 claim(s)',);
            expect(notes.get(2,),).not.toContain('drainpipe',);
            expect(notes.get(2,),).toContain('softens the cat\'s hunger',);
          },
        },),
      ],
    },),
    describe({
      name: repairClaimEvidenceOf.name,
      children: [
        it({
          name: 'READS THE DISPUTES AND THE CLAIM NOTES FROM THE SAME CHUNKS, so a withdrawn slice stays disputed and its '
            + 'minor claim still gets a note',
          fn: async () => {
            /**
             Chunks the repair lane settled, the first withdrawn at assembly.
             */
            const chunks = [
              {
                sliceIndex: 0,
                repairedText: 'The cat sat on the roof.',
                changed: true,
                issues: [INVENTED, SOFTENED,],
                resolvedIssueIds: [INVENTED.issueId,],
              },
            ];
            /**
             Evidence read off the repair lane.
             */
            const { archiveDisputes, acceptedRepairClaimNotes, } = repairClaimEvidenceOf({
              repair: {
                chunks,
                withdrawnSliceIndices: [0,],
              },
            },);
            expect(archiveDisputes.get(0,)?.standInEligible,).toBe(false,);
            expect(acceptedRepairClaimNotes.get(0,),).toContain('softens the cat\'s hunger',);
            expect(acceptedRepairClaimNotes.get(0,),).not.toContain('drainpipe',);
          },
        },),
      ],
    },),
  ],
},);
