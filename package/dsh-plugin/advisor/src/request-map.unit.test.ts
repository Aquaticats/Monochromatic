/**
 Unit tests for the one-shot DSH request construction.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type {
  Api,
  Model,
  TranscriptContext,
} from '@earendil-works/pi-ai';

import { ReasoningEffortId, } from '@deepseek-ai/dsh-llm';

import { SessionId, } from '@deepseek-ai/dsh-session';

import { buildDshAdvisorGenerateOptions, } from '../dist/final/node/index.mjs';

await describe({
  name: buildDshAdvisorGenerateOptions.name,
  children: [
    it({
      name: 'assembles system text and user inputs with the route identity',
      fn: async () => {
        const generate = buildDshAdvisorGenerateOptions({
          context: {
            messages: [
              {
                role: 'system',
                content: 'prompt a',
              },
              {
                role: 'system',
                content: 'prompt b',
              },
              {
                role: 'user',
                content: 'hello',
              },
              {
                role: 'user',
                content: [
                  {
                    type: 'text',
                    text: 'block text',
                  },
                  {
                    type: 'image',
                    mimeType: 'image/png',
                    data: 'x',
                  },
                ],
              },
            ],
          } as unknown as TranscriptContext,
          model: {
            provider: 'route',
            id: 'model-id',
          } as unknown as Model<Api>,
          sessionId: SessionId('session-request-map-a',),
          reasoningEffort: ReasoningEffortId('high',),
          maxTokens: 2_048,
          temperature: 0.5,
        },);
        expect(generate.provider,).toBe('route',);
        expect(generate.model,).toBe('model-id',);
        expect(generate.sessionId,).toBe('session-request-map-a',);
        expect(generate.system,).toBe('prompt a\n\nprompt b',);
        expect(generate.maxTokens,).toBe(2_048,);
        expect(generate.temperature,).toBe(0.5,);
        expect(generate.reasoningEffort,).toBe('high',);
        expect(generate.messages,).toEqual([
          {
            role: 'user',
            content: [
              {
                type: 'text',
                text: 'hello',
              },
            ],
          },
          {
            role: 'user',
            content: [
              {
                type: 'text',
                text: 'block text',
              },
              {
                type: 'text',
                text: '[image: image/png]',
              },
            ],
          },
        ],);
      },
    },),
    it({
      name: 'omits the optional fields when none were supplied',
      fn: async () => {
        const generate = buildDshAdvisorGenerateOptions({
          context: {
            messages: [],
          } as unknown as TranscriptContext,
          model: {
            provider: 'route',
            id: 'model-id',
          } as unknown as Model<Api>,
          sessionId: SessionId('session-request-map-b',),
        },);
        expect('system' in generate,).toBe(false,);
        expect('maxTokens' in generate,).toBe(false,);
        expect('temperature' in generate,).toBe(false,);
        expect('reasoningEffort' in generate,).toBe(false,);
      },
    },),
    it({
      name: 'throws on history roles the one-shot request vocabulary lacks',
      fn: async () => {
        let thrown: unknown;
        try {
          buildDshAdvisorGenerateOptions({
            context: {
              messages: [
                {
                  role: 'assistant',
                  content: [],
                },
              ],
            } as unknown as TranscriptContext,
            model: {
              provider: 'route',
              id: 'model-id',
            } as unknown as Model<Api>,
            sessionId: SessionId('session-request-map-c',),
          },);
        }
        catch (error) {
          thrown = error;
        }
        expect(Error.isError(thrown,),).toBe(true,);
        expect(
          (thrown as Error).message.includes('unsupported role',),
        ).toBe(true,);
      },
    },),
  ],
},);
