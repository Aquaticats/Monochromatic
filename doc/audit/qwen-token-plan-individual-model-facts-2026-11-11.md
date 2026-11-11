# Qwen Token Plan individual model facts, 2026-11-11

Read-only web research record for the Qwen Token Plan (individual plan, international) model
surface. The session ran from 2026-11-11T05:04:39-05:00; every quoted page was fetched in that
session and its quoted text was checked against the fetched page text. No endpoint was called
from this session, so nothing below is a live probe.

The endpoints under audit are `https://token-plan.maas.qwencloudapi.com/compatible-mode/v1` and
`https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1`.

## Fact labels

- measured: an observation of the live endpoints, supplied by the requesting session and not
  re-measured here.
- documented: stated by official documentation, cited inline.
- unknown: no official documentation found; the searches run are listed in
  "Searches performed for question 1".

## Question 1: is there a documented per-model metadata API?

### Answer

Yes for Alibaba Cloud Model Studio (百炼, DashScope), and no for the Token Plan OpenAI-compatible
surface itself. Model Studio documents `GET /api/v1/models`, which returns per-model context
windows, token limits, modalities, capability codes, and prices. The Token Plan
`/compatible-mode/v1/models` listing returns bare ids only, and no Token Plan document describes a
metadata endpoint on the Token Plan hosts.

### Endpoint and authentication

- documented: `GET /api/v1/models` is documented to "Retrieve the list of available models on
  Model Studio", with filtering "by model provider, modality type, model capability, and
  deployment mode", returning "information such as pricing and context length"
  [list-models-intro] [list-models-scope].
- documented: the request URL is per region and carries a workspace id, for example
  `https://{WorkspaceId}.ap-southeast-1.maas.aliyuncs.com/api/v1/models` (Singapore),
  `https://{WorkspaceId}.cn-beijing.maas.aliyuncs.com/api/v1/models`,
  `https://{WorkspaceId}.cn-hongkong.maas.aliyuncs.com/api/v1/models`,
  `https://{WorkspaceId}.eu-central-1.maas.aliyuncs.com/api/v1/models`,
  `https://{WorkspaceId}.ap-northeast-1.maas.aliyuncs.com/api/v1/models`, and
  `https://dashscope-us.aliyuncs.com/api/v1/models` (US Virginia) [list-models].
- documented: authentication is `Authorization: Bearer YOUR_API_KEY` with an API key obtained from
  the Model Studio console [list-models-auth].
- documented: all request parameters go in the query string: `name` (fuzzy model name),
  `model` (exact model id), `language`, `page_no`, `page_size`, `providers`,
  `inference_providers`, `capabilities`, `features`, `context_window`, `service_site`,
  `supports`, `deployment_methods`, `deployment_ptu_service_tiers` [list-models].
- documented: `capabilities` filter values are modality and capability codes such as `TG`
  (text generation), `Reasoning`, `VU` (visual understanding), `IG` (image generation),
  `VG` (video generation), `ASR`, `TTS`, and `Realtime-Chatting` (realtime voice chat)
  [list-models-capabilities] [list-models-tg].

### Request and response shape

- documented: the response envelope is `request_id`, `output.total`, `output.page_no`,
  `output.page_size`, `output.models[]` [list-models].
- documented: each `output.models[]` entry has `model`, `name`, `description`, `provider`,
  `inference_provider`, `capabilities`, `features`, `published_time`, `inference_metadata`,
  `model_info`, `prices`, `equivalent_snapshot`, `inference_offline_info` [list-models].
- documented: `inference_metadata` is "Input and output modality information of the model" with
  `request_modality` and `response_modality` arrays over `Text`, `Image`, `Audio`, `Video`
  [list-models-modality].
- documented: `model_info` is "Context length information of the model" and carries
  `context_window` ("Total context window length"), `max_input_tokens`, `max_output_tokens`,
  `max_reasoning_tokens`, `reasoning_max_input_tokens`, `reasoning_max_output_tokens`; a `null`
  value means no limit or not applicable [list-models-info] [list-models-context].
- documented: `features` codes include `function-calling`, `structured-outputs`, `web-search`,
  `prefix-completion`, `cache`, `batch`, `fine-tuning`, `model-experience` [list-models].

### The Alibaba Cloud OpenAPI alternative

