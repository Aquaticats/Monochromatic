/**
 Tests the evidence lines the lane contest is shown from the
 introduced-defect probe.

 THE CASE IS keyword233, 2026-09-03: two probers corroborated that the repair
 editor had moved a deceased person's paragraph into the present tense, the
 repair shipped as the design says, and the lane contest chose it 7 of 7
 without ever seeing the claim. Later that day the naturalness rewrite did
 the same on another draw, three probers corroborated it under
 `refinementDefects`, and the contest was shown nothing because only the
 accuracy probe was read. Here only corroborated claims become lines, from
 both probes with the edit named, keyed by the slice they concern, and a chunk
 with none contributes nothing.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  RegionDefectTally,
  ScreenedDefectClaim,
} from '@monochromatic-dev/translation-repair-adjudication';
import { damageClaimLinesBySlice, } from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Builds one screened claim with the given admissibility.

 @param admissibility - what the deterministic check made of the quote

 @param evidence - wording quoted from the repair text

 @param category - defect class in the prober's words

 @returns Claim as the screen records it

 @example
 ```ts
 const claim = claimOf({ admissibility: 'corroborated', evidence: 'is', },);
 ```
 */
function claimOf(
  {
    admissibility,
    evidence,
    category = 'tense',
  }: {
    readonly admissibility: ScreenedDefectClaim['admissibility'];
    readonly evidence: string;
    readonly category?: string;
  },
): ScreenedDefectClaim {
  return {
    modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
    category,
    severity: 'moderate',
    evidence,
    omittedText: '',
    reason: 'the page holds past tense',
    admissibility,
  };
}

/**
 Region tally around a set of claims, counts derived from them.

 @param claims - screened claims

 @returns Tally as the probe records it

 @example
 ```ts
 const region = regionOf({ claims: [claimOf({ admissibility: 'corroborated', evidence: 'is', },),], },);
 ```
 */
function regionOf(
  { claims, }: { readonly claims: readonly ScreenedDefectClaim[]; },
): RegionDefectTally {
  return {
    envelopeId: 'envelope/1',
    issueIds: ['issue/1',],
    corroborated: claims.filter((claim,) => claim.admissibility === 'corroborated',).length,
    removalCorroborated: 0,
    contradicted: claims.filter((claim,) => claim.admissibility === 'contradicted',).length,
    unanchored: claims.filter((claim,) => claim.admissibility === 'unanchored',).length,
    preExisting: 0,
    noneFound: 0,
    uncertain: 0,
    claims,
  };
}

