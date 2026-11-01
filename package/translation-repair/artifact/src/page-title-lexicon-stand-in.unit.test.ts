/**
 Tests for the order the page title lexicon reads its voices in when another model stood in for a seat that ran away
 reasoning (ledger B1200): a voice orders by the seat it answered for, never by the model that answered, since that model
 is on no bench of the gather.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { settlePageTitles, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_VISION,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

//region Page title stand-in tests

/**
 Logger the stage writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'page-title-lexicon-stand-in-test', },);

/**
 Four seats, in roster order; the last one runs away.
 */
const BENCH: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY,
  SEAT_HYPER_VISION,
];

/**
 Original naming one title twice.
 */
const SOURCE = '## 猫之歌\n\n小猫唱了《猫之歌》，又读了《猫之歌》。\n';

/**
 The one title, as the page repeats it.
 */
const SPANS = [{ source: '猫之歌', occurrences: 2, },];

/**
 Rendering the two earliest seats give.
 */
const EARLY_RENDERING = 'Song of the Cat';

/**
 Rendering the third seat and the model standing in for the last give, which ties the early one at two voices apiece.
 */
const LATE_RENDERING = 'Cat Song';

/**
 Client whose last bench seat runs away reasoning and whose every other model answers, the third seat and any model
 off the bench with the late rendering.

 @returns Client serving only structured calls

 @example
 ```ts
 const client = clientWhereTheLastSeatRunsAway();
 ```
 */
function clientWhereTheLastSeatRunsAway(): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      if (request.modelId === SEAT_HYPER_VISION) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          reason: 'truncated-completion',
          detail: 'provider reported a truncating completion',
        };
      }

      /**
       Rendering this model gives.
       */
      const rendering = (request.modelId === SEAT_SYNTHETIC_TEXT_EVERYWHERE) || (request.modelId === SEAT_HYPER_ONLY)
        ? EARLY_RENDERING
        : LATE_RENDERING;

      /**
       Scripted reply for this model.
       */
      const value: unknown = { titles: [{ title: 1, rendering, },], };
      if (!request.validate(value,))
        throw new Error('scripted reply failed validator',);
      return { kind: 'ok', value, rawText: JSON.stringify(value,), };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

await describe({
  name: `${settlePageTitles.name} (a stand-in for a seat that ran away)`,
  concurrency: 1,
  children: [
    it({
      name: 'BREAKS A TIE BY THE SEAT A STAND-IN ANSWERED FOR, so the last seat\'s stand-in does not outrank the first '
        + 'two seats (the earliest seat\'s wording wins, as for a seat that answered itself)',
      fn: async () => {
        /**
         What the round settled.
         */
        const lexicon = await settlePageTitles({
          client: clientWhereTheLastSeatRunsAway(),
          modelIds: BENCH,
          sourceText: SOURCE,
          spans: SPANS,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          fanOut: 'whole-bench',
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect({
          heard: lexicon.heard,
          titles: lexicon.titles.map(function renderingOf(title,): string {
            return `${title.rendering} (${String(title.voices,)} voices)`;
          },),
        },).toEqual({
          heard: 4,
          titles: [`${EARLY_RENDERING} (2 voices)`,],
        },);
      },
    },),
  ],
},);

//endregion Page title stand-in tests