- documented: the Model Studio OpenAPI action `ListModels` (version 2026-02-10) is
  `GET /modelstudio/models`, authenticated with Alibaba Cloud signature credentials, filtering by
  `providers`, `capabilities`, `features`, `language`, `contextWindow`, `name`, `nextToken`,
  `maxResults`, `model`, and returning `models`, `nextToken`, `totalCount`
  [openapi-listmodels].

### What the OpenAI-compatible surface returns

- measured: `GET /models` on both Token Plan `/compatible-mode/v1` endpoints returned the same 21
  bare model ids with only `id`, `created`, `owned_by`, `object` (observation from the requesting
  session).
- documented: the OpenAI-compatible Chat API reference documents request and response bodies for
  `/compatible-mode/v1/chat/completions`, including `model` in responses, and documents no
  metadata listing beyond model ids [chatref].
- unknown: no Token Plan document describes a per-model metadata endpoint on
  `token-plan.maas.qwencloudapi.com` or `token-plan.ap-southeast-1.maas.aliyuncs.com`. The Token
  Plan docs cover the OpenAI-compatible, Anthropic-compatible, and dedicated modality endpoints
  [tp-quickstart] [tp-overview].
- unknown: whether a Token Plan `sk-sp-` key can call `GET /api/v1/models` is not documented. The
  docs state that Token Plan keys and pay-as-you-go keys are separate and cannot be used
  interchangeably, so cross-surface use should be treated as unsupported until tested
  [tp-overview].

### Searches performed for question 1

- web searches: `token-plan.maas.qwencloudapi.com Qwen Token Plan`, `Qwen Token Plan individual
  plan models included OpenAI compatible API`, `Alibaba Cloud Model Studio compatible-mode
  GET /models endpoint metadata context window`.
- the full documentation index [docs-index] (919 lines) read, then grepped for model listing,
  model catalog, api-reference, and changelog pages.
- Alibaba Cloud help: "List models" (English), the OpenAPI `ListModels` debug page, and
  "Intelligent Routing".
- QwenCloud docs: Token Plan overview, Token Plan Individual quickstart and FAQ, the whole
  api-reference page listing, "Thinking", "Text generation models", and the per-model pages
  cited below.
- conclusion: the only documented per-model metadata API is Model Studio `GET /api/v1/models`
  (mirrored by the OpenAPI `ListModels` action). The compatible-mode `/models` listing is bare
  model enumeration.

## Question 2: per-model documented facts

### qwen4-max-preview

- documented: "next-generation flagship multimodal model in the Qwen family", covering text
  generation, image and video understanding, and deep reasoning [q4mp].
- documented: preview status: "available through Token Plan only"; pay-as-you-go access is not
  supported; "The preview period runs from 10:00 on November 11, 2026 to 24:00 on November 25,
  2026 (UTC+8)" and the model is retired after the preview ends [q4mp-tp] [q4mp-window].
- documented: context length 1M tokens [q4mp-ctx]; maximum output "393,216 tokens, shared by
  thinking and the final answer" [q4mp-out].
- documented: "Thinking is enabled by default; use reasoning_effort to adjust reasoning
  intensity" [q4mp-think]; accepted `reasoning_effort` values are `low`, `medium`, `xhigh` with
  default `xhigh` [q4mp-effort] [chatref-qwen-effort].
- documented: `reasoning_effort` converts to `thinking_budget` as `low` to 4096, `medium` to
  16384, `xhigh` to 262144; OpenAI standard values map `max` and `high` to `xhigh`, `minimal` to
  `low`, and `none` to disabling thinking [chatref-qwen-effort].
- documented: input is text, images, and video; output is text; tool calling, web search, and
  structured outputs (JSON Object and JSON Schema) are supported [q4mp].
- documented: starting with this model, `max_tokens` is equivalent to `max_completion_tokens` and
  both limit the combined thinking and answer budget [q4mp].
- documented: `preserve_thinking` is enabled by default for this model [thinking].
- documented: the model release changelog entry is dated November 11, 2026 and repeats
  "Available through Token Plan only" [releases-q4mp].
- documented: the Token Plan supported-models page lists it with capabilities "Text Generation,
  Reasoning Model, Visual Understanding" [tp-list].
- note: the preview window start (10:00 on November 11, 2026, UTC+8) is 2026-11-10T21:00-05:00,
  which matches the reported appearance of the id around 2026-11-10 in US time.

### auto

