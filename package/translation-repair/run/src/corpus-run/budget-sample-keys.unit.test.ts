/**
 Tests for the five keys a budget sample reads, all or none.

 A SAMPLE OF SOME PROVIDERS IS NOT RECORDED, because the record is read as a
 statement about all of them and a missing column would be indistinguishable
 from a provider that answered. So the read names each variable and whether it
 is present, and an empty variable is as absent as an unset one. The keys are
 invented stand-ins, never a real one.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { readProviderKeys, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';

/**
 Environment holding all five invented keys.
 */
const ALL_FIVE = {
  TRANSLATION_REPAIR_NEURALWATT_API_KEY: 'whisker-key-0005',
  TRANSLATION_REPAIR_SYNTHETIC_API_KEY: 'whisker-key-0001',
  TRANSLATION_REPAIR_CHARM_HYPER_API_KEY: 'whisker-key-0002',
  TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY: 'whisker-key-0003',
  TRANSLATION_REPAIR_OPENROUTER_API_KEY: 'whisker-key-0004',
} as const;

/**
 Sentence the refusal opens with.
 */
const OPENING = 'every provider key must be set to sample availability, and at least one is not: ';

/**
 Sentence the refusal closes with.
 */
const CLOSING = 'Run under mise so sops injects them. A sample of some providers is not recorded, because the record '
  + 'is read as a statement about all of them and a missing column would be indistinguishable from a provider '
  + 'that answered.';

/**
 Reads the keys of an environment that leaves one out.

 @param env - environment to read

 @returns The refusal's whole text

 @example
 ```ts
 const says = await refusalOver({ env: {}, },);
 ```
 */
async function refusalOver(
  { env, }: { readonly env: Readonly<Record<string, string>>; },
): Promise<string> {
  const refusal = await rejectionOf(function readKeys(): Promise<void> {
    readProviderKeys({ env, },);
    return Promise.resolve();
  },);
  expect(refusal,).toBeInstanceOf(StatedRefusalError,);
  return String(refusal,);
}

