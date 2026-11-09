/**
 Base URLs of the model providers the owner named,
 each with the provider id Pi's registry spells it as.

 These hosts stay outside the tunnel for every program,
 apart from the update hosts in `update-hosts.data.ts`,
 because they are outside it for another reason:
 they carry the requests of the Pi agents this repository runs,
 and the tunnel's exit is not their path.
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
 Pi's `DeepSeek` provider.
 */
export const DEEPSEEK_BASE_URL = 'https://api.deepseek.com';

/**
 NeuralWatt's route,
 which the repository's Pi plugin and its listing share.
 */
export const NEURALWATT_BASE_URL = 'https://api.neuralwatt.com/v1';

/**
 `opencode-go`'s OpenAI-compatible route,
 which every `openai-completions` model of that provider uses.
 */
export const OPENCODE_GO_OPENAI_BASE_URL = 'https://opencode.ai/inference/go/openai/v1';

/**
 `opencode-go`'s Anthropic-compatible route,
 which its `anthropic-messages` models use.
 */
export const OPENCODE_GO_ANTHROPIC_BASE_URL = 'https://opencode.ai/inference/go/anthropic';

/**
 `Qwen Token Plan` and `Qwen Token Plan Individual`,
 which the same regional endpoint serves.
 */
export const QWEN_TOKEN_PLAN_BASE_URL = 'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1';

/**
 `Qwen Token Plan CN`,
 the other regional endpoint Pi's registry lists.
 */
export const QWEN_TOKEN_PLAN_CN_BASE_URL = 'https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1';

/**
 Synthetic's OpenAI-compatible route and its quota endpoint,
 which share one host.
 */
export const SYNTHETIC_BASE_URL = 'https://api.synthetic.new/openai/v1';

/**
 Every recorded provider route,
 one entry per provider and API flavour.
 */
export const PROVIDER_BASE_URLS: readonly ProviderBaseUrl[] = [
  {
    provider: 'deepseek',
    baseUrl: DEEPSEEK_BASE_URL,
  },
  {
    provider: 'neuralwatt',
    baseUrl: NEURALWATT_BASE_URL,
  },
  {
    provider: 'opencode-go',
    baseUrl: OPENCODE_GO_OPENAI_BASE_URL,
  },
  {
    provider: 'opencode-go',
    baseUrl: OPENCODE_GO_ANTHROPIC_BASE_URL,
  },
  {
    provider: 'qwen-token-plan',
    baseUrl: QWEN_TOKEN_PLAN_BASE_URL,
  },
  {
    provider: 'qwen-token-plan-individual',
    baseUrl: QWEN_TOKEN_PLAN_BASE_URL,
  },
  {
    provider: 'qwen-token-plan-cn',
    baseUrl: QWEN_TOKEN_PLAN_CN_BASE_URL,
  },
  {
    provider: 'synthetic',
    baseUrl: SYNTHETIC_BASE_URL,
  },
];

/**
 Every host those providers' requests reach,
 once each and sorted,
 so a route rule is installed per address of each answer.
 */
export const PROVIDER_HOSTS: readonly string[] = [...new Set(PROVIDER_BASE_URLS.map(function hostOf(entry,): string {
  return new URL(entry.baseUrl,).hostname;
},),),].toSorted();