- measured: present in the `GET /models` output of both Token Plan endpoints.
- documented: the Token Plan supported-models page lists `auto` under brand Qwen with capabilities
  "Reasoning Model, Text Generation" and a "New" marker [tp-list].
- documented: official client integration examples use `auto` as a model id on Token Plan,
  including a Qwen Code `modelProviders` entry named "[Token Plan Personal] auto" and a Kilo CLI
  entry `auto` with text input and text output modalities [qwen-code-cn] [kilo-cn].
- measured: `https://www.qwencloud.com/models/auto` returned 404 when fetched on 2026-11-11, so
  there is no model detail page for this id.
- unknown: which models `auto` routes to, its context window, max output, thinking mode, and
  `reasoning_effort` values. No official page describing the bare `auto` id was found.
- caution: the Alibaba Cloud Model Studio "Intelligent Routing" feature is separate; it issues
  per-workspace codes of the form `auto-model-XXXXXXXX` with documented candidate pools and
  strategies, and is not documentation of the bare `auto` id [routing].

### deepseek-v4.1-flash

- documented: lightweight flagship of the DeepSeek V4.1 family, 552B total MoE parameters, a
  causal encoder-decoder asymmetric design with 8B active input and 16B active output parameters,
  and native multimodal visual understanding [ds41].
- documented: "With a 1M-token context window and up to 384K-token maximum output" [ds41-ctx].
- documented: context limits: max input 1000000, max output 393216, context window 1000000, max
  input (thinking mode) 1000000, max output (thinking mode) 393216, max chain-of-thought 393216
  [ds41-limits].
- documented: input modality Text and Image; output modality Text; function calling supported;
  structured outputs JSON Object only; context caching supported; batch inference and fine-tuning
  unsupported [ds41].
- documented: hybrid thinking, thinking enabled by default [thinking].
- documented: `reasoning_effort` default `high`; valid values `max` (maximum-intensity inference),
  `high` (high-intensity inference), `low` (low-intensity inference); `minimal` maps to `low`,
  `medium` and `xhigh` map to `high`, `ultra` maps to `max`
  [chatref-ds-effort] [chatref-ds-map].
- documented: Chat Completions, Responses API, and Anthropic API are supported for this family
  [ds41].

### deepseek-v4.1-flash-1017

- documented: snapshot of the same family: "DeepSeek-V4.1-Flash-1017 is the latest iteration of
  V4.1-Flash", retaining "the same 552B-parameter MoE, native multimodal visual understanding,
  1M-token context window, and 384K-token maximum output" [ds41-1017].
- documented: the same context limits as the base id: max input 1000000, max output 393216,
  context window 1000000, max chain-of-thought 393216 [ds41-1017].
- documented: input Text and Image; output Text; thinking supported; Chat Completions, Responses
  API, and Anthropic API supported [ds41-1017].
- documented: `reasoning_effort` default `high`; valid values `max`, `high`, `low`
  [chatref-ds-effort].
- documented: the model release changelog entry is dated October 18, 2026 [releases].

### qwen3.8-max

- documented: context window 1000000, max input 991808 (983616 in thinking mode), max output
  131072, max chain-of-thought 262144 [q38max-in] [q38max].
- documented: input Image, Text, Video; output Text; function calling, structured outputs,
  prefix completion, and context caching supported [q38max].
- documented: hybrid thinking, thinking enabled by default [thinking-qwen38].
- documented: `reasoning_effort` options `low`, `medium`, `xhigh`, default `xhigh`, converting to
  `thinking_budget` 4096, 16384, 262144, with default `thinking_budget` 131072
  [chatref-qwen-effort].
- documented: `reasoning_effort` and `thinking_budget` cannot be set together for this model
  family; setting both is an error [chatref-qwen-effort].
- documented: snapshot `qwen3.8-max-0902` (alias `qwen3.8-max-2026-09-02`) has the same context
  limits [q38max].

### qwen3.8-flash

- documented: context window 1000000, max input 991808 (983616 in thinking mode), max output
  131072, max chain-of-thought 262144 [q38flash].
- documented: input Image, Text, Video; output Text [q38flash].
- documented: hybrid thinking, thinking enabled by default; `reasoning_effort` options `low`,
  `medium`, `xhigh` with default `xhigh`, same mapping as `qwen3.8-max`
  [thinking] [chatref-qwen-effort].
- documented: compatible with both OpenAI and Anthropic API protocols [q38flash].

### glm-5.3

