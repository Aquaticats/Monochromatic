/**
 Tests for critic attribution, the record of WHICH critic raised each claim.

 The distinction these cases exist to protect is between one critic repeating
 itself and several critics agreeing. Both look like extra emissions of the
 same claim, and a flat list of model ids renders them identically, but they
 mean opposite things: repetition is one voice being noisy, agreement is
 independent support. Measured over 12 settled entries, 83.1% of accepted
 issues rest on a single deduplicated claim, so this is the common case rather
 than an edge one.

 Attribution is calibration data only. It must never reach adjudication, which
 is provenance-blind by design because a real defect can arrive with exactly
 one proposer.

 Fixtures are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildSliceCriticRecords,
  collectClaimAttributions,
  retainAttributions,
  type ClaimEmission,
} from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Critic that finds the sunbathing omission.
 */
const TABBY = SEAT_SYNTHETIC_TEXT_EVERYWHERE;

/**
 Second critic, for independent-support cases.
 */
const CALICO = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Third critic, ordered before both by model id so sorting is observable.
 */
const BENGAL = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Claim the cases attribute.
 */
const NAP_CLAIM = 'issue/nap';

/**
 Second claim identity.
 */
const PURR_CLAIM = 'issue/purr';

/**
 Builds an emission list without repeating the object shape in every case.

 @param pairs - claim id and model id in emission order

 @returns Emissions ready to fold

 @example
 ```ts
 const emissions = emissionsOf([[NAP_CLAIM, TABBY,],],);
 ```
 */
