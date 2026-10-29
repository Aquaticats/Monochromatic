/**
 Tests for when two auditors are talking about the same defect.

 THE CASES THAT DECIDE THIS INSTRUMENT are here, and both were live defects in
 the version this replaces:

 -   The FALSE SPLIT: two voices locating one dropped negation at different
     widths were counted as two lone opinions, because the key was the text
     they typed.
 -   The FALSE MERGE, which is worse: two voices finding DIFFERENT changed
     numbers in one sentence were counted as one twice-confirmed defect,
     because a character floor had forced both to quote the whole sentence.

 A matcher that fixes only the first by loosening the key re-creates the
 second, so the tests for both sit in one file where neither can be relaxed
 without the other failing.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  anchorLocatedSpan,
  type AuditMemberClaim,
  corroborate,
  corroborateByOverlap,
  nearMisses,
  type RenderingAuditCategory,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Original carrying two counts in one sentence, plus a negation.
 */
const SOURCE_TEXT = '三只猫住在书店的阁楼里。她们不吃罐头。晚上两只猫睡在窗台上，一只猫睡在书架上。';

/**
 Rendering that flips the negation and changes BOTH counts.
 */
const CANDIDATE_TEXT = 'Three cats live in the bookshop attic. They eat canned food. '
  + 'At night three cats sleep on the windowsill, two cats sleep on the bookshelf.';

/**
 Sentence both number claims fall inside, which is what a padded quote would
 have collapsed them onto.
 */
const COUNT_SENTENCE_SOURCE = '晚上两只猫睡在窗台上，一只猫睡在书架上';

/**
 Same sentence in the rendering.
 */
const COUNT_SENTENCE_CANDIDATE = 'At night three cats sleep on the windowsill, two cats sleep on the bookshelf';

/**
 Builds one screened claim by anchoring it the way the screen would.

 @param modelId - voice making the claim

 @param category - what it calls the defect

 @param sourceLocator - original span identifying the occurrence

 @param sourceFocus - original span carrying the change

 @param candidateLocator - candidate span identifying the occurrence

 @param candidateFocus - candidate span carrying the change

 @returns Claim in the shape the matcher reads

 @example
 ```ts
 const claim = claimOf({ modelId: 'a', category: 'altered-number', ... },);
 ```
 */
function claimOf(
  {
    modelId,
    category,
    sourceLocator,
    sourceFocus,
    candidateLocator,
    candidateFocus,
  }: {
    readonly modelId: RosterModelId;
    readonly category: RenderingAuditCategory;
    readonly sourceLocator: string;
    readonly sourceFocus: string;
    readonly candidateLocator: string;
    readonly candidateFocus: string;
  },
): AuditMemberClaim {
  /**
   Where the claim sits in the original.
   */
  const source = anchorLocatedSpan({
    text: SOURCE_TEXT,
    locator: sourceLocator,
    focus: sourceFocus,
    side: 'source',
  },);

  /**
   Where it sits in the rendering.
   */
  const candidate = anchorLocatedSpan({
    text: CANDIDATE_TEXT,
    locator: candidateLocator,
    focus: candidateFocus,
    side: 'candidate',
  },);

  if ((!source.anchored) || (!candidate.anchored))
    throw new Error(`fixture claim did not anchor, so the case would prove nothing: ${JSON.stringify([source, candidate,],)}`,);

  return {
    modelId,
    finding: {
      category,
      source: {
        kind: 'anchored',
        locator: source.locator,
        focus: source.focus,
      },
      candidate: {
        kind: 'anchored',
        locator: candidate.locator,
        focus: candidate.focus,
      },
      reason: `${modelId} on ${category}`,
    },
  };
}

/**
 First voice's claim about the two cats becoming three.
 */
const TWO_TO_THREE_A = claimOf({
  modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  category: 'altered-number',
  sourceLocator: COUNT_SENTENCE_SOURCE,
  sourceFocus: '两',
  candidateLocator: COUNT_SENTENCE_CANDIDATE,
  candidateFocus: 'three',
},);

/**
 Second voice's claim about the same count, quoting a narrower locator.
 */
const TWO_TO_THREE_B = claimOf({
  modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
  category: 'altered-number',
  sourceLocator: '晚上两只猫睡在窗台上',
  sourceFocus: '两',
  candidateLocator: 'At night three cats sleep on the windowsill',
  candidateFocus: 'three',
},);

/**
 A DIFFERENT defect in the SAME sentence: the one cat that became two.
 */
