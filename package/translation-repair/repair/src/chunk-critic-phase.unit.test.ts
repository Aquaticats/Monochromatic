/**
 Tests for the critic phase: the fan-out and the deterministic screen over its
 non-translation votes, together.

 This module was extracted from the chunk driver for the file-size budget and
 never covered. Its two halves are each tested elsewhere; what is untested is
 the WIRING, and the wiring is the reason the module exists. The comment at
 its head says so: a vote count that has not been screened is not something
 any caller may act on, and keeping the two adjacent is what stops a later
 caller reading `nonTranslationVotes` straight off the critic result and
 blocking a slice on votes the evidence already contradicted.

 A block is expensive in one direction only. Blocking a faithful translation
 discards the whole slice unrepaired; failing to block a genuinely
 untranslated pair leaves its issues surfaced. So the cases in this file check that
 `votesStand` is true only when the threshold is met AND nothing contradicted
 it.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import { NON_TRANSLATION_BLOCK_VOTES, } from '@monochromatic-dev/translation-repair-archive';
import {
  computeIssueClaimId,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { runChunkCriticPhase, } from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { criticClient, } from '@monochromatic-dev/translation-repair-provider/ts/critic-scripted-client.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the phases under test.
 */
const l = tagged({ tag: 'chunk-critic-phase-test', },);

/**
 Original chunk the critics review.
 */
const SOURCE_TEXT = '猫猫在窗台上睡觉。太阳移动时她会醒来。';

/**
 Translation chunk under review.
 */
const TARGET_TEXT = 'The cat sleeps on the windowsill. She wakes when the sun moves.';

/**
 Parsed pair the claims anchor against.
 */
const DOCUMENTS = {
  source: parseDocument({ text: SOURCE_TEXT, },),
  target: parseDocument({ text: TARGET_TEXT, },),
};

/**
 Critic roster large enough to reach the block threshold and exceed it.
 */
const CRITICS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Wire issue asserting the pair is not a translation at all.

 Carries no quotes, so it never resolves into an anchored claim. That is
 faithful to the real case: a degenerate pair defeats anchoring, which is
 exactly why the vote is counted at wire level rather than after resolution.
 */
const NON_TRANSLATION_ISSUE = {
  category: 'accuracy/non-translation',
  severity: 'critical',
  summary: 'The target is not a translation of the source.',
};

/**
 Non-translation issue whose quotes DO anchor, so unlike
 `NON_TRANSLATION_ISSUE` it resolves into a claim and therefore acquires
 attribution that screening must later take away with it.
 */
const ANCHORED_NON_TRANSLATION = {
  category: 'accuracy/non-translation',
  severity: 'critical',
  summary: 'The target is not a translation of the source.',
  sourceQuote: '猫猫在窗台上睡觉',
  targetQuote: 'The cat sleeps on the windowsill.',
};

/**
 Content critique that anchors into the TARGET and survives screening.

 Deliberately not an omission: `MISSING_TRANSLATION_LEAVES` excludes
 `omission`, `untranslated` and `non-translation` from contradiction counting,
 because those anchor happily onto an untranslated target and so prove
 nothing. A fluency critique only makes sense against text that was in fact
 translated, which is exactly what contradicts the votes.
 */
const CONTENT_CRITIQUE = {
  category: 'fluency/awkward-phrasing',
  severity: 'major',
  summary: 'The waking clause reads oddly.',
  sourceQuote: '太阳移动时她会醒来',
  targetQuote: 'She wakes when the sun moves.',
};

/**
 Runs the phase against a scripted client.

 @param client - scripted critic client

 @param criticModelIds - roster to fan out over

 @returns Phase result

 @example
 ```ts
 const phase = await runPhase({ client, criticModelIds: CRITICS, },);
 ```
 */
