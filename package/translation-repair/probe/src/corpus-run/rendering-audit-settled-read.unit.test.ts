/**
 Tests for the three readings the settled audit owes over persisted audit
 rows.

 These rules were written while the full run was still buying its subjects,
 and the cases here pin them so a later reader can see they were not tuned to
 a tally. The sharpest are the ones the relocation rule must REFUSE: pairing
 across two runs of one entry, and pairing slices that are not neighbours.
 Both would manufacture a relocation nobody's document contains.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  rateByVoice,
  auditRelocationPairs,
  distinctSlicePairs,
  type SettledAuditRow,
  splitFor,
} from '../../dist/final/node/index.mjs';

/**
 Builds one voice's screened answer.

 @param modelId - auditor

 @param categories - category of each claim that anchored

 @param dropped - how many of its claims fell at the screen

 @returns Voice row shaped as the audit returns it

 @example
 ```ts
 const voice = voiceSaying({ modelId: 'hf:cat/Tabby-1', categories: ['omission',], dropped: 0, },);
 ```
 */
function voiceSaying(
  {
    modelId,
    categories,
    dropped,
  }: {
    readonly modelId: string;
    readonly categories: readonly string[];
    readonly dropped: number;
  },
): SettledAuditRow['report']['rows'][number] {
  return {
    modelId,
    verdict: (categories.length === 0) ? 'no-defect-found' : 'defects-found',
    findings: categories.map(function asFinding(category,): Record<string, unknown> {
      return {
        category,
        source: { kind: 'unused', },
        candidate: { kind: 'unused', },
        reason: `the cat ${category}`,
      };
    },),
    dropped: Array.from(
      { length: dropped, },
      function reason(): string {
        return 'quote not found';
      },
    ),
  } as unknown as SettledAuditRow['report']['rows'][number];
}

/**
 Builds one audited slice.

 @param runSet - archive subdirectory

 @param entryId - corpus entry

 @param sliceIndex - slice index

 @param auditsArchiveText - whether this audited the archive's own English

 @param voices - what each auditor said

 @returns Row shaped as the probe persists it

 @example
 ```ts
 const row = rowFor({ runSet: 'first', entryId: 'mittens', sliceIndex: 0, auditsArchiveText: false, voices: [], },);
 ```
 */
function rowFor(
  {
    runSet,
    entryId,
    sliceIndex,
    auditsArchiveText,
    voices,
  }: {
    readonly runSet: string;
    readonly entryId: string;
    readonly sliceIndex: number;
    readonly auditsArchiveText: boolean;
    readonly voices: readonly SettledAuditRow['report']['rows'][number][];
  },
): SettledAuditRow {
  return {
    runSet,
    entryId,
    sliceIndex,
    deliveryKind: auditsArchiveText ? 'incumbent-retained' : 'replacement-shipped',
    auditsArchiveText,
    artifactDigest: 'sha256-tree-v1:cafef00d',
    corpusSha: 'b'.repeat(40,),
    identityKind: 'declared',
    report: {
      corroborated: [],
      agreed: [],
      near: [],
      rows: voices,
      findings: [],
    },
  } as unknown as SettledAuditRow;
}

/**
 A quiet voice, which claims nothing.
 */
