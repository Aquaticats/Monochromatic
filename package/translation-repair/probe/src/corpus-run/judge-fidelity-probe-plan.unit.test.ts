/**
 Tests for what the fidelity probe writes down and says: the bounded matrix,
 the durable plan, one completed row, the kept run and the lines it prints.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { FidelityOutcome, } from '@monochromatic-dev/translation-repair-run';
import {
  boundFidelityMatrix,
  fidelityJudgedLine,
  fidelityKeptLine,
  fidelityKeptRun,
  fidelityPartialWarning,
  fidelityPlan,
  fidelityPreflightLine,
  fidelityRowResult,
  type ReviewedFidelityTrial,
} from '../../dist/final/node/index.mjs';
import { reviewedFixture, } from '../fidelity-reference.test-fixture.ts';

/**
 The invented reviewed reference every row belongs to.
 */
const SPEC = reviewedFixture().spec;

/**
 Build the plan and the kept run claim to have run under.
 */
const DIGEST = 'sha256-tree-v1:00000000000000000000000000000000000000000000000000000000cafef00d';

/**
 Closure the plan and the kept run record.
 */
const CLOSURE = {
  kind: 'read',
  entry: 'judge-fidelity-probe.mjs',
  chunks: ['run-config-abc.mjs',],
} as const;

/**
 Judges the request named.
 */
const JUDGES = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
].map(function toModelId(id,): RosterModelId {
  return id as unknown as RosterModelId;
},);

/**
 Builds one reviewed row of the invented reference.

 @param position - which of the four arrangements the row is

 @returns Row as the matrix holds it

 @example
 ```ts
 const row = rowAt({ position: 0, },);
 ```
 */
function rowAt({ position, }: { readonly position: number; },): ReviewedFidelityTrial {
  return {
    spec: SPEC,
    changedChars: 31,
    damageDetail: 'one sentence removed',
    trial: {
      trialId: `${SPEC.id}/deletion`,
      direction: (position < 2) ? 'preserve' : 'replace',
      damageKind: 'deletion',
      sourceText: '小猫在窗台上睡觉。',
      contextText: '',
      cleanText: 'The cat sleeps on the sill. She purrs.',
      damagedText: 'The cat sleeps on the sill.',
      cleanFirst: (position % 2) === 0,
    },
  };
}

/**
 Four rows, the whole matrix of one reference and one defect.
 */
const MATRIX = [
  rowAt({ position: 0, },),
  rowAt({ position: 1, },),
  rowAt({ position: 2, },),
  rowAt({ position: 3, },),
];

/**
 The row that ran in the cases that keep one.
 */
const FIRST_ROW = rowAt({ position: 0, },);

/**
 What the judges said about the first row.
 */
