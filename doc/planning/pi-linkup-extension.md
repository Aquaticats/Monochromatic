# Planning: Pi Linkup extension

Status:
 resolved implementation plan from grill-me review.
 Revised after GLM review on 2026-06-18.
 Not built.
 Authored 2026-06-18.

## Goal

Build a fresh Pi package that replaces the installed `@aliou/pi-linkup`
package with a smaller Linkup-only extension under `package/pi-plugin/linkup/`.
The package keeps the current Linkup-prefixed tool names that the user expects,
but removes web-answer and balance surfaces.

The extension's job is narrow:

- Search with Linkup's `POST /v1/search` endpoint.
- Fetch pages with Linkup's `POST /v1/fetch` endpoint.
- Apply a global host blocklist to every search request and every fetch attempt.
- Return Linkup's own response object as JSON text for the model,
  except when local policy filtering removes blocked search results.

The extension is not a general web provider layer,
not a research tool,
not a web-answer tool,
and not a Linkup account-management UI.

## Source facts verified before this plan

- `doc/decision/pi-web-search-extension.md` rejects every surveyed third-party package
  because none satisfies the web-answer and global-blocklist constraints together.
- Pi package docs say packages declare resources under `package.json#pi`,
  and existing repo Pi packages use `pi.extensions` pointing at `dist/final/node/index.mjs`.
- Existing Pi packages in this repo live under `package/pi-plugin/<slug>/`,
  with package names such as `@monochromatic-dev/pi-plugin-advisor`
  and `@monochromatic-dev/pi-plugin-terminal-title`.
- Pi extension docs describe `pi.registerTool()` and `prepareArguments()`;
  `prepareArguments()` runs before schema validation and can normalize legacy tool-call shapes.
- Linkup's search reference documents `POST https://api.linkup.so/v1/search`
  with required `q`,
   `depth`,
   and `outputType`,
  plus optional `excludeDomains`,
   `fromDate`,
   `includeDomains`,
   `toDate`,
  `includeImages`,
   and `maxResults`.
- Linkup's max-results changelog says `maxResults` is a first-class `/search`
  parameter for controlling the amount of context passed downstream.
- Linkup's source-filtering guide documents `excludeDomains` for excluding
  URLs or domains,
  but does not specify whether a domain entry matches subdomains by suffix.
- Linkup's fetch reference documents `POST https://api.linkup.so/v1/fetch`
  with required `url`,
  plus optional `renderJs`,
   `extractImages`,
   and `includeRawHtml`.
- Linkup fetch has no `excludeDomains` parameter,
  so fetch blocklist enforcement must happen locally before the network call.
- Existing repo Pi package tests are colocated as `src/*.unit.test.ts`
  and run through package-scoped `mise run` tasks.
- Existing repo Pi packages that log use tagged loggers from
  `@monochromatic-dev/logger-core`.
- `@monochromatic-dev/logger-core` now documents that consumers do not await
  `initPromise` before logging,
  because startup records replay after sinks verify.
- Existing repo Pi packages that declare `typebox` as a peer also install it
  for development when they import it directly.
- Several existing repo Pi packages ship `src/mise.verify-extension.ts` plus a
  `verify:extension` mise task to assert built extension registrations.

## Package shape

Create `package/pi-plugin/linkup/` with these package properties:

- Package name:
   `@monochromatic-dev/pi-linkup`.
- Entry point:
   `src/index.ts`.
- Built extension:
   `dist/final/node/index.mjs`.
- `package.json#pi.extensions`:
   `['./dist/final/node/index.mjs']`.
- Runtime dependencies:
   `@monochromatic-dev/logger-core` plus Node built-ins.
- Peer dependencies:
   `@earendil-works/pi-coding-agent` and `typebox`.
- Dev dependencies follow sibling Pi packages:
  `@earendil-works/pi-coding-agent`,
  `@monochromatic-dev/config-tsdown`,
  `@monochromatic-dev/config-typescript`,
  `@monochromatic-dev/test-harness-core`,
  `@types/node`,
  and `typebox`.

The package should include these files at minimum:

- `package.json`
- `mise.toml`
- `tsdown.node.config.ts`
- `src/index.ts`
- `src/client.ts`
- `src/config.ts`
- `src/domain-policy.ts`
- `src/mise.verify-extension.ts`
- `src/tool-output.ts`
- `src/tools.ts`
- colocated unit tests for each module with branch logic

Modules should import `tagged` from `@monochromatic-dev/logger-core` directly.
Do not add a package root logger module or await `initPromise`;
`@monochromatic-dev/logger-core` buffers startup records until sinks verify.

## Public tools

Register exactly two model-callable tools:

- `linkup_web_search`
- `linkup_web_fetch`

Do not register:

- `linkup_web_answer`
- a generic `web_answer`
- a balance command
- a balance tool
- research tools

This intentionally preserves the user's current Linkup-prefixed tool muscle memory
while removing the low-quality web-answer path.

## Search contract

`linkup_web_search` supports this model-facing input:

```typescript
// doc/planning/pi-linkup-extension.md
type LinkupWebSearchInput = {
  readonly query: string;
  readonly fromDate?: string;
  readonly includeDomains?: readonly string[];
  readonly toDate?: string;
};
```

The request body sent to Linkup is always:

```typescript
// doc/planning/pi-linkup-extension.md
{
  q: input.query,
  depth: 'standard',
  outputType: 'searchResults',
  excludeDomains: config.blocklist,
  ...(input.fromDate === undefined ? {} : { fromDate: input.fromDate }),
  ...(input.includeDomains === undefined ? {} : { includeDomains: input.includeDomains }),
  ...(input.toDate === undefined ? {} : { toDate: input.toDate }),
}
```

Extension-unsupported or fixed search parameters are accepted only as compatibility noise,
ignored,
and warned about in the tool result.
This includes:

- `depth`
- `excludeDomains`
- `includeImages`
- `limit`
- `maxResults`
- `outputType`
- every other extra key

`maxResults` is supported by Linkup,
but it is intentionally not exposed by this extension because the user selected
no per-search result-count control for this v1 plan.
`limit` is different:
it is the incumbent tool's parameter name,
not a Linkup API field.
Both are ignored with a warning so old tool calls fail loudly enough for the model
without changing the request sent to Linkup.

The tool must never let a per-call value override the fixed `standard` depth,
the fixed `searchResults` output type,
or the global blocklist.

Search still sends the normalized global blocklist to Linkup as `excludeDomains`.
After Linkup returns,
the extension also filters `response.results` with the same local suffix matcher
used by fetch preflight.
This local post-filter is required because Linkup's docs do not specify whether
`excludeDomains: ['badwikipedia.invalid']` also excludes
`www.badwikipedia.invalid`.
The model-visible response keeps Linkup's response shape with blocked result
entries removed.
The untouched upstream response stays in non-model-visible details for audit.

## Fetch contract

`linkup_web_fetch` supports this model-facing input:

```typescript
// doc/planning/pi-linkup-extension.md
type LinkupWebFetchInput = {
  readonly url: string;
};
```

The request body sent to Linkup is always:

```typescript
// doc/planning/pi-linkup-extension.md
{
  url: input.url,
  renderJs: true,
  extractImages: false,
  includeRawHtml: false,
}
```

Unsupported fetch parameters are accepted only as compatibility noise,
ignored,
and warned about in the tool result.
This includes:

- `renderJs`
- `extractImages`
- `includeRawHtml`
- every other extra key

Fetch must check the input URL's host against the blocklist before any Linkup call.
If the URL is blocked,
the tool throws an error like:

```text
Blocked by pi-linkup blocklist: badwikipedia.invalid
```

A blocked fetch is a policy refusal,
so it should be a tool error rather than an empty Linkup-shaped response.

## Configuration

Only one config file is read:

```text
~/.pi/agent/extensions/pi-linkup.json
```

The file is optional.
When present,
it has this flat shape:

```json
{
  "apiKey": "optional fallback",
  "blocklist": ["badwikipedia.invalid"]
}
```

API key precedence:

- First:
   `LINKUP_API_KEY` environment variable.
- Second:
   `apiKey` in `pi-linkup.json`.

Do not read project config.
This is deliberate:
project config must not weaken or complicate the user's global web-source policy.

Malformed JSON or invalid config must throw a clear error that includes
`pi-linkup.json` and whether the failure came from parsing,
schema validation,
or blocklist normalization.

## Blocklist grammar

Each blocklist entry is a strict host suffix.
Normalization rules:

- Trim surrounding whitespace.
- Lowercase ASCII letters.
- Strip one trailing dot.
- Reject empty entries.
- Reject schemes.
- Reject slashes.
- Reject ports.
- Reject wildcards.
- Reject empty labels.

Matching rules:

- `badwikipedia.invalid` matches `badwikipedia.invalid`.
- `badwikipedia.invalid` matches `www.badwikipedia.invalid`.
- `badwikipedia.invalid` does not match `notbadwikipedia.invalid`.

The normalized blocklist is sent to Linkup search as `excludeDomains`.
The same normalized blocklist is used locally for search result post-filtering
and fetch preflight.

## Deliberate capability removals

The incumbent skill advertises knobs this package deliberately does not expose.
These are settled grill-me decisions,
not omissions:

- Search depth is always `standard`.
  `fast` and `deep` are ignored with warnings when supplied.
