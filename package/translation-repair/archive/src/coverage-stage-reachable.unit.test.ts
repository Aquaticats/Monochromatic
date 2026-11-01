/**
 Guards class fifty (shi_Yumiaoya4, 2026-09-17): a coverage round takes its
 majority over the seats it could reach, not over seats the router refused
 for want of a wet provider. Four of six heard voices found the death
 passage nowhere on the page, two seats were dark, the denominator stayed at
 eight, and the passage shipped as a recorded gap on a SETTLED page.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { runCoverageStage, } from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the stage writes its progress to.
 */
const l = tagged({ tag: 'coverage-stage-reachable-test', },);

/**
 Translation the roster is asked about.
 */
const TARGET_TEXT = `The cat sleeps on the windowsill each morning and naps on its cushion at noon.

She watches the birds outside.
`;

/**
 That translation, parsed once.
 */
const TARGET = parseDocument({ text: TARGET_TEXT, },);

/**
 Roster the questions go to.
 */
const ROSTER = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
  'hf:cat/Cat-C',
  'hf:cat/Cat-D',
].map(function toModelId(id,): RosterModelId {
  return id as unknown as RosterModelId;
},);

/**
 Seat no provider serves for the whole round.
 */
const DRY_SEAT = 'hf:cat/Cat-C' as unknown as RosterModelId;

/**
 What each reachable model answers.
 */
const SCRIPT: Record<string, { readonly coverage: string; readonly quote: string; }> = {
  'hf:cat/Cat-A': {
    coverage: 'none',
    quote: '',
  },
  'hf:cat/Cat-B': {
    coverage: 'none',
    quote: '',
  },
  'hf:cat/Cat-D': {
    coverage: 'partial',
    quote: 'She watches the birds outside.',
  },
};

/**
 Builds a client answering from the script, with one seat the router refuses.

 @returns Client the stage can be driven with

 @example
 ```ts
 const client = scriptedClient();
 ```
 */
function scriptedClient(): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the coverage stage',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the coverage stage',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (request.modelId === DRY_SEAT) {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this model is out of budget',
        },);
      }

      /**
       Reply this model was scripted to give.
       */
      const scripted = SCRIPT[request.modelId];
      if (scripted === undefined) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          detail: 'scripted silence',
        };
      }

      /**
       Wire value carrying it.
       */
      const value: unknown = {
        coverage: scripted.coverage,
        copiedSpan: scripted.quote,
        assessment: 'fixture',
      };
      if (!request.validate(value,)) {
        return {
          kind: 'schema-mismatch',
          rawText: JSON.stringify(value,),
          detail: 'reply failed the wire guard',
        };
      }
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

await describe({
  name: 'a coverage majority over the seats the router could reach (class fifty)',
  children: [
    it({
      name: 'reports ABSENT on two of three reachable seats when the fourth seat was refused by the '
        + 'router: a seat no provider serves was never asked, so it withholds no vote',
      fn: async () => {
        const answer = await runCoverageStage({
          client: scriptedClient(),
          modelIds: ROSTER,
          fanOut: 'whole-bench',
          sourcePassage: '小猫中午在垫子上打盹。',
          translation: TARGET,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(answer.verdict
          .heard,).toBe(3,);
        expect(answer.verdict
          .absent,).toBe(2,);
        expect(answer.verdict
          .asked,).toBe(3,);
        expect(answer.verdict
          .kind,).toBe('absent',);
      },
    },),
  ],
},);
