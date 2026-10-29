/**
 Tests for the registration gap check that decides whether a provider can be filtered.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type {
  ProviderConfig,
  ProviderModelConfig,
} from '@earendil-works/pi-coding-agent';
import {
  planGap,
  REGISTERABLE,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Build one chat model configuration.

 @param id - model id

 @param api - API type the model is served under, empty to simulate a missing one

 @param baseUrl - endpoint the model reports, empty to simulate a missing one

 @returns configuration shaped like one a plan would register
 */
function chatConfig(
  {
    id,
    api = 'openai-completions',
    baseUrl = 'https://example.invalid',
  }: {
    readonly id: string;
    readonly api?: string;
    readonly baseUrl?: string;
  },
): ProviderModelConfig {
  return {
    id,
    name: id,
    api,
    baseUrl,
    input: ['text'],
    cost: {
      input: 1,
      output: 2,
      cacheRead: 0,
      cacheWrite: 0,
    },
    reasoning: true,
    contextWindow: 128_000,
    maxTokens: 4_096,
    type: 'chat',
  };
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: planGap.name,
      children: [
        it({
          name: 'accepts a plan whose models all carry an api and a baseUrl',
          fn: async function runRegisterablePlan() {
            expect(planGap({
              models: [
                chatConfig({ id: 'glm-5.3', },),
                chatConfig({ id: 'glm-5.3-flash', },),
              ],
              base: {},
            },),).toBe(REGISTERABLE,);
          },
        },),
        it({
          name: 'accepts an empty plan',
          fn: async function runEmptyPlan() {
            expect(planGap({ models: [], base: {}, },),).toBe(REGISTERABLE,);
          },
        },),
        it({
          name: 'names the model that carries no baseUrl',
          fn: async function runMissingBaseUrl() {
            /**
             Gap reported for the measured azure shape.
             */
            const gap = planGap({
              models: [chatConfig({ id: 'gpt-4-turbo', baseUrl: '', },)],
              base: {},
            },);
            expect(gap,).toBe('model gpt-4-turbo carries no baseUrl',);
          },
        },),
        it({
          name: 'names the model that carries no api',
          fn: async function runMissingApi() {
            /**
             Gap reported when neither the model nor the provider resolves an api.
             */
            const gap = planGap({
              models: [chatConfig({ id: 'gpt-4-turbo', api: '', },)],
              base: {},
            },);
            expect(gap,).toBe('model gpt-4-turbo carries no api',);
          },
        },),
        it({
          name: 'accepts a provider-level endpoint as the fallback pi uses',
          fn: async function runProviderFallback() {
            /**
             Incumbent configuration supplying what the models omit.
             */
            const base: ProviderConfig = {
              api: 'openai-responses',
              baseUrl: 'https://provider.invalid/v1',
            };
            expect(planGap({
              models: [chatConfig({
                id: 'gpt-4-turbo',
                api: '',
                baseUrl: '',
              },)],
              base,
            },),).toBe(REGISTERABLE,);
          },
        },),
        it({
          name: 'reports the first gap only',
          fn: async function runFirstGap() {
            /**
             Gap reported for a plan with two unregisterable models.
             */
            const gap = planGap({
              models: [
                chatConfig({ id: 'gpt-4-turbo', baseUrl: '', },),
                chatConfig({ id: 'gpt-4.1', baseUrl: '', },),
              ],
              base: {},
            },);
            expect(gap,).toBe('model gpt-4-turbo carries no baseUrl',);
          },
        },),
      ],
    },),
  ],
},);