await describe({
  name: damageClaimLinesBySlice.name,
  children: [
    it({
      name: 'NAMES A CATEGORY THAT SHOWS A READER NOTHING as unspecified, as it names an empty one: spaces, a '
        + 'zero-width space, a Hangul filler, or both among spaces',
      fn: async () => {
        /**
         Category fillings that show a reader nothing.
         */
        const blank = [
          '',
          '   ',
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
        ];
        /**
         Lines of one probed chunk whose corroborated claims carry those categories.
         */
        const bySlice = damageClaimLinesBySlice({
          lane: { chunks: [
            {
              sliceIndex: 4,
              accuracyPatchSelected: true,
              introducedDefects: {
                regions: [
                  regionOf({
                    claims: blank.map(function claimUnder(category,) {
                      return claimOf({ admissibility: 'corroborated', evidence: 'naps', category, },);
                    },),
                  },),
                ],
              },
            },
          ], },
        },);
        expect([...bySlice.entries(),],).toEqual([
          [
            4,
            blank.map(function lineOf() {
              return '- hf:moonshotai/Kimi-K3 [unspecified] on the accuracy repair quotes "naps": '
                + 'the page holds past tense';
            },),
          ],
        ],);
      },
    },),
    it({
      name: 'RENDERS only corroborated claims, one line each, keyed by the slice they concern, and '
        + 'LEAVES OUT chunks whose claims all failed the screen or that were never probed',
      fn: async () => {
        const bySlice = damageClaimLinesBySlice({
          lane: { chunks: [
            {
              sliceIndex: 1,
              accuracyPatchSelected: true,
              introducedDefects: {
                regions: [
                  regionOf({
                    claims: [
                      claimOf({ admissibility: 'corroborated', evidence: 'is a transgender woman', },),
                      claimOf({ admissibility: 'contradicted', evidence: 'was', },),
                    ],
                  },),
                  regionOf({
                    claims: [
                      claimOf({ admissibility: 'corroborated', evidence: 'where she shares', category: '', },),
                    ],
                  },),
                ],
              },
            },
            {
              sliceIndex: 2,
              accuracyPatchSelected: true,
              introducedDefects: {
                regions: [
                  regionOf({ claims: [claimOf({ admissibility: 'unanchored', evidence: 'nowhere', },),], },),
                ],
              },
            },
            {
              sliceIndex: 3,
              accuracyPatchSelected: false,
            },
          ], },
        },);
        expect([...bySlice.keys(),],).toEqual([1,],);
        expect(bySlice.get(1,),).toEqual([
          '- hf:moonshotai/Kimi-K3 [tense] on the accuracy repair quotes "is a transgender woman": '
            + 'the page holds past tense',
          '- hf:moonshotai/Kimi-K3 [unspecified] on the accuracy repair quotes "where she shares": '
            + 'the page holds past tense',
        ],);
        expect(damageClaimLinesBySlice({ lane: { chunks: [], }, },).size,).toBe(0,);
      },
    },),
    it({
      name: 'RENDERS the naturalness rewrite\'s corroborated claims too, after the accuracy repair\'s and '
        + 'naming their edit, and READS a chunk that carries only the rewrite\'s probe',
      fn: async () => {
        const bySlice = damageClaimLinesBySlice({
          lane: { chunks: [
            {
              sliceIndex: 1,
              accuracyPatchSelected: true,
              introducedDefects: {
                regions: [
                  regionOf({ claims: [claimOf({ admissibility: 'corroborated', evidence: 'she runs', },),], },),
                ],
              },
              refinementDefects: {
                regions: [
                  regionOf({
                    claims: [
                      claimOf({ admissibility: 'corroborated', evidence: 'is a transgender woman', },),
                      claimOf({ admissibility: 'unanchored', evidence: 'nowhere', },),
                    ],
                  },),
                ],
              },
            },
            {
              sliceIndex: 2,
              accuracyPatchSelected: false,
              refinementDefects: {
                regions: [
                  regionOf({ claims: [claimOf({ admissibility: 'corroborated', evidence: 'she shares', },),], },),
                ],
              },
            },
          ], },
        },);
        expect([...bySlice.keys(),],).toEqual([
          1,
          2,
        ],);
        expect(bySlice.get(1,),).toEqual([
          '- hf:moonshotai/Kimi-K3 [tense] on the accuracy repair quotes "she runs": the page holds past tense',
          '- hf:moonshotai/Kimi-K3 [tense] on the naturalness rewrite quotes "is a transgender woman": '
            + 'the page holds past tense',
        ],);
        expect(bySlice.get(2,),).toEqual([
          '- hf:moonshotai/Kimi-K3 [tense] on the naturalness rewrite quotes "she shares": the page holds past tense',
        ],);
      },
    },),
    it({
      name: 'LEAVES OUT the accuracy repair\'s claims where its patch lost (ledger L7): the repair lane then '
        + 'ships the archive or its rewrite, never the patched text the probe quoted, as XingZ6014 slice 3 '
        + 'showed the contest damage its candidate did not carry',
      fn: async () => {
        const bySlice = damageClaimLinesBySlice({
          lane: { chunks: [
            {
              sliceIndex: 3,
              accuracyPatchSelected: false,
              introducedDefects: {
                regions: [
                  regionOf({ claims: [claimOf({ admissibility: 'corroborated', evidence: 'she came out', },),], },),
                ],
              },
            },
            {
              sliceIndex: 4,
              accuracyPatchSelected: false,
              introducedDefects: {
                regions: [
                  regionOf({ claims: [claimOf({ admissibility: 'corroborated', evidence: 'she runs', },),], },),
                ],
              },
              refinementDefects: {
                regions: [
                  regionOf({ claims: [claimOf({ admissibility: 'corroborated', evidence: 'she naps', },),], },),
                ],
              },
            },
          ], },
        },);
        expect([...bySlice.keys(),],).toEqual([4,],);
        expect(bySlice.get(4,),).toEqual([
          '- hf:moonshotai/Kimi-K3 [tense] on the naturalness rewrite quotes "she naps": the page holds past tense',
        ],);
      },
    },),
  ],
},);
