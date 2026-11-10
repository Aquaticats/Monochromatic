import clearnetHosts from './clearnet-hosts.data.json' with { type: 'json', };

/**
 Base URLs of the model providers the owner named,
 each with the provider id Pi's registry spells it as.

 These hosts stay outside the tunnel for every program,
 apart from the update hosts in `update-hosts.data.ts`,
 because they are outside it for another reason:
 they carry the requests of the Pi agents this repository runs,
 and the tunnel's exit is not their path.
 The routes are recorded in `clearnet-hosts.data.json`,
 which the build also copies beside the route watcher so a running watcher reads a changed list again.
 `provider-hosts.unit.test.ts` derives the host list from these URLs and fails
 when a host the repository's own provider sources name is missing.

 Where each was read, on 2026-11-09:

 - `deepseek`, `opencode-go` and the three Qwen token-plan providers: the
   provider definitions in the installed `@earendil-works/pi-coding-agent`
   1.5.0 registry (`dist/bundle/chunks/*`, `createProvider({id: ...,
   baseUrl: ...})`), which is the registry `pi --list-models` reads.
 - `neuralwatt`: `NEURALWATT_BASE_URL` in
   `package/agent-harness-shared/neuralwatt-listing/src/constants.ts`.
 - `synthetic`: `SYNTHETIC_API_BASE_URL` in the installed
   `@benvargas/pi-synthetic-provider` extension.

 A request to any of them answers without a redirect: an unauthenticated
 `GET` of each provider's model-list path through the tunnel returned no
 `Location` header and no followed redirect (`curl --location`), so no
 redirect host joins this list.
 */
export type ProviderBaseUrl = {
  /**
   Provider id as Pi's registry spells it.
   */
  readonly provider: string;

  /**
   Base URL the provider's requests are sent to,
   without a trailing slash so the host is read exactly as written.
   */
  readonly baseUrl: string;
};

/**
 Every recorded provider route,
 one entry per provider and API flavour.
 */
export const PROVIDER_BASE_URLS: readonly ProviderBaseUrl[] = clearnetHosts.providerBaseUrls;

/**
 Every host those providers' requests reach,
 once each and sorted,
 so a route rule is installed per address of each answer.
 */
export const PROVIDER_HOSTS: readonly string[] = [...new Set(PROVIDER_BASE_URLS.map(function hostOf(entry,): string {
  return new URL(entry.baseUrl,).hostname;
},),),].toSorted();