- Search output type is always `searchResults`.
  Web-answer output is not present in the package.
- Search does not expose per-call `maxResults` in v1,
  even though Linkup supports it.
- Fetch always sends `renderJs: true`.
  `renderJs: false` is ignored with a warning when supplied.

## Tool output

Return Linkup's response object as JSON text when no policy filtering changes it.
If search post-filtering removes blocked results,
return the filtered Linkup-shaped response as JSON text instead.

Details should store both forms:

- `details.linkupResponse`:
   the model-visible response object,
  after policy filtering when filtering occurred.
- `details.rawLinkupResponse`:
   the untouched parsed Linkup response object.

Do not surface `details.rawLinkupResponse` to the model when it contains blocked
search results.

If ignored parameters were supplied,
return a separate warning text block before the JSON block.
The warning should name every ignored key and say the fixed behavior that won.
For example:

```text
Warning: ignored extension-unsupported linkup_web_search parameters: depth, limit, maxResults.
This extension always uses depth="standard" and does not expose per-search result-count controls.
```

Then return the response JSON as a separate text content item.
This keeps normal Linkup responses visually raw while still making migration warnings
visible to the model.

Large JSON output should be truncated using Pi's default truncation helpers.
When truncation occurs,
write the full JSON response to a temp file and include the path in the visible text.

## Error handling

The client should throw clear errors for:

- missing API key
- non-2xx Linkup response
- invalid JSON response from Linkup
- aborted request
- blocked fetch URL
- invalid config

Error messages should include endpoint context,
but must not include the API key or authorization header.

Use tagged loggers from `@monochromatic-dev/logger-core` for production logging.
Do not use raw `console.log` or `console.error` in the extension runtime.
Do not await `initPromise` from `@monochromatic-dev/logger-core`;
logging before sink verification is supported by startup replay.
The `src/mise.verify-extension.ts` script may print its final verification result
because it is a user-facing verification script,
not production tool execution.

## Tests

Unit tests should cover the exposed behavior,
not only happy paths.

Config tests:

- absent config uses empty blocklist and no config API key
- environment API key beats config API key
- flat config loads `apiKey` and `blocklist`
- invalid JSON reports config path
- invalid blocklist entry reports offending entry

Domain policy tests:

- exact host match
- subdomain suffix match
- non-boundary suffix does not match
- uppercase and trailing-dot normalization
- schemes,
   ports,
   slashes,
   wildcards,
   and empty labels are rejected

Client tests with mocked `fetch`:

- search sends `q`,
   fixed `standard`,
   fixed `searchResults`,
   and global `excludeDomains`
- search includes `fromDate`,
   `includeDomains`,
   and `toDate` when provided
- search does not send extension-unsupported per-call options,
  including Linkup-supported `maxResults`
- fetch sends fixed `renderJs: true`,
   `extractImages: false`,
   and `includeRawHtml: false`
- non-2xx response throws without leaking secrets

Tool tests:

- extension registers only `linkup_web_search` and `linkup_web_fetch`
- search ignored params produce model-visible warnings
- search local post-filter removes blocked result URLs from model-visible output
- search preserves the untouched upstream response in `details.rawLinkupResponse`
- fetch ignored params produce model-visible warnings
- blocked fetch throws before mocked `fetch` is called
- model-visible response is preserved in `details.linkupResponse`
- visible output is JSON for the model-visible response

Built-extension tests:

- `src/mise.verify-extension.ts` imports the built `dist/final/node/index.mjs`
- the fake Pi API sees only `tool:linkup_web_search` and `tool:linkup_web_fetch`

## Verification plan

Run package-scoped tasks only:

```bash
mise run //package/pi-plugin/linkup:build
mise run //package/pi-plugin/linkup:lint:types
mise run //package/pi-plugin/linkup:lint:oxlint
mise run //package/pi-plugin/linkup:test:unit
mise run //package/pi-plugin/linkup:verify:extension
```

After package tests pass,
verify the extension at the user boundary:

- Load the local package or source extension through Pi.
- Confirm only `linkup_web_search` and `linkup_web_fetch` are registered.
- Run a real search and confirm blocked-result post-filtering with a fixture
  or a mocked Linkup response.
- Run a fetch for an allowed URL.
- Run a fetch for a blocked host and confirm no Linkup network call occurs.
- Install `@monochromatic-dev/pi-linkup` into `~/.pi/agent`.
- Remove `@aliou/pi-linkup` so duplicate `linkup_web_*` tools do not coexist.
- Reload Pi and confirm `linkup_web_answer` is absent.

## Open issues

No user preference questions remain open from grill-me review.
Implementation may still discover API or Pi runtime constraints;
if that happens,
record the source path or docs URL that forced the change before editing behavior.
