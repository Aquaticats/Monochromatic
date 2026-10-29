/**
 Tests that a picture sent to Gemma through OpenRouter keeps zero data
 retention and carries no reasoning knob.

 WHY: Gemma 4 26B A4B is being prepared as a picture reader on OpenRouter
 (2026-10-21). The owner's standing decision of 2026-09-03 is that only
 endpoints with a zero-data-retention policy may serve a corpus passage, and
 a picture is the most identifying corpus content there is. The measuring
 task that seats the reader (`reader-measure` in the bench package) asks
 through this client, so the body it sends is pinned here.

 Fixtures are cat-themed invention. No call reaches OpenRouter.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createOpenRouterClient, } from '../dist/final/node/index.mjs';
import {
  recordingTransport,
  settled,
} from './recording-transport.test-fixture.ts';
import { SEAT_HYPER_TEXT_BEDROCK, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

await describe({
  name: 'OpenRouter picture body',
  children: [
    it({
      name: 'KEEPS zdr true, THE PICTURE PART AND THE OWNER\'S BODY RULES ON A PICTURE CALL TO GEMMA',
      fn: async () => {
        /**
         Bodies the transport recorded.
         */
        const bodies: string[] = [];
        const client = createOpenRouterClient({
          apiKey: 'whiskers',
          transport: recordingTransport({ bodies, },),
          retryPolicy: {
            limit: 0,
            baseMs: 1,
          },
        },);
        await settled(client.chatText({
          modelId: SEAT_HYPER_TEXT_BEDROCK,
          messages: [{
            role: 'user',
            content: [
              {
                type: 'text',
                text: 'Transcribe every word visible in this image.',
              },
              {
                type: 'image_url',
                image_url: { url: 'data:image/webp;base64,UklGRg==', },
              },
            ],
          },],
          signal: new AbortController().signal,
        },),);
        const [bodyJson,] = bodies;
        const body = JSON.parse(bodyJson ?? '{}',) as Record<string, unknown>;
        expect(body.model,).toBe('google/gemma-4-26b-a4b-it',);
        expect(body.provider,).toMatchObject({ zdr: true, },);
        expect(JSON.stringify(body.messages,),).toContain('image_url',);
        expect(typeof body.max_tokens,).toBe('number',);
        for (const absent of ['reasoning', 'reasoning_effort', 'thinking', 'thinking_token_budget',])
          expect(Object.hasOwn(body, absent,),).toBe(false,);
      },
    },),
  ],
},);