function emissionsOf(
  pairs: readonly (readonly [string, string,])[],
): readonly ClaimEmission[] {
  return pairs.map(function toEmission([claimId, modelId,],) {
    return {
      claimId,
      modelId,
    } as ClaimEmission;
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: collectClaimAttributions.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'records ONE proposer with an emission count of two when a single '
            + 'critic emits the same claim twice, rather than two proposers, because '
            + 'a critic repeating itself is not a second opinion and counting it as '
            + 'one would overstate independent support for the claim',
          fn: async () => {
            /**
             Attribution for one critic saying the same thing twice.
             */
            const attributions = collectClaimAttributions({
              emissions: emissionsOf([
                [NAP_CLAIM, TABBY,],
                [NAP_CLAIM, TABBY,],
              ],),
            },);

            expect(attributions,).toHaveLength(1,);
            expect(attributions[0]?.proposers,).toHaveLength(1,);
            expect(attributions[0]?.proposers[0]?.modelId,).toBe(TABBY,);
            expect(attributions[0]?.proposers[0]?.emissionCount,).toBe(2,);
          },
        },),

        it({
          name: 'records TWO proposers when two critics emit the identical claim, '
            + 'which is the case deduplication destroys downstream: aggregateClaims '
            + 'collapses structurally identical claims to one id, so unless the '
            + 'second emitter is recorded here it is unrecoverable afterward',
          fn: async () => {
            /**
             Attribution for two critics agreeing exactly.
             */
            const attributions = collectClaimAttributions({
              emissions: emissionsOf([
                [NAP_CLAIM, TABBY,],
                [NAP_CLAIM, CALICO,],
              ],),
            },);

            expect(attributions,).toHaveLength(1,);
            expect(attributions[0]?.proposers,).toHaveLength(2,);
            for (const proposer of attributions[0]?.proposers ?? [])
              expect(proposer.emissionCount,).toBe(1,);
          },
        },),

        it({
          name: 'SORTS proposers by model id in CODE-UNIT order rather than by '
            + 'arrival, so the same claim attributed by the same critics serializes '
            + 'identically into a cached outcome no matter which critic answered '
            + 'first, and identically on two machines: localeCompare would order '
            + 'these three differently under a different default locale',
          fn: async () => {
            /**
             Same critics, opposite arrival orders.
             */
            const first = collectClaimAttributions({
              emissions: emissionsOf([
                [NAP_CLAIM, TABBY,],
                [NAP_CLAIM, BENGAL,],
                [NAP_CLAIM, CALICO,],
              ],),
            },);

            /**
             Reversed arrival, identical content.
             */
            const second = collectClaimAttributions({
              emissions: emissionsOf([
                [NAP_CLAIM, CALICO,],
                [NAP_CLAIM, BENGAL,],
                [NAP_CLAIM, TABBY,],
              ],),
            },);

            expect(JSON.stringify(first,),).toBe(JSON.stringify(second,),);
            expect(
              first[0]?.proposers.map(function toId(proposer,) {
                return proposer.modelId;
              },),
            ).toStrictEqual([BENGAL, TABBY, CALICO,],);
          },
        },),

        it({
          name: 'orders two proposers of one claim ascending by model id when they '
            + 'already arrive in ascending order',
          fn: async () => {
            /**
             Two critics on NAP_CLAIM, emitted in ascending model-id order.
             */
            const attributions = collectClaimAttributions({
              emissions: emissionsOf([
                [NAP_CLAIM, TABBY,],
                [NAP_CLAIM, CALICO,],
              ],),
            },);

            expect(
              attributions[0]?.proposers.map(function toId(proposer,) {
                return proposer.modelId;
              },),
            ).toStrictEqual([TABBY, CALICO,],);
          },
        },),

        it({
          name: 'orders two distinct claims from one critic ascending by id when '
            + 'they already arrive in ascending order',
          fn: async () => {
            /**
             Two claims from TABBY alone, emitted in ascending id order.
             */
            const attributions = collectClaimAttributions({
              emissions: emissionsOf([
                [NAP_CLAIM, TABBY,],
                [PURR_CLAIM, TABBY,],
              ],),
            },);

            expect(
              attributions.map(function toId(attribution,) {
                return attribution.claimId;
              },),
            ).toStrictEqual([NAP_CLAIM, PURR_CLAIM,],);
          },
        },),

        it({
          name: 'SEPARATES distinct claims and orders them by CLAIM ID rather than '
            + 'by emission, so one critic emitting two different claims produces '
            + 'two attributions rather than a merged one. Emission order follows '
            + 'voice ARRIVAL, and gatherStageVoices orders voices by retry round, '
            + 'so insertion order would serialize identical evidence differently '
            + 'depending on which critic happened to answer first',
          fn: async () => {
            /**
             Two distinct claims from overlapping critics.
             */
            const attributions = collectClaimAttributions({
              emissions: emissionsOf([
                [PURR_CLAIM, TABBY,],
                [NAP_CLAIM, CALICO,],
                [PURR_CLAIM, CALICO,],
              ],),
            },);

            expect(
              attributions.map(function toId(attribution,) {
                return attribution.claimId;
              },),
            ).toStrictEqual([NAP_CLAIM, PURR_CLAIM,],);
            expect(attributions[0]?.proposers,).toHaveLength(1,);
            expect(attributions[1]?.proposers,).toHaveLength(2,);
          },
        },),

        it({
          name: 'serializes identically across two runs that heard the SAME '
            + 'critics in different orders while carrying SEVERAL claims. The '
            + 'other determinism case uses one claim, so its outer array always '
            + 'has length one and it cannot detect outer misordering at all; a '
            + 'retry that changes which critic answers first is the real case',
          fn: async () => {
            /**
             One arrival order over three distinct claims.
             */
            const first = collectClaimAttributions({
              emissions: emissionsOf([
                [PURR_CLAIM, TABBY,],
                ['issue/knead', CALICO,],
                [NAP_CLAIM, BENGAL,],
                [PURR_CLAIM, CALICO,],
              ],),
            },);

            /**
             Same evidence, reversed arrival, as a retry round would produce.
             */
            const second = collectClaimAttributions({
              emissions: emissionsOf([
                [PURR_CLAIM, CALICO,],
                [NAP_CLAIM, BENGAL,],
                ['issue/knead', CALICO,],
                [PURR_CLAIM, TABBY,],
              ],),
            },);

            expect(first,).toHaveLength(3,);
            expect(JSON.stringify(first,),).toBe(JSON.stringify(second,),);
          },
        },),

        it({
          name: 'returns nothing for no emissions, which is what a chunk whose '
            + 'critics all lost their voices looks like and is not a fault',
          fn: async () => {
            expect(collectClaimAttributions({ emissions: [], },),).toHaveLength(0,);
          },
        },),
      ],
    },),

    describe({
      name: retainAttributions.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'DROPS attribution for claims a later screen removed, since an '
            + 'entry pointing at a discarded claim would credit a critic with a '
            + 'hit the pipeline threw away',
          fn: async () => {
            /**
             Attribution for two claims, one of which will be screened out.
             */
            const attributions = collectClaimAttributions({
              emissions: emissionsOf([
                [PURR_CLAIM, TABBY,],
                [NAP_CLAIM, CALICO,],
              ],),
            },);

            /**
             Survivors after screening.
             */
            const kept = retainAttributions({
              attributions,
              claimIds: new Set([NAP_CLAIM,],),
            },);

            expect(kept,).toHaveLength(1,);
            expect(kept[0]?.claimId,).toBe(NAP_CLAIM,);
          },
        },),

        it({
          name: 'returns nothing when every claim was screened out, which is what '
            + 'a chunk whose non-translation votes were contradicted looks like',
          fn: async () => {
            /**
             Attribution that loses every claim.
             */
            const attributions = collectClaimAttributions({
              emissions: emissionsOf([[NAP_CLAIM, TABBY,],],),
            },);

            expect(
              retainAttributions({
                attributions,
                claimIds: new Set<string>(),
              },),
            ).toHaveLength(0,);
          },
        },),

        it({
          name: 'preserves the CANONICAL order among survivors, so filtering never '
            + 'reshuffles what the fold already sorted by claim id and a screened '
            + 'run serializes like an unscreened one',
          fn: async () => {
            /**
             Three claims, middle one screened out.
             */
            const attributions = collectClaimAttributions({
              emissions: emissionsOf([
                [PURR_CLAIM, TABBY,],
                ['issue/knead', CALICO,],
                [NAP_CLAIM, BENGAL,],
              ],),
            },);

            expect(
              retainAttributions({
                attributions,
                claimIds: new Set([PURR_CLAIM, NAP_CLAIM,],),
              },).map(function toId(attribution,) {
                return attribution.claimId;
              },),
            ).toStrictEqual([NAP_CLAIM, PURR_CLAIM,],);
          },
        },),
      ],
    },),

    describe({
      name: buildSliceCriticRecords.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CANONICALIZES the nested arrays rather than inheriting their '
            + 'order, so two callers holding the same evidence in different order '
            + 'serialize to identical bytes. Every producer upstream already sorts, '
            + 'which is exactly why this needs a test: the invariant holds today by '
            + 'convention and this is what makes the artifact boundary enforce it',
          fn: async () => {
            /**
             Records whose nested arrays arrive in one order.
             */
            const forward = buildSliceCriticRecords({
              outcomes: [
                {
                  sliceIndex: 1,
                  heardCriticIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
                  claimAttributions: [
                    {
                      claimId: 'issue/aaa',
                      proposers: [
                        { modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, emissionCount: 1, },
                        { modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, emissionCount: 1, },
                      ],
                    },
                    { claimId: 'issue/bbb', proposers: [{ modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, emissionCount: 1, },], },
                  ],
                },
                { sliceIndex: 0, heardCriticIds: [], claimAttributions: [], },
              ],
            },);

            /**
             The same evidence, every nested array reversed.
             */
            const reversed = buildSliceCriticRecords({
              outcomes: [
                { sliceIndex: 0, heardCriticIds: [], claimAttributions: [], },
                {
                  sliceIndex: 1,
                  heardCriticIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
                  claimAttributions: [
                    { claimId: 'issue/bbb', proposers: [{ modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, emissionCount: 1, },], },
                    {
                      claimId: 'issue/aaa',
                      proposers: [
                        { modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, emissionCount: 1, },
                        { modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, emissionCount: 1, },
                      ],
                    },
                  ],
                },
              ],
            },);

            // Byte identity, not deep equality: this value is serialized into a
            // cached artifact, so what matters is that JSON.stringify agrees.
            expect(JSON.stringify(reversed,),).toBe(JSON.stringify(forward,),);
            expect(forward[0]?.sliceIndex,).toBe(0,);
          },
        },),

        it({
          name: 'keeps two proposers of one claim that share a model id, in '
            + 'their original order, when canonicalizing',
          fn: async () => {
            /**
             One claim whose proposers list carries the same model id twice,
             which the type allows even though no deduplicated Map would.
             */
            const records = buildSliceCriticRecords({
              outcomes: [
                {
                  sliceIndex: 0,
                  heardCriticIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
                  claimAttributions: [
                    {
                      claimId: 'issue/nap',
                      proposers: [
                        { modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, emissionCount: 1, },
                        { modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, emissionCount: 3, },
                      ],
                    },
                  ],
                },
              ],
            },);

            expect(
              records[0]?.claimAttributions[0]?.proposers.map(function toCount(proposer,) {
                return proposer.emissionCount;
              },),
            ).toStrictEqual([1, 3,],);
          },
        },),

        it({
          name: 'keeps two claimAttributions that share a claim id, in their '
            + 'original order, when canonicalizing',
          fn: async () => {
            /**
             One slice outcome whose claimAttributions list carries the same
             claim id twice, which the type allows even though no deduplicated
             Map would.
             */
            const records = buildSliceCriticRecords({
              outcomes: [
                {
                  sliceIndex: 0,
                  heardCriticIds: [],
                  claimAttributions: [
                    {
                      claimId: 'issue/nap',
                      proposers: [{ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, emissionCount: 1, },],
                    },
                    {
                      claimId: 'issue/nap',
                      proposers: [{ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, emissionCount: 2, },],
                    },
                  ],
                },
              ],
            },);

            expect(
              records[0]?.claimAttributions.map(function toFirstCount(attribution,) {
                return attribution.proposers[0]?.emissionCount;
              },),
            ).toStrictEqual([1, 2,],);
          },
        },),
      ],
    },),
  ],
},);
