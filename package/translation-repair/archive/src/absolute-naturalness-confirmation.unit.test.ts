/**
 Tests acceptance confirmation uses distinct same-candidate responsibility.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { confirmAbsoluteNaturalness, } from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

await describe({
  name: confirmAbsoluteNaturalness.name,
  children: [
    it({
      name: 'SENDS DISTINCT PROMPTS to same model for discovery and challenge',
      fn: async () => {
        /** Exact serialized prompts observed at model-facing boundary. */
        const prompts: string[] = [];
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused by confirmation fixture',);
          },
          chatJson: async <ValueT,>(
            request: ChatJsonRequest<ValueT>,
          ): Promise<ChatJsonOutcome<ValueT>> => {
            prompts.push(JSON.stringify(request.messages,),);
            const value: unknown = {
              isAcceptable: true,
              findings: [],
              assessment: 'fixture accepts exact candidate',
            };
            if (!request.validate(value,))
              throw new Error('confirmation fixture response failed validation',);
            return {
              kind: 'ok',
              value,
              rawText: JSON.stringify(value,),
            };
          },
          quotas: async () => {
            throw new Error('quotas unused by confirmation fixture',);
          },
        };
        const confirmed = await confirmAbsoluteNaturalness({
          client,
          modelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
          subject: {
            lineStructured: false,
            sourceText: '猫在睡觉。',
            candidateText: 'The cat is sleeping.',
            paragraphs: ['The cat is sleeping.',],
          },
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'absolute-confirmation-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(confirmed.review.verdict,).toBe('acceptable',);
        expect(confirmed.confirmations,).toHaveLength(1,);
        expect(prompts,).toHaveLength(2,);
        expect(new Set(prompts,).size,).toBe(2,);
      },
    },),
  ],
},);
