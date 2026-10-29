/**
 Tests for restoration grading and the milestone-two repair benchmark.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import type { RepairModels, } from '@monochromatic-dev/translation-repair-assembly';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  contentWords,
  measureSeedRestoration,
} from '@monochromatic-dev/translation-repair-markup';
import {
  applySeededErrors,
  gradeSeedDetection,
  type SeededErrorSpec,
} from '@monochromatic-dev/translation-repair-lane-contest';
import {
  computeRepairScorecard,
  type RepairAttemptRecord,
  type repairTranslation,
  runRepairBenchmark,
  type runRestorationJudge,
} from '../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import {
  quotingFailure,
  WHISKER_KEY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  HOUR_MS,
  stubWallClock,
  WALL_START_MS,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/wall-clock-stub.test-fixture.ts';
import { sliceCoversApplication, } from '@monochromatic-dev/translation-repair-lane-contest/ts/slice-covers-application.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Clean fixture translation the seed deletes from.
 */
const CLEAN_TEXT =
  'The cat naps in the sun. The cat also chases crimson butterflies across the meadow. The bowl stays full.';

/**
 Deletion seed removing the butterfly sentence.
 */
const BUTTERFLY_SEED: SeededErrorSpec = {
  id: 'seed/omission-0',
  category: 'accuracy/omission',
  kind: 'deletion',
  needle: ' The cat also chases crimson butterflies across the meadow.',
  replacement: '',
};

/**
 Characters one fixture span covers, wide enough to overlap the planted
 region without running past the block.
 */
const SPAN_WIDTH = 10;

/**
 Fixture text with the butterfly sentence already deleted.
 */
const SEEDED_TEXT = 'The cat naps in the sun. The bowl stays full.';

/**
 Role roster; identities only matter as distinct voices.
 */
const MODELS: RepairModels = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  judgeModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
  checkerModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
};

/**
 Client stand-in; the injected repair seam keeps it uncalled.
 */
const UNUSED_CLIENT = {
  chatText: async () => {
    throw new Error('unused',);
  },
  chatJson: async () => {
    throw new Error('unused',);
  },
  quotas: async () => {
    throw new Error('unused',);
  },
};

/**
 Judge stub ruling every reference restored through the seam.
 */
