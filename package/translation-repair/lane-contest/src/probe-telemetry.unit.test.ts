/**
 Tests for aggregating shadow-mode probe readings across a run, where the two
 joins that are easy to get silently wrong live.
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

import type {
  RegionDefectTally,
  ScreenedDefectClaim,
} from '@monochromatic-dev/translation-repair-adjudication';
import type { IssueProbeReading, } from '@monochromatic-dev/translation-repair-assembly';
import {
  corroboratingProberCount,
  judgeRegionProbe,
  ProbeTelemetryError,
  summarizeProbeTelemetry,
} from '../dist/final/node/index.mjs';

/**
 Builds upheld claims, one per DISTINCT prober.

 This mirrors what the run actually produces: measured across the 210 distinct
 regions settled so far, no prober ever filed more than one upheld claim on
 one region. Fixtures that pair a count with an empty claim list describe a
 state the screen cannot emit, and they hid the units defect for as long as
 they existed.

 @param count - claims to build

 @param admissibility - what the screen made of each quote

 @param prefix - distinguishes one call's probers from another's

 @returns Claims carrying distinct model ids

 @example
 ```ts
 const claims = catClaims({ count: 2, admissibility: 'corroborated', prefix: 'add', },);
 ```
 */
function catClaims(
  {
    count,
    admissibility,
    prefix,
  }: {
    readonly count: number;
    readonly admissibility: string;
    readonly prefix: string;
  },
): readonly ScreenedDefectClaim[] {
  return Array.from(
    { length: count, },
    function toClaim(
      _unused,
      index,
    ) {
      return {
        modelId: `cat/${prefix}-prober-${String(index,)}`,
        category: 'meaning',
        severity: 'major',
        evidence: 'the cat naps',
        omittedText: '',
        reason: 'the cat did not nap before',
        admissibility,
      } as ScreenedDefectClaim;
    },
  );
}

/**
 Builds one region tally with the counts under test and zeros elsewhere.

 @param envelopeId - envelope the region replaced

 @param corroborated - upheld claims of added damage

 @param removalCorroborated - upheld claims of dropped content

 @param contradicted - claims the screen refuted

 @param claims - overrides the generated claim list, for cases about who
 filed what rather than how many were filed

 @returns Tally the summary reads

 @example
 ```ts
 const tally = catTally({ envelopeId: 'envelope/nap', corroborated: 2, },);
 ```
 */
