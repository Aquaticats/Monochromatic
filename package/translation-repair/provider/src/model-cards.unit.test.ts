/**
 Guards on the model cards: every catalog, cap table and hold set is a
 projection of them, so these hold the cards and the roster lists to each
 other and to the naming rule.

 Fixtures here are the roster itself; no corpus content appears.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  BEDROCK_SERVED_IDS,
  cardsServing,
  COMPLETION_CAP,
  holdSet,
  HYPER_MODELS,
  HYPER_SERVED_IDS,
  MODEL_CARDS,
  OPENROUTER_SERVED_IDS,
  ROSTER_CARDS,
  ROSTER_MODEL_IDS,
  SYNTHETIC_SERVED_IDS,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  BEDROCK_MODELS,
  OPENROUTER_MODELS,
  SYNTHETIC_MODELS,
} from '../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_UNMEASURED, SEAT_HYPER_OPENROUTER_UNMEASURED_HYPER_ID, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { BEDROCK_ONLY_ROSTER_IDS, } from '@monochromatic-dev/translation-repair-shared-contract/ts/roster-buckets.test-fixture.ts';
import { cardOf, } from './catalog-lookups.test-fixture.ts';

/**
 Roster ids that kept their name when a provider moved the model to a newer
 served id, beside the served id each now carries on its first serving
 provider. The owner's instruction of 2026-10-20 ("Migrate to deepseek v4
 flash 1017 where possible") moved Hyper and OpenRouter to the 1017 ids and
 kept the card's name, so caches, ledgers, calibrations and logs keep one
 name for the seat.
 */
const KEPT_ROSTER_NAME: Readonly<Record<string, string>> = {
  [SEAT_HYPER_OPENROUTER_UNMEASURED]: SEAT_HYPER_OPENROUTER_UNMEASURED_HYPER_ID,
};