const ONE_TO_TWO = claimOf({
  modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  category: 'altered-number',
  sourceLocator: COUNT_SENTENCE_SOURCE,
  sourceFocus: '一',
  candidateLocator: COUNT_SENTENCE_CANDIDATE,
  candidateFocus: 'two cats sleep on the bookshelf',
},);

/**
 Builds one screened omission of the denial: the original side anchored the
 way the screen anchors it and the candidate side unused, the one shape the
 screen hands on for a category that rests on the original alone.

 @param modelId - voice making the claim

 @param reason - what the voice says the span amounts to

 @returns Claim in the shape the matcher reads

 @example
 ```ts
 const claim = denialLeftOutBy({ modelId: SEAT_HYPER_VISION, reason: 'the denial is gone', },);
 ```
 */
function denialLeftOutBy(
  {
    modelId,
    reason,
  }: {
    readonly modelId: RosterModelId;
    readonly reason: string;
  },
): AuditMemberClaim {
  /**
   Where the denial sits in the original.
   */
  const source = anchorLocatedSpan({
    text: SOURCE_TEXT,
    locator: '她们不吃罐头',
    focus: '不',
    side: 'source',
  },);

  if (!source.anchored)
    throw new Error(`fixture claim did not anchor, so the case would prove nothing: ${JSON.stringify(source,)}`,);

  return {
    modelId,
    finding: {
      category: 'omission',
      source: {
        kind: 'anchored',
        locator: source.locator,
        focus: source.focus,
      },
      candidate: { kind: 'unused', },
      reason,
    },
  };
}

/**
 First voice's claim that the rendering left the denial out.
 */
const DENIAL_LEFT_OUT_A = denialLeftOutBy({
  modelId: SEAT_HYPER_VISION,
  reason: 'the rendering leaves the denial out',
},);

/**
 Second voice's claim about the same denial, in its own words.
 */
