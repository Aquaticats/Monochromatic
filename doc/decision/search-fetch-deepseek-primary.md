# DeepSeek as the primary search-fetch provider

## Status

Accepted 2026-10-18.
Owner decision recorded in
[`doc/planning/search-fetch-deepseek-primary.md`](../planning/search-fetch-deepseek-primary.md),
which keeps the decision ids (D1 to D11) and the interview question ids
(Q1 to Q13).
Implemented and verified in the same session.

## Context

The shared search and fetch core
(`package/search-fetch/core`) routed search through Exa fast
first and Linkup standard fallback,
and fetch through the `gh` CLI,
Linkup renderJs,
then Exa contents.
The sops/age store at `.env.local.json` already held `DEEPSEEK_API_KEY`,
but the loader requested only `EXA_API_KEY` and `LINKUP_API_KEY`,
so the key could not reach any provider.
DeepSeek Harness's native `web-search-deepseek` provider reads its own
credentials file,
which has no DeepSeek key in this profile,
and the profile disables that provider.

DeepSeek publishes a dedicated Tools API that uses the same bearer
authentication as its model API:

- `POST https://api.deepseek.com/v1/tools/search`
- `POST https://api.deepseek.com/v1/tools/fetch`

Both return raw JSON and bill per successful request,
independently of model
turns.
That made the stored key usable without re-enabling the native provider and
without a second harness surface.

## Decisions

- D1 (Q1,
  Q6):
  DeepSeek is primary for search,
  with Exa then Linkup fallback.
- D2 (Q4):
  use the dedicated Tools API,
  not the Anthropic-compatible Messages
  route.
- D3 (Q8,
  Q10):
  fetch order is `gh`,
  DeepSeek,
  Linkup renderJs,
  Exa contents.
- D4 (Q9):
  every DeepSeek search request sends `count: 50` and
  `need_content: true`,
  including fallback use.
- D5 (Q7):
  `includeDomains`,
  `fromDate`,
  and `toDate` map to `site:`,
  `after:`,
  and `before:` operators while the composed query fits 128 code points;
  dropped filters are enforced again on the returned results.
  A base query longer than 128 code points skips DeepSeek and falls to Exa.
- D6 (Q11):
  results with no publication date stay when a date filter is active;
  known dates outside the window are removed.
- D7 (Q12):
  an empty DeepSeek result list returns as-is.
  Only errors trigger the next provider,
  matching the existing Exa to Linkup
  behavior,
  and DeepSeek does not bill empty searches.
- D8:
  DeepSeek results are deduplicated by host plus path,
  first occurrence
  wins,
  before the provider-neutral blocklist filter runs.
- D9:
  credential precedence is environment then store,
  and a missing DeepSeek
  key leaves the Exa then Linkup chain in place.
  The one-time migration is unchanged because it reads legacy Pi config fields,
  and DeepSeek never had one.
- D10:
  DeepSeek requests time out after 30 seconds with no internal retries.
  Timeouts,
  401,
  402,
  403,
  404,
  408,
  413,
  429,
  and 5xx fall through;
  caller cancellation is rethrown.
  Blocklist filtering stays local,
  and page content keeps its Markdown escapes.
- D11 (Q13):
  planning record,
  this decision record,
  and updated shared,
  MCP,
  pi,
  and DSH bundle READMEs.

## Evidence

Measured with the stored key on 2026-10-18.

- Messages route:
  HTTP 200 in 6.85s with 9,802 input and 1,089 output tokens.
- Tools API search:
  HTTP 200 in 1.08s,
  ten results,
  no model tokens.
- Tools API with `count: 50` and `need_content: true`:
  50 results,
  370,105 content characters,
  573,427 response bytes,
  1.28s.
- Query operators with `site:` OR clauses and `after:` in a 103 character
  query returned only the named hosts and dates inside the window,
  in 1.78s.
- Three query quality comparison:
  DeepSeek relevance was competitive and its
  metadata (dates,
  authority,
  rerank scores,
  600 to 800 character summaries)
  was richer;
  Exa's ranking was more canonical.
  Latency was tied.
- Fetch comparison:
  DeepSeek returned 10,279 characters in 1.18s for the docs
  page and the real JS-heavy Apple documentation page in 4.57s.
  Linkup renderJs returned HTTP 200 with 7,295 characters and no expected
  content for the same JS-heavy page,
  so its shell response never triggered a
  fallback.
- Billing:
  5 yuan per 1,000 successful Search or Fetch requests;
  only non-empty Search results and non-empty Fetch bodies bill;
  both are limited to 5 queries per second per account.

## Consequences

- Search and fetch now work on a machine whose only provider credential is
  `DEEPSEEK_API_KEY`,
  including through the DSH MCP bundle.
- A successful DeepSeek search costs about 0.005 yuan and returns a large
  provider response.
  The existing model-visible cap stays at 100 KiB with the full response in a
  temp file,
  and tool-result details carry the larger provider payload.
- The model-facing tool text now names the DeepSeek-first chains,
  and the
  ignored-key warning reports that fixed behavior.
- The dsh profile keeps its native search disabled;
  the MCP surface remains the
  only search tool in that profile.
- Two scoped `eslint/no-magic-numbers` suppressions record DeepSeek's documented
  API caps (50 results and 128 query characters).
  No lint config value can express an API limit,
  and the named constants are the
  readable form;
  the suppressions sit directly on those constants.

## Rejected alternatives

- The Messages route for search:
  a model turn per search,
  about 11k tokens,
  6.85s,
  and no fetch endpoint.
- Demoting DeepSeek to last fallback:
  gives up the cost motive for normal
  searches.
- Forwarding the blocklist as `-site:` operators:
  the blocklist entries do not
  fit the 128 character query cap,
  and results are filtered locally anyway.
- Linkup renderJs before DeepSeek for fetch:
  the measured shell response shows
  that ordering can return thin content without an error.
- Falling back to Exa on empty DeepSeek results:
  an empty list is a valid
  answer and costs no DeepSeek request.
- Adding a search-order config knob:
  the interview settled a fixed primary,
  and a config surface would need its own validation,
  docs,
  and tests.

## Verification

- `mise run //package/search-fetch/core:lint:types` passes.
- `mise run //package/search-fetch/core:lint:oxlint` passes with
  zero warnings and zero errors.
- `mise run //package/search-fetch/core:test:unit` passes,
  including the new DeepSeek client suite and both fallback-chain suites.
- MCP server `buildAndTest` passes,
  and a stdio smoke call against the built server at protocol revision
  2026-07-28 returned eight JSONL results with page content through DeepSeek.
- The pi extension build,
  unit tests,
  type lint,
  and oxlint pass.
- Live check against the real store keys:
  filtered search stayed inside the requested hosts and dates,
  a plain search returned 50 results with content,
  and a fetch returned 10,279 Markdown characters,
  all through DeepSeek with no
  fallback step.
- User boundary:
  the running DSH MCP row keeps the revision it started with,
  so the new provider order is visible after a row toggle or a new session;
  the pi extension needs its usual reload.