await describe({
  name: 'model-cards',
  children: [
    it({
      name: 'CARRIES ONE CARD PER ROSTER ID and nothing else, in roster order',
      fn: async () => {
        expect(Object.keys(MODEL_CARDS,),).toEqual([...ROSTER_MODEL_IDS,],);
        expect(ROSTER_CARDS.map(function toId(card,): string {
          return card.id;
        },),).toEqual([...ROSTER_MODEL_IDS,],);
        expect(new Set(ROSTER_MODEL_IDS,).size,).toBe(ROSTER_MODEL_IDS.length,);
      },
    },),

    it({
      name: 'NAMES EVERY SERVED SPELLING ON EXACTLY ONE CARD, so a served-id list entry without a card '
        + 'and a card spelling missing from its list both fail here rather than at the first call',
      fn: async () => {
        /**
         Served ids each provider's cards carry.
         */
        const carried = {
          synthetic: cardsServing({ provider: 'synthetic', },).map(function toId(card,): string {
            return card.synthetic.id;
          },),
          hyper: cardsServing({ provider: 'hyper', },).map(function toId(card,): string {
            return card.hyper.id;
          },),
          openrouter: cardsServing({ provider: 'openrouter', },).map(function toId(card,): string {
            return card.openrouter.id;
          },),
          bedrock: cardsServing({ provider: 'bedrock', },).map(function toId(card,): string {
            return card.bedrock.id;
          },),
        };
        expect(carried.synthetic.toSorted(),).toEqual([...SYNTHETIC_SERVED_IDS,].toSorted(),);
        expect(carried.hyper.toSorted(),).toEqual([...HYPER_SERVED_IDS,].toSorted(),);
        expect(carried.openrouter.toSorted(),).toEqual([...OPENROUTER_SERVED_IDS,].toSorted(),);
        expect(carried.bedrock.toSorted(),).toEqual([...BEDROCK_SERVED_IDS,].toSorted(),);
        // THE DECISIONS SPELLINGS ARE A TYPE, NOT A LIST (ledger B31): the
        // type checker holds each card's spelling to the union, and what is
        // left to check here is that no two cards share one.
        /**
         Decisions spellings the cards carry.
         */
        const decisionIds = ROSTER_CARDS.flatMap(function decisionId(card,): readonly string[] {
          return (card.decisions === undefined) ? [] : [card.decisions.id,];
        },);
        expect(decisionIds.length,).toBeGreaterThan(0,);
        expect(new Set(decisionIds,).size,).toBe(decisionIds.length,);
      },
    },),

    it({
      name: 'SPELLS EVERY ROSTER ID THE FIRST SERVING PROVIDER\'S WAY: Synthetic, else Hyper, else Bedrock, '
        + 'else OpenRouter, in the order the providers joined, but for the roster ids that kept their name '
        + 'when the provider moved the model to a newer served id',
      fn: async () => {
        for (const card of ROSTER_CARDS) {
          /**
           Spelling the naming rule demands.
           */
          const expected = card.synthetic?.id
            ?? card.hyper?.id
            ?? card.bedrock?.id
            ?? card.openrouter?.id
            ?? card.anthropic?.id
            ?? card.decisions?.id;
          if (Object.hasOwn(KEPT_ROSTER_NAME, card.id,)) {
            expect(expected,).toBe(KEPT_ROSTER_NAME[card.id],);
            continue;
          }
          expect(card.id,).toBe(expected,);
        }
      },
    },),

    it({
      name: 'KEYS EVERY DERIVED CATALOG BY THE CARDS, one row per served spelling with the card\'s fields',
      fn: async () => {
        expect(Object.keys(SYNTHETIC_MODELS,).toSorted(),).toEqual([...SYNTHETIC_SERVED_IDS,].toSorted(),);
        expect(Object.keys(HYPER_MODELS,).toSorted(),).toEqual([...HYPER_SERVED_IDS,].toSorted(),);
        expect(Object.keys(OPENROUTER_MODELS,).toSorted(),).toEqual([...OPENROUTER_SERVED_IDS,].toSorted(),);
        expect(Object.keys(BEDROCK_MODELS,).toSorted(),).toEqual([...BEDROCK_SERVED_IDS,].toSorted(),);
        expect(Object.keys(COMPLETION_CAP,).toSorted(),).toEqual([...ROSTER_MODEL_IDS,].toSorted(),);
        for (const card of cardsServing({ provider: 'openrouter', },)) {
          expect(OPENROUTER_MODELS[card.openrouter.id].sharedWith,).toBe(card.id,);
          expect(OPENROUTER_MODELS[card.openrouter.id].ignoredEndpoints,).toEqual(card.openrouter.ignoredEndpoints,);
          expect(OPENROUTER_MODELS[card.openrouter.id].preferredEndpoints,).toEqual(card.openrouter.preferredEndpoints,);
        }
        for (const card of cardsServing({ provider: 'bedrock', },)) {
          expect(BEDROCK_MODELS[card.bedrock.id].sharedWith,).toBe(card.id,);
          expect(BEDROCK_MODELS[card.bedrock.id].route,).toBe(card.bedrock.route,);
        }
        for (const card of cardsServing({ provider: 'hyper', },)) {
          expect(HYPER_MODELS[card.hyper.id].sharedWith,).toBe(card.synthetic?.id ?? 'no-synthetic-counterpart',);
          expect(HYPER_MODELS[card.hyper.id].rosterId,).toBe(card.id,);
        }
      },
    },),

    // The origin buckets are test support now (`roster-buckets.test-fixture.ts`,
    // ledger B31), and a case restating their definition against itself could
    // not fail; what the naming rule claims about a Bedrock-spelled card is
    // checked here.
    it({
      name: 'GIVES A BEDROCK-SPELLED ROSTER ID A BEDROCK SIDE ALONE: no Synthetic, Hyper or OpenRouter side',
      fn: async () => {
        expect(BEDROCK_ONLY_ROSTER_IDS.length,).toBeGreaterThan(0,);
        for (const modelId of BEDROCK_ONLY_ROSTER_IDS) {
          /**
           Card of a Bedrock-only model.
           */
          const card = cardOf({ modelId, },);
          expect(card.synthetic,).toBeUndefined();
          expect(card.hyper,).toBeUndefined();
          expect(card.openrouter,).toBeUndefined();
        }
      },
    },),

    it({
      name: 'READS EACH HOLD OFF THE CARDS, in roster order, and an unused hold reads empty',
      fn: async () => {
        for (const card of ROSTER_CARDS) {
          for (const hold of card.holds)
            expect(holdSet({ hold, },).has(card.id,),).toBe(true,);
        }
        expect([...holdSet({ hold: 'judge-unmeasured', },),],).toEqual(ROSTER_CARDS
          .filter(function held(card,): boolean {
            return card.holds.includes('judge-unmeasured',);
          },)
          .map(function toId(card,): string {
            return card.id;
          },),);
      },
    },),
  ],
},);