function catTally(
  {
    envelopeId,
    corroborated = 0,
    removalCorroborated = 0,
    contradicted = 0,
    claims,
  }: {
    readonly envelopeId: string;
    readonly corroborated?: number;
    readonly removalCorroborated?: number;
    readonly contradicted?: number;
    readonly claims?: readonly ScreenedDefectClaim[];
  },
): RegionDefectTally {
  return {
    envelopeId,
    issueIds: ['adjudicated/nap',],
    corroborated,
    removalCorroborated,
    contradicted,
    unanchored: 0,
    preExisting: 0,
    noneFound: 0,
    uncertain: 0,
    claims: claims ?? [
      ...catClaims({
        count: corroborated,
        admissibility: 'corroborated',
        prefix: 'add',
      },),
      ...catClaims({
        count: removalCorroborated,
        admissibility: 'removal-corroborated',
        prefix: 'drop',
      },),
      ...catClaims({
        count: contradicted,
        admissibility: 'contradicted',
        prefix: 'refuted',
      },),
    ],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: judgeRegionProbe.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'measures the majority against the CONFIGURED roster, never the '
            + 'heard one, so two probers can never speak for six',
          fn: async () => {
            /** Two upheld claims where six probers were asked. */
            const tally = catTally({
              envelopeId: 'envelope/nap',
              corroborated: 2,
            },);
            expect(
              judgeRegionProbe({
                tally,
                configuredProbers: 6,
              },),
            ).toBe('minority-introduced',);
            expect(
              judgeRegionProbe({
                tally,
                configuredProbers: 3,
              },),
            ).toBe('majority-introduced',);
          },
        },),

        it({
          name: 'counts dropped-content claims toward the same majority as added '
            + 'damage, since both are damage the edit caused',
          fn: async () => {
            expect(
              judgeRegionProbe({
                tally: catTally({
                  envelopeId: 'envelope/nap',
                  corroborated: 1,
                  removalCorroborated: 1,
                },),
                configuredProbers: 3,
              },),
            ).toBe('majority-introduced',);
          },
        },),

        it({
          name: 'REFUSES to let one prober filing twice carry a roster. Counting '
            + 'claims against a roster size compares claims with voices, and the '
            + 'same tally already counts probers on its other side, so two upheld '
            + 'claims from one model would have read as a majority of three and a '
            + 'gate would have discarded that repair on one opinion',
          fn: async () => {
            /**
             Two upheld claims, both from the same prober.
             */
            const tally = catTally({
              envelopeId: 'envelope/nap',
              corroborated: 2,
              claims: [
                ...catClaims({
                  count: 1,
                  admissibility: 'corroborated',
                  prefix: 'lone',
                },),
                ...catClaims({
                  count: 1,
                  admissibility: 'corroborated',
                  prefix: 'lone',
                },),
              ],
            },);

            expect(tally.claims.length,).toBe(2,);
            expect(corroboratingProberCount({ tally, },),).toBe(1,);
            expect(
              judgeRegionProbe({
                tally,
                configuredProbers: 3,
              },),
            ).toBe('minority-introduced',);
          },
        },),

        it({
          name: 'still reaches a majority when the SAME number of upheld claims '
            + 'comes from different probers, so the stricter numerator narrows the '
            + 'rule to independent voices rather than weakening it',
          fn: async () => {
            expect(
              judgeRegionProbe({
                tally: catTally({
                  envelopeId: 'envelope/nap',
                  corroborated: 2,
                },),
                configuredProbers: 3,
              },),
            ).toBe('majority-introduced',);
          },
        },),

        it({
          name: 'ignores contradicted claims entirely, because a claim the '
            + 'differential refuted is not weak evidence of damage but none',
          fn: async () => {
            expect(
              judgeRegionProbe({
                tally: catTally({
                  envelopeId: 'envelope/nap',
                  contradicted: 3,
                },),
                configuredProbers: 3,
              },),
            ).toBe('none-introduced',);
          },
        },),
      ],
    },),

    describe({
      name: summarizeProbeTelemetry.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'counts one region once however many issues its merged envelope '
            + 'served, since every issue of that envelope carries the same tally '
            + 'and summing over issues would overweight the widest edits',
          fn: async () => {
            /** Tally two issue records both carry, being one shared envelope. */
            const shared = catTally({
              envelopeId: 'envelope/shared',
              corroborated: 3,
            },);

            /** Two records of the same envelope, as buildIssueRecords emits them. */
            const readings: readonly IssueProbeReading[] = [
              {
                heardProbers: 3,
                configuredProbers: 3,
                regions: [shared,],
              },
              {
                heardProbers: 3,
                configuredProbers: 3,
                regions: [shared,],
              },
            ];

            const summary = summarizeProbeTelemetry({
              entries: [
                {
                  entryId: 'Kitten',
                  readings,
                },
              ],
            },);
            expect(summary.regions,).toBe(1,);
            expect(summary.majorityIntroduced,).toBe(1,);
            // Summed over DISTINCT regions, so the shared envelope's three claims
            // are counted once rather than once per issue it served.
            expect(summary.corroborated,).toBe(3,);
          },
        },),

        it({
          name: 'THROWS when two records carry disagreeing copies of one '
            + 'envelope. Every record of a merged envelope carries the same tally '
            + 'and one roster probed them all, so copies that differ cannot both '
            + 'be right, and keeping the first would make the summary depend on '
            + 'the order artifacts happened to be read while reporting a figure '
            + 'that looks settled',
          fn: async () => {
            /**
             What summarizesDisagreement raised, read for its class as well as its wording.
             */
            const refusalOfSummarizesDisagreement = caught(function summarizesDisagreement() {
              summarizeProbeTelemetry({
                entries: [
                  {
                    entryId: 'Kitten',
                    readings: [
                      {
                        heardProbers: 3,
                        configuredProbers: 3,
                        regions: [
                          catTally({
                            envelopeId: 'envelope/shared',
                            corroborated: 3,
                          },),
                        ],
                      },
                      {
                        heardProbers: 3,
                        configuredProbers: 3,
                        regions: [
                          catTally({
                            envelopeId: 'envelope/shared',
                            corroborated: 1,
                          },),
                        ],
                      },
                    ],
                  },
                ],
              },);
            },);

            expect(refusalOfSummarizesDisagreement,).toBeInstanceOf(ProbeTelemetryError,);
            expect((refusalOfSummarizesDisagreement as Error).message,).toContain('envelope/shared',);
          },
        },),

        it({
          name: 'THROWS when two copies of one envelope name DIFFERENT probers in '
            + 'equal numbers. Comparing totals alone would pass them, and the '
            + 'invariant this guard states is that disagreeing copies are refused, '
            + 'not that copies with matching arithmetic are accepted',
          fn: async () => {
            /**
             What summarizesDifferentProbers raised, read for its class as well as its wording.
             */
            const refusalOfSummarizesDifferentProbers = caught(function summarizesDifferentProbers() {
              summarizeProbeTelemetry({
                entries: [
                  {
                    entryId: 'Kitten',
                    readings: [
                      {
                        heardProbers: 3,
                        configuredProbers: 3,
                        regions: [
                          catTally({
                            envelopeId: 'envelope/shared',
                            corroborated: 1,
                            claims: catClaims({
                              count: 1,
                              admissibility: 'corroborated',
                              prefix: 'first',
                            },),
                          },),
                        ],
                      },
                      {
                        heardProbers: 3,
                        configuredProbers: 3,
                        regions: [
                          catTally({
                            envelopeId: 'envelope/shared',
                            corroborated: 1,
                            claims: catClaims({
                              count: 1,
                              admissibility: 'corroborated',
                              prefix: 'second',
                            },),
                          },),
                        ],
                      },
                    ],
                  },
                ],
              },);
            },);

            expect(refusalOfSummarizesDifferentProbers,).toBeInstanceOf(ProbeTelemetryError,);
            expect((refusalOfSummarizesDifferentProbers as Error).message,).toContain('envelope/shared',);
          },
        },),

        it({
          name: 'separates majority from minority findings, which is the split '
            + 'between what a gate would have blocked and what it would not',
          fn: async () => {
            const summary = summarizeProbeTelemetry({
              entries: [
                {
                  entryId: 'Kitten',
                  readings: [
                    {
                      heardProbers: 3,
                      configuredProbers: 3,
                      regions: [
                        catTally({
                          envelopeId: 'envelope/one',
                          corroborated: 3,
                        },),
                        catTally({
                          envelopeId: 'envelope/two',
                          corroborated: 1,
                        },),
                        catTally({ envelopeId: 'envelope/three', },),
                      ],
                    },
                  ],
                },
              ],
            },);
            expect(summary.regions,).toBe(3,);
            expect(summary.majorityIntroduced,).toBe(1,);
            expect(summary.minorityIntroduced,).toBe(1,);
            expect(summary.noneIntroduced,).toBe(1,);
          },
        },),

        it({
          name: 'flags regions whose roster came up short, where an absent verdict '
            + 'is silence rather than a clean bill',
          fn: async () => {
            const summary = summarizeProbeTelemetry({
              entries: [
                {
                  entryId: 'Kitten',
                  readings: [
                    {
                      heardProbers: 2,
                      configuredProbers: 3,
                      regions: [catTally({ envelopeId: 'envelope/short', },),],
                    },
                    {
                      heardProbers: 3,
                      configuredProbers: 3,
                      regions: [catTally({ envelopeId: 'envelope/whole', },),],
                    },
                  ],
                },
              ],
            },);
            expect(summary.degradedRosterRegions,).toBe(1,);
          },
        },),

        it({
          name: 'summarizes an empty run without inventing a region, so a pass '
            + 'that repaired nothing reads as no evidence rather than clean',
          fn: async () => {
            const summary = summarizeProbeTelemetry({ entries: [], },);
            expect(summary.regions,).toBe(0,);
            expect(summary.majorityIntroduced,).toBe(0,);
            expect(summary.noneIntroduced,).toBe(0,);
          },
        },),
        it({
          name: 'counts one envelope id ONCE PER ENTRY rather than once overall. '
            + 'Envelope ids are derived from the text they cover, so two entries '
            + 'sharing a paragraph produce the same id for regions serving '
            + 'different issues; collapsing globally merged unrelated documents '
            + 'and undercounted regions, which is a silent loss rather than an '
            + 'error until the evidence guard happens to meet a disagreeing pair',
          fn: async () => {
            const summary = summarizeProbeTelemetry({
              entries: [
                {
                  entryId: 'Kitten',
                  readings: [
                    {
                      heardProbers: 3,
                      configuredProbers: 3,
                      regions: [catTally({ envelopeId: 'envelope/shared', },),],
                    },
                  ],
                },
                {
                  entryId: 'Mittens',
                  readings: [
                    {
                      heardProbers: 3,
                      configuredProbers: 3,
                      regions: [catTally({ envelopeId: 'envelope/shared', },),],
                    },
                  ],
                },
              ],
            },);
            expect(summary.regions,).toBe(2,);
          },
        },),
      ],
    },),

    describe({
      name: corroboratingProberCount.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'counts probers upholding EITHER direction, because the differential upholds a claim '
            + 'either by finding wording the edit added or by finding wording it '
            + 'dropped, and counting only the added direction was the original '
            + 'defect: an omission claim could never corroborate at all',
          fn: async () => {
            expect(
              corroboratingProberCount({
                tally: catTally({
                  envelopeId: 'envelope/nap',
                  corroborated: 2,
                  removalCorroborated: 3,
                },),
              },),
            ).toBe(5,);
          },
        },),

        it({
          name: 'counts a purely dropped-wording region, the case a forward-only '
            + 'screen would have reported as finding nothing',
          fn: async () => {
            expect(
              corroboratingProberCount({
                tally: catTally({
                  envelopeId: 'envelope/nap',
                  removalCorroborated: 2,
                },),
              },),
            ).toBe(2,);
          },
        },),

        it({
          name: 'IGNORES contradicted claims, so a region whose probers were all '
            + 'refuted by the differential reads as nothing upheld rather than as '
            + 'damage found',
          fn: async () => {
            expect(
              corroboratingProberCount({
                tally: catTally({
                  envelopeId: 'envelope/nap',
                  contradicted: 4,
                },),
              },),
            ).toBe(0,);
          },
        },),

        it({
          name: 'counts zero for a clean region, which is the reading a whole '
            + 'quiet round produces and must not be confused with a broken probe',
          fn: async () => {
            expect(
              corroboratingProberCount({ tally: catTally({ envelopeId: 'envelope/nap', },), },),
            ).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
