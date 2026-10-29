/**
 Tests for the reviewed calibration request check, which refuses an unreviewed
 request before any corpus or provider work, metadata-only preflight included.

 Every refusal case names the boundary that fired and the identifier the
 message names, and asserts the whole message, so a refusal raised by another
 check fails the case that expects this one. Fixtures are cat-themed
 invention; the default-manifest cases read the checked-in manifest's
 metadata only, never a corpus.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { FidelityDamageKind, } from '@monochromatic-dev/translation-repair-repair';
import {
  type FidelityReferenceSpec,
  REVIEWED_FIDELITY_REFERENCES,
  reviewedFidelityRequest,
} from '../dist/final/node/index.mjs';

import {
  expectRefusal,
  reviewedFixture,
} from './fidelity-reference.test-fixture.ts';

//region Request fixtures
// A metadata-only request whose corpus inputs belong to the invented fixture.

/**
 Arguments `reviewedFidelityRequest` takes.
 */
type Request = Parameters<typeof reviewedFidelityRequest>[0];

/**
 The invented spec and a valid zero-call request over it.

 @returns Spec and request that selects it

 @example
 ```ts
 const { spec, request, } = requestFixture();
 ```
 */
function requestFixture(): {
  readonly spec: FidelityReferenceSpec;
  readonly request: Request;
} {
  /**
   Invented reviewed reference.
   */
  const { spec, } = reviewedFixture();
  return {
    spec,
    request: {
      specs: [spec,],
      corpusSha: spec.corpusSha,
      onlyEntryIds: [],
      damageKinds: ['deletion',],
      judgeModelIds: ['independent-judge',],
      cap: 0,
      withContext: false,
    },
  };
}

/**
 Runs a request that must be refused, handing back what it threw.

 @param request - request to make

 @returns What the check threw

 @throws {@link Error} when the check accepts the request

 @example
 ```ts
 const refusal = refusalOfRequest({ request, },);
 ```
 */
function refusalOfRequest({ request, }: { readonly request: Request; },): unknown {
  return caught(function check(): unknown {
    return reviewedFidelityRequest(request,);
  },);
}

/**
 One invalid trial cap.
 */
type CapRow = {
  /**
   Cap handed in.
   */
  readonly cap: number;

  /**
   Cap as the case name spells it.
   */
  readonly spelled: string;
};

/**
 Caps the check refuses: below zero, fractional, not a number, unbounded, and one past the largest safe integer.
 */
const INVALID_CAPS: readonly CapRow[] = [
  {
    cap: -1,
    spelled: '-1',
  },
  {
    cap: 0.5,
    spelled: '0.5',
  },
  {
    cap: Number.NaN,
    spelled: 'NaN',
  },
  {
    cap: Number.POSITIVE_INFINITY,
    spelled: 'Infinity',
  },
  {
    cap: Number.MAX_SAFE_INTEGER + 1,
    spelled: 'one past the largest safe integer',
  },
];

/**
 One judge roster the check refuses.
 */
type RosterRow = {
  /**
   Roster as the case name spells it.
   */
  readonly spelled: string;

  /**
   Roster handed in.
   */
  readonly judgeModelIds: readonly string[];
};

/**
 Rosters with no judge, a repeated judge or a blank judge.
 */
const INVALID_ROSTERS: readonly RosterRow[] = [
  {
    spelled: 'no judge',
    judgeModelIds: [],
  },
  {
    spelled: 'one judge listed twice',
    judgeModelIds: [
      'judge',
      'judge',
    ],
  },
  {
    spelled: 'a judge with an empty identity',
    judgeModelIds: ['',],
  },
  {
    spelled: 'a judge whose identity is spaces only',
    judgeModelIds: [' ',],
  },
];

/**
 One requested family list the check refuses on its own.
 */
type KindsRow = {
  /**
   Families as the case name spells them.
   */
  readonly spelled: string;

  /**
   Families handed in.
   */
  readonly damageKinds: readonly FidelityDamageKind[];
};

/**
 Family lists with no family or a repeated family.
 */
const INVALID_KINDS: readonly KindsRow[] = [
  {
    spelled: 'no family',
    damageKinds: [],
  },
  {
    spelled: 'one family listed twice',
    damageKinds: [
      'deletion',
      'deletion',
    ],
  },
];

