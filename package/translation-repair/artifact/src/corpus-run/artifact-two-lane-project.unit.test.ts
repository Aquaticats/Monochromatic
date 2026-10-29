/**
 Tests for rebuilding live pipeline values as the values version 2 writes.

 WHY THIS MODULE EXISTS AT ALL, and what these cases have to check. The frozen
 vocabulary was introduced with the claim that the compiler enforces it,
 because the builder assigns live values into the frozen types. Half of that
 is true. A live union that GAINS A MEMBER does fail to assign. A live record
 that GAINS A FIELD does not: excess property checking applies to object
 literals, not to values flowing through a variable, so the wider object
 assigns cleanly and `JSON.stringify` writes the new field into every
 artifact, and the version 2 parser then refuses the writer's own output.

 SO THE CENTRAL CASES ARE KEY LISTS, not values. A projection that copied its
 input would pass every value assertion here and still carry a field the
 schema does not describe, which is the exact failure the rebuilding exists to
 stop. Each projection gets a case handing it a live value carrying an extra
 field, asserting the result's own keys.

 THE COPY IS ALSO CHECKED. `undecidedLanes` is copied rather than aliased
 because the artifact outlives the run, and a reader mutating what it read
 would otherwise reach into the comparison the builder returned. A case
 mutates the source afterwards and reads the projection back.

 `pass-entry-artifact.ts` is the only caller and it builds whole artifacts, so every
 rule here reached the suite as a settled file that happened to parse.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SliceLaneComparison, } from '@monochromatic-dev/translation-repair-lane-contest';
import type {
  LaneSliceOutcome,
  SliceDelivery,
  SliceDeliveryRecord,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  toArtifactComparisonRow,
  toArtifactDecisions,
  toArtifactDelivery,
  toArtifactOutcome,
  toArtifactRow,
} from '../../dist/final/node/index.mjs';

/**
 Original of the slice every fixture here is about.
 */
const SOURCE_SILL = '猫猫在窗台上睡觉。';

/**
 Archive's own English for it.
 */
const ARCHIVE_SILL = 'The cat sleeps on the sill.';

/**
 Wording a lane decided on.
 */
const DECIDED_SILL = 'The cat naps on the windowsill.';

/**
 Lists an object's own keys in a stable order, which is what a projection
 dropping an unwanted field is checked by.

 @param value - projected record

 @returns Own keys, sorted

 @example
 ```ts
 expect(keysOf({ value: projected, },),).toEqual(['kind',],);
 ```
 */
