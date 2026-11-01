import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  CLAUDE_KEY_VARIABLE,
  DEEPSEEK_KEY_VARIABLE,
  XIAOMI_KEY_VARIABLE,
} from './direct-key-variables.ts';

//region Budget sample keys
// Reads the five provider keys a budget sample needs, all or none, and the two direct providers' keys, which it
// takes when set and leaves empty otherwise.

/**
 The five provider keys, each non-empty, and the three optional ones.
 */
export type ProviderKeys = {
  /**
   NeuralWatt key.
   */
  readonly neuralwatt: string;

  /**
   Synthetic key.
   */
  readonly synthetic: string;

  /**
   Hyper key.
   */
  readonly hyper: string;

  /**
   Bedrock key.
   */
  readonly bedrock: string;

  /**
   OpenRouter key.
   */
  readonly openrouter: string;

  /**
   DeepSeek key, empty when unset: the provider is optional, so its absence
   refuses nothing and the sample reads it as unconfigured.
   */
  readonly deepseek: string;

  /**
   Xiaomi key, empty when unset, optional likewise.
   */
  readonly xiaomi: string;

  /**
   The owner's Claude key, empty when unset, optional likewise.
   */
  readonly anthropic: string;
};

/**
 Reads every provider's key.

 @param env - environment the keys are read from: `process.env` in a run

 @returns The five keys, and the two optional ones empty where unset

 @throws {@link StatedRefusalError} when any key is absent or empty, naming
 each variable and whether it is present, since a sample of some providers
 cannot answer a question about the others

 @example
 ```ts
 const keys = readProviderKeys({ env: process.env, },);
 ```
 */
export function readProviderKeys(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): ProviderKeys {
  /**
   NeuralWatt's key, from the same place.
   */
  const neuralWattKey = env
    .TRANSLATION_REPAIR_NEURALWATT_API_KEY
    ?? '';

  /**
   First provider's key, injected by mise from the sops-encrypted env.
   */
  const syntheticKey = env
    .TRANSLATION_REPAIR_SYNTHETIC_API_KEY
    ?? '';

  /**
   Second provider's key, from the same place.
   */
  const hyperKey = env
    .TRANSLATION_REPAIR_CHARM_HYPER_API_KEY
    ?? '';

  /**
   Third provider's key, from the same place.
   */
  const openRouterKey = env
    .TRANSLATION_REPAIR_OPENROUTER_API_KEY
    ?? '';

  /**
   Fourth provider's key, from the same place.
   */
  const bedrockKey = env
    .TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY
    ?? '';

  /**
   Whether any provider's key is missing, which makes the sample partial.
   */
  const someKeyMissing = (neuralWattKey === '')
    || (syntheticKey === '')
    || (hyperKey === '')
    || (bedrockKey === '')
    || (openRouterKey === '');
  if (someKeyMissing) {
    throw new StatedRefusalError({
      says: 'every provider key must be set to sample availability, and at least one is not: '
        + `TRANSLATION_REPAIR_NEURALWATT_API_KEY is ${neuralWattKey === '' ? 'absent' : 'present'}, `
        + `TRANSLATION_REPAIR_SYNTHETIC_API_KEY is ${syntheticKey === '' ? 'absent' : 'present'}, `
        + `TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is ${hyperKey === '' ? 'absent' : 'present'}, `
        + `TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY is ${bedrockKey === '' ? 'absent' : 'present'}, `
        + `TRANSLATION_REPAIR_OPENROUTER_API_KEY is ${openRouterKey === '' ? 'absent' : 'present'}. `
        + 'Run under mise so sops injects them. A sample of some providers is not recorded, '
        + 'because the record is read as a statement about all of them and a missing column would '
        + 'be indistinguishable from a provider that answered.',
    },);
  }

  return {
    neuralwatt: neuralWattKey,
    synthetic: syntheticKey,
    hyper: hyperKey,
    bedrock: bedrockKey,
    openrouter: openRouterKey,
    deepseek: env[DEEPSEEK_KEY_VARIABLE] ?? '',
    xiaomi: env[XIAOMI_KEY_VARIABLE] ?? '',
    anthropic: env[CLAUDE_KEY_VARIABLE] ?? '',
  };
}

//endregion Budget sample keys
