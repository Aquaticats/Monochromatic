# @monochromatic-dev/search-fetch-core

Host-neutral search and fetch core:
provider fallback,
global host blocklist,
gh URL routing,
shared tool specs,
and guarded valibot to TypeBox conversion.

## Why

The same search and fetch behavior serves the pi extension
(`@monochromatic-dev/search-fetch-pi-plugin`) and the MCP server
(`@monochromatic-dev/search-fetch-mcp`).
Keeping behavior here makes host differences structural only:
tool contracts,
provider routing,
blocklist enforcement,
and output formatting
have exactly one implementation.

## Provider routing

Search order is DeepSeek,
then Exa fast,
then Linkup standard.
DeepSeek runs through its dedicated Tools API
(`POST https://api.deepseek.com/v1/tools/search`) with the maximum result count
and page content always requested,
and it reads `DEEPSEEK_API_KEY` from the
environment or the encrypted store like the other provider keys.
Filters are sent as DeepSeek query operators while they fit the 128 character
query cap,
and every filter is enforced again on the returned results;
results are also deduplicated by host plus path.
A missing or failing DeepSeek key falls through to Exa and Linkup.

Fetch order is the `gh` CLI for mapped GitHub URLs,
then DeepSeek
(`POST https://api.deepseek.com/v1/tools/fetch`),
then Linkup renderJs,
then Exa contents.
DeepSeek fetch failures,
empty page content,
30 second timeouts,
and HTTP
errors all fall through to the next provider.

## Contents

- `tool-spec.ts` declares the two tool contracts once in valibot,
  including names,
  descriptions,
  and behavior text.
- `tool-exec.ts` runs search and fetch for every host:
  provider fallback,
  blocklist enforcement,
  ignored-key warnings,
  output formatting.
- `valibot-to-typebox.ts` converts the valibot specs into the TypeBox form pi compiles,
  covering exactly the constructs the specs use and throwing on anything else.
- `config.ts`,
  `client.ts`,
  `exa-client.ts`,
  `deepseek-client.ts`,
  `deepseek-query.ts`,
  `deepseek-filter.ts`,
  and `search-fetch-client.ts`
  load config and route providers.
- `gh-client.ts`,
  `gh-*.ts` plan and run gh CLI fetches for mapped GitHub URLs.
- `domain-policy.ts` enforces the global host blocklist.
- `markdown-data-image-filter.ts` strips base64 data-URL images from markdown.
- `tool-output.ts` formats model-visible output with truncation and a full-response temp path.
- `file-mutation-queue.ts` serializes same-path file mutations.

## Schema contract

Valibot is the single source for tool parameters.
Parameter schemas are open objects (`v.object`,
never `v.strictObject`) because ignored keys
are a behavior:
hosts warn about them instead of rejecting the call.

## Usage

```ts
import {
  executeWebSearchTool,
  loadLinkupConfig,
  webSearchToolSpec,
} from '@monochromatic-dev/search-fetch-core/ts';
```

## Tests

- Moved suites cover clients,
  config,
  blocklist policy,
  gh routing plans,
  markdown filtering,
  and output formatting.
- `deepseek-client.unit.test.ts` covers request shape,
  operator composition,
  local filtering,
  deduplication,
  fetch normalization,
  missing-key and HTTP errors,
  and cancellation.
- `search-fetch-client.unit.test.ts` covers both fallback chains and cancellation.
- `valibot-to-typebox.unit.test.ts` proves converter verdicts match valibot and that
  unsupported constructs fail loud.
- `file-mutation-queue.unit.test.ts` proves same-path serialization and cross-path concurrency.
- `tool-exec.unit.test.ts` covers shared execution branches.
