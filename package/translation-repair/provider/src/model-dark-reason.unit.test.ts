/**
 Tests for the reason a model with no usable provider gives.

 THE TWO STOPS READ ALIKE IN A LOG unless the reason names which providers serve the model and how each reads: a model
 whose own providers are all dry is a lost seat, and every provider dry at once ends the run.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  modelDarkReason,
  refuseDarkModel,
} from '../dist/final/node/index.mjs';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

/**
 No provider serves.
 */
const NOBODY_SERVES = {
  neuralwatt: false,
  synthetic: false,
  hyper: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: false,
} as const;

/**
 Every provider serves.
 */
const EVERYBODY_SERVES = {
  neuralwatt: true,
  synthetic: true,
  hyper: true,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  openrouter: true,
} as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: modelDarkReason.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS NO PROVIDER SERVES THE MODEL when none does, whatever the providers read, since that is a '
            + 'roster mistake and not a budget state',
          fn: async () => {
            expect(modelDarkReason({ reach: NOBODY_SERVES, dry: EVERYBODY_SERVES, },),).toBe(
              'no provider serves this model',
            );
          },
        },),

        it({
          name: 'NAMES HOW EACH SERVING PROVIDER READS, then how each provider that does not serve the model reads, '
            + 'in spending order',
          fn: async () => {
            expect(modelDarkReason({
              reach: { ...NOBODY_SERVES, hyper: true, openrouter: true, },
              dry: { ...NOBODY_SERVES, hyper: true, openrouter: true, xiaomi: true, },
            },),).toBe(
              'every provider serving this model is out of budget (hyper dry, openrouter dry); the providers that do '
                + 'not serve it read neuralwatt wet, synthetic wet, bedrock wet, deepseek wet, '
                + 'xiaomi dry, anthropic wet',
            );
          },
        },),

        it({
          name: 'LEAVES OUT THE CLAUSE ABOUT PROVIDERS THAT DO NOT SERVE THE MODEL when every provider serves it',
          fn: async () => {
            expect(modelDarkReason({ reach: EVERYBODY_SERVES, dry: EVERYBODY_SERVES, },),).toBe(
              'every provider serving this model is out of budget (neuralwatt dry, synthetic dry, bedrock dry, '
                + 'deepseek dry, xiaomi dry, anthropic dry, hyper dry, openrouter dry)',
            );
          },
        },),
      ],
    },),

    describe({
      name: refuseDarkModel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES WITH THE LOST-SEAT ERROR AND WARNS WITH THE MODEL AND THE REASON, so one line says which model '
            + 'went dark and how every provider read',
          fn: async () => {
            const lines: string[] = [];
            expect(function refusal(): never {
              return refuseDarkModel({
                modelId: 'minimax-m3',
                reason: modelDarkReason({
                  reach: { ...NOBODY_SERVES, hyper: true, },
                  dry: { ...NOBODY_SERVES, hyper: true, },
                },),
                l: levelCapturingLogger({ lines, },),
              },);
            },).toThrow(NoProviderForModelError,);
            expect(lines.length,).toBe(1,);
            expect(lines[0],).toContain('warn ',);
            expect(lines[0],).toContain(
              'minimax-m3 went dark: every provider serving this model is out of budget (hyper dry); the providers '
                + 'that do not serve it read neuralwatt wet',
            );
          },
        },),
      ],
    },),
  ],
},);
