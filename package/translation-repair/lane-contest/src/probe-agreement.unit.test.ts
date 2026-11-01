/**
 Tests for comparing the probe's verdicts with the human's repair grades,
 where exactly one cell of the table is clean evidence.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ScreenedDefectClaim, } from '@monochromatic-dev/translation-repair-adjudication';
import type { IssueProbeReading, } from '@monochromatic-dev/translation-repair-assembly';
import {
  probeFlaggedIssue,
  scoreProbeAgainstGrades,
} from '../dist/final/node/index.mjs';

/**
 Builds upheld claims, one per DISTINCT prober.

 The majority rule counts probers, so a region's corroborated COUNT has to be
 backed by claims carrying that many distinct model ids. A fixture pairing a
 count with an empty claim list describes a state the screen cannot emit and
 reads as nobody having corroborated anything.

 @param count - claims to build, each from its own prober

 @returns Claims carrying distinct model ids

 @example
 ```ts
 const claims = catClaims({ count: 3, },);
 ```
 */
function catClaims(
  { count, }: { readonly count: number; },
): readonly ScreenedDefectClaim[] {
  return Array.from(
    { length: count, },
    function toClaim(
      _unused,
      index,
    ) {
      return {
        modelId: `cat/prober-${String(index,)}`,
        category: 'meaning',
        severity: 'major',
        evidence: 'the cat naps',
        omittedText: '',
        reason: 'the cat did not nap before',
        admissibility: 'corroborated',
      } as ScreenedDefectClaim;
    },
  );
}

/**
 Builds a probe reading whose single region carries the given corroboration.

 @param corroborated - upheld claims of damage on that region

 @param configuredProbers - roster the majority is measured against

 @returns Reading the agreement scorer reads

 @example
 ```ts
 const reading = catReading({ corroborated: 2, },);
 ```
 */
function catReading(
  {
    corroborated,
    configuredProbers = 3,
  }: {
    readonly corroborated: number;
    readonly configuredProbers?: number;
  },
): IssueProbeReading {
  return {
    heardProbers: configuredProbers,
    configuredProbers,
    regions: [
      {
        envelopeId: 'envelope/nap',
        issueIds: ['adjudicated/nap',],
        corroborated,
        removalCorroborated: 0,
        contradicted: 0,
        unanchored: 0,
        preExisting: 0,
        noneFound: configuredProbers - corroborated,
        uncertain: 0,
        claims: catClaims({ count: corroborated, },),
      },
    ],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: probeFlaggedIssue.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'flags an issue when ANY region serving it drew a majority, since '
            + 'a gate would have rejected the candidate on that one verdict',
          fn: async () => {
            /** Reading whose second region alone is flagged. */
            const mixed: IssueProbeReading = {
              heardProbers: 3,
              configuredProbers: 3,
              regions: [
                ...catReading({ corroborated: 0, },).regions,
                {
                  envelopeId: 'envelope/chase',
                  issueIds: ['adjudicated/nap',],
                  corroborated: 3,
                  removalCorroborated: 0,
                  contradicted: 0,
                  unanchored: 0,
                  preExisting: 0,
                  noneFound: 0,
                  uncertain: 0,
                  claims: catClaims({ count: 3, },),
                },
              ],
            };
            expect(probeFlaggedIssue({ reading: mixed, },),).toBe(true,);
            expect(
              probeFlaggedIssue({ reading: catReading({ corroborated: 0, },), },),
            ).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: scoreProbeAgainstGrades.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'counts a flagged issue the human graded as fixing cleanly as a '
            + 'REFUTATION, because Y states that nothing nearby broke and each one '
            + 'is a correct repair a gate would have discarded',
          fn: async () => {
            const agreement = scoreProbeAgainstGrades({
              items: [
                {
                  verdict: 'fixes',
                  reading: catReading({ corroborated: 3, },),
                },
                {
                  verdict: 'fixes',
                  reading: catReading({ corroborated: 3, },),
                },
              ],
            },);
            expect(agreement.probeFlagged,).toBe(2,);
            expect(agreement.refutedByHuman,).toBe(2,);
            expect(agreement.sharedWithHuman,).toBe(0,);
          },
        },),

        it({
          name: 'keeps a flagged issue graded N in its own count rather than '
            + 'calling it confirmation, since N fires both for a repair that did '
            + 'not fix its target and for one that broke something',
          fn: async () => {
            const agreement = scoreProbeAgainstGrades({
              items: [
                {
                  verdict: 'does-not-fix',
                  reading: catReading({ corroborated: 3, },),
                },
              ],
            },);
            expect(agreement.sharedWithHuman,).toBe(1,);
            expect(agreement.refutedByHuman,).toBe(0,);
          },
        },),

        it({
          name: 'drops an issue whose chunk was never probed from the join rather '
            + 'than scoring it as unflagged, since silence is not a clean verdict',
          fn: async () => {
            const agreement = scoreProbeAgainstGrades({
              items: [
                { verdict: 'fixes', },
                {
                  verdict: 'fixes',
                  reading: catReading({ corroborated: 0, },),
                },
              ],
            },);
            expect(agreement.joined,).toBe(1,);
            expect(agreement.probeFlagged,).toBe(0,);
          },
        },),

        it({
          name: 'keeps unscored flagged items apart from both verdicts, because a '
            + 'grade the human declined proves nothing in either direction',
          fn: async () => {
            const agreement = scoreProbeAgainstGrades({
              items: [
                {
                  verdict: 'unscored',
                  reading: catReading({ corroborated: 3, },),
                },
              ],
            },);
            expect(agreement.flaggedUnscored,).toBe(1,);
            expect(agreement.refutedByHuman,).toBe(0,);
            expect(agreement.sharedWithHuman,).toBe(0,);
          },
        },),

        it({
          name: 'counts unflagged failures as the upper bound on misses it is, '
            + 'inflated by the same ambiguity in N',
          fn: async () => {
            const agreement = scoreProbeAgainstGrades({
              items: [
                {
                  verdict: 'does-not-fix',
                  reading: catReading({ corroborated: 0, },),
                },
              ],
            },);
            expect(agreement.unflaggedFailures,).toBe(1,);
            expect(agreement.probeFlagged,).toBe(0,);
          },
        },),

        it({
          name: 'measures the majority against the configured roster here too, so '
            + 'a six-model roster heard by three cannot flag on two voices',
          fn: async () => {
            const agreement = scoreProbeAgainstGrades({
              items: [
                {
                  verdict: 'fixes',
                  reading: catReading({
                    corroborated: 2,
                    configuredProbers: 6,
                  },),
                },
              ],
            },);
            expect(agreement.probeFlagged,).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