const DENIAL_LEFT_OUT_B = denialLeftOutBy({
  modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  reason: 'the denial is missing from the rendering',
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: corroborate.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'CORROBORATES two voices who located one defect at different widths, which is the false split '
              + 'that keying on quoted text produced: what they have in common is a position, not a phrasing',
          fn: async () => {
            const defects = corroborate({
              claims: [
                TWO_TO_THREE_A,
                TWO_TO_THREE_B,
              ],
            },);
            expect(defects,).toHaveLength(1,);
            expect(defects[0]
              ?.voices,).toBe(2,);
            expect(defects[0]
              ?.members,).toHaveLength(2,);
          },
        },),
        it({
          name:
            'does NOT merge two DIFFERENT defects in one sentence, which is the false merge the character '
              + 'floors caused: both claims quote the same sentence and neither voice agreed with the other '
              + 'about anything, so reporting one twice-confirmed defect would invent the agreement',
          fn: async () => {
            const defects = corroborate({
              claims: [
                TWO_TO_THREE_A,
                ONE_TO_TWO,
              ],
            },);
            expect(defects,).toEqual([],);
          },
        },),
        it({
          name:
            'does not call those two a near miss either, because their FOCUS spans do not touch: a near miss '
              + 'says two claims might be one defect, and two claims about different words of one sentence '
              + 'demonstrably are not. Sharing a locator is not sharing a defect',
          fn: async () => {
            expect(
              nearMisses({
                claims: [
                  TWO_TO_THREE_A,
                  ONE_TO_TWO,
                ],
              },),
            ).toEqual([],);
          },
        },),
        it({
          name:
            'DOES report a near miss when the focus spans genuinely overlap without matching, which is the '
              + 'case a human has to settle: one voice quoting a word and another quoting the phrase it sits in '
              + 'may be one defect or two, and this instrument does not decide it',
          fn: async () => {
            /**
             Claim whose focus contains the other's, on both sides.
             */
            const containing = claimOf({
              modelId: SEAT_HYPER_VISION,
              category: 'altered-number',
              sourceLocator: COUNT_SENTENCE_SOURCE,
              sourceFocus: '两只猫',
              candidateLocator: COUNT_SENTENCE_CANDIDATE,
              candidateFocus: 'three cats',
            },);

            const near = nearMisses({
              claims: [
                TWO_TO_THREE_A,
                containing,
              ],
            },);
            expect(near,).toHaveLength(1,);
            expect(near[0]
              ?.kind,).toBe('overlapping-focus',);
            // AND IT IS NOT CORROBORATION: the spans differ, so the agreement is
            // reported as a question rather than counted as a fact.
            expect(
              corroborate({
                claims: [
                  TWO_TO_THREE_A,
                  containing,
                ],
              },),
            ).toEqual([],);
          },
        },),
        it({
          name:
            'does NOT let one voice corroborate itself by filing the same defect twice, which is what a '
              + 'count over claims rather than over voices would report as agreement nobody reached',
          fn: async () => {
            const defects = corroborate({
              claims: [
                TWO_TO_THREE_A,
                {
                  ...TWO_TO_THREE_A,
                  finding: {
                    ...TWO_TO_THREE_A.finding,
                    reason: 'said again by the same voice',
                  },
                },
              ],
            },);
            expect(defects,).toEqual([],);
          },
        },),
        it({
          name:
            'treats the SAME span under two category names as a near miss rather than as corroboration, '
              + 'since a dropped negator is nameable two ways and merging them would decide a question about '
              + 'the taxonomy that neither voice was asked',
          fn: async () => {
            /**
             Two voices pointing at the identical span and disagreeing on the word
             for it.
             */
            const claims = [
              claimOf({
                modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                category: 'altered-polarity',
                sourceLocator: '她们不吃罐头',
                sourceFocus: '不吃',
                candidateLocator: 'They eat canned food',
                candidateFocus: 'eat',
              },),
              claimOf({
                modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
                category: 'omission',
                sourceLocator: '她们不吃罐头',
                sourceFocus: '不吃',
                candidateLocator: 'They eat canned food',
                candidateFocus: 'eat',
              },),
            ];
            expect(corroborate({ claims, },),).toEqual([],);

            const near = nearMisses({ claims, },);
            expect(near,).toHaveLength(1,);
            expect(near[0]
              ?.kind,).toBe('same-focus-different-category',);
          },
        },),
        it({
          name:
            'does NOT cluster transitively: one wide claim touching two narrow ones does not put those two '
              + 'in a defect together, when they share no text at all',
          fn: async () => {
            /**
             Claim spanning the whole count sentence, which touches both counts.
             */
            const wide = claimOf({
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              category: 'altered-number',
              sourceLocator: COUNT_SENTENCE_SOURCE,
              sourceFocus: '两只猫睡在窗台上，一只猫',
              candidateLocator: COUNT_SENTENCE_CANDIDATE,
              candidateFocus: 'three cats sleep on the windowsill, two cats',
            },);

            /**
             All three claims together.
             */
            const claims = [
              TWO_TO_THREE_A,
              wide,
              ONE_TO_TWO,
            ];
            expect(corroborate({ claims, },),).toEqual([],);

            /**
             Pairs the near-miss pass found.
             */
            const near = nearMisses({ claims, },);

            // TWO PAIRS, NOT THREE: the wide claim touches each narrow one, and the
            // two narrow ones still touch nothing. A transitive rule would have
            // joined them through it.
            expect(near,).toHaveLength(2,);
            expect(
              near.filter(function joinsTheNarrowPair(one,): boolean {
                return [
                  one.left
                    .modelId,
                  one.right
                    .modelId,
                ].every(function isNarrow(modelId,): boolean {
                  return modelId !== SEAT_SYNTHETIC_VISION_WITHHELD;
                },);
              },),
            ).toEqual([],);
          },
        },),
        it({
          name:
            'KEEPS every member claim on a corroborated defect, so a later calibration can re-read what each '
              + 'voice actually said rather than re-running the roster to find out',
          fn: async () => {
            const defects = corroborate({
              claims: [
                TWO_TO_THREE_A,
                TWO_TO_THREE_B,
              ],
            },);
            expect(
              defects[0]
                ?.members
                .map(function toVoice(member,): string {
                  return member.modelId;
                },),
            ).toEqual([
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              SEAT_HYPER_OPENROUTER_UNMEASURED,
            ],);
          },
        },),
        it({
          name: 'reports nothing as a near miss when two claims are about different sentences entirely',
          fn: async () => {
            /**
             Claim about the negation, far from either count.
             */
            const polarity = claimOf({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              category: 'altered-polarity',
              sourceLocator: '她们不吃罐头',
              sourceFocus: '不吃',
              candidateLocator: 'They eat canned food',
              candidateFocus: 'eat',
            },);
            expect(
              nearMisses({
                claims: [
                  polarity,
                  ONE_TO_TWO,
                ],
              },),
            ).toEqual([],);
          },
        },),
        it({
          name: 'READS two one-sided claims naming one defect as one opinion confirmed, since the side the '
            + 'category does not use is one absence rather than a position',
          fn: async () => {
            expect(
              corroborate({
                claims: [
                  DENIAL_LEFT_OUT_A,
                  DENIAL_LEFT_OUT_B,
                ],
              },),
            ).toEqual([
              {
                category: 'omission',
                source: DENIAL_LEFT_OUT_A.finding
                  .source,
                candidate: { kind: 'unused', },
                voices: 2,
                members: [
                  DENIAL_LEFT_OUT_A,
                  DENIAL_LEFT_OUT_B,
                ],
              },
            ],);
          },
        },),
        it({
          name: 'ORDERS the defects by the voices behind them, most agreed first',
          fn: async () => {
            /**
             Third voice locating the two cats that became three.
             */
            const thirdVoice = claimOf({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              category: 'altered-number',
              sourceLocator: COUNT_SENTENCE_SOURCE,
              sourceFocus: '两',
              candidateLocator: COUNT_SENTENCE_CANDIDATE,
              candidateFocus: 'three',
            },);

            // THE TWO-VOICE DEFECT ARRIVES FIRST, so an answer left in arrival
            // order would lead with it.
            expect(
              corroborate({
                claims: [
                  DENIAL_LEFT_OUT_A,
                  DENIAL_LEFT_OUT_B,
                  TWO_TO_THREE_A,
                  TWO_TO_THREE_B,
                  thirdVoice,
                ],
              },),
            ).toEqual([
              {
                category: 'altered-number',
                source: TWO_TO_THREE_A.finding
                  .source,
                candidate: TWO_TO_THREE_A.finding
                  .candidate,
                voices: 3,
                members: [
                  TWO_TO_THREE_A,
                  TWO_TO_THREE_B,
                  thirdVoice,
                ],
              },
              {
                category: 'omission',
                source: DENIAL_LEFT_OUT_A.finding
                  .source,
                candidate: { kind: 'unused', },
                voices: 2,
                members: [
                  DENIAL_LEFT_OUT_A,
                  DENIAL_LEFT_OUT_B,
                ],
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: corroborateByOverlap.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'GATHERS voices who named one defect at different widths, which the strict count cannot: this '
              + 'was measured on the live control, where three auditors independently found one dropped '
              + 'negator, all three called it altered-polarity, and quoted three different spans of it',
          fn: async () => {
            /**
             Third voice quoting a span that contains the other two.
             */
            const wider = claimOf({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              category: 'altered-number',
              sourceLocator: COUNT_SENTENCE_SOURCE,
              sourceFocus: '两只猫睡在窗台上',
              candidateLocator: COUNT_SENTENCE_CANDIDATE,
              candidateFocus: 'three cats sleep on the windowsill',
            },);

            /**
             Same three claims the strict count splits.
             */
            const claims = [
              TWO_TO_THREE_A,
              TWO_TO_THREE_B,
              wider,
            ];

            // The strict count still reports the exact pair only, unchanged.
            expect(corroborate({ claims, },),).toHaveLength(1,);

            const agreed = corroborateByOverlap({ claims, },);
            expect(agreed,).toHaveLength(1,);
            expect(agreed[0]
              ?.voices,).toBe(3,);
            expect(agreed[0]
              ?.category,).toBe('altered-number',);
          },
        },),
        it({
          name:
            'does NOT gather two DIFFERENT defects in one sentence, which is the whole risk of a looser rule: '
              + 'the two changed counts share a sentence and share no focus, so nothing here may join them',
          fn: async () => {
            expect(
              corroborateByOverlap({
                claims: [
                  TWO_TO_THREE_A,
                  ONE_TO_TWO,
                ],
              },),
            ).toEqual([],);
          },
        },),
        it({
          name:
            'does NOT join two narrow claims THROUGH a wide one that touches both, since a group whose '
              + 'members are not all about the same thing is not agreement, however each pair looks',
          fn: async () => {
            /**
             Claim spanning both counts.
             */
            const wide = claimOf({
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              category: 'altered-number',
              sourceLocator: COUNT_SENTENCE_SOURCE,
              sourceFocus: '两只猫睡在窗台上，一只猫',
              candidateLocator: COUNT_SENTENCE_CANDIDATE,
              candidateFocus: 'three cats sleep on the windowsill, two cats',
            },);

            /**
             Groups over the three claims.
             */
            const agreed = corroborateByOverlap({
              claims: [
                TWO_TO_THREE_A,
                wide,
                ONE_TO_TWO,
              ],
            },);

            // Pairs with the wide claim are allowed; a trio is not. Measured on
            // 2026-08-26 before this line was written: the wide claim pairs with
            // each narrow one and the two narrow claims never join, so exactly
            // two groups of two. Pinned first, because the
            // `for (const group of agreed)` loop passes vacuously over an empty
            // result.
            expect(agreed,).toHaveLength(2,);
            for (const group of agreed) {
              expect(group.voices,).toBeLessThan(3,);
              expect(
                group.members
                  .some(function isWide(member,): boolean {
                    return member.modelId === SEAT_SYNTHETIC_VISION_WITHHELD;
                  },),
              ).toBe(true,);
            }
          },
        },),
        it({
          name:
            'does NOT gather two voices who disagree about the CATEGORY, since naming the same span two ways '
              + 'is a question about the taxonomy rather than agreement about a defect',
          fn: async () => {
            const claims = [
              claimOf({
                modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                category: 'altered-polarity',
                sourceLocator: '她们不吃罐头',
                sourceFocus: '不吃',
                candidateLocator: 'They eat canned food',
                candidateFocus: 'eat',
              },),
              claimOf({
                modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                category: 'omission',
                sourceLocator: '她们不吃罐头',
                sourceFocus: '不吃罐头',
                candidateLocator: 'They eat canned food',
                candidateFocus: 'eat canned food',
              },),
            ];
            expect(corroborateByOverlap({ claims, },),).toEqual([],);
          },
        },),
        it({
          name: 'does NOT let one voice form a group with itself by claiming one defect at two widths',
          fn: async () => {
            const claims = [
              TWO_TO_THREE_A,
              {
                ...TWO_TO_THREE_A,
                finding: {
                  ...TWO_TO_THREE_A.finding,
                  reason: 'the same voice, saying it wider',
                },
              },
            ];
            expect(corroborateByOverlap({ claims, },),).toEqual([],);
          },
        },),
        it({
          name: 'READS two claims leaving the same side unused as agreeing about it, since the absence they '
            + 'share is the same absence',
          fn: async () => {
            // THE MEMBERS READ SECOND VOICE FIRST: each claim seeds a group, both
            // groups hold the same two claims, and the group the later seed grew
            // replaces the earlier one under the membership the two share.
            expect(
              corroborateByOverlap({
                claims: [
                  DENIAL_LEFT_OUT_A,
                  DENIAL_LEFT_OUT_B,
                ],
              },),
            ).toEqual([
              {
                category: 'omission',
                voices: 2,
                members: [
                  DENIAL_LEFT_OUT_B,
                  DENIAL_LEFT_OUT_A,
                ],
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: nearMisses.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS two claims of one voice as one opinion rather than a near miss, since filing two '
            + 'findings about one sentence is what atomicity asks for, where the same two claims from two '
            + 'voices are an overlapping near miss',
          fn: async () => {
            /**
             The denial named a second way, on both sides, by the voice that
             also filed it as left out.
             */
            const polarityBySameVoice = claimOf({
              modelId: SEAT_HYPER_VISION,
              category: 'altered-polarity',
              sourceLocator: '她们不吃罐头',
              sourceFocus: '不吃',
              candidateLocator: 'They eat canned food',
              candidateFocus: 'eat',
            },);

            /**
             The same second claim, filed by another voice.
             */
            const polarityByAnotherVoice = claimOf({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              category: 'altered-polarity',
              sourceLocator: '她们不吃罐头',
              sourceFocus: '不吃',
              candidateLocator: 'They eat canned food',
              candidateFocus: 'eat',
            },);
            expect(
              nearMisses({
                claims: [
                  DENIAL_LEFT_OUT_A,
                  polarityBySameVoice,
                ],
              },),
            ).toEqual([],);
            expect(
              nearMisses({
                claims: [
                  DENIAL_LEFT_OUT_A,
                  polarityByAnotherVoice,
                ],
              },),
            ).toEqual([
              {
                kind: 'overlapping-focus',
                left: DENIAL_LEFT_OUT_A,
                right: polarityByAnotherVoice,
              },
            ],);
          },
        },),
        it({
          name: 'READS two voices naming one defect as agreement rather than a near miss, since the same '
            + 'claim twice is confirmation and nothing to reconcile',
          fn: async () => {
            expect(
              nearMisses({
                claims: [
                  DENIAL_LEFT_OUT_A,
                  DENIAL_LEFT_OUT_B,
                ],
              },),
            ).toEqual([],);
          },
        },),
        it({
          name: 'READS a claim leaving the candidate side unused as touching no claim there, so two voices naming '
            + 'defects in different sentences of the original are no near miss whichever side the other uses',
          fn: async () => {
            expect(
              nearMisses({
                claims: [
                  DENIAL_LEFT_OUT_A,
                  TWO_TO_THREE_A,
                ],
              },),
            ).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
