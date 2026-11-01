/**
 Tests for the lexicographic order that decides which candidate ships.

 `selectRepairCandidate` is already covered, and it exercises these branches
 indirectly, but nothing asserted the ORDER OF THE TIERS themselves. That
 order is a design decision rather than an implementation detail: it encodes
 what the pipeline is willing to trade for what, and that trade is itself an open
 question. A test that pins each tier against every tier
 below it turns a future reordering into a deliberate act instead of a silent
 one.

 Each case holds every HIGHER tier equal and makes every LOWER tier favor the
 loser, so passing can only mean the tier under test decided the comparison.
 Every candidate states all five measurements rather than overriding a shared
 base: the comparison IS the five numbers, and a reader should see both sides
 without merging anything in their head.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  compareCandidates,
  type RepairCandidate,
  UNCHANGED_CANDIDATE_ID,
} from '../dist/final/node/index.mjs';

/**
 Builds a candidate with every measurement stated.

 @param candidateId - handle for this candidate

 @param integrityOk - whether the candidate still parses and keeps conventions

 @param resolvedHighSeverity - critical and major issues confirmed fixed

 @param resolvedTotal - issues of any severity confirmed fixed

 @param regressedKnownIssues - known issues the checkers marked worse

 @param touchedRegionChars - total size of touched regions; smaller is more
 conservative

 @returns Candidate carrying those measurements

 @example
 ```ts
 const intact = candidate({
   candidateId: 'candidate/intact',
   integrityOk: true,
   resolvedHighSeverity: 1,
   resolvedTotal: 2,
   regressedKnownIssues: 0,
   touchedRegionChars: 50,
 },);
 ```
 */
function candidate(
  {
    candidateId,
    integrityOk,
    resolvedHighSeverity,
    resolvedTotal,
    regressedKnownIssues,
    touchedRegionChars,
  }: {
    readonly candidateId: string;
    readonly integrityOk: boolean;
    readonly resolvedHighSeverity: number;
    readonly resolvedTotal: number;
    readonly regressedKnownIssues: number;
    readonly touchedRegionChars: number;
  },
): RepairCandidate {
  return {
    candidateId,
    text: 'The cat sleeps on the windowsill.',
    measurements: {
      integrityOk,
      resolvedHighSeverity,
      resolvedTotal,
      regressedKnownIssues,
      touchedRegionChars,
    },
  };
}

/**
 Asserts that one candidate ranks ahead of another, and that reversing the
 arguments reverses the sign.

 Antisymmetry is checked on every case rather than once, because a comparator
 that is merely inconsistent produces a sort order depending on input order,
 which is exactly the failure a stable-looking pipeline would hide.

 @param better - candidate expected to rank first

 @param worse - candidate expected to rank second

 @example
 ```ts
 expectRanksAhead({ better: intact, worse: broken, },);
 ```
 */
function expectRanksAhead(
  {
    better,
    worse,
  }: {
    readonly better: RepairCandidate;
    readonly worse: RepairCandidate;
  },
): void {
  expect(compareCandidates({
    left: better,
    right: worse,
  },),).toBeLessThan(0,);
  expect(compareCandidates({
    left: worse,
    right: better,
  },),).toBeGreaterThan(0,);
}