- documented: "GLM-5.3 is Z.ai's flagship large language model for coding and agentic tasks",
  supporting "a 1M-token context window, 128K-token maximum output, and three reasoning effort
  levels" [glm53-effort].
- documented: context limits: max input 1048576, max output 131072, context window 1048576, max
  chain-of-thought 131072 [glm53-limits].
- documented: input Text only; output Text only [glm53].
- documented: thinking-only: the model always performs thinking; `enable_thinking` supports only
  true and passing false fails the request [chatref-glm53-effort] [thinking].
- documented: `reasoning_effort` default `max`; valid values `max` (deep reasoning), `high`
  (enhanced reasoning), `low` (light reasoning) [chatref-glm53-effort].
- documented: function calling and structured outputs supported; web search unsupported in the
  Singapore deployment scope [glm53].
- documented: the inference service provider for this id is QwenCloud [glm53].

### glm-5.2

- documented: flagship model from Zhipu AI for long-horizon tasks with an ultra-long 1M context
  window [glm52].
- documented: context limits: max input 1048576, max output 131072, context window 1048576, max
  chain-of-thought 131072 [glm52-limits].
- documented: input Text only; output Text only [glm52].
- documented: hybrid thinking, thinking enabled by default [thinking-glm52].
- documented: the GLM series `reasoning_effort` values, which "Applies to glm-5.2, glm-5.1,
  glm-5, deepseek-v4-pro", are `high` (intensive reasoning) and `max` (maximum reasoning);
  `low` and `medium` map to `high`, `xhigh` maps to `max` [chatref-glm-effort].
- documented: function calling and structured outputs supported; web search is supported in the
  Singapore scope and unsupported in the Beijing scope [glm52].

## Question 3: classification of the 21 ids

Type labels come from the Token Plan supported-models page [tp-list] unless stated otherwise.

### Chat (text generation with reasoning)

- `qwen3.8-max`, `qwen3.8-flash`, `qwen3.7-max`, `qwen3.7-plus`, `qwen3.6-flash`.
- `deepseek-v4-pro`, `deepseek-v4-pro-0813`, `deepseek-v4-flash-0731`, `deepseek-v4.1-flash`,
  `deepseek-v4.1-flash-1017`.
- `glm-5.3`, `glm-5.2`.
- `qwen4-max-preview`.
- documented: these carry "Reasoning Model" and "Text Generation" capability labels; the
  vision-capable ones additionally carry "Visual Understanding" (`qwen3.8-max`, `qwen3.8-flash`,
  `qwen3.7-plus`, `qwen3.6-flash`, `deepseek-v4.1-flash`, `deepseek-v4.1-flash-1017`,
  `qwen4-max-preview`) [tp-list].

### Router alias

- `auto`: documented as "Reasoning Model, Text Generation" on the Token Plan page and used as a
  model id in official client configs; classified here as a router alias, with routing behavior
  unknown [tp-list] [qwen-code-cn].

### Image generation

- `wan2.7-image`, `wan2.7-image-pro`, `qwen-image-3.0-pro`.
- documented: labeled "Image Generation"; image generation models are called through the dedicated
  `/api/v1/services/aigc/image-generation/generation` endpoint and are not compatible with
  `/compatible-mode/v1/chat/completions` [tp-list] [tp-quickstart].

### Video generation

- `wan3.0-video`.
- documented: labeled "Video Generation"; video generation models are called through
  `/api/v1/services/aigc/video-generation/video-synthesis` [tp-list] [tp-quickstart].

### Text to speech

- `qwen-audio-3.0-tts-plus`.
- documented: labeled "Text-to-Speech" [tp-list].

### Speech to text

- `qwen-audio-3.0-asr-flash`.
- documented: labeled "Speech Recognition" [tp-list].

### Realtime voice

- `qwen-audio-3.0-realtime-plus`.
- documented: labeled "Realtime Chatting"; the Token Plan docs describe a real-time voice
  conversation integration path for it [tp-list] [tp-overview].

### Set differences between the measured and documented lists

- measured: the 21 ids include `qwen4-max-preview` and exclude the HappyHorse video models.
- documented: the Token Plan supported-models page additionally lists `happyhorse-1.1-r2v`,
  `happyhorse-1.1-t2v`, `happyhorse-1.1-i2v` as "Video Generation" [tp-list].
- unknown: why the HappyHorse ids are absent from the `GET /models` output while documented as
  Token Plan models (plan tier, region, or listing behavior).