async function runPhase(
  {
    client,
    criticModelIds,
  }: {
    readonly client: ModelClient;
    readonly criticModelIds: readonly typeof CRITICS[number][];
  },
) {
  return await runChunkCriticPhase({
    client,
    criticModelIds,
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
    documents: DOCUMENTS,
    sliceIndex: 0,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    // THE WHOLE BENCH, since these cases count votes over every scripted
    // critic; production asks the window of quorum plus one, pinned in
    // `stage-quorum.unit.test.ts`.
    fanOut: 'whole-bench',
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: runChunkCriticPhase.name,
  children: [
    it({
      name: 'reports no standing votes when every critic found the pair a '
        + 'real translation, so a clean slice proceeds to repair',
      fn: async () => {
        /**
         Phase where every critic reported nothing at all.
         */
        const phase = await runPhase({
          client: criticClient({ reportFor: () => ({ issues: [], }), },),
          criticModelIds: CRITICS,
        },);

        expect(phase.nonTranslationVotes,).toBe(0,);
        expect(phase.contradicted,).toBe(false,);
        expect(phase.votesStand,).toBe(false,);
        expect(phase.claims,).toStrictEqual([],);
        expect(phase.heardCritics,).toBe(CRITICS.length,);
      },
    },),

    it({
      name: 'lets votes STAND once the threshold is met with nothing '
        + 'contradicting them, which is the case the block exists for: a '
        + 'genuinely untranslated pair defeats anchoring, so the vote is all '
        + 'the evidence there is',
      fn: async () => {
        /**
         Critics that vote, exactly meeting the threshold.
         */
        const voters: ReadonlySet<string> = new Set(CRITICS.slice(
          0,
          NON_TRANSLATION_BLOCK_VOTES,
        ),);

        /**
         Phase at exactly the block threshold.
         */
        const phase = await runPhase({
          client: criticClient({
            reportFor: (modelId,) => (
              {
                issues: voters.has(modelId,)
                  ? [NON_TRANSLATION_ISSUE,]
                  : [],
              }
            ),
          },),
          criticModelIds: CRITICS,
        },);

        expect(phase.nonTranslationVotes,).toBe(NON_TRANSLATION_BLOCK_VOTES,);
        expect(phase.contradicted,).toBe(false,);
        expect(phase.votesStand,).toBe(true,);
      },
    },),

    it({
      name: 'refuses to block one vote below the threshold, because a false '
        + 'block discards a faithful translation whole while a missed block '
        + 'only leaves a garbage pair with its issues still surfaced',
      fn: async () => {
        /**
         Critics that vote, one short of the threshold.
         */
        const voters: ReadonlySet<string> = new Set(CRITICS.slice(
          0,
          NON_TRANSLATION_BLOCK_VOTES - 1,
        ),);

        /**
         Phase one vote below the block threshold.
         */
        const phase = await runPhase({
          client: criticClient({
            reportFor: (modelId,) => (
              {
                issues: voters.has(modelId,)
                  ? [NON_TRANSLATION_ISSUE,]
                  : [],
              }
            ),
          },),
          criticModelIds: CRITICS,
        },);

        expect(phase.nonTranslationVotes,).toBe(NON_TRANSLATION_BLOCK_VOTES - 1,);
        expect(phase.votesStand,).toBe(false,);
      },
    },),

    it({
      name: 'counts the vote at WIRE level, before anchoring, so a report '
        + 'whose non-translation issue carries no resolvable quote still votes: '
        + 'a degenerate pair is exactly the case that defeats anchoring, and '
        + 'requiring an anchor would silence the vote precisely when it is '
        + 'right',
      fn: async () => {
        /**
         Phase where every voting report resolves to zero claims.
         */
        const phase = await runPhase({
          client: criticClient({
            reportFor: () => ({ issues: [NON_TRANSLATION_ISSUE,], }),
          },),
          criticModelIds: CRITICS,
        },);

        expect(phase.claims,).toStrictEqual([],);
        expect(phase.nonTranslationVotes,).toBe(CRITICS.length,);
        expect(phase.votesStand,).toBe(true,);
      },
    },),

    it({
      name: 'ignores a non-translation issue that is not CRITICAL, since the '
        + 'block is reserved for the whole-pair verdict rather than for a '
        + 'critic hedging about one sentence',
      fn: async () => {
        /**
         Phase where every critic hedged at major rather than critical.
         */
        const phase = await runPhase({
          client: criticClient({
            reportFor: () => ({
              issues: [
                {
                  ...NON_TRANSLATION_ISSUE,
                  severity: 'major',
                },
              ],
            }),
          },),
          criticModelIds: CRITICS,
        },);

        expect(phase.nonTranslationVotes,).toBe(0,);
        expect(phase.votesStand,).toBe(false,);
      },
    },),

    it({
      name: 'reports heard critics for the caller\'s degradation accounting, '
        + 'so a slice decided by half the roster is visible as such rather '
        + 'than reading like a full-roster verdict',
      fn: async () => {
        /**
         Phase over a roster of two.
         */
        const phase = await runPhase({
          client: criticClient({ reportFor: () => ({ issues: [], }), },),
          criticModelIds: CRITICS.slice(
            0,
            2,
          ),
        },);

        expect(phase.heardCritics,).toBe(2,);
      },
    },),

    it({
      name: 'carries the critics\' own findings through, so resolution '
        + 'failures reach the scorecard rather than being dropped when the '
        + 'phase adds its screening findings',
      fn: async () => {
        /**
         Phase where every critic quoted text that is not in either document.
         */
        const phase = await runPhase({
          client: criticClient({
            reportFor: () => ({
              issues: [
                {
                  category: 'accuracy/omission',
                  severity: 'major',
                  summary: 'A clause about the garden is missing.',
                  sourceQuote: '花园里有狗在叫。',
                  targetQuote: 'A dog barks in the garden.',
                },
              ],
            }),
          },),
          criticModelIds: CRITICS,
        },);

        expect(phase.claims,).toStrictEqual([],);
        expect(phase.findings.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'DROPS attribution for claims screening removed, so a critic keeps '
        + 'credit only for claims that survived. Attribution is collected at '
        + 'resolution time, before the screen runs, so without the filter a '
        + 'contradicted non-translation claim would still be counted as a hit '
        + 'for whoever raised it',
      fn: async () => {
        /**
         Phase where every critic raises both an anchored non-translation
         claim and a content critique, so the votes are contradicted and the
         non-translation claim is filtered out while the critique stands.
         */
        const phase = await runPhase({
          client: criticClient({
            reportFor: () => ({
              issues: [
                ANCHORED_NON_TRANSLATION,
                CONTENT_CRITIQUE,
                CONTENT_CRITIQUE,
              ],
            }),
          },),
          criticModelIds: CRITICS,
        },);

        expect(phase.contradicted,).toBe(true,);

        /**
         Identities still standing after the screen.
         */
        const survivingIds = new Set(phase.claims
          .map(function toClaimId(claim,) {
            return computeIssueClaimId({ claim, },);
          },),);

        expect(phase.claimAttributions.length,).toBeGreaterThan(0,);
        for (const attribution of phase.claimAttributions)
          expect(survivingIds.has(attribution.claimId,),).toBe(true,);
        for (const claim of phase.claims)
          expect(claim.category.endsWith('/non-translation',),).toBe(false,);
      },
    },),
  ],
},);