await describe({
  name: readProviderKeys.name,
  children: [
    it({
      name: 'READS each provider\'s key from its own variable when all five are set',
      fn: async () => {
        expect(readProviderKeys({ env: ALL_FIVE, },),).toEqual({
          neuralwatt: 'whisker-key-0005',
          synthetic: 'whisker-key-0001',
          hyper: 'whisker-key-0002',
          bedrock: 'whisker-key-0003',
          openrouter: 'whisker-key-0004',
          deepseek: '',
          xiaomi: '',
          anthropic: '',
        },);
      },
    },),
    it({
      name: 'READS the three direct providers\' keys when set and refuses nothing when they are not',
      fn: async () => {
        expect(readProviderKeys({
          env: {
            ...ALL_FIVE,
            DEEPSEEK_API_KEY: 'whisker-key-0006',
            TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY: 'whisker-key-0007',
            TRANSLATION_REPAIR_CLAUDE_API_KEY: 'whisker-key-0008',
          },
        },),).toEqual({
          neuralwatt: 'whisker-key-0005',
          synthetic: 'whisker-key-0001',
          hyper: 'whisker-key-0002',
          bedrock: 'whisker-key-0003',
          openrouter: 'whisker-key-0004',
          deepseek: 'whisker-key-0006',
          xiaomi: 'whisker-key-0007',
          anthropic: 'whisker-key-0008',
        },);
      },
    },),
    it({
      name: 'REFUSES naming only the NeuralWatt variable as absent when that one is left out',
      fn: async () => {
        expect(await refusalOver({
          env: {
            ...ALL_FIVE,
            TRANSLATION_REPAIR_NEURALWATT_API_KEY: '',
          },
        },),).toBe(
          `StatedRefusalError: ${OPENING}TRANSLATION_REPAIR_NEURALWATT_API_KEY is absent, `
            + `TRANSLATION_REPAIR_SYNTHETIC_API_KEY is present, TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is present, `
            + `TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY is present, TRANSLATION_REPAIR_OPENROUTER_API_KEY is present. ${
             CLOSING}`,
        );
      },
    },),
    it({
      name: 'REFUSES naming only the Synthetic variable as absent when that one is left out',
      fn: async () => {
        expect(await refusalOver({
          env: {
            ...ALL_FIVE,
            TRANSLATION_REPAIR_SYNTHETIC_API_KEY: '',
          },
        },),).toBe(
          `StatedRefusalError: ${OPENING}TRANSLATION_REPAIR_NEURALWATT_API_KEY is present, TRANSLATION_REPAIR_SYNTHETIC_API_KEY is absent, `
            + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is present, TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY is '
            + `present, TRANSLATION_REPAIR_OPENROUTER_API_KEY is present. ${CLOSING}`,
        );
      },
    },),
    it({
      name: 'REFUSES naming only the Hyper variable as absent when that one is unset',
      fn: async () => {
        expect(await refusalOver({
          env: {
            TRANSLATION_REPAIR_NEURALWATT_API_KEY: ALL_FIVE.TRANSLATION_REPAIR_NEURALWATT_API_KEY,
            TRANSLATION_REPAIR_SYNTHETIC_API_KEY: ALL_FIVE.TRANSLATION_REPAIR_SYNTHETIC_API_KEY,
            TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY: ALL_FIVE.TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY,
            TRANSLATION_REPAIR_OPENROUTER_API_KEY: ALL_FIVE.TRANSLATION_REPAIR_OPENROUTER_API_KEY,
          },
        },),).toBe(
          `StatedRefusalError: ${OPENING}TRANSLATION_REPAIR_NEURALWATT_API_KEY is present, TRANSLATION_REPAIR_SYNTHETIC_API_KEY is present, `
            + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is absent, TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY is '
            + `present, TRANSLATION_REPAIR_OPENROUTER_API_KEY is present. ${CLOSING}`,
        );
      },
    },),
    it({
      name: 'REFUSES naming only the Bedrock variable as absent when that one is left out',
      fn: async () => {
        expect(await refusalOver({
          env: {
            ...ALL_FIVE,
            TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY: '',
          },
        },),).toBe(
          `StatedRefusalError: ${OPENING}TRANSLATION_REPAIR_NEURALWATT_API_KEY is present, TRANSLATION_REPAIR_SYNTHETIC_API_KEY is present, `
            + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is present, TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY is '
            + `absent, TRANSLATION_REPAIR_OPENROUTER_API_KEY is present. ${CLOSING}`,
        );
      },
    },),
    it({
      name: 'REFUSES naming only the OpenRouter variable as absent when that one is left out',
      fn: async () => {
        expect(await refusalOver({
          env: {
            ...ALL_FIVE,
            TRANSLATION_REPAIR_OPENROUTER_API_KEY: '',
          },
        },),).toBe(
          `StatedRefusalError: ${OPENING}TRANSLATION_REPAIR_NEURALWATT_API_KEY is present, TRANSLATION_REPAIR_SYNTHETIC_API_KEY is present, `
            + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is present, TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY is '
            + `present, TRANSLATION_REPAIR_OPENROUTER_API_KEY is absent. ${CLOSING}`,
        );
      },
    },),
    it({
      name: 'REFUSES naming all five as absent for an empty environment',
      fn: async () => {
        expect(await refusalOver({ env: {}, },),).toBe(
          `StatedRefusalError: ${OPENING}TRANSLATION_REPAIR_NEURALWATT_API_KEY is absent, TRANSLATION_REPAIR_SYNTHETIC_API_KEY is absent, `
            + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is absent, TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY is '
            + `absent, TRANSLATION_REPAIR_OPENROUTER_API_KEY is absent. ${CLOSING}`,
        );
      },
    },),
  ],
},);
