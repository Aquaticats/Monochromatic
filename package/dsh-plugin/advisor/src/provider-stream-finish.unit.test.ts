/**
 Unit tests for DSH stream terminal classification and message construction.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  FinishReason,
  GenerateOptions,
} from '@deepseek-ai/dsh-llm';

import {
  advisorStopReasonForFinish,
  buildAdvisorAssistantMessage,
  emptyAdvisorUsage,
} from '../dist/final/node/index.mjs';

await describe({
  name: advisorStopReasonForFinish.name,
  children: [
    it({
      name: 'maps stop, tool-calls, and max-tokens to done reasons',
      fn: async () => {
        expect(
          advisorStopReasonForFinish({
            kind: 'stop',
          },),
        ).toEqual({
          kind: 'done',
          doneReason: 'stop',
        },);
        expect(
          advisorStopReasonForFinish({
            kind: 'tool-calls',
          },),
        ).toEqual({
          kind: 'done',
          doneReason: 'toolUse',
        },);
        expect(
          advisorStopReasonForFinish({
            kind: 'max-tokens',
          },),
        ).toEqual({
          kind: 'done',
          doneReason: 'length',
        },);
      },
    },),
    it({
      name: 'maps failures and marks quota codes with the credit marker',
      fn: async () => {
        const quota = advisorStopReasonForFinish({
          kind: 'error',
          failure: {
            code: 'quota',
            message: 'payment required',
          },
        } as unknown as FinishReason,);
        expect(quota.kind,).toBe('failure',);
        expect(
          quota.kind === 'failure' ? quota.errorMessage : '',
        ).toBe('payment required; insufficient_quota',);
        const aborted = advisorStopReasonForFinish({
          kind: 'aborted',
          failure: {
            code: 'CANCELLED',
            message: 'cancelled',
          },
        } as unknown as FinishReason,);
        expect(
          aborted.kind === 'failure' ? aborted.failureReason : '',
        ).toBe('aborted',);
        expect(
          aborted.kind === 'failure' ? aborted.errorMessage : '',
        ).toBe('cancelled',);
      },
    },),
    it({
      name: 'records a diagnostic for a finish reason the vocabulary lacks',
      fn: async () => {
        const unmapped = advisorStopReasonForFinish({
          kind: 'future-kind',
        } as unknown as FinishReason,);
        expect(unmapped.kind,).toBe('failure',);
        expect(
          unmapped.kind === 'failure'
            ? unmapped.errorMessage.includes('unmapped DSH finish reason',)
            : false,
        ).toBe(true,);
      },
    },),
  ],
},);

await describe({
  name: buildAdvisorAssistantMessage.name,
  children: [
    it({
      name: 'builds a terminal assistant message over the collected state',
      fn: async () => {
        const message = buildAdvisorAssistantMessage({
          generate: {
            provider: 'route',
            model: 'm',
            messages: [],
          } as unknown as GenerateOptions,
          content: [
            {
              type: 'text',
              text: 'answer',
            },
          ],
          usage: emptyAdvisorUsage(),
          stopReason: 'stop',
          errorMessage: 'boom',
          timestampMs: 99,
        },);
        expect(message.role,).toBe('assistant',);
        expect(message.provider,).toBe('route',);
        expect(message.model,).toBe('m',);
        expect(message.errorMessage,).toBe('boom',);
        expect(message.timestamp,).toBe(99,);
      },
    },),
    it({
      name: 'omits the error message when the response finished cleanly',
      fn: async () => {
        const message = buildAdvisorAssistantMessage({
          generate: {
            provider: 'route',
            model: 'm',
            messages: [],
          } as unknown as GenerateOptions,
          content: [],
          usage: emptyAdvisorUsage(),
          stopReason: 'toolUse',
          timestampMs: 5,
        },);
        expect('errorMessage' in message,).toBe(false,);
        expect(message.stopReason,).toBe('toolUse',);
      },
    },),
  ],
},);
