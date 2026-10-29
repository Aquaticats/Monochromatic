/**
 Tests for the WIRING that keeps each critic's identity attached to the claim
 it raised, as opposed to the fold that counts them once they are attached.

 `collectClaimAttributions` is tested on its own, and passing those cases
 proves nothing about whether `runCriticStage` ever calls it with real
 emissions. Measured: deleting the single `emissions.push` in
 `runCriticStage` left the whole suite green, so the exact discard this work
 exists to fix was reintroducible without any test noticing. These cases close
 that, and they are the ones a future edit has to keep passing.

 That hole is the same shape as the defect it followed: the stage-call cases
 asserted only that a voice was LOST and never read what the loss recorded,
 which is how an uninformative warning survived having tests.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  computeIssueClaimId,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { runCriticStage, } from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { criticClient, } from '@monochromatic-dev/translation-repair-provider/ts/critic-scripted-client.test-fixture.ts';
import { proposerIdsOf, } from '@monochromatic-dev/translation-repair-adjudication/ts/claim-attribution-proposer-ids.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the stage under test.
 */
const l = tagged({ tag: 'critic-stage-attribution-test', },);

/**
 Original chunk the critics review.
 */
const SOURCE_TEXT = '猫猫喜欢晒太阳，也喜欢追蝴蝶。\n\n猫猫会打呼噜。\n';

/**
 Translation chunk under review, missing the butterfly clause.
 */
const TARGET_TEXT = 'The cat likes to nap in the sun.\n\nThe cat purrs.\n';

/**
 Parsed pair the claims anchor against.
 */
const DOCUMENTS = {
  source: parseDocument({ text: SOURCE_TEXT, },),
  target: parseDocument({ text: TARGET_TEXT, },),
};

/**
 Two critics, enough to tell independent support from self-repetition.
 */
const CRITICS = [
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
] as const;

/**
 Wire issue whose quotes both anchor, so resolution succeeds.
 */
const ANCHORING_WIRE = {
  category: 'accuracy/omission',
  severity: 'major',
  summary: '追蝴蝶那句没有翻译。',
  sourceQuote: '也喜欢追蝴蝶',
  targetQuote: 'The cat likes to nap in the sun.',
};

/**
 Wire issue quoting text that appears in neither document, so resolution
 fails and it must contribute no attribution.
 */
const UNANCHORABLE_WIRE = {
  category: 'accuracy/omission',
  severity: 'major',
  summary: 'A clause about kneading is missing.',
  sourceQuote: '猫猫踩奶',
  targetQuote: 'The cat kneads the blanket.',
};

/**
 Runs the critic stage against a scripted client.

 @param reportFor - report each model returns

 @returns Stage result including its attribution

 @example
 ```ts
 const critic = await runStage({ reportFor: () => ({ issues: [], }), },);
 ```
 */
async function runStage(
  {
    reportFor,
  }: {
    readonly reportFor: (modelId: string,) => unknown;
  },
) {
  return await runCriticStage({
    client: criticClient({ reportFor, },),
    criticModelIds: [...CRITICS,],
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
    documents: DOCUMENTS,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: `${runCriticStage.name} attribution`,
  children: [
    it({
      name: 'CARRIES the speaker through resolution, so two critics raising '
        + 'the same defect are recorded as two proposers of one claim. Without '
        + 'this the stage returns no attribution at all, which is the exact '
        + 'discard this work exists to fix and which no other case detects',
      fn: async () => {
        /**
         Stage result where both critics raise the identical issue.
         */
        const critic = await runStage({
          reportFor: () => ({ issues: [ANCHORING_WIRE,], }),
        },);

        expect(critic.claimAttributions,).toHaveLength(1,);
        expect(critic.claimAttributions[0]?.proposers,).toHaveLength(2,);
        expect(
          critic.claimAttributions[0]?.proposers
            .map(function toId(proposer,) {
              return proposer.modelId;
            },),
        ).toStrictEqual([...CRITICS,].toSorted(),);
      },
    },),

    it({
      name: 'keys attribution by the id computed over the RESOLVED claim, not '
        + 'the wire issue, so the key matches what downstream stages hold: the '
        + 'deterministic identity is defined over anchored spans that only '
        + 'exist after resolution. The stage itself does NOT deduplicate, so '
        + 'two critics leave two identical claims here while attribution has '
        + 'already collapsed to the one id they share, which is exactly why it '
        + 'must be collected before aggregateClaims runs',
      fn: async () => {
        /**
         Stage result where both critics raise the identical issue.
         */
        const critic = await runStage({
          reportFor: () => ({ issues: [ANCHORING_WIRE,], }),
        },);

        expect(critic.claims,).toHaveLength(2,);
        expect(critic.claimAttributions,).toHaveLength(1,);
        for (const claim of critic.claims)
          expect(critic.claimAttributions[0]?.claimId,)
            .toBe(computeIssueClaimId({ claim, },),);
      },
    },),

    it({
      name: 'records ONE proposer with an emission count of two when a single '
        + 'critic repeats itself while the other stays silent, since crediting '
        + 'that as two proposers would manufacture independent support the '
        + 'ensemble never had',
      fn: async () => {
        /**
         Stage result where one critic double-reports and one says nothing.
         */
        const critic = await runStage({
          reportFor: (modelId,) => ((modelId === CRITICS[0])
            ? { issues: [ANCHORING_WIRE, ANCHORING_WIRE,], }
            : { issues: [], }),
        },);

        expect(critic.claimAttributions,).toHaveLength(1,);
        expect(critic.claimAttributions[0]?.proposers,).toHaveLength(1,);
        expect(critic.claimAttributions[0]?.proposers[0]?.modelId,)
          .toBe(CRITICS[0],);
        expect(critic.claimAttributions[0]?.proposers[0]?.emissionCount,)
          .toBe(2,);
      },
    },),

    it({
      name: 'attributes NOTHING for an issue whose quotes do not anchor, '
        + 'because a claim the pipeline discarded must not count toward any '
        + 'critic\'s recorded hits, and resolution failure is the commonest '
        + 'way a claim is discarded',
      fn: async () => {
        /**
         Stage result where every issue fails to resolve.
         */
        const critic = await runStage({
          reportFor: () => ({ issues: [UNANCHORABLE_WIRE,], }),
        },);

        expect(critic.claims,).toHaveLength(0,);
        expect(critic.claimAttributions,).toHaveLength(0,);
        expect(critic.findings.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'records WHICH critics answered, not merely how many, because that '
        + 'is the denominator attribution cannot supply: a critic heard that '
        + 'raised nothing and a critic never heard both produce no attribution '
        + 'entry, so without the roster hits can be counted but rates cannot',
      fn: async () => {
        /**
         Stage result where only one of the two critics raises anything.
         */
        const critic = await runStage({
          reportFor: (modelId,) => ((modelId === CRITICS[0])
            ? { issues: [ANCHORING_WIRE,], }
            : { issues: [], }),
        },);

        expect(critic.heardCriticIds,).toStrictEqual([...CRITICS,].toSorted(),);
        expect(critic.heardCritics,).toBe(CRITICS.length,);

        /**
         Critics that actually raised a claim.
         */
        const raisers = new Set(critic.claimAttributions
          .flatMap(function toIds(attribution,) {
            return proposerIdsOf(attribution,);
          },),);

        // The silent critic is in the roster and absent from attribution, which
        // is the whole point: that difference is a measurable rate.
        expect(raisers.has(CRITICS[0],),).toBe(true,);
        expect(raisers.has(CRITICS[1],),).toBe(false,);
      },
    },),
  ],
},);