## Question 4: official model-listing pages for the individual plan

- documented: the international Token Plan model list is the "Supported Models" section of the
  Token Plan benefits page: "Token Plan supports Qwen and other mainstream third-party models,
  covering text, image, video, speech recognition, and real-time voice", with brand, model id,
  and capability columns [tp-list-intro] [tp-list].
- documented: the Token Plan pricing page carries the same supported-models listing per plan
  tier ("Supported models are listed below") [tp-pricing].
- documented: the Token Plan Individual quickstart gives endpoints and multimodal integration
  guidance but has no model table [tp-quickstart].
- documented: the China-side Aliyun Token Plan benefit page advertises the Qwen4-Max preview
  launch and plan tiers; its fetched content on 2026-11-11 showed no model table [aliyun-benefit].
- documented: an Aliyun developer community article dated 2026-09-22 reproduces a Token Plan
  individual model table with brand, model id, and capability columns; treat it as secondary
  material [aliyun-article].

## Sources

[docs-index]: https://docs.qwencloud.com/llms.txt
[list-models]: https://www.alibabacloud.com/help/en/model-studio/list-models
[list-models-intro]: https://www.alibabacloud.com/help/en/model-studio/list-models#:~:text=Retrieve%20the%20list%20of%20available%20models%20on%20Model%20Studio
[list-models-scope]: https://www.alibabacloud.com/help/en/model-studio/list-models#:~:text=get%20information%20such%20as%20pricing%20and%20context%20length
[list-models-auth]: https://www.alibabacloud.com/help/en/model-studio/list-models#:~:text=set%20Authorization%3A%20Bearer%20YOUR_API_KEY%20in%20the%20request%20header
[list-models-modality]: https://www.alibabacloud.com/help/en/model-studio/list-models#:~:text=Input%20and%20output%20modality%20information%20of%20the%20model
[list-models-info]: https://www.alibabacloud.com/help/en/model-studio/list-models#:~:text=Context%20length%20information%20of%20the%20model
[list-models-context]: https://www.alibabacloud.com/help/en/model-studio/list-models#:~:text=context_window%20%28Integer%29%3A%20Total%20context%20window%20length
[list-models-capabilities]: https://www.alibabacloud.com/help/en/model-studio/list-models#:~:text=Filter%20by%20modality%20type
[list-models-tg]: https://www.alibabacloud.com/help/en/model-studio/list-models#:~:text=TG%3A%20Text%20Generation
[openapi-listmodels]: https://error-center.aliyun.com/api/ModelStudio/2026-02-10/ListModels?RegionId=cn-beijing
[q4mp]: https://www.alibabacloud.com/help/en/model-studio/qwen4-max-preview
[q4mp-tp]: https://www.alibabacloud.com/help/en/model-studio/qwen4-max-preview#:~:text=available%20through%20Token%20Plan%20only
[q4mp-window]: https://www.alibabacloud.com/help/en/model-studio/qwen4-max-preview#:~:text=The%20preview%20period%20runs%20from%2010%3A00%20on%20November%2011%2C%202026
[q4mp-ctx]: https://www.alibabacloud.com/help/en/model-studio/qwen4-max-preview#:~:text=1M%20tokens
[q4mp-out]: https://www.alibabacloud.com/help/en/model-studio/qwen4-max-preview#:~:text=393%2C216%20tokens%2C%20shared%20by%20thinking%20and%20the%20final%20answer
[q4mp-think]: https://www.alibabacloud.com/help/en/model-studio/qwen4-max-preview#:~:text=Thinking%20is%20enabled%20by%20default%3B%20use%20reasoning_effort%20to%20adjust%20reasoning%20intensity
[q4mp-effort]: https://www.alibabacloud.com/help/en/model-studio/qwen4-max-preview#:~:text=Choose%20low%2C%20medium%2C%20or%20xhigh%20%28default%29
[ds41]: https://www.alibabacloud.com/help/en/model-studio/deepseek-v4-1-flash
[ds41-ctx]: https://www.alibabacloud.com/help/en/model-studio/deepseek-v4-1-flash#:~:text=With%20a%201M-token%20context%20window%20and%20up%20to%20384K-token%20maximum%20output
[ds41-limits]: https://www.alibabacloud.com/help/en/model-studio/deepseek-v4-1-flash#:~:text=Max%20Output%20Length
[ds41-1017]: https://www.alibabacloud.com/help/en/model-studio/deepseek-v4-1-flash#:~:text=DeepSeek-V4.1-Flash-1017%20is%20the%20latest%20iteration%20of%20V4.1-Flash
[tp-list]: https://www.qwencloud.com/benefits/tokenplan
[tp-list-intro]: https://www.qwencloud.com/benefits/tokenplan#:~:text=Token%20Plan%20supports%20Qwen%20and%20other%20mainstream%20third-party%20models
[tp-pricing]: https://www.qwencloud.com/pricing/token-plan
[tp-quickstart]: https://docs.qwencloud.com/token-plan/personal/token-plan-personal-quickstart
[tp-overview]: https://docs.qwencloud.com/token-plan/overview
[chatref]: https://docs.qwencloud.com/api-reference/chat/openai-chat
[chatref-qwen-effort]: https://docs.qwencloud.com/api-reference/chat/openai-chat#:~:text=Options%3A%20low%2C%20medium%2C%20xhigh.%20Default%3A%20xhigh
[chatref-glm-effort]: https://docs.qwencloud.com/api-reference/chat/openai-chat#:~:text=Applies%20to%20glm-5.2%2C%20glm-5.1%2C%20glm-5%2C%20deepseek-v4-pro.
[chatref-glm53-effort]: https://docs.qwencloud.com/api-reference/chat/openai-chat#:~:text=Default%20value%3A%20max.%20Valid%20values
[chatref-ds-effort]: https://docs.qwencloud.com/api-reference/chat/openai-chat#:~:text=deepseek-v4.1-flash%2C%20deepseek-v4.1-flash-1017%3A%20Default%20value%3A%20high
[chatref-ds-map]: https://docs.qwencloud.com/api-reference/chat/openai-chat#:~:text=minimal%20is%20mapped%20to%20low
[thinking]: https://docs.qwencloud.com/developer-guides/text-generation/thinking
[thinking-glm52]: https://docs.qwencloud.com/developer-guides/text-generation/thinking#:~:text=Hybrid%2C%20thinking%20enabled%20by%20default%3A%20glm-5.2
[thinking-qwen38]: https://docs.qwencloud.com/developer-guides/text-generation/thinking#:~:text=hybrid%2C%20thinking%20enabled%20by%20default%29%3A%20qwen3.8-max
[glm53]: https://docs.qwencloud.com/models/glm-5-3
[glm53-effort]: https://docs.qwencloud.com/models/glm-5-3#:~:text=three%20reasoning%20effort%20levels
[glm53-limits]: https://docs.qwencloud.com/models/glm-5-3#:~:text=Max%20Output%20Length
[glm52]: https://docs.qwencloud.com/models/glm-5-2
[glm52-limits]: https://docs.qwencloud.com/models/glm-5-2#:~:text=Max%20Output%20Length
[q38max]: https://docs.qwencloud.com/models/qwen3-8-max
[q38max-in]: https://docs.qwencloud.com/models/qwen3-8-max#:~:text=991808
[q38flash]: https://docs.qwencloud.com/models/qwen3-8-flash
[releases]: https://docs.qwencloud.com/changelog/models
[releases-q4mp]: https://docs.qwencloud.com/changelog/models#:~:text=Available%20through%20Token%20Plan%20only
[textgen]: https://docs.qwencloud.com/developer-guides/getting-started/text-generation-models
[routing]: https://help.aliyun.com/en/model-studio/model-routing
[qwen-code-cn]: https://platform.qianwenai.com/docs/developer-guides/clients-and-developer-tools/qwen-code
[kilo-cn]: https://platform.qianwenai.com/docs/developer-guides/clients-and-developer-tools/kilo-cli
[aliyun-benefit]: https://www.aliyun.com/benefit/scene/tokenplan
[aliyun-article]: https://developer.aliyun.com/article/1765656

## Unknowns and limits

- unknown: the routing policy behind `auto`, and its context window, max output, and thinking
  controls (no doc page; the model detail URL 404s).
- unknown: whether Token Plan `sk-sp-` keys can call Model Studio `GET /api/v1/models`.
- unknown: why HappyHorse ids are documented for Token Plan but missing from the measured
  `GET /models` output.
- limit: this session made no live endpoint calls; "measured" facts are the requesting session's
  observations.
- limit: documentation pages are live documents; the quoted text was fetched in one session
  beginning 2026-11-11T05:04:39-05:00 and can change later.
