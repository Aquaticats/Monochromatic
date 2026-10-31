# Plan: DeepSeek as the primary search-fetch provider

## Status

Proposed 2026-10-18.
Interview settled the decisions below,
and the work landed in the same session.
Question ids (Q1 to Q13) are kept so the interview and the decision record can
be traced against each other.
The accepted record is
[`doc/decision/search-fetch-deepseek-primary.md`](../decision/search-fetch-deepseek-primary.md).
No key values appear in this document.

## Problem

The shared search and fetch core
(`package/search-fetch/core`) routes search through Exa fast
first and Linkup standard fallback,
and fetch through the `gh` CLI,
Linkup
renderJs,
then Exa contents.
`DEEPSEEK_API_KEY` already exists in the sops/age store at `.env.local.json`,
but the loader only requests `EXA_API_KEY` and `LINKUP_API_KEY`.
DeepSeek Harness's native `web-search-deepseek` provider cannot use the stored
key because the harness resolves credentials from its own environment and
credentials file,
and that provider is disabled in this profile.
A machine whose only provider credential is the DeepSeek key therefore has no
working search,
and can fetch nothing outside the `gh` routes.

DeepSeek now publishes a dedicated Search and Fetch API that uses the same
bearer authentication as its model API:
`POST https://api.deepseek.com/v1/tools/search` and
`POST https://api.deepseek.com/v1/tools/fetch`.
Both are raw JSON endpoints,
bill per successful request,
and are independent
of model turns.

## Requirements

- Use `DEEPSEEK_API_KEY` as the primary credential for `web_search` and
  `web_fetch`,
  read with the existing precedence:
  environment first,
  then the
  encrypted store.
- Keep Exa and Linkup as fallbacks,
  and keep `gh` routing for mapped GitHub
  URLs.
- Always request the maximum result count with page content from DeepSeek.
- Honor the existing search filters (`fromDate`,
  `toDate`,
  `includeDomains`)
  under DeepSeek's query limit.
- Preserve the global host blocklist and the existing output,
  truncation,
  and
  resource behavior.
- Cover every new branch with tests,
  and keep the package READMEs accurate.

## Evidence

All probes used the stored key and the real endpoints on 2026-10-18.

Search surface comparison,
same query,
ten results:

- Messages route (`/anthropic/v1/messages` with `web_search_20250305`):
  HTTP 200 in 6.85s,
  9,802 input and 1,089 output tokens,
  ten sources with
  citation snippets,
  no fetch endpoint.
- Tools API (`/v1/tools/search`):
  HTTP 200 in 1.08s,
  ten structured results,
  no model tokens.

Tools API with the required request shape (`count: 50`,
`need_content: true`):

- 50 results,
  all with content,
  370,105 content characters in total,
  573,427 response bytes,
  1.28s.
- Three duplicate host plus path pairs in the 50 results.

Search quality on three queries (DeepSeek Tools API against Exa fast):

- Relevance was competitive;
  Exa's ranking was more canonical,
  surfacing the
  official release page and repository docs,
  while DeepSeek repeated a docs
  changelog three ways and returned some raw Markdown and package-registry
  noise.
- DeepSeek returns publication dates,
  authority 0 to 4,
  rerank scores 0 to 3,
  and 600 to 800 character summaries;
  Exa fast returned titles and URLs without
  dates,
  scores,
  or text.
- Latency was tied:
  DeepSeek 0.51 to 1.10 seconds,
  Exa 0.57 to 1.02 seconds.

Query operators,
measured with `site:` OR clauses plus `after:` in a
103 character query:
the request returned 50 results restricted to the named
hosts and to dates inside the window,
in 1.78s.
The query hard limit is 128 Unicode characters.

Fetch comparison on two pages:

- DeepSeek docs page:
  DeepSeek 10,279 characters in 1.18s with a title;
  Linkup renderJs 12,080 characters in 1.59s without a title.
- JS-heavy Apple documentation page:
  DeepSeek fetched the real page with 61
  expected content occurrences in 4.57s;
  Linkup renderJs returned HTTP 200
  with 7,295 characters and zero expected occurrences,
  so it returned a shell
  without an error.

Billing and limits from the official guide:

- Search and Fetch each cost 5 yuan per 1,000 successful requests.
- Only a Search with a non-empty result list and a Fetch with page text bill.
- Failed requests,
  empty searches,
  and timeouts do not bill.
- Search and Fetch are each limited to 5 queries per second per account.
- Search accepts 1 to 50 results and caps the query at 128 characters.
- Error statuses are 400,
  401,
  402,
  403,
  404,
  408,
  413,
  429,
  500,
  and 504;
  the guide recommends retrying 408,
  429,
  and 5xx.

## Settled decisions

- D1 (Q1,
  Q6):
  DeepSeek is the primary search provider.
  Search order becomes DeepSeek,
  Exa,
  Linkup.
  The quality comparison did not show DeepSeek clearly worse,
  and the cost
  motive still holds.
- D2 (Q4):
  Use the dedicated Tools API for both tools.
  The Messages route costs a model turn,
  produces no fetch result,
  and is
  roughly six times slower.
