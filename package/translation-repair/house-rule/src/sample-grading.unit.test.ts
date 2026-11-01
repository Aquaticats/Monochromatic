import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  allocateBandQuota,
  assertSourceBytes,
  type BandQuota,
  classifyBand,
  classifySourceAnchor,
  drawStratifiedSample,
  extractGradingCandidate,
  formatGradingSheet,
  type GradableIssue,
  type GradableRepair,
  type GradingCandidate,
  MEDIUM_BAND_MAX_BYTES,
  type SizeBand,
  SMALL_BAND_MAX_BYTES,
  sourceBytesOf,
} from '../dist/final/node/index.mjs';

/**
 Bands a raw byte count, stating the unit explicitly.

 These cases probe the band BOUNDARIES, which are byte counts by
 definition and cannot be produced from text, so the assertion is the
 honest way to reach `classifyBand` rather than a cast around its guard.

 @param count - UTF-8 byte length under test

 @returns Band that count falls in

 @example
 ```ts
 expect(bandAt(1_842,),).toBe('small',);
 ```
 */
function bandAt(count: number,): SizeBand {
  assertSourceBytes(count,);
  return classifyBand({ sourceBytes: count, },);
}


/**
 Builds a grading candidate for the given entry and band; only the fields a
 test reads carry meaning, the rest default to cat-themed filler.
 */
function catCandidate(
  {
    entryId,
    band,
    issueId,
  }: {
    readonly entryId: string;
    readonly band: SizeBand;
    readonly issueId: string;
  },
): GradingCandidate {
  return {
    entryId,
    band,
    issueId,
    category: 'accuracy/omission',
    severity: 'minor',
    summary: 'A purr is dropped from the greeting.',
    sourceAnchor: 'quoted',
    sourceQuotes: ['呼噜',],
    targetQuotes: [],
  };
}

/**
 Builds a pool of `perEntry` candidates for each of `entries`, all in one
 band, with deterministic ids so draws are checkable.
 */
function catPool(
  {
    band,
    entries,
    perEntry,
  }: {
    readonly band: SizeBand;
    readonly entries: readonly string[];
    readonly perEntry: number;
  },
): readonly GradingCandidate[] {
  return entries.flatMap(function forEntry(entryId,) {
    return [
      ...Array.from({ length: perEntry, },)
        .keys(),
    ]
      .map(function forIssue(issueIndex,) {
        return catCandidate({
          entryId,
          band,
          issueId: `${entryId}/issue/${String(issueIndex,)}`,
        },);
      },);
  },);
}

/**
 Builds an adjudicated issue from a list of `[side, quote]` span pairs and a
 primary category, for exercising candidate extraction.
 */
function catIssue(
  {
    issueId,
    category,
    summary,
    spans,
  }: {
    readonly issueId: string;
    readonly category: AdjudicatedIssue['claims'][number]['claim']['category'];
    readonly summary: string;
    readonly spans: readonly (readonly ['source' | 'target', string,])[];
  },
): AdjudicatedIssue {
  return {
    issueId,
    status: 'accepted',
    severity: 'major',
    claims: [
      {
        claimId: `${issueId}/claim`,
        claim: {
          category,
          severity: 'major',
          summary,
          spans: spans.map(function toSpan(pair,) {
            return {
              side: pair[0],
              nodeId: 'block/0',
              nodeHash: 'hash/whisker',
              startOffset: 0,
              endOffset: pair[1]
                .length,
              quotedText: pair[1],
            };
          },),
        },
      },
    ],
    tallies: {},
  };
}

/**
 Builds a gradable issue from span descriptions, so a case reads as the shape
 it is testing rather than as nested boilerplate.

 @param spans - spans across the issue's claims, one claim per span

 @returns Issue whose source anchoring is under classification

 @example
 ```ts
 const issue = catAnchoredIssue({ spans: [{ side: 'source', quotedText: '猫猫', },], },);
 ```
 */