const restoringJudge: typeof runRestorationJudge = async ({ references, },) =>
  Object.fromEntries(references.map(function toVerdict(reference,) {
    return [
      reference.seedId,
      {
        verdict: 'restored' as const,
        judged: true,
        votes: 3,
      },
    ];
  },),)
  ;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: contentWords.name,
      children: [
        it({
          name: 'collects distinct long words and drops short ones',
          fn: async () => {
            /** Vocabulary of a short sentence. */
            const words = contentWords({ text: 'The cat naps; the CAT chases butterflies!', },);
            expect(words.has('chases',),).toBe(true,);
            expect(words.has('butterflies',),).toBe(true,);
            expect(words.has('naps',),).toBe(true,);
            expect(words.has('cat',),).toBe(false,);
            expect(words.has('the',),).toBe(false,);
          },
        },),
        it({
          name: 'folds an accented word to one word, composed or combining, instead of cutting it at the accent (ledger B18)',
          fn: async () => {
            expect([...contentWords({ text: 'The château kept Émilie; the cha\u{0302}teau kept Emilie\'s yarn.', },),].toSorted(),)
              .toEqual(['chateau', 'emilie', 'emilie\'s', 'kept', 'yarn',],);
          },
        },),
      ],
    },),

    describe({
      name: measureSeedRestoration.name,
      children: [
        it({
          name: 'grades a re-translated restoration as restored',
          fn: async () => {
            /** Repair wording differs but restores the distinctive vocabulary. */
            const grade = measureSeedRestoration({
              needle: BUTTERFLY_SEED.needle,
              seededText: SEEDED_TEXT,
              repairedText:
                'The cat naps in the sun. The cat loves to chase crimson butterflies over the meadow. The bowl stays full.',
            },);
            expect(grade.measurable,).toBe(true,);
            // Disappeared vocabulary: also, chases, crimson, butterflies,
            // across, meadow; the repair returns crimson, butterflies, meadow.
            expect(grade.restored,).toBe(true,);
            expect(grade.returnedWords,).toBe(3,);
          },
        },),

        it({
          name: 'grades an untouched text as not restored',
          fn: async () => {
            /** Repair that never brought the sentence back. */
            const grade = measureSeedRestoration({
              needle: BUTTERFLY_SEED.needle,
              seededText: SEEDED_TEXT,
              repairedText: SEEDED_TEXT,
            },);
            expect(grade.measurable,).toBe(true,);
            expect(grade.restored,).toBe(false,);
            expect(grade.returnedWords,).toBe(0,);
          },
        },),

        it({
          name: 'marks a needle whose vocabulary survives elsewhere as unmeasurable',
          fn: async () => {
            /** Needle repeating vocabulary the seeded text keeps. */
            const grade = measureSeedRestoration({
              needle: 'The bowl stays full.',
              seededText: SEEDED_TEXT,
              repairedText: SEEDED_TEXT,
            },);
            expect(grade.measurable,).toBe(false,);
            expect(grade.restored,).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: computeRepairScorecard.name,
      children: [
        it({
          name: 'rates restoration over measurable seeds and reports coverage',
          fn: async () => {
            /** Two dispatched attempts and one budget skip. */
            const records: readonly RepairAttemptRecord[] = [
              {
                entryId: 'whiskers',
                outcomeKind: 'ok',
                status: 'repaired',
                seedJudgments: {
                  'seed/omission-0': {
                    verdict: 'restored',
                    judged: true,
                    votes: 3,
                  },
                  'seed/omission-1': {
                    verdict: 'partial',
                    judged: true,
                    votes: 3,
                  },
                },
                seedDerivability: {},
                seedGrades: {
                  'seed/omission-0': {
                    measurable: true,
                    disappearedWords: 6,
                    returnedWords: 4,
                    restored: true,
                  },
                  'seed/omission-1': {
                    measurable: false,
                    disappearedWords: 0,
                    returnedWords: 0,
                    restored: false,
                  },
                },
                seedDetection: {
                  'seed/omission-0': 'accepted',
                  'seed/omission-1': 'accepted',
                },
                issueCount: 3,
                resolvedIssueCount: 2,
                detail: '',
              },
              {
                entryId: 'mittens',
                outcomeKind: 'ok',
                status: 'unchanged',
                seedJudgments: {
                  // One judged absent, one that never reached quorum.
                  'seed/omission-0': {
                    verdict: 'absent',
                    judged: true,
                    votes: 3,
                  },
                  'seed/omission-1': {
                    verdict: 'absent',
                    judged: false,
                    votes: 0,
                  },
                },
                seedDerivability: {},
                seedGrades: {
                  'seed/omission-0': {
                    measurable: true,
                    disappearedWords: 5,
                    returnedWords: 0,
                    restored: false,
                  },
                },
                seedDetection: { 'seed/omission-0': 'declined-protective', },
                issueCount: 1,
                resolvedIssueCount: 0,
                detail: '',
              },
              {
                entryId: 'shadow',
                outcomeKind: 'skipped',
                seedJudgments: {},
                seedDerivability: {},
                seedGrades: {},
                seedDetection: {},
                issueCount: 0,
                resolvedIssueCount: 0,
                detail: 'run-budget-exhausted',
              },
            ];
            /** Scorecard over the mixed records. */
            const scorecard = computeRepairScorecard({ records, },);
            expect(scorecard.dispatchedEntries,).toBe(2,);
            expect(scorecard.coverage,).toBe(2 / 3,);
            // Judge denominator excludes the unjudged (no-quorum) seed: 3 judged.
            expect(scorecard.judgedSeeds,).toBe(3,);
            expect(scorecard.restoredSeeds,).toBe(1,);
            expect(scorecard.partialSeeds,).toBe(1,);
            expect(scorecard.seededRepairRate,).toBe(1 / 3,);
            expect(scorecard.seededRepairRateLenient,).toBe(2 / 3,);
            // Lexical grade kept for comparison: 2 measurable, 1 restored.
            expect(scorecard.lexicalUniverse,).toBe(2,);
            expect(scorecard.lexicalRestoredSeeds,).toBe(1,);
            expect(scorecard.lexicalRepairRate,).toBe(1 / 2,);
            expect(scorecard.plantedSeeds,).toBe(3,);
            expect(scorecard.detectedSeeds,).toBe(2,);
            expect(scorecard.seedDetectionRate,).toBe(2 / 3,);
            // The protective decline stays IN the raw denominator and is reported
            // beside it, so a verdict can cite either number but never silently
            // swap one for the other.
            expect(scorecard.policyDeclinedSeeds,).toBe(1,);
            expect(scorecard.seedDetectionRateExcludingPolicy,).toBe(1,);
            expect(scorecard.statusCounts.repaired,).toBe(1,);
            expect(scorecard.statusCounts.unchanged,).toBe(1,);
            // Records written before the probe was wired carry no derivability,
            // and an absent verdict counts as derivable rather than as unfair, so
            // the fair denominator equals the repairable one here.
            expect(scorecard.nonDerivableSeeds,).toBe(0,);
            expect(scorecard.seedDetectionRateExcludingUnfair,).toBe(1,);
          },
        },),

        it({
          name: 'scores a benchmark over no entries as fully covered and every rate as zero: coverage asks '
            + 'what share of the entries ran, and none were skipped, while a rate asks what share of seeds '
            + 'came back, and none were planted',
          fn: async () => {
            expect(computeRepairScorecard({ records: [], },),).toEqual({
              dispatchedEntries: 0,
              coverage: 1,
              plantedSeeds: 0,
              detectedSeeds: 0,
              seedDetectionRate: 0,
              policyDeclinedSeeds: 0,
              seedDetectionRateExcludingPolicy: 0,
              nonDerivableSeeds: 0,
              seedDetectionRateExcludingUnfair: 0,
              judgedSeeds: 0,
              restoredSeeds: 0,
              partialSeeds: 0,
              seededRepairRate: 0,
              seededRepairRateLenient: 0,
              lexicalUniverse: 0,
              lexicalRestoredSeeds: 0,
              lexicalRepairRate: 0,
              statusCounts: {},
            },);
          },
        },),

        it({
          name: 'holds a NOT-DERIVABLE seed out of the fair denominator, since the '
            + 'benchmark plants seeds by deleting published English and published '
            + 'English may carry a translator addition the Chinese never stated. '
            + 'Deleting one leaves a hole no reader of the source could find, so '
            + 'counting it as a miss scores the pipeline on a question nobody asked',
          fn: async () => {
            /**
             One entry: two seeds detected, one missed and not derivable.
             */
            const records: readonly RepairAttemptRecord[] = [
              {
                entryId: 'mittens',
                outcomeKind: 'ok',
                status: 'repaired',
                seedJudgments: {},
                seedDerivability: {
                  'seed/omission-0': {
                    verdict: 'derivable',
                    judged: true,
                    votes: 3,
                  },
                  'seed/omission-1': {
                    verdict: 'not-derivable',
                    judged: true,
                    votes: 3,
                  },
                },
                seedGrades: {},
                seedDetection: {
                  'seed/omission-0': 'accepted',
                  'seed/omission-1': 'undetected',
                },
                issueCount: 1,
                resolvedIssueCount: 1,
                detail: '',
              },
            ];

            /** Scorecard over the one entry. */
            const scorecard = computeRepairScorecard({ records, },);

            expect(scorecard.nonDerivableSeeds,).toBe(1,);
            // Raw rate counts the unfindable seed against detection: 1 of 2.
            expect(scorecard.seedDetectionRate,).toBe(1 / 2,);
            // The fair rate does not: 1 of 1.
            expect(scorecard.seedDetectionRateExcludingUnfair,).toBe(1,);
          },
        },),

        it({
          name: 'counts a seed that is BOTH policy-declined and not-derivable only '
            + 'ONCE. Subtracting the two tallies from the total would remove it '
            + 'twice, shrinking the denominator and making the rate too generous '
            + 'by exactly the overlap, which is why the two records are joined per '
            + 'seed id rather than counted separately',
          fn: async () => {
            /**
             One entry whose second seed is excluded for both reasons at once.
             */
            const records: readonly RepairAttemptRecord[] = [
              {
                entryId: 'whiskers',
                outcomeKind: 'ok',
                status: 'repaired',
                seedJudgments: {},
                seedDerivability: {
                  'seed/omission-1': {
                    verdict: 'not-derivable',
                    judged: true,
                    votes: 3,
                  },
                },
                seedGrades: {},
                seedDetection: {
                  'seed/omission-0': 'accepted',
                  'seed/omission-1': 'declined-protective',
                },
                issueCount: 1,
                resolvedIssueCount: 1,
                detail: '',
              },
            ];

            /** Scorecard over the overlapping exclusion. */
            const scorecard = computeRepairScorecard({ records, },);

            expect(scorecard.policyDeclinedSeeds,).toBe(1,);
            expect(scorecard.nonDerivableSeeds,).toBe(1,);
            // One seed remains fair, not zero. Naive subtraction would give
            // 2 - 1 - 1 = 0 and then a rate of 0 for a run that detected
            // everything it could.
            expect(scorecard.seedDetectionRateExcludingUnfair,).toBe(1,);
          },
        },),

        it({
          name: 'ignores an UNJUDGED not-derivable verdict, so a probe that lost '
            + 'its quorum cannot excuse a detection miss. An unheard probe and a '
            + 'probe that found the seed underivable must never read alike',
          fn: async () => {
            /**
             One entry whose miss carries an unjudged derivability verdict.
             */
            const records: readonly RepairAttemptRecord[] = [
              {
                entryId: 'shadow',
                outcomeKind: 'ok',
                status: 'repaired',
                seedJudgments: {},
                seedDerivability: {
                  'seed/omission-1': {
                    verdict: 'not-derivable',
                    judged: false,
                    votes: 0,
                  },
                },
                seedGrades: {},
                seedDetection: {
                  'seed/omission-0': 'accepted',
                  'seed/omission-1': 'undetected',
                },
                issueCount: 1,
                resolvedIssueCount: 1,
                detail: '',
              },
            ];

            /** Scorecard over the unjudged verdict. */
            const scorecard = computeRepairScorecard({ records, },);

            expect(scorecard.nonDerivableSeeds,).toBe(0,);
            expect(scorecard.seedDetectionRateExcludingUnfair,).toBe(1 / 2,);
          },
        },),
      ],
    },),

    describe({
      name: gradeSeedDetection.name,
      children: [
        it({
          name: 'marks seeds with accepted issues at their region and only those',
          fn: async () => {
            /** Sectioned fixture translation the seed deletes from. */
            const sectionedTarget = `## Introduction

    The cat naps in the sun. The cat also chases crimson butterflies across the meadow. The bowl stays full.
    `;
            /** Deletion planted into the sectioned fixture. */
            const { seededText, applications, } = applySeededErrors({
              text: sectionedTarget,
              specs: [BUTTERFLY_SEED,],
            },);
            /** Application region of the planted seed. */
            const [application,] = applications;
            if (application === undefined)
              throw new Error('fixture planting failed',);
            /** Accepted issue anchored at the deletion point. */
            const nearIssue = {
              sliceIndex: 0,
              resolved: false,
              repairRegions: [],
              repairDisposition: 'no-region' as const,
              refined: false,
              issue: {
                issueId: 'adjudicated/near',
                status: 'accepted' as const,
                severity: 'major' as const,
                claims: [
                  {
                    claimId: 'issue/near',
                    claim: {
                      category: 'accuracy/omission' as const,
                      severity: 'major' as const,
                      summary: 'The butterfly sentence is missing.',
                      spans: [
                        {
                          side: 'target' as const,
                          nodeId: 'block/1',
                          nodeHash: hashContent({ content: 'invented', },),
                          startOffset: application.startOffset,
                          endOffset: application.startOffset + 10,
                          quotedText: seededText.slice(
                            application.startOffset,
                            application.startOffset + 10,
                          ),
                        },
                      ],
                    },
                  },
                ],
                tallies: {},
              },
            };
            /** Detection with the accepted near issue. */
            const detected = gradeSeedDetection({
              sourceText: '## 简介\n\n猫猫在太阳下打盹。猫猫也追蝴蝶。碗是满的。\n',
              seededText,
              applications,
              issues: [nearIssue,],
            },);
            expect(detected[BUTTERFLY_SEED.id],).toBe('accepted',);
            /** Detection when the same issue is rejected. */
            const rejected = gradeSeedDetection({
              sourceText: '## 简介\n\n猫猫在太阳下打盹。猫猫也追蝴蝶。碗是满的。\n',
              seededText,
              applications,
              issues: [
                {
                  ...nearIssue,
                  issue: {
                    ...nearIssue.issue,
                    status: 'rejected' as const,
                  },
                },
              ],
            },);
            expect(rejected[BUTTERFLY_SEED.id],).toBe('declined-other',);
          },
        },),

        it({
          name: 'resolves spans through the SLICES the pipeline repaired, not the '
            + 'aligned pairs, so a seed past the first slice is still seen',
          fn: async () => {
            /**
             Filler paragraph long enough that the document subdivides; the
             slice budget is 400 target characters.
             */
            const filler = Array.from(
              { length: 6, },
              function toParagraph(
                _unused,
                index,
              ) {
                return `Paragraph ${String(index,)} about the cat, long enough to push the slicer past its budget so the document splits into several slices instead of one.`;
              },
            ).join('\n\n',);

            /** Sectioned target whose seed sits AFTER the filler. */
            const sectionedTarget =
              `## Introduction\n\n${filler}\n\nThe cat naps in the sun. The cat also chases crimson butterflies across the meadow. The bowl stays full.\n`;

            /** Deletion planted into the late paragraph. */
            const { seededText, applications, } = applySeededErrors({
              text: sectionedTarget,
              specs: [BUTTERFLY_SEED,],
            },);

            /** Application region of the planted seed. */
            const [application,] = applications;
            if (application === undefined)
              throw new Error('fixture planting failed',);

            /** Original the seeded translation is graded against. */
            const sourceText = `## 简介\n\n${filler}\n\n猫猫在太阳下打盹。猫猫也追蝴蝶。碗是满的。\n`;

            /** Slices the repair entry prepares over the seeded pair. */
            const { slices, } = prepareDocumentPair({
              sourceText,
              targetText: seededText,
            },);

            /** Slice whose target region covers the planted seed. */
            const slicePosition = slices.findIndex(function covers(slice,) {
              return sliceCoversApplication({ slice, application, },);
            },);
            // The whole point of the fixture: the seed must NOT be in slice zero,
            // because slice zero is the one case the old pair indexing got right.
            expect(slicePosition,).toBeGreaterThan(0,);

            /** Slice the seed landed in, present since `slicePosition` is past zero. */
            const slice = slices[slicePosition];
            if (slice === undefined)
              throw new Error('fixture lost its slice',);

            /** Accepted issue anchored at the deletion, in slice-local offsets. */
            const localStart = application.startOffset - slice.target.startOffset;

            /** Detection over an issue reported against that slice. */
            const detected = gradeSeedDetection({
              sourceText,
              seededText,
              applications,
              issues: [
                {
                  sliceIndex: slicePosition,
                  resolved: false,
                  repairRegions: [],
                  repairDisposition: 'no-region' as const,
                  refined: false,
                  issue: {
                    issueId: 'adjudicated/late',
                    status: 'accepted' as const,
                    severity: 'major' as const,
                    claims: [
                      {
                        claimId: 'issue/late',
                        claim: {
                          category: 'accuracy/omission' as const,
                          severity: 'major' as const,
                          summary: 'The butterfly sentence is missing.',
                          spans: [
                            {
                              side: 'target' as const,
                              nodeId: 'block/1',
                              nodeHash: hashContent({ content: 'invented', },),
                              startOffset: localStart,
                              endOffset: localStart + SPAN_WIDTH,
                              quotedText: seededText.slice(
                                application.startOffset,
                                application.startOffset + SPAN_WIDTH,
                              ),
                            },
                          ],
                        },
                      },
                    ],
                    tallies: {},
                  },
                },
              ],
            },);
            expect(detected[BUTTERFLY_SEED.id],).toBe('accepted',);
          },
        },),

        it({
          name: 'separates a seed nobody reported from one the panel declined on '
            + 'protective grounds, so house policy is not scored as a miss',
          fn: async () => {
            /** Sectioned fixture translation the seed deletes from. */
            const sectionedTarget = `## Introduction

    The cat naps in the sun. The cat also chases crimson butterflies across the meadow. The bowl stays full.
    `;
            /** Deletion planted into the sectioned fixture. */
            const { seededText, applications, } = applySeededErrors({
              text: sectionedTarget,
              specs: [BUTTERFLY_SEED,],
            },);

            /** Application region of the planted seed. */
            const [application,] = applications;
            if (application === undefined)
              throw new Error('fixture planting failed',);

            /** Original the seeded translation is graded against. */
            const sourceText = '## 简介\n\n猫猫在太阳下打盹。猫猫也追蝴蝶。碗是满的。\n';

            /**
             Planted region's start, bound out here because `const` narrowing
             does not reach into a function declaration (AGENTS.md TY8).
             */
            const regionStart = application.startOffset;

            /** Bytes the span quotes from the seeded translation. */
            const quotedText = seededText.slice(
              regionStart,
              regionStart + SPAN_WIDTH,
            );

            /**
             Builds the issue anchored at the deletion point, its status left to
             the caller so one fixture covers both declines.

             @param status - adjudication status the panel landed on

             @returns Issue record covering the seeded region

             @example
             ```ts
             const record = issueWithStatus('source-defect',);
             ```
             */
            function issueWithStatus(status: 'source-defect' | 'rejected',) {
              return {
                sliceIndex: 0,
                resolved: false,
                repairRegions: [],
                repairDisposition: 'no-region' as const,
                refined: false,
                issue: {
                  issueId: 'adjudicated/near',
                  status,
                  severity: 'major' as const,
                  claims: [
                    {
                      claimId: 'issue/near',
                      claim: {
                        category: 'accuracy/omission' as const,
                        severity: 'major' as const,
                        summary: 'The butterfly sentence is missing.',
                        spans: [
                          {
                            side: 'target' as const,
                            nodeId: 'block/1',
                            nodeHash: hashContent({ content: 'invented', },),
                            startOffset: regionStart,
                            endOffset: regionStart + SPAN_WIDTH,
                            quotedText,
                          },
                        ],
                      },
                    },
                  ],
                  tallies: {},
                },
              };
            }

            // The panel saw this region and ruled the ORIGINAL at fault, which is
            // where a policy-driven protective omission lands. Recording it as a
            // plain miss would score the pipeline's own rule as a failure.
            /** Detection when the panel declined protectively. */
            const protective = gradeSeedDetection({
              sourceText,
              seededText,
              applications,
              issues: [issueWithStatus('source-defect',),],
            },);
            expect(protective[BUTTERFLY_SEED.id],).toBe('declined-protective',);

            /** Detection when no issue was reported at the region at all. */
            const silent = gradeSeedDetection({
              sourceText,
              seededText,
              applications,
              issues: [],
            },);
            expect(silent[BUTTERFLY_SEED.id],).toBe('undetected',);
          },
        },),
      ],
    },),

    describe({
      name: runRepairBenchmark.name,
      concurrency: 1,
      children: [
        it({
          name: 'grades a scripted restoring repair through the seam',
          fn: async () => {
            /** Scripted repair that restores the butterfly sentence. */
            const restoringRepair: typeof repairTranslation = async ({ targetText, },) => {
              /** Document as this stub returns it. */
              const repairedText = `${targetText} The cat also chases crimson butterflies across the meadow.`;

              return {
                repairedText,
                status: 'repaired',
                issues: [],
                findings: [],
                sliceCritics: [],

                // No slice outcome, because this stub never runs one: it rewrites
                // the document whole. An empty list here says the same thing the
                // `sliceCount` field does, from the other side.
                chunks: [],

                // One slice, since this stub does not slice at all: the count is
                // what every index set below is out of, and a benchmark reading a
                // rate needs the denominator as much as the numerator.
                sliceCount: 1,

                // The stub rewrites the document whole rather than by slice, so it
                // names the one slice that stands for it. A changed document with
                // no shipped slice would state a thing the contract cannot mean,
                // and the `sliceTexts` entry is that slice as both sides saw it.
                changedSliceIndices: [0,],
                withdrawnSliceIndices: [],
                trimmedReplacements: [],
                sliceTexts: [{
                  sliceIndex: 0,
                  incumbentKind: 'present',
                  incumbentText: targetText,
                  outcome: {
                    kind: 'decided',
                    acceptedText: repairedText,
                  },
                },],
              };
            };
            /** Benchmark over one entry. */
            const { records, scorecard, } = await runRepairBenchmark({
              client: UNUSED_CLIENT,
              judgeModelIds: MODELS.judgeModelIds,
              entries: [
                {
                  entryId: 'whiskers',
                  sourceText: '猫猫在太阳下打盹。猫猫也追蝴蝶。碗是满的。',
                  targetText: CLEAN_TEXT,
                  seeds: [BUTTERFLY_SEED,],
                },
              ],
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              repair: restoringRepair,
              judge: restoringJudge,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(records[0]?.outcomeKind,).toBe('ok',);
            expect(records[0]?.seedJudgments['seed/omission-0']?.verdict,).toBe('restored',);
            expect(records[0]?.seedGrades['seed/omission-0']?.restored,).toBe(true,);
            expect(scorecard.seededRepairRate,).toBe(1,);
            expect(scorecard.coverage,).toBe(1,);
          },
        },),

        it({
          name: 'skips entries the budget cannot fit and records thrown repairs as errors',
          fn: async () => {
            /** Scripted repair that always throws. */
            const throwingRepair: typeof repairTranslation = async () => {
              throw new Error('scripted transport collapse',);
            };
            /** Benchmark whose budget is already exhausted at start. */
            const skipped = await runRepairBenchmark({
              client: UNUSED_CLIENT,
              judgeModelIds: MODELS.judgeModelIds,
              entries: [
                {
                  entryId: 'whiskers',
                  sourceText: '猫',
                  targetText: CLEAN_TEXT,
                  seeds: [BUTTERFLY_SEED,],
                },
              ],
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              runBudgetMs: 0,
              repair: throwingRepair,
              judge: restoringJudge,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(skipped.records[0]?.outcomeKind,).toBe('skipped',);
            expect(skipped.scorecard.coverage,).toBe(0,);
            /** Benchmark whose repair throws. */
            const errored = await runRepairBenchmark({
              client: UNUSED_CLIENT,
              judgeModelIds: MODELS.judgeModelIds,
              entries: [
                {
                  entryId: 'whiskers',
                  sourceText: '猫',
                  targetText: CLEAN_TEXT,
                  seeds: [BUTTERFLY_SEED,],
                },
              ],
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              repair: throwingRepair,
              judge: restoringJudge,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(errored.records[0]?.outcomeKind,).toBe('error',);
            expect(errored.records[0]?.detail,).toBe('refused by Error',);
          },
        },),

        it({
          name: 'RECORDS A THROWN REPAIR BY CLASS AND HTTP STATUS and never by the message, which a provider body or an '
            + 'unsendable header can fill',
          fn: async () => {
            /** Entry whose repair collapses. */
            const entry = {
              entryId: 'whiskers',
              sourceText: '猫',
              targetText: CLEAN_TEXT,
              seeds: [BUTTERFLY_SEED,],
            };
            /** Benchmark whose repair raises a runtime failure quoting a header value. */
            const headerRefusal = await runRepairBenchmark({
              client: UNUSED_CLIENT,
              judgeModelIds: MODELS.judgeModelIds,
              entries: [entry,],
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              repair: async () => {
                throw quotingFailure();
              },
              judge: restoringJudge,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            /** Benchmark whose repair raises a provider status failure carrying a body excerpt. */
            const statusRefusal = await runRepairBenchmark({
              client: UNUSED_CLIENT,
              judgeModelIds: MODELS.judgeModelIds,
              entries: [entry,],
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              repair: async () => {
                throw new ProviderHttpError({
                  provider: 'synthetic',
                  status: 429,
                  bodyText: `the cat key ${WHISKER_KEY} is over its weekly credit`,
                },);
              },
              judge: restoringJudge,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(headerRefusal.records[0]?.detail,).toBe('refused by TypeError',);
            expect(statusRefusal.records[0]?.detail,).toBe('refused by ProviderHttpError from synthetic with HTTP 429',);
          },
        },),

        it({
          name: 'LOGS A THROWN REPAIR BY CLASS AND HTTP STATUS, then the provider\'s own words labelled as its own, '
            + 'where its record keeps the class and status alone',
          fn: async () => {
            const { result, warned, } = await warnLinesDuring({
              run: async () =>
                runRepairBenchmark({
                  client: UNUSED_CLIENT,
                  judgeModelIds: MODELS.judgeModelIds,
                  entries: [
                    {
                      entryId: 'whiskers',
                      sourceText: '猫',
                      targetText: CLEAN_TEXT,
                      seeds: [BUTTERFLY_SEED,],
                    },
                  ],
                  models: MODELS,
                  signal: new AbortController().signal,
                  perCallTimeoutMs: HANG_STOP_MS,
                  repair: async () => {
                    throw new ProviderHttpError({ provider: 'synthetic', status: 429, bodyText: 'the cat is over its weekly credit', },);
                  },
                  judge: restoringJudge,
                  stageSettings: SCRIPTED_STAGE_SETTINGS,
                },),
            },);
            expect(warned.filter(function thrown(line,): boolean {
              return line.includes('repair threw',);
            },),).toEqual([
              '[translation-repair-repair-benchmark] [runRepairBenchmark] whiskers: repair threw refused by '
                + 'ProviderHttpError from synthetic with HTTP 429 (the provider said: "the cat is over its weekly '
                + 'credit")',
            ],);
            expect(result.records[0]?.detail,).toBe('refused by ProviderHttpError from synthetic with HTTP 429',);
          },
        },),

        it({
          name: 'counts the issues a run reported and, apart, those its checkers confirmed fixed, so a '
            + 'record says how much of what was found was mended',
          fn: async () => {
            /**
             Issue record as the driver reports one, fixed or not.

             @param issueId - issue this record is about

             @param resolved - whether the checkers confirmed the shipped text fixes it

             @returns Record the scripted repair reports
             */
            function reportedIssue(
              {
                issueId,
                resolved,
              }: {
                readonly issueId: string;
                readonly resolved: boolean;
              },
            ) {
              return {
                sliceIndex: 0,
                resolved,
                repairRegions: [],
                repairDisposition: 'no-region' as const,
                refined: false,
                issue: {
                  issueId,
                  status: 'accepted' as const,
                  severity: 'major' as const,
                  claims: [
                    {
                      claimId: `claim/${issueId}`,
                      claim: {
                        category: 'accuracy/omission' as const,
                        severity: 'major' as const,
                        summary: 'The sentence about the full bowl is missing.',
                        spans: [
                          {
                            side: 'target' as const,
                            nodeId: 'block/1',
                            nodeHash: hashContent({ content: 'invented', },),
                            startOffset: 0,
                            endOffset: 3,
                            quotedText: 'The',
                          },
                        ],
                      },
                    },
                  ],
                  tallies: {},
                },
              };
            }
            /** Scripted repair reporting one fixed issue and one it did not fix. */
            const reportingRepair: typeof repairTranslation = async ({ targetText, },) => {
              /** Document as this stub returns it. */
              const repairedText = `${targetText} The cat also chases crimson butterflies across the meadow.`;
              return {
                repairedText,
                status: 'repaired',
                issues: [
                  reportedIssue({
                    issueId: 'adjudicated/tail',
                    resolved: true,
                  },),
                  reportedIssue({
                    issueId: 'adjudicated/ears',
                    resolved: false,
                  },),
                ],
                findings: [],
                sliceCritics: [],
                chunks: [],
                sliceCount: 1,
                changedSliceIndices: [0,],
                withdrawnSliceIndices: [],
                trimmedReplacements: [],
                sliceTexts: [{
                  sliceIndex: 0,
                  incumbentKind: 'present',
                  incumbentText: targetText,
                  outcome: {
                    kind: 'decided',
                    acceptedText: repairedText,
                  },
                },],
              };
            };
            /** Benchmark over one entry. */
            const { records, } = await runRepairBenchmark({
              client: UNUSED_CLIENT,
              judgeModelIds: MODELS.judgeModelIds,
              entries: [
                {
                  entryId: 'whiskers',
                  sourceText: '猫猫在太阳下打盹。猫猫也追蝴蝶。碗是满的。',
                  targetText: CLEAN_TEXT,
                  seeds: [BUTTERFLY_SEED,],
                },
              ],
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              repair: reportingRepair,
              judge: restoringJudge,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect([
              records[0]?.issueCount,
              records[0]?.resolvedIssueCount,
            ],).toEqual([2, 1,],);
          },
        },),

        it({
          name: 'RAISES THE CALLER\'S ABORT REASON when an entry fails under an aborted signal, not the '
            + 'exchange the abort tore down, so a caller tells a stop it asked for from a fault by identity',
          fn: async () => {
            /** Caller steering the benchmark. */
            const controller = new AbortController();
            /** Why the caller stopped it. */
            const stop = new Error('scripted steering stop',);
            /** Scripted repair whose exchange the caller's abort tears down. */
            const abortedRepair: typeof repairTranslation = async () => {
              controller.abort(stop,);
              throw new Error('scripted exchange torn down by the abort',);
            };
            await expect(runRepairBenchmark({
              client: UNUSED_CLIENT,
              judgeModelIds: MODELS.judgeModelIds,
              entries: [
                {
                  entryId: 'whiskers',
                  sourceText: '猫',
                  targetText: CLEAN_TEXT,
                  seeds: [BUTTERFLY_SEED,],
                },
              ],
              models: MODELS,
              signal: controller.signal,
              perCallTimeoutMs: HANG_STOP_MS,
              repair: abortedRepair,
              judge: restoringJudge,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },),).rejects.toBe(stop,);
          },
        },),
      ],
    },),

    describe({
      name: 'the run budget on a clock the system time cannot move (ledger B78)',
      children: [
        it({
          name: 'RUNS THE NEXT ENTRY when the system clock is set forward two hours during the first, under a budget '
            + 'of an hour that milliseconds of real work leave almost whole, where the wall clock read the budget '
            + 'spent and skipped it',
          fn: async ctx => {
            const wall = stubWallClock({ sinon: ctx.sinon, atMs: WALL_START_MS, },);
            /** Scripted repair that sets the clock forward, then fails as a transport would. */
            const steppingRepair: typeof repairTranslation = async () => {
              wall.step({ byMs: 2 * HOUR_MS, },);
              throw new Error('scripted transport collapse',);
            };
            /** Benchmark over two entries on an hour's budget. */
            const { records, } = await runRepairBenchmark({
              client: UNUSED_CLIENT,
              judgeModelIds: MODELS.judgeModelIds,
              entries: [
                {
                  entryId: 'whiskers',
                  sourceText: '猫',
                  targetText: CLEAN_TEXT,
                  seeds: [BUTTERFLY_SEED,],
                },
                {
                  entryId: 'mittens',
                  sourceText: '猫',
                  targetText: CLEAN_TEXT,
                  seeds: [BUTTERFLY_SEED,],
                },
              ],
              models: MODELS,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              runBudgetMs: HOUR_MS,
              repair: steppingRepair,
              judge: restoringJudge,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(records.map(function kindOf(record,): string {
              return record.outcomeKind;
            },),).toEqual([
              'error',
              'error',
            ],);
          },
        },),
      ],
    },),
  ],
},);
