/**
 Unit tests for DSH transcript capture and serialization seams.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { Message as DshMessage, } from '@deepseek-ai/dsh-llm';

import {
  captureDshAdvisorTranscript,
  createDshAdvisorTranscript,
  dshAdvisorTranscriptSerialization,
} from '../dist/final/node/index.mjs';

await describe({
  name: captureDshAdvisorTranscript.name,
  children: [
    it({
      name: 'captures derived messages with an empty degradation reason',
      fn: async () => {
        const snapshot = captureDshAdvisorTranscript({
          source: {
            deriveMessages: () => [
              {
                id: 'm1',
                role: 'user',
                content: [
                  {
                    type: 'text',
                    text: 'hi',
                  },
                ],
                source: {
                  kind: 'user',
                },
              },
            ] as unknown as DshMessage[],
          },
        },);
        expect(snapshot.degradedReason,).toBe('',);
        expect(snapshot.entries.length,).toBe(1,);
      },
    },),
    it({
      name: 'degrades to an empty entry list and reports the failure',
      fn: async () => {
        const reasons: string[] = [];
        const snapshot = captureDshAdvisorTranscript({
          source: {
            deriveMessages: () => {
              throw new Error('derive failed',);
            },
          },
          onDegraded: (reason) => {
            reasons.push(reason,);
          },
        },);
        expect(snapshot.entries,).toEqual([],);
        expect(snapshot.degradedReason.includes('derive failed',),).toBe(true,);
        expect(reasons.length,).toBe(1,);
      },
    },),
  ],
},);

await describe({
  name: createDshAdvisorTranscript.name,
  children: [
    it({
      name: 'exposes the captured entries through the shared port',
      fn: async () => {
        const transcript = createDshAdvisorTranscript({
          source: {
            deriveMessages: () => [
              {
                id: 'm1',
                role: 'user',
                content: 'hello',
              },
            ] as unknown as DshMessage[],
          },
        },);
        expect(transcript.buildContextEntries().length,).toBe(1,);
      },
    },),
  ],
},);

await describe({
  name: 'DSH transcript serialization seam',
  children: [
    it({
      name: 'keeps only pi message roles in the conversion seam',
      fn: async () => {
        const converted = dshAdvisorTranscriptSerialization.convertToLlm([
          {
            role: 'user',
            content: 'hello',
          },
          {
            role: 'assistant',
            content: [],
            api: 'dsh',
            provider: 'p',
            model: 'm',
            usage: {},
            stopReason: 'stop',
            timestamp: 1,
          },
          {
            role: 'system',
            content: 'prompt',
          },
        ] as unknown as Parameters<
          typeof dshAdvisorTranscriptSerialization.convertToLlm
          >[0],);
        expect(converted.map((message,) => message.role,),).toEqual([
          'user',
          'assistant',
          'system',
        ],);
      },
    },),
    it({
      name: 'serializes through the conversation text serializer',
      fn: async () => {
        const text = dshAdvisorTranscriptSerialization.serializeConversation([
          {
            role: 'user',
            content: [
              {
                type: 'text',
                text: 'hello',
              },
            ],
          },
        ] as unknown as Parameters<
          typeof dshAdvisorTranscriptSerialization.serializeConversation
          >[0],);
        expect(text,).toBe('[User]: hello',);
      },
    },),
  ],
},);
