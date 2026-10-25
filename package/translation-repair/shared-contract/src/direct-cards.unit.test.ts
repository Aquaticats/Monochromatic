/**
 Tests the two direct sides of the model cards, DeepSeek's own API and
 Xiaomi's own API, and THE LOCK each owner grant puts on them: a key granted for
 one model reaches that model and no other.

 The owner, 2026-10-22: "You can also use the DEEPSEEK_API_KEY as you wish, but
 you may only use deepseek v4.1 flash 1017 with that key" and "You may only use
 v2.6-flash with this key" (the Xiaomi MiMo key). Neither key is model-scoped,
 so the lock is in the cards: each side appears on exactly one card, with
 exactly one served id.

 Facts: the DeepSeek pricing page (`https://api-docs.deepseek.com/quick_start/pricing`)
 and Xiaomi's pay-as-you-go page (`https://mimo.mi.com/docs/en-US/price/pay-as-you-go`),
 both read 2026-10-22.

 Fixtures are the roster itself; no corpus content appears.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  CARD_PROVIDERS,
  cardsServing,
  DEEPSEEK_SERVED_IDS,
  MODEL_CARDS,
  ROSTER_MODEL_IDS,
  servedRecord,
  XIAOMI_SERVED_IDS,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'direct cards',
  children: [
    it({
      name: 'LOCKS EACH KEY TO ONE SPELLING: the served-id list of each provider holds the one model the owner '
        + 'granted, and no other, though both keys could reach more',
      fn: async () => {
        expect([...DEEPSEEK_SERVED_IDS,],).toEqual(['deepseek-flash',],);
        expect([...XIAOMI_SERVED_IDS,],).toEqual(['mimo-v2.6-flash',],);
      },
    },),

    it({
      name: 'PUTS EACH SIDE ON EXACTLY ONE CARD with exactly the granted id: DeepSeek V4.1 Flash for the '
        + 'DeepSeek key and MiMo V2.6 Flash for the Xiaomi key, and on no other roster model',
      fn: async () => {
        expect(cardsServing({ provider: 'deepseek', },).map(function toSide(card,): readonly [string, string] {
          return [card.id, card.deepseek.id,];
        },),).toEqual([['deepseek-v4.1-flash', 'deepseek-flash',],],);
        expect(cardsServing({ provider: 'xiaomi', },).map(function toSide(card,): readonly [string, string] {
          return [card.id, card.xiaomi.id,];
        },),).toEqual([['xiaomi/mimo-v2.6-flash', 'mimo-v2.6-flash',],],);
        expect(CARD_PROVIDERS.includes('deepseek',),).toBe(true,);
        expect(CARD_PROVIDERS.includes('xiaomi',),).toBe(true,);
      },
    },),

    it({
      name: 'BUILDS ONE ROW PER PROVIDER FROM THE CARDS: a served record over each provider has exactly the '
        + 'granted spelling as its only key',
      fn: async () => {
        expect(
          Object.keys(servedRecord({
            provider: 'deepseek',
            toRow: function row(card,): string {
              return card.id;
            },
          },),),
        ).toEqual(['deepseek-flash',],);
        expect(
          Object.keys(servedRecord({
            provider: 'xiaomi',
            toRow: function row(card,): string {
              return card.id;
            },
          },),),
        ).toEqual(['mimo-v2.6-flash',],);
      },
    },),

    it({
      name: 'ADDS NO ROSTER IDENTITY and lifts or adds no hold: the two cards are the existing DeepSeek V4.1 '
        + 'Flash and MiMo V2.6 Flash, each held out of a picture-reader seat as before',
      fn: async () => {
        // The roster is untouched: every id the cards declare is one the roster already held, each once.
        expect(new Set(ROSTER_MODEL_IDS,).size,).toBe(ROSTER_MODEL_IDS.length,);
        expect(ROSTER_MODEL_IDS,).toContain('deepseek-v4.1-flash',);
        expect(ROSTER_MODEL_IDS,).toContain('xiaomi/mimo-v2.6-flash',);
        expect(Object.keys(MODEL_CARDS,).length,).toBe(ROSTER_MODEL_IDS.length,);
        expect(MODEL_CARDS['deepseek-v4.1-flash'].holds,).toEqual(['reader-unmeasured',],);
        expect(MODEL_CARDS['xiaomi/mimo-v2.6-flash'].holds,).toEqual([
          'judge-unmeasured',
          'writer-unmeasured',
          'reader-unmeasured',
        ],);
      },
    },),

    it({
      name: 'RECORDS THE PAGES\' PRICES: DeepSeek at its peak rates, which the provider package halves for a call '
        + 'that started and completed outside every weekday peak window, and Xiaomi at the overseas real-time rates, '
        + 'with image input probed on both',
      fn: async () => {
        expect(MODEL_CARDS['deepseek-v4.1-flash'].deepseek,).toEqual({
          id: 'deepseek-flash',
          readsImages: true,
          maxOutputLength: 393_216,
          promptCacheHitUsdPerMillion: 0.006,
          promptCacheMissUsdPerMillion: 0.3,
          completionUsdPerMillion: 1.2,
        },);
        expect(MODEL_CARDS['xiaomi/mimo-v2.6-flash'].xiaomi,).toEqual({
          id: 'mimo-v2.6-flash',
          readsImages: true,
          maxOutputLength: 131_072,
          promptCacheHitUsdPerMillion: 0.0028,
          promptCacheMissUsdPerMillion: 0.14,
          completionUsdPerMillion: 0.28,
        },);
      },
    },),
  ],
},);
