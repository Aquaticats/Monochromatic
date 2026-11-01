import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Providers left off
// A PROVIDER THE OWNER RULED TO LEAVE OFF, named once with the ruling, the way
// `OWNER_CULLED` names the models the owner culled: a dated decision held as a
// constant in code, so a key left in the environment does not turn it back on.
//
// OFF MEANS NO CLIENT. `configureProviders` reads an off provider's key as
// unset, and an unset key already has one meaning everywhere: the provider has
// no caller, its meter reads dry with nothing to report (`UNCONFIGURED_METER`),
// the router never sends it a call, and the seat readers name a model only it
// serves as unserved before any call is made. So nothing here teaches the router
// or the seat readers a new state; it feeds them the state they have.

/**
 One provider left off, with the words that say why.
 */
export type ProviderOff = {
  /**
   Provider no run asks while it is off.
   */
  readonly provider: ProviderName;

  /**
   Why it is off, written for the launch line.
   */
  readonly reason: string;
};

/**
 Providers every run leaves off.

 BEDROCK SINCE 2026-10-20. Its key expired (HTTP 401 "The security token
 included in the request has expired" on every call of zheermao19,
 CuspariaKLSY20, CuspariaKLSY21 and CuspariaKLSY22), and the owner ruled to
 leave Bedrock off rather than issue a new key. A run that still asked it once
 was refused and held it out for the rest of the run, and spent the one ask on
 nothing. The client and its tests stay: turning Bedrock on again is deleting
 this entry, not rebuilding a provider.
 */
export const PROVIDERS_LEFT_OFF: readonly ProviderOff[] = [
  {
    provider: 'bedrock',
    reason: 'the owner ruled on 2026-10-20 to leave Bedrock off rather than issue a new key, since its token expired '
      + '(HTTP 401 on every call); google.gemma-4-e2b is served only there and so has no provider',
  },
];

/**
 Whether a provider is among those left off.

 @param off - providers left off

 @param provider - provider asked about

 @returns True where the provider is off

 @example
 ```ts
 const off = isProviderOff({ off: PROVIDERS_LEFT_OFF, provider: 'bedrock', },);
 // => true
 ```
 */
export function isProviderOff(
  {
    off,
    provider,
  }: {
    readonly off: readonly ProviderOff[];
    readonly provider: ProviderName;
  },
): boolean {
  return off.some(function names(entry,): boolean {
    return entry.provider === provider;
  },);
}

//endregion Providers left off