function keysOf({ value, }: { readonly value: object; },): readonly string[] {
  return Object.keys(value,)
    .toSorted();
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: toArtifactOutcome.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'carries a decided outcome`s accepted wording, which is the only '
            + 'outcome that has anything to carry',
          fn: async () => {
            expect(toArtifactOutcome({
              outcome: {
                kind: 'decided',
                acceptedText: DECIDED_SILL,
              },
            },),)
              .toEqual({
                kind: 'decided',
                acceptedText: DECIDED_SILL,
              },);
          },
        },),

        it({
          name: 'rebuilds each of the four outcomes that carry nothing but their '
            + 'name, so a case per arm rather than a case per shape',
          fn: async () => {
            expect([
              'not-evaluated',
              'unfilled',
              'incumbent-fallback',
              'not-applicable',
            ].map(function project(kind,): unknown {
              return toArtifactOutcome({ outcome: { kind, } as LaneSliceOutcome, },);
            },),)
              .toEqual([
                { kind: 'not-evaluated', },
                { kind: 'unfilled', },
                { kind: 'incumbent-fallback', },
                { kind: 'not-applicable', },
              ],);
          },
        },),

        it({
          name: 'CARRIES THE FLOOR MARK of an unfilled outcome and of no other, so the artifact tells a slice the '
            + 'select bench never bought a candidate for from one the panel declined',
          fn: async () => {
            expect({
              marked: toArtifactOutcome({
                outcome: {
                  kind: 'unfilled',
                  benchBelowFloor: true,
                },
              },),
              declined: toArtifactOutcome({ outcome: { kind: 'unfilled', }, },),
            },)
              .toEqual({
                marked: {
                  kind: 'unfilled',
                  benchBelowFloor: true,
                },
                declined: { kind: 'unfilled', },
              },);
          },
        },),

        it({
          name: 'DROPS a field the live outcome carries and this schema does not '
            + 'describe, which no assignment would have done and is the whole '
            + 'reason these projections rebuild through literals',
          fn: async () => {
            expect(keysOf({
              value: toArtifactOutcome({
                outcome: {
                  kind: 'decided',
                  acceptedText: DECIDED_SILL,
                  whiskerCount: 12,
                } as LaneSliceOutcome,
              },),
            },),)
              .toEqual([
                'acceptedText',
                'kind',
              ],);
          },
        },),

        it({
          name: 'REFUSES an outcome member no projection describes, naming which '
            + 'union it was reading. Unreachable while the arms stay exhaustive, '
            + 'and reached here by casting past the never binding that guarantees '
            + 'they do',
          fn: async () => {
            const refusalOfNewMember = caught(function newMember() {
              toArtifactOutcome({ outcome: { kind: 'napped', } as unknown as LaneSliceOutcome, },);
            },);

            expect(refusalOfNewMember,).toBeInstanceOf(Error,);
            expect((refusalOfNewMember as Error).message,).toContain('lane outcome',);
          },
        },),

        it({
          name: 'NAMES THE FIELDS of an unknown member and none of their values. The '
            + 'unions this guards carry text, and the message reaches the pass '
            + 'stdout through the refusal path, so a stringified member would print '
            + 'corpus wording on the day a union grows',
          fn: async () => {
            const refusalOfNewMember = caught(function newMember() {
              toArtifactOutcome({
                outcome: {
                  kind: 'napped',
                  acceptedText: DECIDED_SILL,
                } as unknown as LaneSliceOutcome,
              },);
            },);

            /**
             What the refusal says.
             */
            const said = (refusalOfNewMember as Error).message;

            expect(said,).toContain('kind napped',);
            expect(said,).toContain('fields [acceptedText, kind]',);
            expect(said.includes(DECIDED_SILL,),).toBe(false,);
          },
        },),

        it({
          name: 'NAMES ONLY THE TYPE of a member that is not an object at all, which has no fields to list',
          fn: async () => {
            const refusalOfNewMember = caught(function newMember() {
              toArtifactOutcome({ outcome: DECIDED_SILL as unknown as LaneSliceOutcome, },);
            },);

            /**
             What the refusal says.
             */
            const said = (refusalOfNewMember as Error).message;

            expect(said,).toContain('lane outcome carries a member version 2 does not describe: a string',);
            expect(said.includes(DECIDED_SILL,),).toBe(false,);
          },
        },),

        it({
          name: 'SAYS A MEMBER CARRIES NO KIND rather than reading a missing discriminant as one, and still names '
            + 'its fields and none of their values',
          fn: async () => {
            const refusalOfNewMember = caught(function newMember() {
              toArtifactOutcome({
                outcome: { acceptedText: DECIDED_SILL, } as unknown as LaneSliceOutcome,
              },);
            },);

            /**
             What the refusal says.
             */
            const said = (refusalOfNewMember as Error).message;

            expect(said,).toContain('kind (no kind), fields [acceptedText]',);
            expect(said.includes(DECIDED_SILL,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: toArtifactDelivery.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'carries a withdrawal`s reason, which is what separates the '
            + 'assembly guard taking one back from a run refused as a whole',
          fn: async () => {
            expect(toArtifactDelivery({
              delivery: {
                kind: 'replacement-withdrawn',
                reason: 'assembly-integrity',
              },
            },),)
              .toEqual({
                kind: 'replacement-withdrawn',
                reason: 'assembly-integrity',
              },);
          },
        },),

        it({
          name: 'rebuilds each of the three deliveries that carry nothing but '
            + 'their name',
          fn: async () => {
            expect([
              'replacement-shipped',
              'incumbent-retained',
              'gap-remains',
            ].map(function project(kind,): unknown {
              return toArtifactDelivery({ delivery: { kind, } as SliceDelivery, },);
            },),)
              .toEqual([
                { kind: 'replacement-shipped', },
                { kind: 'incumbent-retained', },
                { kind: 'gap-remains', },
              ],);
          },
        },),

        it({
          name: 'DROPS a field the live delivery carries and this schema does not '
            + 'describe',
          fn: async () => {
            expect(keysOf({
              value: toArtifactDelivery({
                delivery: {
                  kind: 'replacement-withdrawn',
                  reason: 'assembly-integrity',
                  withdrawnAt: 4,
                } as SliceDelivery,
              },),
            },),)
              .toEqual([
                'kind',
                'reason',
              ],);
          },
        },),

        it({
          name: 'REFUSES a delivery member no projection describes, naming which '
            + 'union it was reading',
          fn: async () => {
            const refusalOfNewMember = caught(function newMember() {
              toArtifactDelivery({ delivery: { kind: 'napped', } as unknown as SliceDelivery, },);
            },);

            expect(refusalOfNewMember,).toBeInstanceOf(Error,);
            expect((refusalOfNewMember as Error).message,).toContain('slice delivery',);
          },
        },),
      ],
    },),

    describe({
      name: toArtifactDecisions.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'carries a comparable reading`s verdict',
          fn: async () => {
            expect(toArtifactDecisions({
              decisionComparison: {
                kind: 'comparable',
                verdict: 'same',
              },
            },),)
              .toEqual({
                kind: 'comparable',
                verdict: 'same',
              },);
          },
        },),

        it({
          name: 'carries which lanes decided nothing, in the order the live '
            + 'reading gave them',
          fn: async () => {
            expect(toArtifactDecisions({
              decisionComparison: {
                kind: 'not-comparable',
                undecidedLanes: [
                  'repair',
                  'translate',
                ],
              },
            },),)
              .toEqual({
                kind: 'not-comparable',
                undecidedLanes: [
                  'repair',
                  'translate',
                ],
              },);
          },
        },),

        it({
          name: 'COPIES the undecided lanes rather than aliasing them, since the '
            + 'artifact outlives the run and a reader mutating what it read would '
            + 'otherwise reach into the comparison the builder returned',
          fn: async () => {
            /**
             Live list the projection is handed, mutated after it returns.
             */
            const lanes: ('repair' | 'translate')[] = ['repair',];

            /**
             Projection taken before the mutation.
             */
            const projected = toArtifactDecisions({
              decisionComparison: {
                kind: 'not-comparable',
                undecidedLanes: lanes,
              },
            },);
            lanes.push('translate',);

            expect(projected,)
              .toEqual({
                kind: 'not-comparable',
                undecidedLanes: ['repair',],
              },);
          },
        },),

        it({
          name: 'DROPS a field the live reading carries and this schema does not '
            + 'describe',
          fn: async () => {
            expect(keysOf({
              value: toArtifactDecisions({
                decisionComparison: {
                  kind: 'comparable',
                  verdict: 'different',
                  decidedBy: 'the panel',
                } as SliceLaneComparison['decisionComparison'],
              },),
            },),)
              .toEqual([
                'kind',
                'verdict',
              ],);
          },
        },),

        it({
          name: 'REFUSES a decision reading no projection describes, naming which '
            + 'union it was reading',
          fn: async () => {
            const refusalOfNewMember = caught(function newMember() {
              toArtifactDecisions({
                decisionComparison: { kind: 'napped', } as unknown as SliceLaneComparison['decisionComparison'],
              },);
            },);

            expect(refusalOfNewMember,).toBeInstanceOf(Error,);
            expect((refusalOfNewMember as Error).message,).toContain('decision comparison',);
          },
        },),
      ],
    },),

    describe({
      name: toArtifactRow.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'rebuilds a ledger row as the seven fields this schema describes '
            + 'and NOTHING ELSE, so a live record that grew a field does not write '
            + 'that field into every artifact',
          fn: async () => {
            /**
             Live row carrying a field version 2 knows nothing about.
             */
            const record = {
              sliceIndex: 3,
              sourceText: SOURCE_SILL,
              incumbentKind: 'present',
              incumbentText: ARCHIVE_SILL,
              outcome: {
                kind: 'decided',
                acceptedText: DECIDED_SILL,
              },
              shippedText: DECIDED_SILL,
              delivery: { kind: 'replacement-shipped', },
              roundsSpent: 2,
            } as SliceDeliveryRecord;

            expect(toArtifactRow({ record, },),)
              .toEqual({
                sliceIndex: 3,
                sourceText: SOURCE_SILL,
                incumbentKind: 'present',
                incumbentText: ARCHIVE_SILL,
                outcome: {
                  kind: 'decided',
                  acceptedText: DECIDED_SILL,
                },
                shippedText: DECIDED_SILL,
                delivery: { kind: 'replacement-shipped', },
              },);
            expect(keysOf({ value: toArtifactRow({ record, },), },),)
              .toEqual([
                'delivery',
                'incumbentKind',
                'incumbentText',
                'outcome',
                'shippedText',
                'sliceIndex',
                'sourceText',
              ],);
          },
        },),
      ],
    },),

    describe({
      name: toArtifactComparisonRow.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RENAMES the live `verdict` to `laneRelation`, which is a rename '
            + 'at the artifact boundary only: the collision existed in the JSON, '
            + 'where a bare key sat beside a sibling of the same name',
          fn: async () => {
            /**
             Live comparison row carrying a field version 2 knows nothing about.
             */
            const row = {
              sliceIndex: 1,
              incumbentKind: 'present',
              incumbentText: ARCHIVE_SILL,
              repairText: ARCHIVE_SILL,
              translateText: DECIDED_SILL,
              verdict: 'translate-only',
              repairOutcome: { kind: 'incumbent-fallback', },
              translateOutcome: {
                kind: 'decided',
                acceptedText: DECIDED_SILL,
              },
              decisionComparison: {
                kind: 'comparable',
                verdict: 'different',
              },
              repairDelivery: { kind: 'incumbent-retained', },
              translateDelivery: { kind: 'replacement-shipped', },
              comparedAt: 'yesterday',
            } as SliceLaneComparison;

            /**
             Row as version 2 records it.
             */
            const projected = toArtifactComparisonRow({ row, },);

            expect((projected as { readonly laneRelation: string; }).laneRelation,)
              .toBe('translate-only',);
            expect(keysOf({ value: projected, },),)
              .toEqual([
                'decisionComparison',
                'incumbentKind',
                'incumbentText',
                'laneRelation',
                'repairDelivery',
                'repairOutcome',
                'repairText',
                'sliceIndex',
                'translateDelivery',
                'translateOutcome',
                'translateText',
              ],);
          },
        },),

        it({
          name: 'projects BOTH lanes` outcomes and BOTH lanes` deliveries rather '
            + 'than passing either through, so a live member growing on one side '
            + 'cannot reach an artifact by riding the other',
          fn: async () => {
            /**
             Live row whose two lanes each carry an undescribed field.
             */
            const row = {
              sliceIndex: 0,
              incumbentKind: 'absent',
              incumbentText: '',
              repairText: '',
              translateText: DECIDED_SILL,
              verdict: 'translate-only',
              repairOutcome: {
                kind: 'not-applicable',
                reasonCode: 7,
              },
              translateOutcome: {
                kind: 'decided',
                acceptedText: DECIDED_SILL,
                producedBy: 'a cat',
              },
              decisionComparison: {
                kind: 'not-comparable',
                undecidedLanes: ['repair',],
              },
              repairDelivery: {
                kind: 'gap-remains',
                noticedAt: 3,
              },
              translateDelivery: {
                kind: 'replacement-shipped',
                shippedAt: 4,
              },
            } as SliceLaneComparison;

            /**
             Row as version 2 records it.
             */
            const projected = toArtifactComparisonRow({ row, },) as {
              readonly repairOutcome: object;
              readonly translateOutcome: object;
              readonly repairDelivery: object;
              readonly translateDelivery: object;
            };

            expect(keysOf({ value: projected.repairOutcome, },),).toEqual(['kind',],);
            expect(keysOf({ value: projected.translateOutcome, },),)
              .toEqual([
                'acceptedText',
                'kind',
              ],);
            expect(keysOf({ value: projected.repairDelivery, },),).toEqual(['kind',],);
            expect(keysOf({ value: projected.translateDelivery, },),).toEqual(['kind',],);
          },
        },),
      ],
    },),
  ],
},);
