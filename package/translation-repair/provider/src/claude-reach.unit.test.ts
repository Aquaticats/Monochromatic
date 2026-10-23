/**
 Tests for where the Claude key reaches the roster: the eight Claude cards, spelled as themselves, and no other
 roster entry, so a call for any other model never goes to the key. Nothing reaches a provider.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  ANTHROPIC_SERVED_IDS,
  ROSTER_MODEL_IDS,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  claudeIdFor,
  claudeShowsPictures,
} from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_TEXT_EVERYWHERE, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

await describe({
  name: 'claude reach',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'SPELLS each Claude card as itself and no other roster model, found by walking every roster entry',
      fn: async () => {
        expect(claudeIdFor({ modelId: 'claude-sonnet-5-5-effort-high', },),)
          .toEqual({ served: true, id: 'claude-sonnet-5-5-effort-high', },);
        expect(claudeIdFor({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },),).toEqual({ served: false, },);
        expect(ROSTER_MODEL_IDS.filter((modelId,) => claudeIdFor({ modelId, },).served),)
          .toEqual([...ANTHROPIC_SERVED_IDS,],);
      },
    },),
    it({
      name: 'SHOWS pictures on the eight Claude cards, which read images, and on no other model',
      fn: async () => {
        for (const modelId of ROSTER_MODEL_IDS)
          expect(claudeShowsPictures({ modelId, },),).toBe((ANTHROPIC_SERVED_IDS as readonly string[]).includes(modelId,),);
      },
    },),
  ],
},);
