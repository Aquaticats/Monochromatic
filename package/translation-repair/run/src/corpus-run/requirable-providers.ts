import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Requirable providers
// WHICH PROVIDERS AN ARM MAY REQUIRE WET, in a file of its own so the command line table can name them without
// loading the clients `required-providers.ts` reads meters with.

/**
 Provider identities a validation arm may require before calls: every provider but the three optional direct ones.
 */
export type RequiredProvider = Exclude<ProviderName, 'anthropic' | 'deepseek' | 'xiaomi'>;

/**
 Providers a validation arm may require wet, in the order the pass names them.

 DeepSeek, Xiaomi and the Claude key are left out on purpose: all three are optional direct routes that a run uses when
 their key is set and passes over otherwise (`doc/decision/translation-repair-direct-providers.md`),
 so no arm can depend on one being ready.
 */
export const REQUIRABLE_PROVIDERS: readonly RequiredProvider[] = [
  'neuralwatt',
  'synthetic',
  'bedrock',
  'hyper',
  'openrouter',
];

/**
 Whether a name is a provider an arm may require wet.

 POSITIONAL RATHER THAN DESTRUCTURED: TypeScript refuses a type predicate that names an element of a binding
 pattern (TS1230), so a guard takes its subject directly.

 @param provider - name read from the command line

 @returns True where the name is one of {@link REQUIRABLE_PROVIDERS}

 @example
 ```ts
 isRequirable('hyper',);
 ```
 */
export function isRequirable(provider: string,): provider is RequiredProvider {
  return REQUIRABLE_PROVIDERS.some(function named(candidate,): boolean {
    return candidate === provider;
  },);
}

//endregion Requirable providers