await describe({
  name: compareCandidates.name,
  children: [
    it({
      name: 'INTEGRITY outranks everything: a candidate that stopped parsing '
        + 'loses even when it resolves more issues, regresses fewer, and '
        + 'touches less text',
      fn: async () => {
        expectRanksAhead({
          better: candidate({
            candidateId: 'candidate/intact',
            integrityOk: true,
            resolvedHighSeverity: 0,
            resolvedTotal: 0,
            regressedKnownIssues: 3,
            touchedRegionChars: 900,
          },),
          worse: candidate({
            candidateId: 'candidate/broken',
            integrityOk: false,
            resolvedHighSeverity: 9,
            resolvedTotal: 9,
            regressedKnownIssues: 0,
            touchedRegionChars: 1,
          },),
        },);
      },
    },),

    it({
      name: 'HIGH-SEVERITY RESOLUTION outranks regressions, total resolution, '
        + 'and preservation: this is the substantive trade the order encodes, '
        + 'that fixing one more critical or major issue is worth regressing '
        + 'known issues elsewhere',
      fn: async () => {
        expectRanksAhead({
          better: candidate({
            candidateId: 'candidate/high',
            integrityOk: true,
            resolvedHighSeverity: 2,
            resolvedTotal: 2,
            regressedKnownIssues: 5,
            touchedRegionChars: 900,
          },),
          worse: candidate({
            candidateId: 'candidate/low',
            integrityOk: true,
            resolvedHighSeverity: 1,
            resolvedTotal: 9,
            regressedKnownIssues: 0,
            touchedRegionChars: 1,
          },),
        },);
      },
    },),

    it({
      name: 'REGRESSIONS outrank total resolution and preservation, so once '
        + 'high-severity resolution ties, breaking known issues costs more '
        + 'than any number of minor fixes buys',
      fn: async () => {
        expectRanksAhead({
          better: candidate({
            candidateId: 'candidate/clean',
            integrityOk: true,
            resolvedHighSeverity: 1,
            resolvedTotal: 2,
            regressedKnownIssues: 0,
            touchedRegionChars: 900,
          },),
          worse: candidate({
            candidateId: 'candidate/regressing',
            integrityOk: true,
            resolvedHighSeverity: 1,
            resolvedTotal: 9,
            regressedKnownIssues: 1,
            touchedRegionChars: 1,
          },),
        },);
      },
    },),

    it({
      name: 'TOTAL RESOLUTION outranks preservation, so a candidate fixing '
        + 'more minor issues wins even though it changed far more text',
      fn: async () => {
        expectRanksAhead({
          better: candidate({
            candidateId: 'candidate/thorough',
            integrityOk: true,
            resolvedHighSeverity: 1,
            resolvedTotal: 5,
            regressedKnownIssues: 0,
            touchedRegionChars: 900,
          },),
          worse: candidate({
            candidateId: 'candidate/timid',
            integrityOk: true,
            resolvedHighSeverity: 1,
            resolvedTotal: 4,
            regressedKnownIssues: 0,
            touchedRegionChars: 1,
          },),
        },);
      },
    },),

    it({
      name: 'PRESERVATION breaks the remaining tie: with every other '
        + 'measurement equal, the candidate that touched less text wins, which '
        + 'is the conservatism the whole pipeline claims',
      fn: async () => {
        expectRanksAhead({
          better: candidate({
            candidateId: 'candidate/small',
            integrityOk: true,
            resolvedHighSeverity: 1,
            resolvedTotal: 2,
            regressedKnownIssues: 0,
            touchedRegionChars: 10,
          },),
          worse: candidate({
            candidateId: 'candidate/large',
            integrityOk: true,
            resolvedHighSeverity: 1,
            resolvedTotal: 2,
            regressedKnownIssues: 0,
            touchedRegionChars: 11,
          },),
        },);
      },
    },),

    it({
      name: 'UNCHANGED wins a perfect tie against a repair, because shipping '
        + 'an edit that demonstrably proves nothing is worse than shipping '
        + 'nothing at all; and it wins regardless of whether its id sorts '
        + 'before or after the other, so the preference is by identity rather '
        + 'than alphabetical accident',
      fn: async () => {
        // 'candidate/unchanged' sorts AFTER 'candidate/editor-a' but BEFORE
        // 'candidate/zzz-editor'. If the unchanged preference were an artifact
        // of the id tiebreak, one of these two would fail.
        for (const otherId of [
          'candidate/editor-a',
          'candidate/zzz-editor',
        ])
          expectRanksAhead({
            better: candidate({
              candidateId: UNCHANGED_CANDIDATE_ID,
              integrityOk: true,
              resolvedHighSeverity: 0,
              resolvedTotal: 0,
              regressedKnownIssues: 0,
              touchedRegionChars: 0,
            },),
            worse: candidate({
              candidateId: otherId,
              integrityOk: true,
              resolvedHighSeverity: 0,
              resolvedTotal: 0,
              regressedKnownIssues: 0,
              touchedRegionChars: 0,
            },),
          },);
      },
    },),

    it({
      name: 'breaks a tie between two equal repairs by id, so a slate of '
        + 'indistinguishable candidates still sorts the same way every run '
        + 'and the reported winner is reproducible',
      fn: async () => {
        expectRanksAhead({
          better: candidate({
            candidateId: 'candidate/editor-a',
            integrityOk: true,
            resolvedHighSeverity: 1,
            resolvedTotal: 2,
            regressedKnownIssues: 0,
            touchedRegionChars: 50,
          },),
          worse: candidate({
            candidateId: 'candidate/editor-b',
            integrityOk: true,
            resolvedHighSeverity: 1,
            resolvedTotal: 2,
            regressedKnownIssues: 0,
            touchedRegionChars: 50,
          },),
        },);
      },
    },),

    it({
      name: 'returns zero only for candidates sharing an id, which is the one '
        + 'case where no deterministic order exists to find',
      fn: async () => {
        expect(compareCandidates({
          left: candidate({
            candidateId: 'candidate/editor-a',
            integrityOk: true,
            resolvedHighSeverity: 1,
            resolvedTotal: 2,
            regressedKnownIssues: 0,
            touchedRegionChars: 50,
          },),
          right: candidate({
            candidateId: 'candidate/editor-a',
            integrityOk: true,
            resolvedHighSeverity: 1,
            resolvedTotal: 2,
            regressedKnownIssues: 0,
            touchedRegionChars: 50,
          },),
        },),).toBe(0,);
      },
    },),
  ],
},);