function catAnchoredIssue(
  { spans, }: { readonly spans: readonly { readonly side: 'source' | 'target'; readonly quotedText: string; }[]; },
): GradableIssue {
  return {
    issueId: 'adjudicated/nap',
    severity: 'major',
    claims: spans.map(function toClaim(span,) {
      return {
        claim: {
          category: 'accuracy/omission',
          summary: 'A clause about the windowsill is missing.',
          spans: [span,],
        },
      };
    },),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: '',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        //region classifyBand

        describe({
          name: classifyBand.name,
          children: [
            it({
              name: 'is small below the small cut',
              fn: async () => {
                expect(bandAt(SMALL_BAND_MAX_BYTES - 1,),)
                  .toBe('small',);
                expect(bandAt(0,),)
                  .toBe('small',);
              },
            },),
            it({
              name: 'is medium from the small cut up to the medium cut',
              fn: async () => {
                expect(bandAt(SMALL_BAND_MAX_BYTES,),)
                  .toBe('medium',);
                expect(bandAt(MEDIUM_BAND_MAX_BYTES - 1,),)
                  .toBe('medium',);
              },
            },),
            it({
              name: 'is large at and above the medium cut',
              fn: async () => {
                expect(bandAt(MEDIUM_BAND_MAX_BYTES,),)
                  .toBe('large',);
                expect(bandAt(40_700,),)
                  .toBe('large',);
              },
            },),
          ],
        },),

        //endregion classifyBand

        //region assertSourceBytes

        describe({
          name: assertSourceBytes.name,
          children: [
            it({
              name: 'refuses a fractional or negative byte count, since neither measures bytes',
              fn: async () => {
                for (const count of [-1, 1.5,]) {
                  /**
                   What the assertion raised, read for its class and its
                   whole wording.
                   */
                  const refusal = caught(function assertCount(): unknown {
                    return assertSourceBytes(count,);
                  },);
                  expect(refusal,).toBeInstanceOf(RangeError,);
                  expect(String(refusal,),).toBe(
                    `RangeError: A UTF-8 byte length is a non-negative safe integer; received ${String(count,)}. `
                      + 'A fractional or negative value means the caller measured something other than bytes.',
                  );
                }
              },
            },),
          ],
        },),

        //endregion assertSourceBytes

        //region sourceBytesOf

        describe({
          name: sourceBytesOf.name,
          children: [
            it({
              name: 'MEASURES the UTF-8 bytes of a text, the yardstick the bands are cut at',
              fn: async () => {
                expect(sourceBytesOf({ text: '雨', },),).toBe(3,);
                expect(sourceBytesOf({ text: 'rain', },),).toBe(4,);
              },
            },),
          ],
        },),

        //endregion sourceBytesOf

        //region extractGradingCandidate

        describe({
          name: extractGradingCandidate.name,
          children: [
            it({
              name: 'lifts primary category and summary from the first claim',
              fn: async () => {
                const candidate = extractGradingCandidate({
                  issue: catIssue({
                    issueId: 'adjudicated/paw',
                    category: 'terminology/wrong-term',
                    summary: 'Whisker rendered as antenna.',
                    spans: [['source', '胡须',], ['target', 'antenna',],],
                  },),
                  entryId: 'Kitten',
                  band: 'small',
                },);
                expect(candidate.category,).toBe('terminology/wrong-term',);
                expect(candidate.summary,).toBe('Whisker rendered as antenna.',);
                expect(candidate.entryId,).toBe('Kitten',);
                expect(candidate.band,).toBe('small',);
              },
            },),
            it({
              name: 'reads the uncategorized and no-summary placeholders where the issue names no claim',
              fn: async () => {
                const candidate = extractGradingCandidate({
                  issue: {
                    issueId: 'adjudicated/empty',
                    severity: 'minor',
                    claims: [],
                  },
                  entryId: 'Kitten',
                  band: 'small',
                },);
                expect(candidate.category,).toBe('(uncategorized)',);
                expect(candidate.summary,).toBe('(no claim summary)',);
              },
            },),
            it({
              name: 'CARRIES the repair context where the issue has one, and omits the field where it '
                + 'has none',
              fn: async () => {
                /**
                 Repair context the issue carries.
                 */
                const repair: GradableRepair = {
                  disposition: 'shipped',
                  regions: [],
                  refined: true,
                };
                const carried = extractGradingCandidate({
                  issue: catIssue({
                    issueId: 'adjudicated/paw',
                    category: 'terminology/wrong-term',
                    summary: 'Whisker rendered as antenna.',
                    spans: [['source', '胡须',], ['target', 'antenna',],],
                  },),
                  repair,
                  entryId: 'Kitten',
                  band: 'small',
                },);
                expect(carried.repair,).toBe(repair,);
                const withoutRepair = extractGradingCandidate({
                  issue: catIssue({
                    issueId: 'adjudicated/paw',
                    category: 'terminology/wrong-term',
                    summary: 'Whisker rendered as antenna.',
                    spans: [['source', '胡须',], ['target', 'antenna',],],
                  },),
                  entryId: 'Kitten',
                  band: 'small',
                },);
                expect(Object.hasOwn(withoutRepair, 'repair',),).toBe(false,);
              },
            },),
            it({
              name: 'gathers distinct source and target quotes in first-seen order',
              fn: async () => {
                const candidate = extractGradingCandidate({
                  issue: catIssue({
                    issueId: 'adjudicated/tail',
                    category: 'accuracy/omission',
                    summary: 'Repeated meow dropped.',
                    spans: [
                      ['source', '喵',],
                      ['target', 'meow',],
                      ['source', '喵',],
                      ['target', 'purr',],
                    ],
                  },),
                  entryId: 'Kitten',
                  band: 'medium',
                },);
                expect(candidate.sourceQuotes,).toEqual(['喵',],);
                expect(candidate.targetQuotes,).toEqual(['meow', 'purr',],);
              },
            },),
            it({
              name: 'drops empty insertion-anchor quotes',
              fn: async () => {
                const candidate = extractGradingCandidate({
                  issue: catIssue({
                    issueId: 'adjudicated/empty',
                    category: 'accuracy/addition',
                    summary: 'Fabricated hiss inserted.',
                    spans: [['target', '',], ['target', 'hiss',],],
                  },),
                  entryId: 'Kitten',
                  band: 'large',
                },);
                expect(candidate.targetQuotes,).toEqual(['hiss',],);
                expect(candidate.sourceQuotes,).toEqual([],);
              },
            },),
            it({
              name: 'tells an unanchored claim apart from a correctly anchored insertion',
              fn: async () => {
                // A bare "(none)" on the sheet conflated these, which made one
                // graded item ungradable for the wrong reason.
                /**
                 Claim anchoring a real position in the original that holds no
                 text, which is how an insertion is correctly anchored.
                 */
                const insertion = extractGradingCandidate({
                  issue: catIssue({
                    issueId: 'adjudicated/insertion',
                    category: 'accuracy/addition',
                    summary: 'Fabricated hiss inserted.',
                    spans: [['source', '',], ['target', 'hiss',],],
                  },),
                  entryId: 'Kitten',
                  band: 'large',
                },);

                /**
                 Claim pointing at nothing in the original at all.
                 */
                const unanchored = extractGradingCandidate({
                  issue: catIssue({
                    issueId: 'adjudicated/unanchored',
                    category: 'accuracy/addition',
                    summary: 'Fabricated hiss inserted.',
                    spans: [['target', 'hiss',],],
                  },),
                  entryId: 'Kitten',
                  band: 'large',
                },);

                expect(insertion.sourceAnchor,).toBe('insertion-point',);
                expect(unanchored.sourceAnchor,).toBe('unanchored',);
              },
            },),
          ],
        },),

        //endregion extractGradingCandidate

        //region allocateBandQuota

        describe({
          name: allocateBandQuota.name,
          children: [
            it({
              name: 'splits fifty near-evenly when every band has spare',
              fn: async () => {
                const quota = allocateBandQuota({
                  available: { small: 200, medium: 180, large: 90, },
                  size: 50,
                },);
                expect(quota,).toEqual({ small: 17, medium: 17, large: 16, },);
              },
            },),
            it({
              name: 'caps a scarce band and redistributes its slots to the others',
              fn: async () => {
                const quota = allocateBandQuota({
                  available: { small: 100, medium: 100, large: 4, },
                  size: 50,
                },);
                expect(quota.large,).toBe(4,);
                expect(quota.small + quota.medium + quota.large,).toBe(50,);
                expect(Math.abs(quota.small - quota.medium,),).toBeLessThanOrEqual(1,);
              },
            },),
            it({
              name: 'falls to total available when the pool is smaller than the size',
              fn: async () => {
                const quota = allocateBandQuota({
                  available: { small: 3, medium: 2, large: 1, },
                  size: 50,
                },);
                expect(quota,).toEqual({ small: 3, medium: 2, large: 1, },);
              },
            },),
            it({
              name: 'draws entirely from the only stocked band',
              fn: async () => {
                const quota = allocateBandQuota({
                  available: { small: 0, medium: 0, large: 90, },
                  size: 50,
                },);
                expect(quota,).toEqual({ small: 0, medium: 0, large: 50, },);
              },
            },),
          ],
        },),

        //endregion allocateBandQuota

        //region drawStratifiedSample

        describe({
          name: drawStratifiedSample.name,
          children: [
            it({
              name: 'is deterministic for a fixed pool and seed',
              fn: async () => {
                const pool = [
                  ...catPool({ band: 'small', entries: ['A', 'B',], perEntry: 10, },),
                  ...catPool({ band: 'medium', entries: ['C', 'D',], perEntry: 10, },),
                  ...catPool({ band: 'large', entries: ['E', 'F',], perEntry: 10, },),
                ];
                const first = drawStratifiedSample({ candidates: pool, size: 30, seed: 'meow', },);
                const second = drawStratifiedSample({ candidates: pool, size: 30, seed: 'meow', },);
                /**
                 Sampled issue ids from the first draw.
                 */
                const firstIds = first.map(function toId(candidate,) {
                  return candidate.issueId;
                },);
                /**
                 Sampled issue ids from the second draw.
                 */
                const secondIds = second.map(function toId(candidate,) {
                  return candidate.issueId;
                },);
                expect(firstIds,).toEqual(secondIds,);
              },
            },),
            it({
              name: 'honours the per-band quotas',
              fn: async () => {
                const pool = [
                  ...catPool({ band: 'small', entries: ['A',], perEntry: 40, },),
                  ...catPool({ band: 'medium', entries: ['C',], perEntry: 40, },),
                  ...catPool({ band: 'large', entries: ['E',], perEntry: 40, },),
                ];
                const sample = drawStratifiedSample({ candidates: pool, size: 50, seed: 'meow', },);
                /**
                 Sampled count per band.
                 */
                const counts: BandQuota = {
                  small: sample.filter(function s(c,) { return c.band === 'small'; },).length,
                  medium: sample.filter(function m(c,) { return c.band === 'medium'; },).length,
                  large: sample.filter(function l(c,) { return c.band === 'large'; },).length,
                };
                expect(counts,).toEqual({ small: 17, medium: 17, large: 16, },);
                expect(sample.length,).toBe(50,);
              },
            },),
            it({
              name: 'spreads across entries before taking a second issue from any one',
              fn: async () => {
                const pool = [
                  ...catPool({
                    band: 'large',
                    entries: ['Heavy', 'Light',],
                    perEntry: 1,
                  },),
                  ...catPool({ band: 'large', entries: ['Heavy',], perEntry: 4, },),
                ];
                const sample = drawStratifiedSample({ candidates: pool, size: 2, seed: 'meow', },);
                /**
                 Distinct entry ids in the two-slot draw.
                 */
                const entryIds = new Set(
                  sample.map(function toEntry(c,) { return c.entryId; },),
                );
                expect(entryIds.has('Light',),).toBe(true,);
                expect(entryIds.size,).toBe(2,);
              },
            },),
            it({
              name: 'never selects more than the pool holds',
              fn: async () => {
                const pool = catPool({ band: 'small', entries: ['A',], perEntry: 3, },);
                const sample = drawStratifiedSample({ candidates: pool, size: 50, seed: 'meow', },);
                expect(sample.length,).toBe(3,);
              },
            },),
          ],
        },),

        //endregion drawStratifiedSample

        //region formatGradingSheet

        describe({
          name: formatGradingSheet.name,
          children: [
            it({
              name: 'records seed, bar, corpus pin, and per-issue grade boxes',
              fn: async () => {
                const sample = [
                  catCandidate({ entryId: 'Kitten', band: 'small', issueId: 'i/1', },),
                  catCandidate({ entryId: 'Tabby', band: 'large', issueId: 'i/2', },),
                ];
                const sheet = formatGradingSheet({
                  sample,
                  seed: 'meow',
                  bar: 0.9,
                  corpusSha: 'feedfac',
                  drawDigest: 'digest-of-this-draw',
                },);
                expect(sheet,).toContain('Draw seed: meow',);
                expect(sheet,).toContain('Precision bar: 0.9',);
                expect(sheet,).toContain('Corpus pin: feedfac',);
                expect(sheet,).toContain('Sample size: 2',);
                expect(sheet,).toContain('entry: Kitten',);
                expect(sheet,).toContain('entry: Tabby',);
                expect(
                  sheet.split('grade: [ ]',).length - 1,
                ).toBe(2,);
              },
            },),
          ],
        },),

        //endregion formatGradingSheet
      ],
    },),

    describe({
      name: classifySourceAnchor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reports a quoted anchor when the issue names source text, which '
            + 'is the anchoring a grader can actually check against the original',
          fn: async () => {
            expect(
              classifySourceAnchor({
                issue: catAnchoredIssue({
                  spans: [
                    {
                      side: 'source',
                      quotedText: '猫猫在窗台上睡觉',
                    },
                  ],
                },),
              },),
            ).toBe('quoted',);
          },
        },),

        it({
          name: 'reports an INSERTION POINT for an empty source span, because an '
            + 'empty span is a real place in the original with no text at it, '
            + 'which is exactly how a correctly anchored addition claim looks: '
            + 'collapsing it into unanchored would condemn every correct insertion',
          fn: async () => {
            expect(
              classifySourceAnchor({
                issue: catAnchoredIssue({
                  spans: [
                    {
                      side: 'source',
                      quotedText: '',
                    },
                  ],
                },),
              },),
            ).toBe('insertion-point',);
          },
        },),

        it({
          name: 'reports UNANCHORED when no span points into the original at all, '
            + 'which is the claim that points at nothing and is the thing worth '
            + 'separating from a correct insertion',
          fn: async () => {
            expect(
              classifySourceAnchor({
                issue: catAnchoredIssue({
                  spans: [
                    {
                      side: 'target',
                      quotedText: 'The cat sleeps on the windowsill.',
                    },
                  ],
                },),
              },),
            ).toBe('unanchored',);
          },
        },),

        it({
          name: 'reports unanchored for an issue with no spans whatsoever, rather '
            + 'than throwing on a shape the adjudicator can produce',
          fn: async () => {
            expect(
              classifySourceAnchor({ issue: catAnchoredIssue({ spans: [], },), },),
            ).toBe('unanchored',);
          },
        },),

        it({
          name: 'prefers QUOTED when an issue carries both a quoted source span '
            + 'and an empty one, since one real quote is enough for a grader to '
            + 'check the original and the sheet has something to show',
          fn: async () => {
            expect(
              classifySourceAnchor({
                issue: catAnchoredIssue({
                  spans: [
                    {
                      side: 'source',
                      quotedText: '',
                    },
                    {
                      side: 'source',
                      quotedText: '猫猫在窗台上睡觉',
                    },
                  ],
                },),
              },),
            ).toBe('quoted',);
          },
        },),

        it({
          name: 'looks across EVERY claim of the issue rather than the first, so a '
            + 'merged issue whose source anchor arrived on a later claim is not '
            + 'reported as unanchored',
          fn: async () => {
            expect(
              classifySourceAnchor({
                issue: catAnchoredIssue({
                  spans: [
                    {
                      side: 'target',
                      quotedText: 'The cat sleeps.',
                    },
                    {
                      side: 'source',
                      quotedText: '猫猫在窗台上睡觉',
                    },
                  ],
                },),
              },),
            ).toBe('quoted',);
          },
        },),
      ],
    },),
  ],
},);
