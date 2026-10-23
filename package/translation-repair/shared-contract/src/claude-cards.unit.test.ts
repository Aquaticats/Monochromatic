/**
 Tests the Anthropic side of the model cards, the first-party route on the owner's own Claude key, and THE LOCK the
 owner's grant puts on it: the key lists Opus, Fable and older models, and this pipeline may spend it on Sonnet 5.5
 and Haiku 5.5 only.

 The owner, 2026-10-22: "I've added TRANSLATION_REPAIR_CLAUDE_API_KEY . You may use latest Haiku and latest Sonnet
 with this key. This key would have $200 every month so be careful what you use." and, of effort, "Why don't you
 bench?", so the roster carries one card per model and effort for the bench to seat, and one card per model that sends
 no effort at all.

 Prices: the Anthropic pricing page as the owner's brief carries it (read 2026-10-22).

 Fixtures are the roster itself; no corpus content appears.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  ANTHROPIC_EFFORTS,
  ANTHROPIC_SERVED_IDS,
  ANTHROPIC_WIRE_MODELS,
  CARD_PROVIDERS,
  cardsServing,
  MODEL_CARDS,
  ROSTER_MODEL_IDS,
  servedRecord,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'anthropic cards',
  children: [
    it({
      name: 'LOCKS THE KEY TO TWO WIRE MODELS: the list of model ids a request may carry holds Sonnet 5.5 and '
        + 'Haiku 5.5 and no other, though the key lists Opus, Fable and older models',
      fn: async () => {
        expect([...ANTHROPIC_WIRE_MODELS,],).toEqual(['claude-sonnet-5-5', 'claude-haiku-5-5',],);
      },
    },),

    it({
      name: 'NAMES THREE EFFORTS for the bench to seat, and not the levels the owner did not ask about',
      fn: async () => {
        expect([...ANTHROPIC_EFFORTS,],).toEqual(['low', 'medium', 'high',],);
      },
    },),

    it({
      name: 'SEATS ONE CARD PER MODEL WITH NO EFFORT AND ONE PER MODEL AND EFFORT, eight in all, each served '
        + 'under its own roster spelling and each naming one of the two wire models',
      fn: async () => {
        expect([...ANTHROPIC_SERVED_IDS,],).toEqual([
          'claude-sonnet-5-5',
          'claude-sonnet-5-5-effort-low',
          'claude-sonnet-5-5-effort-medium',
          'claude-sonnet-5-5-effort-high',
          'claude-haiku-5-5',
          'claude-haiku-5-5-effort-low',
          'claude-haiku-5-5-effort-medium',
          'claude-haiku-5-5-effort-high',
        ],);
        expect(cardsServing({ provider: 'anthropic', },).map(function toSide(card,): readonly [
          string,
          string,
          string,
          string,
        ] {
          return [card.id, card.anthropic.id, card.anthropic.wireModel, card.anthropic.effort ?? 'none',];
        },),).toEqual([
          ['claude-sonnet-5-5', 'claude-sonnet-5-5', 'claude-sonnet-5-5', 'none',],
          ['claude-sonnet-5-5-effort-low', 'claude-sonnet-5-5-effort-low', 'claude-sonnet-5-5', 'low',],
          ['claude-sonnet-5-5-effort-medium', 'claude-sonnet-5-5-effort-medium', 'claude-sonnet-5-5', 'medium',],
          ['claude-sonnet-5-5-effort-high', 'claude-sonnet-5-5-effort-high', 'claude-sonnet-5-5', 'high',],
          ['claude-haiku-5-5', 'claude-haiku-5-5', 'claude-haiku-5-5', 'none',],
          ['claude-haiku-5-5-effort-low', 'claude-haiku-5-5-effort-low', 'claude-haiku-5-5', 'low',],
          ['claude-haiku-5-5-effort-medium', 'claude-haiku-5-5-effort-medium', 'claude-haiku-5-5', 'medium',],
          ['claude-haiku-5-5-effort-high', 'claude-haiku-5-5-effort-high', 'claude-haiku-5-5', 'high',],
        ],);
        expect(CARD_PROVIDERS.includes('anthropic',),).toBe(true,);
      },
    },),

    it({
      name: 'BUILDS ONE ROW PER SERVED SPELLING FROM THE CARDS: a served record over the provider has the eight '
        + 'spellings as its keys, so no wire model is reachable from a spelling the cards do not name',
      fn: async () => {
        expect(
          Object.keys(servedRecord({
            provider: 'anthropic',
            toRow: function row(card,): string {
              return card.anthropic.wireModel;
            },
          },),),
        ).toEqual([...ANTHROPIC_SERVED_IDS,],);
      },
    },),

    it({
      name: 'ADDS EIGHT ROSTER IDENTITIES, each once, each with a card, and no card names a wire model off the list',
      fn: async () => {
        expect(new Set(ROSTER_MODEL_IDS,).size,).toBe(ROSTER_MODEL_IDS.length,);
        for (const id of ANTHROPIC_SERVED_IDS)
          expect(ROSTER_MODEL_IDS,).toContain(id,);
        expect(Object.keys(MODEL_CARDS,).length,).toBe(ROSTER_MODEL_IDS.length,);
        for (const card of cardsServing({ provider: 'anthropic', },))
          expect(ANTHROPIC_WIRE_MODELS,).toContain(card.anthropic.wireModel,);
      },
    },),

    it({
      name: 'PRICES SONNET 5.5 AT 2 AND 10 USD A MILLION with a 2.50 five-minute cache write and a 0.10 cache hit, '
        + 'the same on every Sonnet card',
      fn: async () => {
        for (const id of ANTHROPIC_SERVED_IDS.filter(function isSonnet(candidate,): boolean {
          return candidate.startsWith('claude-sonnet',);
        },)) {
          expect(MODEL_CARDS[id].anthropic?.prices,).toEqual({
            inputUsdPerMillion: 2,
            outputUsdPerMillion: 10,
            cacheWriteUsdPerMillion: 2.5,
            cacheHitUsdPerMillion: 0.1,
          },);
        }
      },
    },),

    it({
      name: 'PRICES HAIKU 5.5 IN TWO TIERS: up to 100,000 prompt tokens at 0.10 and 0.50, and past them at 0.50 and '
        + '2.50, with the cache write and hit scaled with them, the same on every Haiku card',
      fn: async () => {
        for (const id of ANTHROPIC_SERVED_IDS.filter(function isHaiku(candidate,): boolean {
          return candidate.startsWith('claude-haiku',);
        },)) {
          expect(MODEL_CARDS[id].anthropic?.prices,).toEqual({
            inputUsdPerMillion: 0.1,
            outputUsdPerMillion: 0.5,
            cacheWriteUsdPerMillion: 0.125,
            cacheHitUsdPerMillion: 0.01,
            longPrompt: {
              aboveTokens: 100_000,
              inputUsdPerMillion: 0.5,
              outputUsdPerMillion: 2.5,
              cacheWriteUsdPerMillion: 0.625,
              cacheHitUsdPerMillion: 0.05,
            },
          },);
        }
      },
    },),

    it({
      name: 'SEATS ONLY THE TWO CARDS THE OWNER NAMED: Sonnet with no effort holds no seat but the editors\', '
        + 'Haiku with no effort holds the wide seats and the checker bench, and the six effort cards sit on no bench '
        + 'until the effort bench names a level',
      fn: async () => {
        expect(MODEL_CARDS['claude-sonnet-5-5'].holds,).toEqual([
          'judge-unmeasured',
          'writer-unmeasured',
          'reader-unmeasured',
        ],);
        expect(MODEL_CARDS['claude-haiku-5-5'].holds,).toEqual([
          'writer-unmeasured',
          'reader-unmeasured',
          'late-judge-dropped',
        ],);
        for (const id of ANTHROPIC_SERVED_IDS.filter(function isEffortCard(candidate,): boolean {
          return candidate.includes('-effort-',);
        },)) {
          expect(MODEL_CARDS[id].holds,).toEqual([
            'judge-unmeasured',
            'writer-unmeasured',
            'reader-unmeasured',
          ],);
        }
      },
    },),
  ],
},);
