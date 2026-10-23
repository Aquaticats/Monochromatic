/**
 Tests for the Claude card sides themselves: the eight rows keyed by roster id, the two price tables and the
 fields every row shares. How the rows seat, price and lock is held by `claude-cards.unit.test.ts` through the
 assembled cards; this file holds the module's own exports.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  ANTHROPIC_SERVED_IDS,
  CLAUDE_CARDS,
  CLAUDE_COMPLETION_CAP,
  CLAUDE_FIRST_BYTE_ALLOWANCE_MS,
  CLAUDE_FLOOR_TOKENS_PER_SECOND,
  claudeCallDeadlineMs,
  COMPLETION_CAP,
  HAIKU_5_5_PRICES,
  SONNET_5_5_PRICES,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'claude card sides',
  children: [
    it({
      name: 'KEYS the eight rows by the eight roster spellings, in the order the spellings are listed',
      fn: async () => {
        expect(Object.keys(CLAUDE_CARDS,),).toEqual([...ANTHROPIC_SERVED_IDS,],);
      },
    },),

    it({
      name: 'GIVES EVERY ROW the shared fields: it reads images and states the 128,000-token output ceiling',
      fn: async () => {
        for (const id of ANTHROPIC_SERVED_IDS) {
          const card = CLAUDE_CARDS[id];
          expect(card.anthropic?.readsImages,).toBe(true,);
          expect(card.anthropic?.maxOutputLength,).toBe(128_000,);
        }
      },
    },),

    it({
      name: 'CAPS EVERY ROW\'S COMPLETION AT 32,000 TOKENS, not at the pooled p99 of 13,082: thinking counts toward '
        + 'max_tokens on both models, so a cap measured on providers that hide no thinking would be spent thinking and '
        + 'return nothing (stop_reason max_tokens after a thinking block), billed in full and asked again',
      fn: async () => {
        for (const id of ANTHROPIC_SERVED_IDS) {
          expect(CLAUDE_CARDS[id].completionCap,).toBe(32_000,);
          expect(COMPLETION_CAP[id],).toBe(32_000,);
        }
      },
    },),

    it({
      name: 'SIZES THE CLAUDE CALL DEADLINE FROM THE COMPLETION CAP (R1): the cap at the floor rate of 80 tokens a '
        + 'second, 400 s for 32,000 tokens, plus a 60 s allowance for the prompt and the first byte, 460,000 ms; a '
        + 'smaller ceiling asks for less, and the cap, the rate and the allowance are the exported constants',
      fn: async () => {
        expect(CLAUDE_COMPLETION_CAP,).toBe(32_000,);
        expect(CLAUDE_FLOOR_TOKENS_PER_SECOND,).toBe(80,);
        expect(CLAUDE_FIRST_BYTE_ALLOWANCE_MS,).toBe(60_000,);
        expect(claudeCallDeadlineMs({ maxTokens: CLAUDE_COMPLETION_CAP, },),).toBe(460_000,);
        expect(claudeCallDeadlineMs({ maxTokens: 800, },),).toBe(70_000,);
        expect(claudeCallDeadlineMs({ maxTokens: 1, },),).toBe(60_013,);
      },
    },),

    it({
      name: 'TIES each row to the price table of its model, the same on every effort card of one model',
      fn: async () => {
        for (const id of ANTHROPIC_SERVED_IDS) {
          expect(CLAUDE_CARDS[id].anthropic?.prices,).toBe(
            id.startsWith('claude-sonnet',) ? SONNET_5_5_PRICES : HAIKU_5_5_PRICES,
          );
        }
      },
    },),

    it({
      name: 'HOLDS BACK ONLY SONNET\'S AND THE EFFORT CARDS\' JUDGING: Haiku with no effort keeps the judge seats and '
        + 'drops the late rounds, and the other seven carry the judge hold',
      fn: async () => {
        expect(CLAUDE_CARDS['claude-haiku-5-5'].holds,).toContain('late-judge-dropped',);
        expect(CLAUDE_CARDS['claude-haiku-5-5'].holds,).not.toContain('judge-unmeasured',);
        for (const id of ANTHROPIC_SERVED_IDS.filter(function notHaiku(candidate,): boolean {
          return candidate !== 'claude-haiku-5-5';
        },))
          expect(CLAUDE_CARDS[id].holds,).toContain('judge-unmeasured',);
      },
    },),
  ],
},);