const OUTCOME: FidelityOutcome = {
  trialId: `${SPEC.id}/deletion`,
  direction: 'preserve',
  damageKind: 'deletion',
  cleanFirst: true,
  verdict: 'clean',
  correct: true,
  ballots: [
    {
      modelId: nonNullishOrThrow(JUDGES[0],),
      picked: 'clean',
      reason: 'covers the purr',
      weight: 1,
    },
    {
      modelId: nonNullishOrThrow(JUDGES[1],),
      picked: 'declined',
      reason: 'cannot tell',
      weight: 1,
    },
  ],
  declineReason: '',
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: boundFidelityMatrix.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE LEADING ROWS A SMALLER CAP ALLOWS and says the matrix is not complete',
          fn: async () => {
            expect(boundFidelityMatrix({
              matrix: MATRIX,
              cap: 2,
            },),).toEqual({
              planned: [
                rowAt({ position: 0, },),
                rowAt({ position: 1, },),
              ],
              complete: false,
            },);
          },
        },),
        it({
          name: 'KEEPS EVERY ROW WHEN THE CAP EQUALS THE MATRIX and says it is complete',
          fn: async () => {
            expect(boundFidelityMatrix({
              matrix: MATRIX,
              cap: 4,
            },),).toEqual({
              planned: MATRIX,
              complete: true,
            },);
          },
        },),
        it({
          name: 'KEEPS EVERY ROW WHEN THE CAP EXCEEDS THE MATRIX and says it is complete',
          fn: async () => {
            expect(boundFidelityMatrix({
              matrix: MATRIX,
              cap: 16,
            },),).toEqual({
              planned: MATRIX,
              complete: true,
            },);
          },
        },),
        it({
          name: 'PLANS NOTHING FROM AN EMPTY MATRIX and calls that complete, since no row was left out',
          fn: async () => {
            expect(boundFidelityMatrix({
              matrix: [],
              cap: 16,
            },),).toEqual({
              planned: [],
              complete: true,
            },);
          },
        },),
      ],
    },),

    describe({
      name: fidelityPreflightLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS ONE SPECIFICATION IN THE SINGULAR, since a count of one takes the singular noun',
          fn: async () => {
            expect(fidelityPreflightLine({ selected: 1, },),).toBe(
              'preflight only: 1 reviewed reference specification selected; no corpus or model calls',
            );
          },
        },),
        it({
          name: 'SAYS HOW MANY SPECIFICATIONS were selected, in the plural, and that nothing was asked',
          fn: async () => {
            expect(fidelityPreflightLine({ selected: 3, },),).toBe(
              'preflight only: 3 reviewed reference specifications selected; no corpus or model calls',
            );
          },
        },),
      ],
    },),

    describe({
      name: fidelityPartialWarning.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS THE ROWS OF THE WHOLE MATRIX in the plural beside the rows planned',
          fn: async () => {
            expect(fidelityPartialWarning({
              planned: 2,
              total: 4,
            },),).toBe('partial reviewed matrix: 2 of 4 rows; not complete admission evidence',);
          },
        },),
        it({
          name: 'COUNTS ONE ROW IN THE SINGULAR when the whole matrix holds one',
          fn: async () => {
            expect(fidelityPartialWarning({
              planned: 0,
              total: 1,
            },),).toBe('partial reviewed matrix: 0 of 1 row; not complete admission evidence',);
          },
        },),
      ],
    },),

    describe({
      name: fidelityPlan.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES THE WHOLE PLAN of a bounded run: the identity, the manifest, the roster and one entry per '
            + 'planned row, each at its position',
          fn: async () => {
            expect(fidelityPlan({
              startedAt: '2026-10-06T06:00:00.000Z',
              pipelineDigest: DIGEST,
              runnerClosure: CLOSURE,
              referenceManifestDigest: 'manifest-digest',
              specs: [SPEC,],
              judgeModelIds: JUDGES,
              matrix: MATRIX,
              bound: boundFidelityMatrix({
                matrix: MATRIX,
                cap: 2,
              },),
            },),).toEqual({
              status: 'planned',
              startedAt: '2026-10-06T06:00:00.000Z',
              pipelineDigest: DIGEST,
              runnerClosure: CLOSURE,
              referenceManifestDigest: 'manifest-digest',
              referenceManifest: [SPEC,],
              requestedRoster: JUDGES,
              completeRequestedMatrix: false,
              fullMatrixRows: 4,
              plannedRows: 2,
              rows: [
                {
                  position: 0,
                  trialId: `${SPEC.id}/deletion`,
                  direction: 'preserve',
                  cleanFirst: true,
                  damageKind: 'deletion',
                },
                {
                  position: 1,
                  trialId: `${SPEC.id}/deletion`,
                  direction: 'preserve',
                  cleanFirst: false,
                  damageKind: 'deletion',
                },
              ],
            },);
          },
        },),
        it({
          name: 'WRITES A COMPLETE PLAN with every row of the matrix and the complete flag set',
          fn: async () => {
            /**
             Plan of the whole matrix.
             */
            const plan = fidelityPlan({
              startedAt: '2026-10-06T06:00:00.000Z',
              pipelineDigest: DIGEST,
              runnerClosure: {
                kind: 'unavailable',
                reason: 'no entry path was given',
              },
              referenceManifestDigest: 'manifest-digest',
              specs: [SPEC,],
              judgeModelIds: JUDGES,
              matrix: MATRIX,
              bound: boundFidelityMatrix({
                matrix: MATRIX,
                cap: 16,
              },),
            },);
            expect(plan.completeRequestedMatrix,).toBe(true,);
            expect(plan.plannedRows,).toBe(4,);
            expect(plan.runnerClosure,).toEqual({
              kind: 'unavailable',
              reason: 'no entry path was given',
            },);
            expect(plan.rows.map((row,) => [
              row.position,
              row.direction,
              row.cleanFirst,
            ],),).toEqual([
              [
                0,
                'preserve',
                true,
              ],
              [
                1,
                'preserve',
                false,
              ],
              [
                2,
                'replace',
                true,
              ],
              [
                3,
                'replace',
                false,
              ],
            ],);
          },
        },),
      ],
    },),

    describe({
      name: fidelityRowResult.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'JOINS THE REFERENCE\'S PROVENANCE TO THE OUTCOME, whole, with the outcome\'s own fields after it',
          fn: async () => {
            expect(fidelityRowResult({
              row: FIRST_ROW,
              outcome: OUTCOME,
            },),).toEqual({
              referenceId: SPEC.id,
              entryId: SPEC.entryId,
              sourceRange: SPEC.source,
              archiveRange: SPEC.archive,
              referenceHash: SPEC.referenceHash,
              changedChars: 31,
              damageDetail: 'one sentence removed',
              ...OUTCOME,
            },);
          },
        },),
      ],
    },),

    describe({
      name: fidelityJudgedLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS ONE ROW IN THE SINGULAR',
          fn: async () => {
            expect(fidelityJudgedLine({ count: 1, },),).toBe(
              'fidelity: 1 reviewed trial row; use individual ballots for per-model calibration',
            );
          },
        },),
        it({
          name: 'COUNTS SEVERAL ROWS IN THE PLURAL',
          fn: async () => {
            expect(fidelityJudgedLine({ count: 4, },),).toBe(
              'fidelity: 4 reviewed trial rows; use individual ballots for per-model calibration',
            );
          },
        },),
      ],
    },),

    describe({
      name: fidelityKeptLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS ONE ROW IN THE SINGULAR beside where the run was kept',
          fn: async () => {
            expect(fidelityKeptLine({
              count: 1,
              keptAt: '/runs/a.json',
            },),).toBe('kept 1 reviewed row at /runs/a.json',);
          },
        },),
        it({
          name: 'COUNTS SEVERAL ROWS IN THE PLURAL beside where the run was kept',
          fn: async () => {
            expect(fidelityKeptLine({
              count: 4,
              keptAt: '/runs/a.json',
            },),).toBe('kept 4 reviewed rows at /runs/a.json',);
          },
        },),
      ],
    },),

    describe({
      name: fidelityKeptRun.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE WHOLE RUN: the plan\'s identity, the roster, the subject with its counts and the rows',
          fn: async () => {
            /**
             The plan a bounded run wrote first.
             */
            const plan = fidelityPlan({
              startedAt: '2026-10-06T06:00:00.000Z',
              pipelineDigest: DIGEST,
              runnerClosure: CLOSURE,
              referenceManifestDigest: 'manifest-digest',
              specs: [SPEC,],
              judgeModelIds: JUDGES,
              matrix: MATRIX,
              bound: boundFidelityMatrix({
                matrix: MATRIX,
                cap: 2,
              },),
            },);
            /**
             The one row that ran.
             */
            const row = fidelityRowResult({
              row: FIRST_ROW,
              outcome: OUTCOME,
            },);
            expect(fidelityKeptRun({
              plan,
              rows: [row,],
              finishedAt: '2026-10-06T06:05:00.000Z',
              promptPayloadDir: '/runs/judge-fidelity-x/payloads',
              corpusPin: 'b'.repeat(40,),
              entriesRequested: ['starlit-cat',],
              trialCap: 2,
              damageKinds: ['deletion',],
            },),).toEqual({
              startedAt: '2026-10-06T06:00:00.000Z',
              finishedAt: '2026-10-06T06:05:00.000Z',
              pipelineDigest: DIGEST,
              runnerClosure: CLOSURE,
              roster: JUDGES,
              subject: {
                status: 'completed',
                completeRequestedMatrix: false,
                fullMatrixRows: 4,
                plannedRows: 2,
                completedRows: 1,
                referenceManifestDigest: 'manifest-digest',
                promptPayloadDir: '/runs/judge-fidelity-x/payloads',
                corpusPin: 'b'.repeat(40,),
                referenceManifest: [SPEC,],
                entriesRequested: ['starlit-cat',],
                trialCap: 2,
                damageKinds: ['deletion',],
                withContext: false,
              },
              rows: [row,],
            },);
          },
        },),
      ],
    },),
  ],
},);
