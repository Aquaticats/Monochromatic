import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  CLAUDE_KEY_VARIABLE,
  DEEPSEEK_KEY_VARIABLE,
  XIAOMI_KEY_VARIABLE,
} from './direct-key-variables.ts';
import {
  isProviderOff,
  PROVIDERS_LEFT_OFF,
  type ProviderOff,
} from './providers-off.ts';

//region Budget sample keys
// Reads the five provider keys a budget sample needs, all or none, and the two direct providers' keys, which it
// takes when set and leaves empty otherwise. A provider left off is the exception to all or none (B1185): the run
// asks it nothing, so its key is read as unset and never required.

/**
 The five provider keys, each non-empty unless the provider is left off, and the three optional ones.
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
   Bedrock key, empty while Bedrock is left off.
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
 One of the five variables a sample reads, as the refusal names it.
 */
type KeyRow = Readonly<{
  /**
   Provider the variable belongs to.
   */
  provider: ProviderName;

  /**
   Name of the environment variable.
   */
  variable: string;

  /**
   What the sample uses: the environment's value, or empty where the provider is left off or the variable is unset.
   */
  key: string;

  /**
   How the refusal words the variable: `present`, `absent`, or `left off`.
   */
  state: 'absent' | 'left off' | 'present';
}>;

/**
 Reads one provider's variable, as unset where the provider is left off.

 @param provider - provider the variable belongs to

 @param variable - name of the environment variable

 @param held - environment's value, empty when unset

 @param off - providers left off

 @returns Row for the refusal and the key the sample uses

 @example
 ```ts
 const row = keyRowOf({ provider: 'bedrock', variable: 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY', held: 'k', off, },);
 // => { provider: 'bedrock', variable: '...', key: '', state: 'left off', } while Bedrock is off
 ```
 */
function keyRowOf(
  {
    provider,
    variable,
    held,
    off,
  }: {
    readonly provider: ProviderName;
    readonly variable: string;
    readonly held: string;
    readonly off: readonly ProviderOff[];
  },
): KeyRow {
  if (isProviderOff({
    off,
    provider,
  },)) {
    return {
      provider,
      variable,
      key: '',
      state: 'left off',
    };
  }
  return {
    provider,
    variable,
    key: held,
    state: (held === '') ? 'absent' : 'present',
  };
}

/**
 Reads one of the three optional keys, as unset where its provider is left off (B1209).

 @param off - providers left off

 @param provider - provider whose optional key is read

 @param held - the environment's value, empty where unset

 @returns The key, empty where the provider is left off or the variable is unset

 @example
 ```ts
 const key = optionalKeyOf({ off, provider: 'deepseek', held: env.DEEPSEEK_KEY ?? '', },);
 ```
 */
function optionalKeyOf(
  {
    off,
    provider,
    held,
  }: {
    readonly off: readonly ProviderOff[];
    readonly provider: ProviderName;
    readonly held: string;
  },
): string {
  return isProviderOff({
    off,
    provider,
  },)
    ? ''
    : held;
}

/**
 Reads every provider's key.

 A PROVIDER LEFT OFF IS READ AS UNSET AND NEVER REQUIRED, as the run reads it (`keyUnlessOff` in `run-providers.ts`):
 the run asks it nothing, so a sample that asked it would print `bedrock wet` for a provider the launch holds off, and
 one that required its key would refuse a reading over a key the run does not use (B1185). Its variable is named in a
 refusal for another key as left off.

 @param env - environment the keys are read from: `process.env` in a run

 @param off - providers left off, whose keys are read as unset and never required; the owner's rulings by default
 (`PROVIDERS_LEFT_OFF`), an empty list in a case that exercises an off provider's client

 @returns Five keys, an off provider's empty, and the three optional ones empty where unset

 @throws {@link StatedRefusalError} when any key of a provider that is not off is absent or empty, naming
 each variable and whether it is present, since a sample of some providers
 cannot answer a question about the others

 @example
 ```ts
 const keys = readProviderKeys({ env: process.env, },);
 ```
 */
export function readProviderKeys(
  {
    env,
    off = PROVIDERS_LEFT_OFF,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly off?: readonly ProviderOff[];
  },
): ProviderKeys {
  /**
   The five providers' rows, in the order the refusal names them: NeuralWatt, Synthetic, Hyper, Bedrock, OpenRouter.
   Each key is injected by mise from the sops-encrypted env, so an unset variable reads as empty.
   */
  const [
    neuralwatt,
    synthetic,
    hyper,
    bedrock,
    openrouter,
  ] = [
    keyRowOf({
      provider: 'neuralwatt',
      variable: 'TRANSLATION_REPAIR_NEURALWATT_API_KEY',
      held: env.TRANSLATION_REPAIR_NEURALWATT_API_KEY ?? '',
      off,
    },),
    keyRowOf({
      provider: 'synthetic',
      variable: 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY',
      held: env.TRANSLATION_REPAIR_SYNTHETIC_API_KEY ?? '',
      off,
    },),
    keyRowOf({
      provider: 'hyper',
      variable: 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY',
      held: env.TRANSLATION_REPAIR_CHARM_HYPER_API_KEY ?? '',
      off,
    },),
    keyRowOf({
      provider: 'bedrock',
      variable: 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY',
      held: env.TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY ?? '',
      off,
    },),
    keyRowOf({
      provider: 'openrouter',
      variable: 'TRANSLATION_REPAIR_OPENROUTER_API_KEY',
      held: env.TRANSLATION_REPAIR_OPENROUTER_API_KEY ?? '',
      off,
    },),
  ] as const;

  /**
   The five rows in the refusal's order.
   */
  const rows: readonly KeyRow[] = [
    neuralwatt,
    synthetic,
    hyper,
    bedrock,
    openrouter,
  ];
  if (rows.some(function absent({ state, },): boolean {
    return state === 'absent';
  },)) {
    /**
     Each variable beside how it reads, in the refusal's order.
     */
    const spelled = rows.map(function spell({
      variable,
      state,
    },): string {
      return `${variable} is ${state}`;
    },);
    throw new StatedRefusalError({
      says: 'every provider key must be set to sample availability, and at least one is not: '
        + `${spelled.join(', ',)}. `
        + 'Run under mise so sops injects them. A sample of some providers is not recorded, '
        + 'because the record is read as a statement about all of them and a missing column would '
        + 'be indistinguishable from a provider that answered.',
    },);
  }

  return {
    neuralwatt: neuralwatt.key,
    synthetic: synthetic.key,
    hyper: hyper.key,
    bedrock: bedrock.key,
    openrouter: openrouter.key,
    // THE THREE OPTIONAL KEYS ARE READ AS UNSET TOO WHEN THEIR PROVIDER IS LEFT OFF (B1209), so no provider left off is
    // sampled whichever key the environment holds.
    deepseek: optionalKeyOf({
      off,
      provider: 'deepseek',
      held: env[DEEPSEEK_KEY_VARIABLE] ?? '',
    },),
    xiaomi: optionalKeyOf({
      off,
      provider: 'xiaomi',
      held: env[XIAOMI_KEY_VARIABLE] ?? '',
    },),
    anthropic: optionalKeyOf({
      off,
      provider: 'anthropic',
      held: env[CLAUDE_KEY_VARIABLE] ?? '',
    },),
  };
}

//endregion Budget sample keys