const QUIET = voiceSaying({
  modelId: 'hf:cat/Quiet-1',
  categories: [],
  dropped: 0,
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: splitFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEPARATES slices carrying the archive\'s own English from slices carrying a fresh '
            + 'rendering, because the instrument was built for the second kind and one denominator over '
            + 'both would blur its first real measurement',
          fn: async () => {
            /**
             Two archive slices and one fresh one, all claimed against.
             */
            const rows = [
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 0,
                auditsArchiveText: true,
                voices: [voiceSaying({
                  modelId: 'hf:cat/Tabby-1',
                  categories: ['omission',],
                  dropped: 0,
                },),],
              },),
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 1,
                auditsArchiveText: true,
                voices: [QUIET,],
              },),
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 2,
                auditsArchiveText: false,
                voices: [voiceSaying({
                  modelId: 'hf:cat/Tabby-1',
                  categories: ['omission', 'unsupported-addition',],
                  dropped: 0,
                },),],
              },),
            ];

            /**
             The archive half.
             */
            const archive = splitFor({
              rows,
              audits: 'archive',
            },);
            expect(archive.subjects,).toBe(2,);
            expect(archive.claimed,).toBe(1,);

            /**
             The fresh half.
             */
            const fresh = splitFor({
              rows,
              audits: 'fresh',
            },);
            expect(fresh.subjects,).toBe(1,);
            expect(fresh.claimed,).toBe(2,);
          },
        },),

        it({
          name: 'counts SLICES THAT DREW A CLAIM apart from claims, since one slice drawing five is a '
            + 'different finding from five slices drawing one and a total cannot tell them apart',
          fn: async () => {
            /**
             One noisy slice and two quiet ones.
             */
            const rows = [
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 0,
                auditsArchiveText: false,
                voices: [voiceSaying({
                  modelId: 'hf:cat/Tabby-1',
                  categories: ['omission', 'omission', 'altered-time',],
                  dropped: 0,
                },),],
              },),
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 1,
                auditsArchiveText: false,
                voices: [QUIET,],
              },),
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 2,
                auditsArchiveText: false,
                voices: [QUIET,],
              },),
            ];

            /**
             Every slice, all of them fresh.
             */
            const fresh = splitFor({
              rows,
              audits: 'fresh',
            },);
            expect(fresh.subjects,).toBe(3,);
            expect(fresh.claimed,).toBe(3,);
            expect(fresh.subjectsWithClaims,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: rateByVoice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reports how often each auditor thought a rendering was worth a claim, which is the '
            + 'reading that once found three voices disagreeing by more than an order of magnitude',
          fn: async () => {
            /**
             Two slices, one auditor speaking on both and one on neither.
             */
            const rows = [
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 0,
                auditsArchiveText: false,
                voices: [
                  voiceSaying({
                    modelId: 'hf:cat/Loud-1',
                    categories: ['omission', 'altered-time',],
                    dropped: 1,
                  },),
                  QUIET,
                ],
              },),
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 1,
                auditsArchiveText: false,
                voices: [
                  voiceSaying({
                    modelId: 'hf:cat/Loud-1',
                    categories: ['omission',],
                    dropped: 0,
                  },),
                  QUIET,
                ],
              },),
            ];

            /**
             Both auditors' rates.
             */
            const rates = rateByVoice({ rows, },);
            expect(rates.length,).toBe(2,);

            /**
             The one that kept claiming.
             */
            const loud = rates.find(function isLoud(rate,): boolean {
              return rate.modelId === 'hf:cat/Loud-1';
            },);
            expect(loud?.asked,).toBe(2,);
            expect(loud?.spoke,).toBe(2,);
            expect(loud?.claims,).toBe(3,);
            expect(loud?.dropped,).toBe(1,);

            /**
             The one that never did, which is asked twice and speaks never.
             */
            const quiet = rates.find(function isQuiet(rate,): boolean {
              return rate.modelId === 'hf:cat/Quiet-1';
            },);
            expect(quiet?.asked,).toBe(2,);
            expect(quiet?.spoke,).toBe(0,);
            expect(quiet?.claims,).toBe(0,);
          },
        },),

        it({
          name: 'REFUSES to invent a row for an auditor that never answered, because a fabricated zero '
            + 'says it was asked and stayed quiet, which is a different claim from never being reached '
            + 'and would read as the very silence the per-voice rates exist to measure',
          fn: async () => {
            /**
             One slice, answered by one voice only.
             */
            const rows = [rowFor({
              runSet: 'first',
              entryId: 'mittens',
              sliceIndex: 0,
              auditsArchiveText: false,
              voices: [voiceSaying({
                modelId: 'hf:cat/Tabby-1',
                categories: [],
                dropped: 0,
              },),],
            },),];

            /**
             Rates, which must name exactly the voice that spoke.
             */
            const rates = rateByVoice({ rows, },);
            expect(rates.length,).toBe(1,);
            expect(rates[0]?.modelId,).toBe('hf:cat/Tabby-1',);
            expect(rates[0]?.asked,).toBe(1,);
            expect(rates[0]?.answered,).toBe(1,);
          },
        },),

        it({
          name: 'COUNTS A LOST VOICE AGAINST THE ROSTER the run recorded: asked is the subject count for '
            + 'every member and answered is what arrived, so a member never heard is a row at zero '
            + 'rather than an absence, because the run file says it was asked. The printed asked= '
            + 'used to be answered, and per-model loss was unreadable from the report',
          fn: async () => {
            /**
             Two subjects; one voice answers both, one answers the first only,
             and one the roster names never answers.
             */
            const rows = [
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 0,
                auditsArchiveText: false,
                voices: [
                  voiceSaying({
                    modelId: 'hf:cat/Loud-1',
                    categories: ['omission',],
                    dropped: 0,
                  },),
                  voiceSaying({
                    modelId: 'hf:cat/Flaky-1',
                    categories: [],
                    dropped: 0,
                  },),
                ],
              },),
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 1,
                auditsArchiveText: false,
                voices: [voiceSaying({
                  modelId: 'hf:cat/Loud-1',
                  categories: [],
                  dropped: 0,
                },),],
              },),
            ];

            /**
             Rates over the recorded roster, in its order.
             */
            const rates = rateByVoice({
              rows,
              roster: [
                'hf:cat/Loud-1',
                'hf:cat/Flaky-1',
                'hf:cat/Dark-1',
              ],
            },);

            expect(rates.map(function named(rate,): string {
              return rate.modelId;
            },),).toEqual([
              'hf:cat/Loud-1',
              'hf:cat/Flaky-1',
              'hf:cat/Dark-1',
            ],);
            expect(rates.map(function counted(rate,): readonly [number, number,] {
              return [
                rate.asked,
                rate.answered,
              ];
            },),).toEqual([
              [2, 2,],
              [2, 1,],
              [2, 0,],
            ],);
          },
        },),

        it({
          name: 'APPENDS a voice the roster never listed after the roster, asked only where it answered, '
            + 'since a run file that names a roster and then hears someone else is evidence about the '
            + 'run rather than a reason to lose the rows',
          fn: async () => {
            /**
             One subject answered by a voice outside the recorded roster.
             */
            const rows = [rowFor({
              runSet: 'first',
              entryId: 'mittens',
              sliceIndex: 0,
              auditsArchiveText: false,
              voices: [voiceSaying({
                modelId: 'hf:cat/Stray-1',
                categories: [],
                dropped: 0,
              },),],
            },),];

            /**
             Rates over a roster that names one member only.
             */
            const rates = rateByVoice({
              rows,
              roster: ['hf:cat/Loud-1',],
            },);

            expect(rates.map(function named(rate,): string {
              return rate.modelId;
            },),).toEqual([
              'hf:cat/Loud-1',
              'hf:cat/Stray-1',
            ],);
            expect(rates[1]?.asked,).toBe(1,);
            expect(rates[1]?.answered,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: auditRelocationPairs.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'pairs a passage called MISSING on one slice with a passage called UNSUPPORTED on the '
            + 'next, which is what one relocation looks like to per-slice judging that cannot see the '
            + 'move',
          fn: async () => {
            /**
             A dropped passage and, next door, an unaccounted one.
             */
            const rows = [
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 3,
                auditsArchiveText: true,
                voices: [voiceSaying({
                  modelId: 'hf:cat/Tabby-1',
                  categories: ['omission',],
                  dropped: 0,
                },),],
              },),
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 4,
                auditsArchiveText: true,
                voices: [voiceSaying({
                  modelId: 'hf:cat/Tabby-1',
                  categories: ['unsupported-addition',],
                  dropped: 0,
                },),],
              },),
            ];

            /**
             What the rule names.
             */
            const candidates = auditRelocationPairs({ rows, },);
            expect(candidates.length,).toBe(1,);
            expect(candidates[0]?.omissionAt,).toBe(3,);
            expect(candidates[0]?.additionAt,).toBe(4,);
            // The reasons travel, so a reader can judge the pairing without opening
            // the run it came from.
            expect(candidates[0]?.omissionReason,).toContain('omission',);
            expect(candidates[0]?.additionReason,).toContain('unsupported-addition',);
            expect(distinctSlicePairs({ pairs: candidates, },),).toBe(1,);
          },
        },),

        it({
          name: 'COUNTS CLAIM PAIRS AND SLICE PAIRS APART when several voices file on both slices: '
            + 'three omissions beside two additions are six claim pairs over one pair of slices, and a '
            + 'heading reading six would be quoted as six relocations',
          fn: async () => {
            /**
             Three voices calling one passage missing, two calling it invented
             next door.
             */
            const rows = [
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 3,
                auditsArchiveText: true,
                voices: [
                  'hf:cat/Tabby-1',
                  'hf:cat/Loud-1',
                  'hf:cat/Quiet-1',
                ].map(function omitting(modelId,) {
                  return voiceSaying({
                    modelId,
                    categories: ['omission',],
                    dropped: 0,
                  },);
                },),
              },),
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 4,
                auditsArchiveText: true,
                voices: [
                  'hf:cat/Tabby-1',
                  'hf:cat/Loud-1',
                ].map(function adding(modelId,) {
                  return voiceSaying({
                    modelId,
                    categories: ['unsupported-addition',],
                    dropped: 0,
                  },);
                },),
              },),
            ];

            /**
             Every claim pairing the rule names.
             */
            const candidates = auditRelocationPairs({ rows, },);
            expect(candidates.length,).toBe(6,);
            expect(distinctSlicePairs({ pairs: candidates, },),).toBe(1,);
          },
        },),

        it({
          name: 'REFUSES to pair across two runs of one entry, since both write the same entry id and '
            + 'the same slice indices, and a pairing that crossed them would report a relocation no '
            + 'document contains',
          fn: async () => {
            /**
             The same two slice indices, settled in two different runs.
             */
            const rows = [
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 3,
                auditsArchiveText: true,
                voices: [voiceSaying({
                  modelId: 'hf:cat/Tabby-1',
                  categories: ['omission',],
                  dropped: 0,
                },),],
              },),
              rowFor({
                runSet: 'second',
                entryId: 'mittens',
                sliceIndex: 4,
                auditsArchiveText: true,
                voices: [voiceSaying({
                  modelId: 'hf:cat/Tabby-1',
                  categories: ['unsupported-addition',],
                  dropped: 0,
                },),],
              },),
            ];

            expect(auditRelocationPairs({ rows, },).length,).toBe(0,);
          },
        },),

        it({
          name: 'REFUSES to pair slices that are not neighbours, because a moved passage lands next '
            + 'door and a wider reach would let the rule absorb unrelated findings until nothing was '
            + 'ever a defect',
          fn: async () => {
            /**
             An omission and an addition three slices apart.
             */
            const rows = [
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 0,
                auditsArchiveText: true,
                voices: [voiceSaying({
                  modelId: 'hf:cat/Tabby-1',
                  categories: ['omission',],
                  dropped: 0,
                },),],
              },),
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 3,
                auditsArchiveText: true,
                voices: [voiceSaying({
                  modelId: 'hf:cat/Tabby-1',
                  categories: ['unsupported-addition',],
                  dropped: 0,
                },),],
              },),
            ];

            expect(auditRelocationPairs({ rows, },).length,).toBe(0,);
          },
        },),

        it({
          name: 'names nothing when an omission has no addition beside it, so a genuine dropped passage '
            + 'is not quietly explained away as a move',
          fn: async () => {
            /**
             One omission, with a quiet neighbour.
             */
            const rows = [
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 0,
                auditsArchiveText: true,
                voices: [voiceSaying({
                  modelId: 'hf:cat/Tabby-1',
                  categories: ['omission',],
                  dropped: 0,
                },),],
              },),
              rowFor({
                runSet: 'first',
                entryId: 'mittens',
                sliceIndex: 1,
                auditsArchiveText: true,
                voices: [QUIET,],
              },),
            ];

            expect(auditRelocationPairs({ rows, },).length,).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