//endregion Request fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: reviewedFidelityRequest.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS a copy of each selected specification for a zero-call preflight',
          fn: async () => {
            const { spec, request, } = requestFixture();
            const selected = reviewedFidelityRequest(request,);
            expect(selected,).toEqual([spec,],);
            expect(selected[0],).not.toBe(spec,);
          },
        },),
        it({
          name: 'ACCEPTS a positive trial cap and the largest safe integer',
          fn: async () => {
            const { spec, request, } = requestFixture();
            expect(reviewedFidelityRequest({
              ...request,
              cap: 12,
            },),).toEqual([spec,],);
            expect(reviewedFidelityRequest({
              ...request,
              cap: Number.MAX_SAFE_INTEGER,
            },),).toEqual([spec,],);
          },
        },),
        ...INVALID_CAPS.map(function capCase(row,) {
          return it({
            name: `REFUSES a trial cap of ${row.spelled} at the request boundary, before corpus or provider work`,
            fn: async () => {
              const { request, } = requestFixture();
              expectRefusal({
                refusal: refusalOfRequest({
                  request: {
                    ...request,
                    cap: row.cap,
                  },
                },),
                referenceId: 'trial cap',
                operation: 'request',
              },);
            },
          },);
        },),
        ...INVALID_ROSTERS.map(function rosterCase(row,) {
          return it({
            name: `REFUSES a judge roster of ${row.spelled} at the request boundary`,
            fn: async () => {
              const { request, } = requestFixture();
              expectRefusal({
                refusal: refusalOfRequest({
                  request: {
                    ...request,
                    judgeModelIds: row.judgeModelIds,
                  },
                },),
                referenceId: 'judge roster',
                operation: 'request',
              },);
            },
          },);
        },),
        ...INVALID_KINDS.map(function kindsCase(row,) {
          return it({
            name: `REFUSES a request for ${row.spelled} at the request boundary`,
            fn: async () => {
              const { request, } = requestFixture();
              expectRefusal({
                refusal: refusalOfRequest({
                  request: {
                    ...request,
                    damageKinds: row.damageKinds,
                  },
                },),
                referenceId: 'damage selection',
                operation: 'request',
              },);
            },
          },);
        },),
        it({
          name: 'REFUSES requested context at the request boundary, since no context was reviewed',
          fn: async () => {
            const { request, } = requestFixture();
            expectRefusal({
              refusal: refusalOfRequest({
                request: {
                  ...request,
                  withContext: true,
                },
              },),
              referenceId: 'unreviewed context',
              operation: 'request',
            },);
          },
        },),
        ...['fixture-author', 'provider/fixture-author',].map(function authorCase(author,) {
          return it({
            name: `REFUSES ${author} as judge, since fixture-author wrote the reference's corrections, naming the `
              + 'reference',
            fn: async () => {
              const { request, } = requestFixture();
              expectRefusal({
                refusal: refusalOfRequest({
                  request: {
                    ...request,
                    judgeModelIds: [author,],
                  },
                },),
                referenceId: 'invented-reference',
                operation: 'request',
              },);
            },
          },);
        },),
        it({
          name: 'KEEPS a judge whose identity only ends with a correction author\'s name, without a slash before it',
          fn: async () => {
            const { spec, request, } = requestFixture();
            expect(reviewedFidelityRequest({
              ...request,
              judgeModelIds: [
                'other-fixture-author',
                'provider/other-fixture-author',
              ],
            },),).toEqual([spec,],);
          },
        },),
        it({
          name: 'REFUSES a request whose only reviewed family is not the one requested, naming the family',
          fn: async () => {
            const { spec, request, } = requestFixture();
            expectRefusal({
              refusal: refusalOfRequest({
                request: {
                  ...request,
                  damageKinds: ['alteration',],
                  specs: [{
                    ...spec,
                    damages: spec.damages.filter((damage,) => damage.kind === 'deletion',),
                  },],
                },
              },),
              referenceId: 'damage selection (alteration)',
              operation: 'request',
            },);
          },
        },),
        it({
          name: 'REFUSES a mixed request when one requested family is reviewed nowhere in the selection, naming '
            + 'only that family',
          fn: async () => {
            const { spec, request, } = requestFixture();
            expectRefusal({
              refusal: refusalOfRequest({
                request: {
                  ...request,
                  damageKinds: [
                    'deletion',
                    'alteration',
                  ],
                  specs: [{
                    ...spec,
                    damages: spec.damages.filter((damage,) => damage.kind === 'deletion',),
                  },],
                },
              },),
              referenceId: 'damage selection (alteration)',
              operation: 'request',
            },);
          },
        },),
        it({
          name: 'NAMES every family reviewed nowhere in the selection, in the order requested, joined by a comma '
            + 'and a space',
          fn: async () => {
            const { spec, request, } = requestFixture();
            expectRefusal({
              refusal: refusalOfRequest({
                request: {
                  ...request,
                  damageKinds: [
                    'alteration',
                    'deletion',
                    'insertion',
                  ],
                  specs: [{
                    ...spec,
                    damages: spec.damages.filter((damage,) => damage.kind === 'deletion',),
                  },],
                },
              },),
              referenceId: 'damage selection (alteration, insertion)',
              operation: 'request',
            },);
          },
        },),
        it({
          name: 'KEEPS a family reviewed by any one selected reference, since the request is of the population',
          fn: async () => {
            const { spec, request, } = requestFixture();
            const deletionOnly = {
              ...spec,
              id: 'deletion-only',
              entryId: 'second-cat',
              damages: spec.damages.filter((damage,) => damage.kind === 'deletion',),
            };
            expect(reviewedFidelityRequest({
              ...request,
              damageKinds: [
                'deletion',
                'alteration',
              ],
              specs: [
                deletionOnly,
                spec,
              ],
            },),).toEqual([
              deletionOnly,
              spec,
            ],);
          },
        },),
        it({
          name: 'REFUSES a different corpus revision at the pin boundary, naming the reference it was reviewed for',
          fn: async () => {
            const { request, } = requestFixture();
            expectRefusal({
              refusal: refusalOfRequest({
                request: {
                  ...request,
                  corpusSha: 'different-pin',
                },
              },),
              referenceId: 'invented-reference',
              operation: 'pin',
            },);
          },
        },),
        it({
          name: 'NAMES the first selected reference whose review is for another corpus revision',
          fn: async () => {
            const { spec, request, } = requestFixture();
            const elsewhere = {
              ...spec,
              id: 'elsewhere-reference',
              entryId: 'second-cat',
              corpusSha: 'b'.repeat(spec.corpusSha.length,),
            };
            expectRefusal({
              refusal: refusalOfRequest({
                request: {
                  ...request,
                  specs: [
                    spec,
                    elsewhere,
                  ],
                },
              },),
              referenceId: 'elsewhere-reference',
              operation: 'pin',
            },);
          },
        },),
        it({
          name: 'KEEPS only the entries named by the filter and refuses the filter\'s unreviewed entries',
          fn: async () => {
            const { spec, request, } = requestFixture();
            const second = {
              ...spec,
              id: 'second',
              entryId: 'second-cat',
            };
            expect(reviewedFidelityRequest({
              ...request,
              specs: [
                spec,
                second,
              ],
              onlyEntryIds: ['second-cat',],
            },),).toEqual([second,],);
            expectRefusal({
              refusal: refusalOfRequest({
                request: {
                  ...request,
                  onlyEntryIds: ['unreviewed',],
                },
              },),
              referenceId: 'unreviewed',
              operation: 'request',
            },);
          },
        },),
        it({
          name: 'REFUSES a manifest with no references at the request boundary, naming the manifest',
          fn: async () => {
            const { request, } = requestFixture();
            expectRefusal({
              refusal: refusalOfRequest({
                request: {
                  ...request,
                  specs: [],
                },
              },),
              referenceId: 'manifest',
              operation: 'request',
            },);
          },
        },),
        it({
          name: 'READS the checked-in manifest when none is given, selecting every reference for its own revision',
          fn: async () => {
            const first = nonNullishOrThrow(REVIEWED_FIDELITY_REFERENCES[0],);
            expect(reviewedFidelityRequest({
              corpusSha: first.corpusSha,
              onlyEntryIds: [],
              damageKinds: ['deletion',],
              judgeModelIds: ['independent-judge',],
              cap: 0,
              withContext: false,
            },),).toEqual(REVIEWED_FIDELITY_REFERENCES,);
          },
        },),
        it({
          name: 'REFUSES the checked-in manifest\'s own correction author as judge, naming the reference they corrected',
          fn: async () => {
            const corrected = nonNullishOrThrow(REVIEWED_FIDELITY_REFERENCES.find((spec,) => spec.edits.length > 0,),);
            const { author, } = nonNullishOrThrow(corrected.edits[0],);
            expectRefusal({
              refusal: refusalOfRequest({
                request: {
                  corpusSha: corrected.corpusSha,
                  onlyEntryIds: [],
                  damageKinds: ['deletion',],
                  judgeModelIds: [author,],
                  cap: 0,
                  withContext: false,
                },
              },),
              referenceId: corrected.id,
              operation: 'request',
            },);
          },
        },),
      ],
    },),
  ],
},);