- D3 (Q8,
  Q10):
  Fetch order becomes `gh`,
  DeepSeek,
  Linkup renderJs,
  Exa
  contents.
  The `gh` route runs first for mapped GitHub URLs.
  DeepSeek runs before Linkup because it returned real content where Linkup
  renderJs returned a shell,
  and it costs the same per successful request as
  Search.
- D4 (Q9):
  Every DeepSeek search request sends `count: 50` and
  `need_content: true`,
  including when DeepSeek runs as a fallback.
- D5 (Q7):
  Tool filters map to DeepSeek query operators only while the composed
  query fits in 128 Unicode characters.
  Operators that do not fit are dropped,
  and every filter is then enforced by
  post-filtering the returned results.
  A base query longer than 128 characters skips DeepSeek and falls through to
  Exa.
- D6 (Q11):
  When a date filter is active and post-filtering runs,
  results with
  no date stay.
  Results whose date is present and outside the window are removed.
- D7 (Q12):
  A successful but empty DeepSeek result list is returned as-is.
  Only errors trigger the next provider,
  matching the current Exa to Linkup
  behavior,
  and empty searches do not bill.
- D8:
  DeepSeek results are deduplicated by host plus path,
  first occurrence
  wins,
  before they leave the client.
  The blocklist filter and output formatting stay provider-neutral in
  `tool-exec.ts`.
- D9:
  Credential precedence is environment then store,
  and a missing DeepSeek
  key leaves the current Exa then Linkup chain in place.
  The one-time migration is unchanged because it reads legacy Pi config fields,
  and DeepSeek never had one;
  operators add the key through the existing
  encrypted store.
- D10:
  DeepSeek requests time out after 30 seconds and are not retried inside
  the client.
  A timeout,
  401,
  402,
  403,
  404,
  408,
  413,
  429,
  or 5xx falls through to the
  next provider,
  and a caller cancellation is rethrown instead of falling back.
  The global blocklist remains a local post-filter for DeepSeek results.
  DeepSeek page content is passed through with its Markdown escapes intact.
- D11 (Q13):
  Write this planning record,
  add a decision record when the work
  lands,
  and update the shared,
  MCP,
  pi,
  and DSH bundle READMEs.

## Rejected alternatives

- The Messages route for search (Q4):
  a model turn per search,
  about 11k tokens,
  6.85s,
  and no fetch endpoint.
- Demoting DeepSeek to the last fallback (Q6):
  keeps Exa ranking first but
  gives up the cost motive for normal searches.
- Forwarding the blocklist as `-site:` operators:
  the 23 normalized entries do
  not fit the 128 character query cap,
  and results are already filtered
  locally.
- Linkup renderJs before DeepSeek for fetch (Q10):
  the JS-heavy probe shows
  Linkup can return a 200 shell that never triggers a fallback.
- Falling back to Exa on empty DeepSeek results (Q12):
  an empty list is a valid
  answer,
  matches current provider semantics,
  and requires no billed call.
- Adding a search-order config knob:
  the interview settled a fixed primary,
  and
  a config surface would need its own validation,
  docs,
  and tests.

## Change surface

- `package/search-fetch/core/src/deepseek-client.ts` and
  `deepseek-http.ts`:
  request bodies,
  bearer authentication,
  error formatting,
  30 second timeout,
  result post-filtering,
  dedupe,
  and fetch normalization.
- `config-constants.ts`,
  `config-types.ts`,
  and `config.ts`:
  the
  `DEEPSEEK_API_KEY` name,
  `deepseekApiKey`,
  and origin metadata.
- `search-fetch-types.ts`,
  `search-fetch-client.ts`,
  and
  `provider-fallback.ts`:
  the `deepseek` provider id,
  client options,
  runtime
  fields,
  and both fallback chains.
- `tool-spec.ts`:
  model-facing search and fetch text for the new chains.
- `package/search-fetch/mcp/src/index.ts` and
  `package/search-fetch/pi-plugin/src/index.ts`:
  pass `deepseekApiKey`.
- Unit tests for the new client,
  routing,
  config precedence,
  and output,
  plus
  updates to existing suites that assert provider order or behavior text.
- README updates in the shared package,
  MCP server,
  pi extension,
  and DSH
  bundle.
- `doc/decision/` record when the work lands.

## Verification plan

- `mise run //package/search-fetch/core:lint:types`
- `mise run //package/search-fetch/core:test:unit`
- Build the shared package and the MCP server.
- Live check against the store keys:
  search with a query that exercises
  `includeDomains` and `fromDate`,
  a 50 result content search,
  and one fetch,
  asserting provider and result counts.
- MCP stdio smoke test of the built server.
- User boundary:
  the running DSH MCP row keeps the revision it started with,
  so the new behavior is visible after a row toggle or a new session;
  the pi
  extension needs its usual reload.

## Open questions

None.

## Next action

Landed.
Verification results and consequences are recorded in
[`doc/decision/search-fetch-deepseek-primary.md`](../decision/search-fetch-deepseek-primary.md).
The only remaining user step is restarting or respawning the DSH MCP row,
and
reloading the pi extension,
so the new provider order becomes visible at the
tool boundary.
