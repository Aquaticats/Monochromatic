/**
 Tests for lexicographic candidate selection.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type CandidateMeasurements,
  CandidateSlateError,
  type RepairCandidate,
  selectRepairCandidate,
  UNCHANGED_CANDIDATE_ID,
  UNCHANGED_MEASUREMENTS,
  winnerChangedText,
} from '../dist/final/node/index.mjs';

/**
 The always-competing unchanged candidate.
 */
const UNCHANGED: RepairCandidate = {
  candidateId: UNCHANGED_CANDIDATE_ID,
  text: 'The cat naps in the sun.',
  measurements: UNCHANGED_MEASUREMENTS,
};

/**
 Repaired candidate with chosen measurements.
 */
function repaired(
  {
    suffix,
    measurements,
  }: {
    readonly suffix: string;
    readonly measurements: CandidateMeasurements;
  },
): RepairCandidate {
  return {
    candidateId: `candidate/${suffix}`,
    text: `The cat naps in the warm sun. (${suffix})`,
    measurements,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: selectRepairCandidate.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'prefers a proven repair over the unchanged translation',
          fn: async () => {
            /** Repair resolving one critical issue cleanly. */
            const fixer = repaired({
              suffix: 'fixer',
              measurements: {
                integrityOk: true,
                resolvedHighSeverity: 1,
                resolvedTotal: 1,
                regressedKnownIssues: 0,
                touchedRegionChars: 20,
              },
            },);
            /** Selection over both. */
            const { winner, } = selectRepairCandidate({
              candidates: [UNCHANGED, fixer,],
              incumbentText: UNCHANGED.text,
            },);
            expect(winner.candidateId,).toBe('candidate/fixer',);
          },
        },),

        it({
          name: 'returns the unchanged translation when nothing demonstrably beats it',
          fn: async () => {
            /** Repair that resolved nothing and changed text anyway. */
            const churner = repaired({
              suffix: 'churner',
              measurements: {
                integrityOk: true,
                resolvedHighSeverity: 0,
                resolvedTotal: 0,
                regressedKnownIssues: 0,
                touchedRegionChars: 35,
              },
            },);
            /** Selection over both. */
            const { winner, } = selectRepairCandidate({
              candidates: [churner, UNCHANGED,],
              incumbentText: UNCHANGED.text,
            },);
            expect(winner.candidateId,).toBe(UNCHANGED_CANDIDATE_ID,);
          },
        },),

        it({
          name: 'ranks integrity above every resolution count',
          fn: async () => {
            /** Repair resolving everything but breaking document structure. */
            const breaker = repaired({
              suffix: 'breaker',
              measurements: {
                integrityOk: false,
                resolvedHighSeverity: 5,
                resolvedTotal: 8,
                regressedKnownIssues: 0,
                touchedRegionChars: 10,
              },
            },);
            /** Selection over both. */
            const { winner, ranking, } = selectRepairCandidate({
              candidates: [breaker, UNCHANGED,],
              incumbentText: UNCHANGED.text,
            },);
            expect(winner.candidateId,).toBe(UNCHANGED_CANDIDATE_ID,);
            expect(ranking.at(-1,)?.candidateId,).toBe('candidate/breaker',);
          },
        },),

        it({
          name: 'ranks high-severity resolution above regression count and preservation above ties',
          fn: async () => {
            /** Repair resolving two criticals with one regression. */
            const bold = repaired({
              suffix: 'bold',
              measurements: {
                integrityOk: true,
                resolvedHighSeverity: 2,
                resolvedTotal: 2,
                regressedKnownIssues: 1,
                touchedRegionChars: 60,
              },
            },);
            /** Repair resolving one critical cleanly. */
            const timid = repaired({
              suffix: 'timid',
              measurements: {
                integrityOk: true,
                resolvedHighSeverity: 1,
                resolvedTotal: 1,
                regressedKnownIssues: 0,
                touchedRegionChars: 15,
              },
            },);
            /** Same measurements as timid but touching more text. */
            const sprawling = repaired({
              suffix: 'sprawling',
              measurements: {
                integrityOk: true,
                resolvedHighSeverity: 1,
                resolvedTotal: 1,
                regressedKnownIssues: 0,
                touchedRegionChars: 90,
              },
            },);
            /** Selection over the full slate. */
            const { ranking, } = selectRepairCandidate({
              candidates: [
                sprawling,
                timid,
                UNCHANGED,
                bold,
              ],
              incumbentText: UNCHANGED.text,
            },);
            expect(ranking.map(function toId(candidate,) {
              return candidate.candidateId;
            },),).toEqual([
              'candidate/bold',
              'candidate/timid',
              'candidate/sprawling',
              UNCHANGED_CANDIDATE_ID,
            ],);
          },
        },),

        it({
          name: 'throws when the unchanged candidate is missing from the slate',
          fn: async () => {
            /** Lone repaired candidate. */
            const only = repaired({
              suffix: 'only',
              measurements: UNCHANGED_MEASUREMENTS,
            },);
            /**
             What selectWithoutUnchanged raised, read for its class as well as its wording.
             */
            const refusalOfSelectWithoutUnchanged = caught(function selectWithoutUnchanged() {
              selectRepairCandidate({
                candidates: [only,],
                incumbentText: UNCHANGED.text,
              },);
            },);

            expect(refusalOfSelectWithoutUnchanged,).toBeInstanceOf(CandidateSlateError,);
            expect((refusalOfSelectWithoutUnchanged as Error).message,).toContain('always competes',);
          },
        },),

        it({
          name: 'REFUSES a slate whose unchanged candidate carries some other wording, since that '
            + 'identifier is what every later reader means by nothing having needed repair here: one '
            + 'carrying an edit would win ties on the strength of its name and ship text nobody ranked',
          fn: async () => {
            /** Entry claiming to be the archive while carrying an edit. */
            const impostor: RepairCandidate = {
              candidateId: UNCHANGED_CANDIDATE_ID,
              text: 'The cat naps in the warm sun.',
              measurements: UNCHANGED_MEASUREMENTS,
            };
            /**
             What selectWithDishonestUnchanged raised, read for its class as well as its wording.
             */
            const refusalOfSelectWithDishonestUnchanged = caught(function selectWithDishonestUnchanged() {
              selectRepairCandidate({
                candidates: [impostor,],
                incumbentText: UNCHANGED.text,
              },);
            },);

            expect(refusalOfSelectWithDishonestUnchanged,).toBeInstanceOf(CandidateSlateError,);
            expect((refusalOfSelectWithDishonestUnchanged as Error).message,).toContain('wording other than the archive text',);
          },
        },),

        it({
          name: 'REFUSES a slate holding TWO candidates under the unchanged identifier, whichever '
            + 'order they arrive in. Checking the first one found leaves the second free to carry an '
            + 'edit, win on its measurements, and be reported as the candidate that changed nothing; '
            + 'which of the two is checked would depend on slate order alone',
          fn: async () => {
            /** Honest archive entry. */
            const archive: RepairCandidate = {
              candidateId: UNCHANGED_CANDIDATE_ID,
              text: UNCHANGED.text,
              measurements: UNCHANGED_MEASUREMENTS,
            };

            /** Second entry wearing the same identifier, with an edit and a better case. */
            const wearingTheName: RepairCandidate = {
              candidateId: UNCHANGED_CANDIDATE_ID,
              text: 'The cat naps in the warm sun.',
              measurements: {
                integrityOk: true,
                resolvedHighSeverity: 3,
                resolvedTotal: 3,
                regressedKnownIssues: 0,
                touchedRegionChars: 5,
              },
            };
            /**
             What honestFirst raised, read for its class as well as its wording.
             */
            const refusalOfHonestFirst = caught(function honestFirst() {
              selectRepairCandidate({
                candidates: [archive, wearingTheName,],
                incumbentText: UNCHANGED.text,
              },);
            },);

            expect(refusalOfHonestFirst,).toBeInstanceOf(CandidateSlateError,);
            expect((refusalOfHonestFirst as Error).message,).toContain('under the unchanged identifier',);
            /**
             What impostorFirst raised, read for its class as well as its wording.
             */
            const refusalOfImpostorFirst = caught(function impostorFirst() {
              selectRepairCandidate({
                candidates: [wearingTheName, archive,],
                incumbentText: UNCHANGED.text,
              },);
            },);

            expect(refusalOfImpostorFirst,).toBeInstanceOf(CandidateSlateError,);
            expect((refusalOfImpostorFirst as Error).message,).toContain('under the unchanged identifier',);
          },
        },),

        it({
          name: 'ACCEPTS an unchanged candidate whose measurements say the ARCHIVE fails integrity, '
            + 'which is not a slate defect: an archive that does not parse is exactly the document '
            + 'this lane exists to repair, and refusing the slate would refuse the repair with it',
          fn: async () => {
            /** Archive measured honestly rather than intact by definition. */
            const brokenArchive: RepairCandidate = {
              candidateId: UNCHANGED_CANDIDATE_ID,
              text: UNCHANGED.text,
              measurements: {
                ...UNCHANGED_MEASUREMENTS,
                integrityOk: false,
              },
            };
            /** Repair that parses, against an archive that does not. */
            const fixer = repaired({
              suffix: 'fixer',
              measurements: UNCHANGED_MEASUREMENTS,
            },);
            /** Selection over both. */
            const { winner, } = selectRepairCandidate({
              candidates: [brokenArchive, fixer,],
              incumbentText: UNCHANGED.text,
            },);
            expect(winner.candidateId,).toBe('candidate/fixer',);
          },
        },),
      ],
    },),

    describe({
      name: winnerChangedText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads the unchanged winner as no change, which is the ordinary case',
          fn: async () => {
            expect(winnerChangedText({
              winner: UNCHANGED,
              incumbentText: UNCHANGED.text,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'reads a patched winner carrying different wording as a change',
          fn: async () => {
            expect(winnerChangedText({
              winner: {
                candidateId: 'candidate/chunk-0',
                text: 'The cat is asleep in the sun.',
                measurements: UNCHANGED_MEASUREMENTS,
              },
              incumbentText: UNCHANGED.text,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'reads a patched winner whose text equals the archive as NO change, which is the case '
            + 'the patch gate cannot catch: it refuses an operation that rewrites its own region to '
            + 'itself, and two operations in adjacent envelopes can each change their own region while '
            + 'concatenating back to the archive text. A slice nothing happened in must not be counted '
            + 'as one this lane changed',
          fn: async () => {
            expect(winnerChangedText({
              winner: {
                candidateId: 'candidate/chunk-0',
                text: UNCHANGED.text,
                measurements: UNCHANGED_MEASUREMENTS,
              },
              incumbentText: UNCHANGED.text,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'reads the TEXT rather than the identifier: a winner labelled unchanged while carrying '
            + 'other wording is a change, because the document would carry that wording. An earlier '
            + 'version short-circuited on the identifier, which is the same answer only while the '
            + 'slate is honest, and answering `no change` over text that moved hides a rewrite from '
            + 'every index set',
          fn: async () => {
            expect(winnerChangedText({
              winner: {
                candidateId: UNCHANGED_CANDIDATE_ID,
                text: 'The cat naps in the warm sun.',
                measurements: UNCHANGED_MEASUREMENTS,
              },
              incumbentText: UNCHANGED.text,
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
