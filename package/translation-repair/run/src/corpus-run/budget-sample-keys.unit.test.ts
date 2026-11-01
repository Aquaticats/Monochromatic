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
} from '@monochromatic-dev/test-harness-core/ts';

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

 @returns Refusal's whole text

 @example
 ```ts
 const says = await refusalOver({ env: {}, },);
 ```
 */
async function refusalOver(
  { env, }: { readonly env: Readonly<Record<string, string>>; },
): Promise<string> {
  const refusal = await rejectionOf(function readKeys(): Promise<void> {
    // NOTHING IS LEFT OFF HERE, so the refusal names all five variables as it did before Bedrock was left off.
    readProviderKeys({
      env,
      off: [],
    },);
    return Promise.resolve();
  },);
  expect(refusal,).toBeInstanceOf(StatedRefusalError,);
  return String(refusal,);
}

await describe({
  name: readProviderKeys.name,
  children: [
    it({
      name: 'READS each provider\'s key from its own variable when all five are set and none is left off',
      fn: async () => {
        expect(readProviderKeys({
          env: ALL_FIVE,
          off: [],
        },),).toEqual({
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
          off: [],
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
      name: 'READS THE KEY OF A PROVIDER THE OWNER LEFT OFF AS UNSET, as the run does (`keyUnlessOff`), so a key still '
        + 'in the environment turns nothing back on (Bedrock, since 2026-10-20)',
      fn: async () => {
        expect(readProviderKeys({ env: ALL_FIVE, },).bedrock,).toBe('',);
        expect(readProviderKeys({ env: ALL_FIVE, },).synthetic,).toBe('whisker-key-0001',);
      },
    },),
    it({
      name: 'REFUSES NOTHING FOR THE ABSENT KEY OF A PROVIDER LEFT OFF, since the run asks that provider nothing '
        + 'and a refusal here would stop the pre-launch reading on a key the run does not use',
      fn: async () => {
        expect(readProviderKeys({
          env: {
            ...ALL_FIVE,
            TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY: '',
          },
        },).bedrock,).toBe('',);
      },
    },),
    it({
      name: 'STILL REFUSES NAMING THE BEDROCK VARIABLE when no provider is left off and its key is absent, which is '
        + 'the all-or-none rule for every provider the run asks',
      fn: async () => {
        const refusal = await rejectionOf(function readKeys(): Promise<void> {
          readProviderKeys({
            env: {
              ...ALL_FIVE,
              TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY: '',
            },
            off: [],
          },);
          return Promise.resolve();
        },);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toContain('TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY is absent',);
      },
    },),
    it({
      name: 'NAMES THE VARIABLE OF A PROVIDER LEFT OFF AS LEFT OFF in the refusal for another key, whether its key is '
        + 'set or not, so a reader does not hunt for a key the run does not use',
      fn: async () => {
        const refusal = await rejectionOf(function readKeys(): Promise<void> {
          readProviderKeys({
            env: {
              ...ALL_FIVE,
              TRANSLATION_REPAIR_SYNTHETIC_API_KEY: '',
            },
          },);
          return Promise.resolve();
        },);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${OPENING}TRANSLATION_REPAIR_NEURALWATT_API_KEY is present, TRANSLATION_REPAIR_SYNTHETIC_API_KEY is absent, `
            + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is present, TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY is '
            + `left off, TRANSLATION_REPAIR_OPENROUTER_API_KEY is present. ${CLOSING}`,
        );
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
